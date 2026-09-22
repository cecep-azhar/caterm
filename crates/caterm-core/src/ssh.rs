//! Real SSH command surface implemented with `ssh2` and Tokio async channel routing.
//! Active sessions are maintained in an in-memory session registry.
//!
//! `connect` takes only a saved host id — never raw connection details from the frontend.
//! It resolves the host record and its decrypted credential via
//! `crate::store::load_host_for_connect` internally, so a plaintext password/passphrase
//! never has to cross the Tauri IPC boundary on every connect (it was already saved,
//! encrypted, once via `store::save_host`).
//!
//! Two kinds of session live here and they are deliberately kept apart:
//!
//! * **PTY sessions** (`SESSIONS`) back an interactive terminal pane. Their `ssh2::Session`
//!   is switched to non-blocking and is owned by a reader thread that polls it continuously.
//! * **Exec sessions** (`EXEC_SESSIONS`, via [`with_exec_session`]) back every *non*-interactive
//!   consumer — SFTP, monitoring, AI step execution. They are blocking and pooled per host.
//!
//! Sharing one `ssh2::Session` between the two is not safe: a consumer that flips the shared
//! session to blocking mode leaves the PTY reader thread parked inside `read()` while it still
//! holds the channel lock, which freezes the terminal permanently. Every non-interactive caller
//! must therefore go through [`with_exec_session`] and must never touch a PTY session.

use crate::error::{CatermError, ValidationError};
use crate::store::{AuthMethod, HostRecord};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SshSession {
    pub session_id: String,
    pub host_id: String,
}

/// Something that happened on a live PTY session, pushed out as it happens instead of
/// waiting for the frontend to ask. `caterm-app` turns these into Tauri events (K2-1: the
/// core stays GUI-free and only knows about this callback).
#[derive(Debug, Clone)]
pub enum SshEvent {
    Output { session_id: String, data: String },
    Closed { session_id: String },
    OsDetected { host_id: String, os: String },
}

type EventSink = Box<dyn Fn(SshEvent) + Send + Sync + 'static>;

static EVENT_SINK: Lazy<Mutex<Option<EventSink>>> = Lazy::new(|| Mutex::new(None));

/// Registers the process-wide sink that receives PTY output as it arrives. Installing a sink
/// switches sessions from "buffer and wait for [`read`]" to push delivery — the buffer is then
/// left empty on purpose so the same bytes are never delivered twice.
///
/// # Infallible
/// Stores a callback in a process-wide slot; a poisoned lock leaves the previous sink in place
/// rather than failing, because there is no caller-recoverable outcome to report at startup.
pub fn set_event_sink(sink: impl Fn(SshEvent) + Send + Sync + 'static) {
    if let Ok(mut slot) = EVENT_SINK.lock() {
        *slot = Some(Box::new(sink));
    }
}

fn has_event_sink() -> bool {
    EVENT_SINK.lock().map(|s| s.is_some()).unwrap_or(false)
}

fn emit(event: SshEvent) {
    if let Ok(sink) = EVENT_SINK.lock()
        && let Some(sink) = sink.as_ref()
    {
        sink(event);
    }
}

pub(crate) struct SessionHandle {
    pub(crate) tx: std::sync::mpsc::Sender<Vec<u8>>,
    pub(crate) output_buffer: Arc<Mutex<Vec<u8>>>,
    pub(crate) input_buffer: Arc<Mutex<String>>,
    pub(crate) channel: Arc<Mutex<ssh2::Channel>>,
    pub(crate) host_id: String,
}

pub(crate) static SESSIONS: Lazy<Arc<Mutex<HashMap<String, SessionHandle>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

/// Pooled blocking sessions for non-interactive work, one per host id.
static EXEC_SESSIONS: Lazy<Mutex<HashMap<String, Arc<Mutex<ssh2::Session>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

fn require_non_empty(field: &str, value: &str) -> Result<(), CatermError> {
    if value.trim().is_empty() {
        return Err(CatermError::Validation(ValidationError::Generic(format!(
            "{field} cannot be empty"
        ))));
    }
    Ok(())
}

fn invalid(message: String) -> CatermError {
    CatermError::Validation(ValidationError::Generic(message))
}

fn generate_session_id(host_id: &str) -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("ssh-{host_id}-{nanos:x}")
}

/// libssh2 records a non-default port as `[host]:port` but only when it is told to. Its
/// `checkp` lookup applies that transformation internally while `add` does not, so the name
/// has to be normalised here — otherwise every connect to a non-22 port reports `NotFound`
/// and silently re-adds a duplicate TOFU entry, which defeats the whole check.
fn known_hosts_name(address: &str, port: u16) -> String {
    if port == 22 {
        address.to_string()
    } else {
        format!("[{address}]:{port}")
    }
}

/// TCP connect + handshake + TOFU host key verification + user authentication for a saved
/// host. Returns a **blocking** authenticated session; interactive callers switch it to
/// non-blocking themselves once their channel is up.
///
/// This is the single place where a CATerm SSH connection is established. Every consumer
/// (PTY, SFTP, monitoring, AI) goes through it so host-key verification can never be
/// accidentally skipped by one of them.
pub(crate) fn open_authenticated_session(
    host_id: &str,
) -> Result<(ssh2::Session, HostRecord), CatermError> {
    require_non_empty("host_id", host_id)?;

    let (host, secret) = crate::store::load_host_for_connect(host_id)?;

    let port = if host.port == 0 { 22 } else { host.port };
    let addr = format!("{}:{port}", host.address);

    let socket_addr = addr
        .parse()
        .map_err(|e| invalid(format!("Invalid address {addr}: {e}")))?;

    let tcp = std::net::TcpStream::connect_timeout(&socket_addr, Duration::from_secs(10))
        .map_err(|e| invalid(format!("Connection failed to {addr}: {e}")))?;

    tcp.set_nonblocking(false)
        .map_err(|e| invalid(format!("Failed to set blocking TCP stream: {e}")))?;

    let mut sess =
        ssh2::Session::new().map_err(|e| invalid(format!("SSH session creation failed: {e}")))?;

    sess.set_tcp_stream(tcp);
    sess.handshake()
        .map_err(|e| invalid(format!("SSH handshake failed: {e}")))?;

    verify_host_key(&sess, &host, port)?;
    authenticate(&sess, &host, secret)?;

    if !sess.authenticated() {
        return Err(invalid(format!(
            "Authentication failed for user {}",
            host.username
        )));
    }

    Ok((sess, host))
}

/// TOFU Host Key Verification (REQ-18, T2-SSH-04).
fn verify_host_key(sess: &ssh2::Session, host: &HostRecord, port: u16) -> Result<(), CatermError> {
    let data_info = crate::paths::resolve_data_dir()?;
    let kh_file = data_info.path.join("known_hosts");
    if let Some(parent) = kh_file.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let mut known_hosts = sess
        .known_hosts()
        .map_err(|e| invalid(format!("Failed to initialize known_hosts: {e}")))?;

    if kh_file.exists() {
        let _ = known_hosts.read_file(&kh_file, ssh2::KnownHostFileKind::OpenSSH);
    }

    let Some((key, key_type)) = sess.host_key() else {
        return Err(invalid(
            "Remote server did not present a host key.".to_string(),
        ));
    };

    match known_hosts.check_port(&host.address, port, key) {
        ssh2::CheckResult::Match => Ok(()),
        ssh2::CheckResult::NotFound => {
            // Trust On First Use: record the new key.
            known_hosts
                .add(
                    &known_hosts_name(&host.address, port),
                    key,
                    &format!("Added by CATerm for {}", host.label),
                    key_type.into(),
                )
                .map_err(|e| invalid(format!("Failed to record TOFU host key: {e}")))?;
            let _ = known_hosts.write_file(&kh_file, ssh2::KnownHostFileKind::OpenSSH);
            Ok(())
        }
        ssh2::CheckResult::Mismatch => Err(invalid(format!(
            "WARNING: REMOTE HOST IDENTIFICATION HAS CHANGED! Host key for {} ({}) does not match known_hosts record.",
            host.label, host.address
        ))),
        ssh2::CheckResult::Failure => Err(invalid(
            "Host key verification check failed unexpectedly.".to_string(),
        )),
    }
}

fn authenticate(
    sess: &ssh2::Session,
    host: &HostRecord,
    secret: Option<String>,
) -> Result<(), CatermError> {
    match &host.auth_method {
        AuthMethod::Password => {
            let password = secret.ok_or_else(|| {
                invalid(format!(
                    "Password belum diset untuk host '{}' — edit host dan isi password terlebih dahulu.",
                    host.label
                ))
            })?;
            sess.userauth_password(&host.username, &password)
                .map_err(|e| invalid(format!("Auth failed for user {}: {e}", host.username)))
        }
        AuthMethod::Key { path } => {
            let expanded = crate::paths::expand_tilde(path);
            sess.userauth_pubkey_file(
                &host.username,
                None,
                Path::new(&expanded),
                secret.as_deref(),
            )
            .map_err(|e| {
                invalid(format!(
                    "SSH key authentication failed for {} ({expanded}): {e}",
                    host.username
                ))
            })
        }
        AuthMethod::KeyId { id } => {
            let priv_pem = crate::keys::get_private_key(id)?;
            let temp_path = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
            std::fs::write(&temp_path, priv_pem.as_bytes()).map_err(|e| {
                invalid(format!("Failed to prepare temporary key for authentication: {e}"))
            })?;
            let auth_res = sess.userauth_pubkey_file(&host.username, None, &temp_path, None);
            // Best-effort shred: the key must not outlive the auth attempt on disk.
            let _ = std::fs::write(&temp_path, vec![0u8; priv_pem.len()]);
            std::fs::remove_file(&temp_path).ok();
            auth_res.map_err(|e| {
                invalid(format!(
                    "Vault Key ID authentication failed for {}: {e}",
                    host.username
                ))
            })
        }
    }
}

fn evict_exec_session(host_id: &str) {
    if let Ok(mut pool) = EXEC_SESSIONS.lock() {
        pool.remove(host_id);
    }
}

/// `keepalive_send` is a cheap round trip that fails as soon as the transport is gone — the one
/// thing a cached handle cannot tell on its own.
fn session_is_alive(session: &Arc<Mutex<ssh2::Session>>) -> bool {
    match session.lock() {
        Ok(sess) => sess.authenticated() && sess.keepalive_send().is_ok(),
        Err(_) => false,
    }
}

/// Returns a pooled session only if it is still alive.
fn live_exec_session(host_id: &str) -> Option<Arc<Mutex<ssh2::Session>>> {
    let cached = EXEC_SESSIONS.lock().ok()?.get(host_id).map(Arc::clone)?;
    if session_is_alive(&cached) {
        Some(cached)
    } else {
        evict_exec_session(host_id);
        None
    }
}

fn open_exec_session(host_id: &str) -> Result<Arc<Mutex<ssh2::Session>>, CatermError> {
    let (sess, _host) = open_authenticated_session(host_id)?;
    sess.set_blocking(true);
    sess.set_timeout(20_000);
    sess.set_keepalive(true, 30);

    let arc = Arc::new(Mutex::new(sess));
    if let Ok(mut pool) = EXEC_SESSIONS.lock() {
        pool.insert(host_id.to_string(), Arc::clone(&arc));
    }
    Ok(arc)
}

/// Runs `f` against a blocking SSH session dedicated to non-interactive work on `host_id`.
///
/// The session is pooled and reused across calls, and is completely independent of any
/// terminal pane: SFTP and monitoring therefore work whether or not a PTY tab for that host
/// happens to be open, and can never stall the terminal. If a reused session turns out to be
/// dead, the operation is retried once on a freshly opened one, so a connection that died while
/// idle surfaces as a reconnect rather than an error. Errors raised by a still-live session are
/// the operation's own and are returned as-is.
pub fn with_exec_session<T, F>(host_id: &str, f: F) -> Result<T, CatermError>
where
    F: Fn(&ssh2::Session) -> Result<T, CatermError>,
{
    require_non_empty("host_id", host_id)?;

    if let Some(cached) = live_exec_session(host_id) {
        let outcome = match cached.lock() {
            Ok(sess) => f(&sess),
            Err(_) => Err(invalid("Exec session mutex poisoned".to_string())),
        };
        match outcome {
            Ok(value) => return Ok(value),
            Err(err) => {
                // Distinguish "the link died" from "the operation legitimately failed". Only the
                // former is worth a reconnect; retrying a genuine `No such file` would throw away
                // a perfectly good connection on every mistyped path and report it twice as slowly.
                if session_is_alive(&cached) {
                    return Err(err);
                }
                evict_exec_session(host_id);
            }
        }
    }

    let fresh = open_exec_session(host_id)?;
    let sess = fresh
        .lock()
        .map_err(|_| invalid("Exec session mutex poisoned".to_string()))?;
    f(&sess)
}

/// Splits off every complete UTF-8 character currently in `buf`, leaving a partial trailing
/// sequence behind for the next chunk. Without this a multi-byte character straddling a 4 KiB
/// read boundary would be delivered as two replacement characters.
fn drain_utf8(buf: &mut Vec<u8>) -> String {
    match std::str::from_utf8(buf) {
        Ok(text) => {
            let out = text.to_string();
            buf.clear();
            out
        }
        Err(e) => {
            let valid_len = e.valid_up_to();
            if e.error_len().is_none() {
                // Truncated trailing sequence — keep it and wait for the rest.
                let rest = buf.split_off(valid_len);
                let out = String::from_utf8_lossy(buf).into_owned();
                *buf = rest;
                out
            } else {
                // Genuinely invalid byte (binary output): degrade lossily rather than stall.
                let out = String::from_utf8_lossy(buf).into_owned();
                buf.clear();
                out
            }
        }
    }
}

/// Upper bound on how much decoded output is held before being pushed out mid-burst. Output is
/// otherwise flushed the moment the stream goes quiet, so interactive echo is never delayed;
/// this only bounds a continuous firehose (`cat` on a large file, `tail -f` on a busy log),
/// which would otherwise emit one IPC event per 4 KiB read and flood the webview.

/// Takes bytes off the wire: decoded into `pending` for push delivery, or appended to the
/// session's buffer when no sink is installed.
fn absorb_pty_bytes(
    chunk: &[u8],
    push: bool,
    carry: &mut Vec<u8>,
    pending: &mut String,
    buffer: &Arc<Mutex<Vec<u8>>>,
) {
    if push {
        carry.extend_from_slice(chunk);
        pending.push_str(&drain_utf8(carry));
    } else if let Ok(mut out) = buffer.lock() {
        out.extend_from_slice(chunk);
    }
}

/// No-op when there is nothing pending, which is also the case whenever no sink is installed.
fn flush_pty_output(session_id: &str, pending: &mut String) {
    if pending.is_empty() {
        return;
    }
    emit(SshEvent::Output {
        session_id: session_id.to_string(),
        data: std::mem::take(pending),
    });
}

/// Opens a real SSH connection & PTY channel via `ssh2` for a saved host. Looks the host
/// (and its decrypted credential, if any) up via `crate::store::load_host_for_connect` —
/// the frontend only ever supplies a `host_id`.
pub fn connect(host_id: &str) -> Result<SshSession, CatermError> {
    let (sess, host) = open_authenticated_session(host_id)?;

    let session_id = generate_session_id(&host.id);

    let mut channel = sess
        .channel_session()
        .map_err(|e| invalid(format!("Channel creation failed: {e}")))?;

    channel
        .request_pty("xterm-256color", None, Some((80, 24, 0, 0)))
        .map_err(|e| invalid(format!("PTY request failed: {e}")))?;

    channel
        .shell()
        .map_err(|e| invalid(format!("Shell request failed: {e}")))?;

    let (tx, rx) = std::sync::mpsc::channel::<Vec<u8>>();
    let output_buffer = Arc::new(Mutex::new(Vec::<u8>::new()));

    // Set session to non-blocking so our reader loop doesn't hold the lock forever. The
    // session is moved into the reader thread afterwards and is never shared with SFTP,
    // monitoring or AI execution — those use `with_exec_session` instead.
    sess.set_blocking(false);

    let channel_arc = Arc::new(Mutex::new(channel));

    // Spawn reader background thread
    let channel_read = Arc::clone(&channel_arc);
    let buffer_read = Arc::clone(&output_buffer);
    let reader_session_id = session_id.clone();
    thread::spawn(move || {
        // Owns the session for the lifetime of the PTY: dropping it here is what tears the
        // transport down once the channel is closed. (`ssh2::Channel` holds its own Arc on the
        // session internals, so the writer thread stays valid regardless of this handle.)
        let _owned_session = sess;
        let push = has_event_sink();
        let mut buf = [0u8; 4096];
        let mut err_buf = [0u8; 4096];
        let mut carry = Vec::<u8>::new();
        let mut pending = String::new();

        loop {
            let (read_res, is_eof) = {
                if let Ok(mut ch) = channel_read.lock() {
                    if let Ok(n_err) = ch.stderr().read(&mut err_buf)
                        && n_err > 0
                        && let Some(chunk) = err_buf.get(..n_err)
                    {
                        absorb_pty_bytes(chunk, push, &mut carry, &mut pending, &buffer_read);
                    }
                    let res = ch.read(&mut buf);
                    let eof = ch.eof();
                    (res, eof)
                } else {
                    break;
                }
            };

            match read_res {
                Ok(0) => {
                    // Stream went quiet: hand over whatever we have before idling, so a single
                    // keystroke echo is never held back waiting for more bytes.
                    flush_pty_output(&reader_session_id, &mut pending);
                    if is_eof {
                        break;
                    }
                    thread::sleep(Duration::from_millis(1));
                }
                Ok(n) => {
                    if let Some(chunk) = buf.get(..n) {
                        absorb_pty_bytes(chunk, push, &mut carry, &mut pending, &buffer_read);
                    }
                    // Instant flush on any read for zero-delay typing feedback
                    flush_pty_output(&reader_session_id, &mut pending);
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::Interrupted
                    {
                        flush_pty_output(&reader_session_id, &mut pending);
                        thread::sleep(Duration::from_millis(1));
                    } else {
                        break;
                    }
                }
            }
        }

        flush_pty_output(&reader_session_id, &mut pending);
        if push {
            emit(SshEvent::Closed {
                session_id: reader_session_id.clone(),
            });
        }
    });

    // Spawn writer background thread
    let channel_write = Arc::clone(&channel_arc);
    thread::spawn(move || {
        while let Ok(bytes) = rx.recv() {
            let mut written = 0;
            while written < bytes.len() {
                let (res, is_eof) = {
                    if let Ok(mut ch) = channel_write.lock() {
                        let pending = bytes.get(written..).unwrap_or(&[]);
                        let r = ch.write(pending);
                        let eof = ch.eof();
                        (r, eof)
                    } else {
                        return;
                    }
                };

                match res {
                    Ok(0) => {
                        if is_eof {
                            break;
                        }
                        thread::sleep(Duration::from_millis(1));
                    }
                    Ok(n) => {
                        written += n;
                    }
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::WouldBlock
                            || e.kind() == std::io::ErrorKind::Interrupted
                        {
                            thread::sleep(Duration::from_millis(1));
                        } else {
                            break;
                        }
                    }
                }
            }
            if let Ok(mut ch) = channel_write.lock() {
                let _ = ch.flush();
            }
        }
    });

    let handle = SessionHandle {
        tx,
        output_buffer,
        input_buffer: Arc::new(Mutex::new(String::new())),
        channel: channel_arc,
        host_id: host.id.clone(),
    };

    if let Ok(mut sessions) = SESSIONS.lock() {
        sessions.insert(session_id.clone(), handle);
    }

    // Automatically detect remote OS in the background without blocking terminal startup
    let hid_detect = host.id.clone();
    thread::spawn(move || {
        let _ = detect_host_os(&hid_detect);
    });

    Ok(SshSession {
        session_id,
        host_id: host.id,
    })
}

/// Write data to active SSH channel.
pub fn write(session_id: &str, data: &str) -> Result<String, CatermError> {
    require_non_empty("session_id", session_id)?;

    let (tx, input_buffer, host_id) = {
        let sessions = SESSIONS
            .lock()
            .map_err(|_| invalid("Lock failure".to_string()))?;
        let handle = sessions
            .get(session_id)
            .ok_or_else(|| invalid(format!("Session {session_id} not found")))?;
        (
            handle.tx.clone(),
            Arc::clone(&handle.input_buffer),
            handle.host_id.clone(),
        )
    };

    if !data.is_empty() {
        if let Ok(mut buf) = input_buffer.lock() {
            if data == "\r" || data == "\n" {
                if !buf.is_empty() {
                    let _ = crate::audit::log_event("PTY_COMMAND", Some(&host_id), &buf);
                    buf.clear();
                }
            } else if data == "\x7F" || data == "\x08" {
                // Backspace
                buf.pop();
            } else if !data.contains('\x1b') {
                // Ignore escape sequences
                buf.push_str(data);
            }
        }
        let _ = tx.send(data.as_bytes().to_vec());
    }

    Ok(String::new())
}

/// Read available output from an active SSH channel without writing.
///
/// Only meaningful when no event sink is installed (CLI, tests): with push delivery active
/// the reader thread never fills the buffer, so this returns an empty string rather than
/// handing the same bytes out twice.
pub fn read(session_id: &str) -> Result<String, CatermError> {
    require_non_empty("session_id", session_id)?;
    let output_buffer = {
        let sessions = SESSIONS
            .lock()
            .map_err(|_| invalid("Lock failure".to_string()))?;
        let handle = sessions
            .get(session_id)
            .ok_or_else(|| invalid(format!("Session {session_id} not found")))?;
        Arc::clone(&handle.output_buffer)
    };

    let mut out_bytes = Vec::new();
    if let Ok(mut buf) = output_buffer.lock()
        && !buf.is_empty()
    {
        match std::str::from_utf8(&buf) {
            Ok(_) => {
                out_bytes = std::mem::take(&mut *buf);
            }
            Err(e) => {
                let valid_len = e.valid_up_to();
                if e.error_len().is_none() {
                    out_bytes = buf.drain(..valid_len).collect();
                } else {
                    out_bytes = std::mem::take(&mut *buf);
                }
            }
        }
    }
    Ok(String::from_utf8_lossy(&out_bytes).to_string())
}

/// Resize active terminal PTY window.
pub fn resize(session_id: &str, cols: u16, rows: u16) -> Result<(), CatermError> {
    require_non_empty("session_id", session_id)?;

    let sessions = SESSIONS
        .lock()
        .map_err(|_| invalid("Lock failure".to_string()))?;
    if let Some(handle) = sessions.get(session_id)
        && let Ok(mut ch) = handle.channel.lock()
    {
        let _ = ch.request_pty_size(cols as u32, rows as u32, None, None);
    }
    Ok(())
}

/// Disconnect and remove active SSH session.
pub fn disconnect(session_id: &str) -> Result<(), CatermError> {
    require_non_empty("session_id", session_id)?;

    if let Ok(mut sessions) = SESSIONS.lock()
        && let Some(handle) = sessions.remove(session_id)
        && let Ok(mut ch) = handle.channel.lock()
    {
        let _ = ch.close();
    }
    Ok(())
}

/// Parses raw remote probe output (from `/etc/os-release` or `uname`) into a canonical OS identifier.
pub fn parse_os_key(raw: &str) -> String {
    let lower = raw.to_lowercase();

    // 1. Look for ID= line in os-release
    for line in lower.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("id=") {
            let id = rest.trim_matches('"').trim_matches('\'').trim();
            if !id.is_empty() {
                if id.contains("ubuntu") {
                    return "ubuntu".to_string();
                }
                if id.contains("debian") {
                    return "debian".to_string();
                }
                if id.contains("alpine") {
                    return "alpine".to_string();
                }
                if id.contains("arch") {
                    return "arch".to_string();
                }
                if id.contains("fedora") {
                    return "fedora".to_string();
                }
                if id.contains("centos") {
                    return "centos".to_string();
                }
                if id.contains("rocky") {
                    return "rocky".to_string();
                }
                if id.contains("alma") {
                    return "almalinux".to_string();
                }
                if id.contains("rhel") || id.contains("redhat") {
                    return "redhat".to_string();
                }
                if id.contains("manjaro") {
                    return "manjaro".to_string();
                }
                if id.contains("opensuse") || id.contains("suse") {
                    return "opensuse".to_string();
                }
                if id.contains("mint") {
                    return "mint".to_string();
                }
                if id.contains("kali") {
                    return "kali".to_string();
                }
                if id.contains("pop") {
                    return "popos".to_string();
                }
                if id.contains("gentoo") {
                    return "gentoo".to_string();
                }
                if id.contains("nixos") {
                    return "nixos".to_string();
                }
                if id.contains("raspbian") || id.contains("raspberry") {
                    return "raspberry".to_string();
                }
                if id.contains("amzn") || id.contains("amazon") {
                    return "amazon".to_string();
                }
                if id.contains("oracle") {
                    return "oracle".to_string();
                }
                if id.contains("void") {
                    return "void".to_string();
                }
                if id.contains("endeavour") {
                    return "endeavour".to_string();
                }
                if id.contains("elementary") {
                    return "elementary".to_string();
                }
                if id.contains("zorin") {
                    return "zorin".to_string();
                }
                if id.contains("freebsd") {
                    return "freebsd".to_string();
                }
                if id.contains("openbsd") {
                    return "openbsd".to_string();
                }
                if id.contains("netbsd") {
                    return "netbsd".to_string();
                }
                return id.to_string();
            }
        }
    }

    // 2. Check ID_LIKE= line
    for line in lower.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("id_like=") {
            let like = rest.trim_matches('"').trim_matches('\'').trim();
            if like.contains("ubuntu") {
                return "ubuntu".to_string();
            }
            if like.contains("debian") {
                return "debian".to_string();
            }
            if like.contains("rhel") || like.contains("centos") || like.contains("fedora") {
                return "redhat".to_string();
            }
            if like.contains("arch") {
                return "arch".to_string();
            }
            if like.contains("suse") {
                return "opensuse".to_string();
            }
        }
    }

    // 3. Fallback checks across full output
    if lower.contains("ubuntu") {
        return "ubuntu".to_string();
    }
    if lower.contains("debian") {
        return "debian".to_string();
    }
    if lower.contains("alpine") {
        return "alpine".to_string();
    }
    if lower.contains("arch") {
        return "arch".to_string();
    }
    if lower.contains("fedora") {
        return "fedora".to_string();
    }
    if lower.contains("centos") {
        return "centos".to_string();
    }
    if lower.contains("rocky") {
        return "rocky".to_string();
    }
    if lower.contains("alma") {
        return "almalinux".to_string();
    }
    if lower.contains("red hat") || lower.contains("rhel") {
        return "redhat".to_string();
    }
    if lower.contains("manjaro") {
        return "manjaro".to_string();
    }
    if lower.contains("opensuse") || lower.contains("suse") {
        return "opensuse".to_string();
    }
    if lower.contains("mint") {
        return "mint".to_string();
    }
    if lower.contains("kali") {
        return "kali".to_string();
    }
    if lower.contains("pop!_os") || lower.contains("popos") {
        return "popos".to_string();
    }
    if lower.contains("gentoo") {
        return "gentoo".to_string();
    }
    if lower.contains("nixos") {
        return "nixos".to_string();
    }
    if lower.contains("raspbian") || lower.contains("raspberry") {
        return "raspberry".to_string();
    }
    if lower.contains("amazon") || lower.contains("amzn") {
        return "amazon".to_string();
    }
    if lower.contains("oracle") {
        return "oracle".to_string();
    }
    if lower.contains("void") {
        return "void".to_string();
    }
    if lower.contains("darwin") || lower.contains("macos") || lower.contains("apple") {
        return "macos".to_string();
    }
    if lower.contains("windows")
        || lower.contains("microsoft")
        || lower.contains("cygwin")
        || lower.contains("mingw")
    {
        return "windows".to_string();
    }
    if lower.contains("freebsd") {
        return "freebsd".to_string();
    }
    if lower.contains("openbsd") {
        return "openbsd".to_string();
    }
    if lower.contains("netbsd") {
        return "netbsd".to_string();
    }
    if lower.contains("linux") {
        return "linux".to_string();
    }

    "linux".to_string()
}

/// Detects the remote host's operating system via SSH non-interactive exec session,
/// saves it to the local encrypted SQLite store, and emits an `OsDetected` event.
pub fn detect_host_os(host_id: &str) -> Result<String, CatermError> {
    require_non_empty("host_id", host_id)?;

    const OS_DETECT_SCRIPT: &str = r#"
if [ -f /etc/os-release ]; then
    cat /etc/os-release
elif [ -f /usr/lib/os-release ]; then
    cat /usr/lib/os-release
elif type uname >/dev/null 2>&1; then
    uname -s
elif [ -n "$COMSPEC" ] || [ -n "$OS" ]; then
    echo "windows"
else
    echo "linux"
fi
"#;

    let raw_output = with_exec_session(host_id, |sess| {
        let mut channel = sess
            .channel_session()
            .map_err(|e| invalid(format!("Failed to open SSH channel for OS detection: {e}")))?;
        channel
            .exec(OS_DETECT_SCRIPT)
            .map_err(|e| invalid(format!("Failed to execute OS detection command: {e}")))?;
        let mut out = String::new();
        channel.read_to_string(&mut out).unwrap_or_default();
        channel.wait_close().unwrap_or_default();
        Ok(out)
    })?;

    let detected = parse_os_key(&raw_output);
    if !detected.is_empty() && detected != "unknown" {
        let _ = crate::store::update_host_os(host_id, &detected);
        emit(SshEvent::OsDetected {
            host_id: host_id.to_string(),
            os: detected.clone(),
        });
    }

    Ok(detected)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connect_rejects_empty_host_id() {
        // Fails at the `require_non_empty` guard, before ever touching the real on-disk
        // store — unlike a made-up-but-non-empty id, which would hit `store::load_host_for_connect`
        // and thus the real per-OS data dir (not something a unit test should touch; there is
        // no dependency-injection seam here yet to point it at a temp dir instead).
        assert!(connect("").is_err());
    }

    #[test]
    fn write_rejects_empty_session_id() {
        assert!(write("", "ls").is_err());
    }

    #[test]
    fn read_rejects_empty_session_id() {
        assert!(read("").is_err());
    }

    #[test]
    fn resize_rejects_empty_session_id() {
        assert!(resize("", 80, 24).is_err());
    }

    #[test]
    fn disconnect_rejects_empty_session_id() {
        assert!(disconnect("").is_err());
    }

    #[test]
    fn with_exec_session_rejects_empty_host_id() {
        let result = with_exec_session("", |_| Ok(()));
        assert!(result.is_err());
    }

    #[test]
    fn known_hosts_name_only_brackets_non_default_ports() {
        // libssh2's `checkp` derives this form itself; `add` does not, so the two only agree
        // when the name is normalised here.
        assert_eq!(known_hosts_name("10.0.0.1", 22), "10.0.0.1");
        assert_eq!(known_hosts_name("10.0.0.1", 2222), "[10.0.0.1]:2222");
    }

    #[test]
    fn drain_utf8_holds_back_a_split_multibyte_char() {
        // "é" is 0xC3 0xA9 — arriving one byte at a time across two reads.
        let mut buf = vec![b'a', 0xC3];
        assert_eq!(drain_utf8(&mut buf), "a");
        assert_eq!(buf, vec![0xC3]);

        buf.push(0xA9);
        assert_eq!(drain_utf8(&mut buf), "é");
        assert!(buf.is_empty());
    }

    #[test]
    fn drain_utf8_does_not_stall_on_invalid_bytes() {
        let mut buf = vec![b'o', b'k', 0xFF];
        let out = drain_utf8(&mut buf);
        assert!(out.starts_with("ok"));
        assert!(buf.is_empty());
    }

    #[test]
    fn parse_os_key_debian() {
        let os_release = r#"
PRETTY_NAME="Debian GNU/Linux 12 (bookworm)"
NAME="Debian GNU/Linux"
VERSION_ID="12"
VERSION="12 (bookworm)"
ID=debian
HOME_URL="https://www.debian.org/"
"#;
        assert_eq!(parse_os_key(os_release), "debian");
    }

    #[test]
    fn parse_os_key_ubuntu() {
        let os_release = r#"
NAME="Ubuntu"
VERSION="24.04 LTS (Noble Numbat)"
ID=ubuntu
ID_LIKE=debian
PRETTY_NAME="Ubuntu 24.04 LTS"
"#;
        assert_eq!(parse_os_key(os_release), "ubuntu");
    }

    #[test]
    fn parse_os_key_alpine_and_arch() {
        assert_eq!(parse_os_key("ID=alpine\nNAME=\"Alpine Linux\""), "alpine");
        assert_eq!(parse_os_key("ID=arch\nNAME=\"Arch Linux\""), "arch");
        assert_eq!(parse_os_key("Linux"), "linux");
        assert_eq!(parse_os_key("Darwin"), "macos");
    }
}
