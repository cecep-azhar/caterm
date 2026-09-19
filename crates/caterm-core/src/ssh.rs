//! Real SSH command surface implemented with `ssh2` and Tokio async channel routing.
//! Active sessions are maintained in an in-memory session registry.

use crate::error::{CatermError, ValidationError};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use once_cell::sync::Lazy;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SshConnectRequest {
    pub host_id: String,
    pub address: String,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SshSession {
    pub session_id: String,
    pub host_id: String,
}

struct SessionHandle {
    tx: tokio::sync::mpsc::Sender<Vec<u8>>,
    output_buffer: Arc<Mutex<Vec<u8>>>,
    channel: Arc<Mutex<ssh2::Channel>>,
}

static SESSIONS: Lazy<Arc<Mutex<HashMap<String, SessionHandle>>>> =
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

/// Open a real SSH connection & PTY channel via `ssh2`.
pub fn connect(request: SshConnectRequest) -> Result<SshSession, CatermError> {
    require_non_empty("host_id", &request.host_id)?;
    require_non_empty("address", &request.address)?;
    require_non_empty("username", &request.username)?;

    let session_id = generate_session_id(&request.host_id);
    let addr = format!("{}:{}", request.address, if request.port == 0 { 22 } else { request.port });

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

    if let Some(ref pass) = request.password {
        sess.userauth_password(&request.username, pass)
            .map_err(|e| CatermError::Validation(ValidationError::Generic(format!("Auth failed for user {}: {e}", request.username))))?;
    } else {
        // Fallback to agent auth or unauthenticated prompt handling
        let _ = sess.userauth_agent(&request.username);
    }

    if !sess.authenticated() {
        return Err(CatermError::Validation(ValidationError::Generic(
            format!("Authentication failed for user {}", request.username)
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
                    if let Ok(mut out) = buffer_read.lock() {
                        out.extend_from_slice(&buf[..n]);
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
    };

    if let Ok(mut sessions) = SESSIONS.lock() {
        sessions.insert(session_id.clone(), handle);
    }

    Ok(SshSession {
        session_id,
        host_id: request.host_id,
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
    if let Some(handle) = sessions.get(session_id) {
        if let Ok(mut ch) = handle.channel.lock() {
            let _ = ch.request_pty_size(cols as u32, rows as u32, None, None);
        }
    }
    Ok(())
}

/// Disconnect and remove active SSH session.
pub fn disconnect(session_id: &str) -> Result<(), CatermError> {
    require_non_empty("session_id", session_id)?;

    if let Ok(mut sessions) = SESSIONS.lock() {
        if let Some(handle) = sessions.remove(session_id) {
            if let Ok(mut ch) = handle.channel.lock() {
                let _ = ch.close();
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connect_rejects_empty_host_id() {
        let result = connect(SshConnectRequest {
            host_id: "".into(),
            address: "127.0.0.1".into(),
            port: 22,
            username: "root".into(),
            password: None,
        });
        assert!(result.is_err());
    }

    #[test]
    fn write_rejects_empty_session_id() {
        assert!(write("", "ls").is_err());
    }
}
