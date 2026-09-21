//! Port Forwarding / Tunneling Manager (`T2-TOOL-02`).
//! Supports Local, Remote, and Dynamic (SOCKS5 proxy) port forwarding over SSH.
//! Safe lifecycle management: guarantees clean closing upon exit or vault lock.

use crate::error::{CatermError, ValidationError};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ForwardType {
    Local,
    Remote,
    Dynamic, // SOCKS5
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TunnelRecord {
    pub id: String,
    pub host_id: String,
    pub forward_type: ForwardType,
    pub bind_addr: String,
    pub bind_port: u16,
    pub target_addr: String,
    pub target_port: u16,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TunnelInput {
    pub host_id: String,
    pub forward_type: ForwardType,
    pub bind_addr: Option<String>,
    pub bind_port: u16,
    pub target_addr: String,
    pub target_port: u16,
}

struct ActiveTunnel {
    shutdown_signal: Arc<AtomicBool>,
}

static ACTIVE_TUNNELS: Lazy<Arc<Mutex<HashMap<String, ActiveTunnel>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

fn generate_id() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("tun-{nanos:x}")
}

/// Create the `tunnels` database table if it doesn't already exist.
pub fn init_table(conn: &rusqlite::Connection) -> Result<(), CatermError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS tunnels (
            id TEXT PRIMARY KEY,
            host_id TEXT NOT NULL,
            forward_type TEXT NOT NULL,
            bind_addr TEXT NOT NULL,
            bind_port INTEGER NOT NULL,
            target_addr TEXT NOT NULL,
            target_port INTEGER NOT NULL
        );",
    )
    .map_err(|e| {
        CatermError::Validation(ValidationError::Generic(format!(
            "Failed to create tunnels table: {e}"
        )))
    })?;
    Ok(())
}

/// List all configured tunnels along with active live state.
pub fn list_tunnels() -> Result<Vec<TunnelRecord>, CatermError> {
    let data_info = crate::paths::resolve_data_dir()?;
    let key = crate::vault::load_or_create_local_key(&data_info.path)?;
    let conn = crate::db::open_encrypted(&data_info.path, &key)?;
    init_table(&conn)?;

    let mut stmt = conn
        .prepare("SELECT id, host_id, forward_type, bind_addr, bind_port, target_addr, target_port FROM tunnels")
        .map_err(|e| CatermError::Validation(ValidationError::Generic(e.to_string())))?;

    let active_map = ACTIVE_TUNNELS.lock();

    let rows = stmt
        .query_map([], |row| {
            let ftype_str: String = row.get(2)?;
            let forward_type = match ftype_str.as_str() {
                "Remote" => ForwardType::Remote,
                "Dynamic" => ForwardType::Dynamic,
                _ => ForwardType::Local,
            };

            let id: String = row.get(0)?;
            let is_active = active_map.contains_key(&id);

            Ok(TunnelRecord {
                id,
                host_id: row.get(1)?,
                forward_type,
                bind_addr: row.get(3)?,
                bind_port: row.get(4)?,
                target_addr: row.get(5)?,
                target_port: row.get(6)?,
                is_active,
            })
        })
        .map_err(|e| CatermError::Validation(ValidationError::Generic(e.to_string())))?;

    let mut result = Vec::new();
    for r in rows {
        result
            .push(r.map_err(|e| CatermError::Validation(ValidationError::Generic(e.to_string())))?);
    }
    Ok(result)
}

/// Save a tunnel configuration in the vault.
pub fn save_tunnel(input: TunnelInput) -> Result<TunnelRecord, CatermError> {
    let bind_addr = input.bind_addr.unwrap_or_else(|| "127.0.0.1".to_string());
    let forward_type_str = match input.forward_type {
        ForwardType::Local => "Local",
        ForwardType::Remote => "Remote",
        ForwardType::Dynamic => "Dynamic",
    };

    let id = generate_id();

    let data_info = crate::paths::resolve_data_dir()?;
    let key = crate::vault::load_or_create_local_key(&data_info.path)?;
    let conn = crate::db::open_encrypted(&data_info.path, &key)?;
    init_table(&conn)?;

    conn.execute(
        "INSERT INTO tunnels (id, host_id, forward_type, bind_addr, bind_port, target_addr, target_port)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            id,
            input.host_id,
            forward_type_str,
            bind_addr,
            input.bind_port,
            input.target_addr,
            input.target_port
        ],
    )
    .map_err(|e| CatermError::Validation(ValidationError::Generic(format!("Failed to save tunnel: {e}"))))?;

    Ok(TunnelRecord {
        id,
        host_id: input.host_id,
        forward_type: input.forward_type,
        bind_addr,
        bind_port: input.bind_port,
        target_addr: input.target_addr,
        target_port: input.target_port,
        is_active: false,
    })
}

/// Delete a tunnel configuration and close if currently active.
pub fn delete_tunnel(id: &str) -> Result<(), CatermError> {
    let _ = stop_tunnel(id);

    let data_info = crate::paths::resolve_data_dir()?;
    let key = crate::vault::load_or_create_local_key(&data_info.path)?;
    let conn = crate::db::open_encrypted(&data_info.path, &key)?;
    init_table(&conn)?;

    let affected = conn
        .execute("DELETE FROM tunnels WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| CatermError::Validation(ValidationError::Generic(e.to_string())))?;

    if affected == 0 {
        return Err(CatermError::Validation(ValidationError::Generic(format!(
            "Tunnel '{id}' not found"
        ))));
    }

    Ok(())
}

fn connect_host_session(host_id: &str) -> Result<ssh2::Session, CatermError> {
    let (host, secret) = crate::store::load_host_for_connect(host_id)?;
    let port = if host.port == 0 { 22 } else { host.port };
    let addr = format!("{}:{port}", host.address);

    let tcp = TcpStream::connect_timeout(
        &addr.parse().map_err(|e| {
            CatermError::Validation(ValidationError::Generic(format!(
                "Invalid address {addr}: {e}"
            )))
        })?,
        Duration::from_secs(10),
    )
    .map_err(|e| {
        CatermError::Validation(ValidationError::Generic(format!(
            "Connection failed to {addr}: {e}"
        )))
    })?;

    let mut sess = ssh2::Session::new().map_err(|e| {
        CatermError::Validation(ValidationError::Generic(format!(
            "SSH session creation failed: {e}"
        )))
    })?;

    sess.set_tcp_stream(tcp);
    sess.handshake().map_err(|e| {
        CatermError::Validation(ValidationError::Generic(format!(
            "SSH handshake failed: {e}"
        )))
    })?;

    match &host.auth_method {
        crate::store::AuthMethod::Password => {
            let password = secret.ok_or_else(|| {
                CatermError::Validation(ValidationError::Generic(format!(
                    "Password host '{}' belum diset.",
                    host.label
                )))
            })?;
            sess.userauth_password(&host.username, &password)
                .map_err(|e| {
                    CatermError::Validation(ValidationError::Generic(format!("Auth failed: {e}")))
                })?;
        }
        crate::store::AuthMethod::Key { path } => {
            let expanded = crate::paths::expand_tilde(path);
            sess.userauth_pubkey_file(
                &host.username,
                None,
                Path::new(&expanded),
                secret.as_deref(),
            )
            .map_err(|e| {
                CatermError::Validation(ValidationError::Generic(format!(
                    "Auth via key failed: {e}"
                )))
            })?;
        }
        crate::store::AuthMethod::KeyId { id } => {
            let priv_pem = crate::keys::get_private_key(id)?;
            sess.userauth_pubkey_memory(&host.username, None, &priv_pem, None)
                .map_err(|e| {
                    CatermError::Validation(ValidationError::Generic(format!(
                        "Auth via key id failed: {e}"
                    )))
                })?;
        }
    }

    if !sess.authenticated() {
        return Err(CatermError::Validation(ValidationError::Generic(
            "Authentication failed".into(),
        )));
    }

    Ok(sess)
}

/// Start an active port forward tunnel listener.
pub fn start_tunnel(id: &str) -> Result<(), CatermError> {
    if ACTIVE_TUNNELS.lock().contains_key(id) {
        return Ok(()); // Already running
    }

    let tunnel = get_tunnel(id)?;

    // Log audit event
    let _ = crate::audit::log_event(
        "TUNNEL_START",
        Some(&tunnel.host_id),
        &format!(
            "Started tunnel {} ({}) on port {}",
            tunnel.name, tunnel.tunnel_type, tunnel.local_port
        ),
    );

    let tun = tunnels
        .iter()
        .find(|t| t.id == id)
        .ok_or_else(|| {
            CatermError::Validation(ValidationError::Generic(format!("Tunnel '{id}' not found")))
        })?
        .clone();

    let shutdown_signal = Arc::new(AtomicBool::new(false));
    let shutdown_clone = Arc::clone(&shutdown_signal);

    match tun.forward_type {
        ForwardType::Local => {
            let bind_str = format!("{}:{}", tun.bind_addr, tun.bind_port);
            let listener = TcpListener::bind(&bind_str).map_err(|e| {
                CatermError::Validation(ValidationError::Generic(format!(
                    "Failed to bind local port {bind_str}: {e}"
                )))
            })?;

            listener.set_nonblocking(true).ok();

            let host_id = tun.host_id.clone();
            let target_addr = tun.target_addr.clone();
            let target_port = tun.target_port;

            let sess = connect_host_session(&host_id)?;
            let sess_arc = Arc::new(Mutex::new(sess));

            thread::spawn(move || {
                while !shutdown_clone.load(Ordering::Relaxed) {
                    match listener.accept() {
                        Ok((mut local_stream, _)) => {
                            let sess_inner = Arc::clone(&sess_arc);
                            let t_addr = target_addr.clone();
                            let shutdown_worker = Arc::clone(&shutdown_clone);

                            thread::spawn(move || {
                                let mut channel = {
                                    let guard = sess_inner.lock();
                                    match guard.channel_direct_tcpip(
                                        &t_addr,
                                        target_port as u16,
                                        None,
                                    ) {
                                        Ok(c) => c,
                                        Err(_) => return,
                                    }
                                };

                                local_stream.set_nonblocking(true).ok();
                                let mut buf = [0u8; 8192];

                                while !shutdown_worker.load(Ordering::Relaxed) {
                                    let mut active = false;

                                    // Local -> Remote
                                    match local_stream.read(&mut buf) {
                                        Ok(0) => break,
                                        Ok(n) => {
                                            active = true;
                                            if channel.write_all(&buf[..n]).is_err() {
                                                break;
                                            }
                                        }
                                        Err(ref e)
                                            if e.kind() == std::io::ErrorKind::WouldBlock => {}
                                        Err(_) => break,
                                    }

                                    // Remote -> Local
                                    match channel.read(&mut buf) {
                                        Ok(0) => break,
                                        Ok(n) => {
                                            active = true;
                                            if local_stream.write_all(&buf[..n]).is_err() {
                                                break;
                                            }
                                        }
                                        Err(ref e)
                                            if e.kind() == std::io::ErrorKind::WouldBlock => {}
                                        Err(_) => break,
                                    }

                                    if !active {
                                        thread::sleep(Duration::from_millis(5));
                                    }
                                }
                            });
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(50));
                        }
                        Err(_) => break,
                    }
                }
            });
        }
        ForwardType::Remote => {
            let sess = connect_host_session(&tun.host_id)?;
            let mut listener_channel = sess
                .channel_forward_listen(tun.bind_port, Some(&tun.bind_addr), None)
                .map_err(|e| {
                    CatermError::Validation(ValidationError::Generic(format!(
                        "Failed to remote bind: {e}"
                    )))
                })?;

            let target_addr = tun.target_addr.clone();
            let target_port = tun.target_port;

            thread::spawn(move || {
                while !shutdown_clone.load(Ordering::Relaxed) {
                    if let Ok(mut remote_stream) = listener_channel.accept() {
                        let t_addr = target_addr.clone();
                        let shutdown_worker = Arc::clone(&shutdown_clone);
                        thread::spawn(move || {
                            let mut local_stream =
                                match TcpStream::connect(format!("{}:{}", t_addr, target_port)) {
                                    Ok(s) => s,
                                    Err(_) => return,
                                };
                            local_stream.set_nonblocking(true).ok();
                            let mut buf = [0u8; 8192];
                            while !shutdown_worker.load(Ordering::Relaxed) {
                                let mut active = false;
                                match local_stream.read(&mut buf) {
                                    Ok(0) => break,
                                    Ok(n) => {
                                        active = true;
                                        if remote_stream.write_all(&buf[..n]).is_err() {
                                            break;
                                        }
                                    }
                                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                                    Err(_) => break,
                                }
                                match remote_stream.read(&mut buf) {
                                    Ok(0) => break,
                                    Ok(n) => {
                                        active = true;
                                        if local_stream.write_all(&buf[..n]).is_err() {
                                            break;
                                        }
                                    }
                                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                                    Err(_) => break,
                                }
                                if !active {
                                    thread::sleep(Duration::from_millis(5));
                                }
                            }
                        });
                    } else {
                        thread::sleep(Duration::from_millis(50));
                    }
                }
            });
        }
        ForwardType::Dynamic => {
            let bind_str = format!("{}:{}", tun.bind_addr, tun.bind_port);
            let listener = TcpListener::bind(&bind_str).map_err(|e| {
                CatermError::Validation(ValidationError::Generic(format!(
                    "Failed to bind local port {bind_str}: {e}"
                )))
            })?;
            listener.set_nonblocking(true).ok();

            let sess = connect_host_session(&tun.host_id)?;
            let sess_arc = Arc::new(Mutex::new(sess));

            thread::spawn(move || {
                while !shutdown_clone.load(Ordering::Relaxed) {
                    match listener.accept() {
                        Ok((mut local_stream, _)) => {
                            let sess_inner = Arc::clone(&sess_arc);
                            let shutdown_worker = Arc::clone(&shutdown_clone);

                            thread::spawn(move || {
                                local_stream.set_nonblocking(false).ok();
                                let mut buf = [0u8; 512];
                                if local_stream.read_exact(&mut buf[0..2]).is_err() || buf[0] != 5 {
                                    return;
                                }
                                let nmethods = buf[1] as usize;
                                if local_stream.read_exact(&mut buf[0..nmethods]).is_err() {
                                    return;
                                }
                                if local_stream.write_all(&[5, 0]).is_err() {
                                    return;
                                }

                                if local_stream.read_exact(&mut buf[0..4]).is_err()
                                    || buf[0] != 5
                                    || buf[1] != 1
                                {
                                    return;
                                }
                                let atyp = buf[3];
                                let mut target_host = String::new();
                                if atyp == 1 {
                                    if local_stream.read_exact(&mut buf[0..4]).is_err() {
                                        return;
                                    }
                                    target_host =
                                        format!("{}.{}.{}.{}", buf[0], buf[1], buf[2], buf[3]);
                                } else if atyp == 3 {
                                    if local_stream.read_exact(&mut buf[0..1]).is_err() {
                                        return;
                                    }
                                    let len = buf[0] as usize;
                                    if local_stream.read_exact(&mut buf[0..len]).is_err() {
                                        return;
                                    }
                                    target_host = String::from_utf8_lossy(&buf[0..len]).to_string();
                                } else if atyp == 4 {
                                    if local_stream.read_exact(&mut buf[0..16]).is_err() {
                                        return;
                                    }
                                    return; // IPv6 not implemented
                                } else {
                                    return;
                                }

                                if local_stream.read_exact(&mut buf[0..2]).is_err() {
                                    return;
                                }
                                let target_port = u16::from_be_bytes([buf[0], buf[1]]);

                                let mut channel = {
                                    let guard = sess_inner.lock();
                                    match guard.channel_direct_tcpip(
                                        &target_host,
                                        target_port,
                                        None,
                                    ) {
                                        Ok(c) => c,
                                        Err(_) => return,
                                    }
                                };

                                if local_stream
                                    .write_all(&[5, 0, 0, 1, 0, 0, 0, 0, 0, 0])
                                    .is_err()
                                {
                                    return;
                                }

                                local_stream.set_nonblocking(true).ok();
                                let mut buf2 = [0u8; 8192];
                                while !shutdown_worker.load(Ordering::Relaxed) {
                                    let mut active = false;
                                    match local_stream.read(&mut buf2) {
                                        Ok(0) => break,
                                        Ok(n) => {
                                            active = true;
                                            if channel.write_all(&buf2[..n]).is_err() {
                                                break;
                                            }
                                        }
                                        Err(ref e)
                                            if e.kind() == std::io::ErrorKind::WouldBlock => {}
                                        Err(_) => break,
                                    }
                                    match channel.read(&mut buf2) {
                                        Ok(0) => break,
                                        Ok(n) => {
                                            active = true;
                                            if local_stream.write_all(&buf2[..n]).is_err() {
                                                break;
                                            }
                                        }
                                        Err(ref e)
                                            if e.kind() == std::io::ErrorKind::WouldBlock => {}
                                        Err(_) => break,
                                    }
                                    if !active {
                                        thread::sleep(Duration::from_millis(5));
                                    }
                                }
                            });
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(50));
                        }
                        Err(_) => break,
                    }
                }
            });
        }
    }

    ACTIVE_TUNNELS
        .lock()
        .insert(id.to_string(), ActiveTunnel { shutdown_signal });
    Ok(())
}

/// Stop an active port forward tunnel listener.
pub fn stop_tunnel(id: &str) -> Result<(), CatermError> {
    if let Some(active) = ACTIVE_TUNNELS.lock().remove(id) {
        active.shutdown_signal.store(true, Ordering::Relaxed);

        if let Ok(tunnel) = get_tunnel(id) {
            let _ = crate::audit::log_event(
                "TUNNEL_STOP",
                Some(&tunnel.host_id),
                &format!(
                    "Stopped tunnel {} ({}) on port {}",
                    tunnel.name, tunnel.tunnel_type, tunnel.local_port
                ),
            );
        }
    }
    Ok(())
}

/// Stop all active tunnels cleanly (e.g. called on vault lock or app exit).
pub fn stop_all_tunnels() -> Result<(), CatermError> {
    let mut tunnels = ACTIVE_TUNNELS.lock();
    for (_, tunnel) in tunnels.drain() {
        tunnel.shutdown_signal.store(true, Ordering::Relaxed);
    }
    Ok(())
}
