//! Real SSH command surface implemented with `ssh2` and Tokio async channel routing.
//! Active sessions are maintained in an in-memory session registry.
//!
//! `connect` takes only a saved host id — never raw connection details from the frontend.
//! It resolves the host record and its decrypted credential via
//! `crate::store::load_host_for_connect` internally, so a plaintext password/passphrase
//! never has to cross the Tauri IPC boundary on every connect (it was already saved,
//! encrypted, once via `store::save_host`).

use crate::error::{CatermError, ValidationError};
use crate::store::AuthMethod;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use once_cell::sync::Lazy;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SshSession {
    pub session_id: String,
    pub host_id: String,
}

pub(crate) struct SessionHandle {
    pub(crate) tx: tokio::sync::mpsc::Sender<Vec<u8>>,
    pub(crate) output_buffer: Arc<Mutex<Vec<u8>>>,
    pub(crate) channel: Arc<Mutex<ssh2::Channel>>,
    pub(crate) session: Arc<Mutex<ssh2::Session>>,
    pub(crate) host_id: String,
}

pub(crate) static SESSIONS: Lazy<Arc<Mutex<HashMap<String, SessionHandle>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

fn require_non_empty(field: &str, value: &str) -> Result<(), CatermError> {
    if value.trim().is_empty() {
        return Err(CatermError::Validation(ValidationError::Generic(format!(
            "{field} tidak boleh kosong"
        ))));
    }
    Ok(())
}

fn generate_session_id(host_id: &str) -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("ssh-{host_id}-{nanos:x}")
}

/// Opens a real SSH connection & PTY channel via `ssh2` for a saved host. Looks the host
/// (and its decrypted credential, if any) up via `crate::store::load_host_for_connect` —
/// the frontend only ever supplies a `host_id`.
pub fn connect(host_id: &str) -> Result<SshSession, CatermError> {
    require_non_empty("host_id", host_id)?;

    let (host, secret) = crate::store::load_host_for_connect(host_id)?;

    let session_id = generate_session_id(&host.id);
    let port = if host.port == 0 { 22 } else { host.port };
    let addr = format!("{}:{port}", host.address);

    let tcp = std::net::TcpStream::connect_timeout(
        &addr.parse().map_err(|e| CatermError::Validation(ValidationError::Generic(format!("Invalid address {addr}: {e}"))))?,
        Duration::from_secs(10),
    ).map_err(|e| CatermError::Validation(ValidationError::Generic(format!("Connection failed to {addr}: {e}"))))?;

    tcp.set_nonblocking(false)
        .map_err(|e| CatermError::Validation(ValidationError::Generic(format!("Failed to set blocking TCP stream: {e}"))))?;

    let mut sess = ssh2::Session::new()
        .map_err(|e| CatermError::Validation(ValidationError::Generic(format!("SSH session creation failed: {e}"))))?;

    sess.set_tcp_stream(tcp);
    sess.handshake()
        .map_err(|e| CatermError::Validation(ValidationError::Generic(format!("SSH handshake failed: {e}"))))?;

    // TOFU Host Key Verification (REQ-18, T2-SSH-04)
    let data_info = crate::paths::resolve_data_dir()?;
    let kh_file = data_info.path.join("known_hosts");
    if let Some(parent) = kh_file.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let mut known_hosts = sess.known_hosts()
        .map_err(|e| CatermError::Validation(ValidationError::Generic(format!("Failed to initialize known_hosts: {e}"))))?;

    if kh_file.exists() {
        let _ = known_hosts.read_file(&kh_file, ssh2::KnownHostFileKind::OpenSSH);
    }

    if let Some((key, key_type)) = sess.host_key() {
        let check = known_hosts.check_port(&host.address, port, key);
        match check {
            ssh2::CheckResult::Match => {
                // Verified successfully against known_hosts
            }
            ssh2::CheckResult::NotFound => {
                // Trust On First Use: record the new key
                known_hosts.add(&host.address, key, &format!("Added by CATerm for {}", host.label), key_type.into())
                    .map_err(|e| CatermError::Validation(ValidationError::Generic(format!("Failed to record TOFU host key: {e}"))))?;
                let _ = known_hosts.write_file(&kh_file, ssh2::KnownHostFileKind::OpenSSH);
            }
            ssh2::CheckResult::Mismatch => {
                return Err(CatermError::Validation(ValidationError::Generic(format!(
                    "WARNING: REMOTE HOST IDENTIFICATION HAS CHANGED! Host key for {} ({}) does not match known_hosts record.",
                    host.label, host.address
                ))));
            }
            ssh2::CheckResult::Failure => {
                return Err(CatermError::Validation(ValidationError::Generic(
                    "Host key verification check failed unexpectedly.".into()
                )));
            }
        }
    } else {
        return Err(CatermError::Validation(ValidationError::Generic(
            "Remote server did not present a host key.".into()
        )));
    }

    match &host.auth_method {
        AuthMethod::Password => {
            let password = secret.ok_or_else(|| {
                CatermError::Validation(ValidationError::Generic(format!(
                    "Password belum diset untuk host '{}' — edit host dan isi password terlebih dahulu.",
                    host.label
                )))
            })?;
            sess.userauth_password(&host.username, &password).map_err(|e| {
                CatermError::Validation(ValidationError::Generic(format!(
                    "Auth failed for user {}: {e}",
                    host.username
                )))
            })?;
        }
        AuthMethod::Key { path } => {
            let expanded = crate::paths::expand_tilde(path);
            sess.userauth_pubkey_file(&host.username, None, Path::new(&expanded), secret.as_deref())
                .map_err(|e| {
                    CatermError::Validation(ValidationError::Generic(format!(
                        "Auth via SSH key gagal untuk {} ({}): {e}",
                        host.username, expanded
                    )))
                })?;
        }
        AuthMethod::KeyId { id } => {
            let priv_pem = crate::keys::get_private_key(id)?;
            sess.userauth_pubkey_memory(&host.username, None, &priv_pem, None)
                .map_err(|e| {
                    CatermError::Validation(ValidationError::Generic(format!(
                        "Auth via Vault Key ID gagal untuk {}: {e}",
                        host.username
                    )))
                })?;
        }
    }

    if !sess.authenticated() {
        return Err(CatermError::Validation(ValidationError::Generic(
            format!("Authentication failed for user {}", host.username)
        )));
    }

    let mut channel = sess.channel_session()
        .map_err(|e| CatermError::Validation(ValidationError::Generic(format!("Channel creation failed: {e}"))))?;

    channel.request_pty("xterm-256color", None, Some((80, 24, 0, 0)))
        .map_err(|e| CatermError::Validation(ValidationError::Generic(format!("PTY request failed: {e}"))))?;

    channel.shell()
        .map_err(|e| CatermError::Validation(ValidationError::Generic(format!("Shell request failed: {e}"))))?;

    let (tx, mut rx) = tokio::sync::mpsc::channel::<Vec<u8>>(100);
    let output_buffer = Arc::new(Mutex::new(Vec::<u8>::new()));

    // Set session to non-blocking so our reader loop doesn't hold the lock forever
    sess.set_blocking(false);

    let channel_arc = Arc::new(Mutex::new(channel));

    // Spawn reader background thread
    let channel_read = Arc::clone(&channel_arc);
    let buffer_read = Arc::clone(&output_buffer);
    thread::spawn(move || {
        let mut buf = [0u8; 4096];
        loop {
            let read_res = {
                if let Ok(mut ch) = channel_read.lock() {
                    ch.read(&mut buf)
                } else {
                    break;
                }
            };

            match read_res {
                Ok(0) => break, // EOF
                Ok(n) => {
                    if let (Ok(mut out), Some(chunk)) = (buffer_read.lock(), buf.get(..n)) {
                        out.extend_from_slice(chunk);
                    }
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock {
                        thread::sleep(Duration::from_millis(10));
                    } else {
                        break;
                    }
                }
            }
        }
    });

    // Spawn writer background thread
    let channel_write = Arc::clone(&channel_arc);
    thread::spawn(move || {
        while let Some(bytes) = rx.blocking_recv() {
            if let Ok(mut ch) = channel_write.lock() {
                let _ = ch.write_all(&bytes);
                let _ = ch.flush();
            }
        }
    });

    let handle = SessionHandle {
        tx,
        output_buffer,
        channel: channel_arc,
        session: Arc::new(Mutex::new(sess)),
        host_id: host.id.clone(),
    };

    if let Ok(mut sessions) = SESSIONS.lock() {
        sessions.insert(session_id.clone(), handle);
    }

    Ok(SshSession {
        session_id,
        host_id: host.id,
    })
}

/// Write data to active SSH channel and read available output.
pub fn write(session_id: &str, data: &str) -> Result<String, CatermError> {
    require_non_empty("session_id", session_id)?;

    let (tx, output_buffer) = {
        let sessions = SESSIONS.lock().map_err(|_| {
            CatermError::Validation(ValidationError::Generic("Lock failure".into()))
        })?;
        let handle = sessions.get(session_id).ok_or_else(|| {
            CatermError::Validation(ValidationError::Generic(format!("Session {session_id} not found")))
        })?;
        (handle.tx.clone(), Arc::clone(&handle.output_buffer))
    };

    if !data.is_empty() {
        let _ = tx.blocking_send(data.as_bytes().to_vec());
    }

    // Give a brief window for response output
    thread::sleep(Duration::from_millis(20));

    let mut out_bytes = Vec::new();
    if let Ok(mut buf) = output_buffer.lock() {
        out_bytes = std::mem::take(&mut *buf);
    }

    Ok(String::from_utf8_lossy(&out_bytes).to_string())
}

/// Read available output from active SSH channel without writing.
pub fn read(session_id: &str) -> Result<String, CatermError> {
    require_non_empty("session_id", session_id)?;
    let output_buffer = {
        let sessions = SESSIONS.lock().map_err(|_| {
            CatermError::Validation(ValidationError::Generic("Lock failure".into()))
        })?;
        let handle = sessions.get(session_id).ok_or_else(|| {
            CatermError::Validation(ValidationError::Generic(format!("Session {session_id} not found")))
        })?;
        Arc::clone(&handle.output_buffer)
    };

    let mut out_bytes = Vec::new();
    if let Ok(mut buf) = output_buffer.lock() {
        out_bytes = std::mem::take(&mut *buf);
    }
    Ok(String::from_utf8_lossy(&out_bytes).to_string())
}

/// Resize active terminal PTY window.
pub fn resize(session_id: &str, cols: u16, rows: u16) -> Result<(), CatermError> {
    require_non_empty("session_id", session_id)?;

    let sessions = SESSIONS.lock().map_err(|_| {
        CatermError::Validation(ValidationError::Generic("Lock failure".into()))
    })?;
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
}
