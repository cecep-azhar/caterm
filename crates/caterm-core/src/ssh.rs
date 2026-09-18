//! Placeholder SSH command surface (Fase 2 will replace this with a real
//! `russh`/`ssh2` transport). Every function below is a stateless stand-in —
//! no socket is ever opened, `write` just echoes its input back after
//! validating the session id looks sane — so the Tauri binding layer and the
//! frontend have a stable, typed contract to build against before any real
//! transport exists.

use crate::error::{CatermError, ValidationError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SshConnectRequest {
    pub host_id: String,
    pub address: String,
    pub port: u16,
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SshSession {
    pub session_id: String,
    pub host_id: String,
}

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

/// Open a (placeholder) SSH session for a host. Never touches the network yet.
pub fn connect(request: SshConnectRequest) -> Result<SshSession, CatermError> {
    require_non_empty("host_id", &request.host_id)?;
    let session_id = generate_session_id(&request.host_id);
    Ok(SshSession {
        session_id,
        host_id: request.host_id,
    })
}

/// Echoes `data` back verbatim. Fase 2 will forward these bytes over the real
/// SSH channel instead of echoing them.
pub fn write(session_id: &str, data: &str) -> Result<String, CatermError> {
    require_non_empty("session_id", session_id)?;
    Ok(data.to_string())
}

/// Records a terminal resize. No-op until a real PTY exists on the other end.
pub fn resize(session_id: &str, _cols: u16, _rows: u16) -> Result<(), CatermError> {
    require_non_empty("session_id", session_id)
}

/// Tears down a (placeholder) session. No-op until a real connection exists.
pub fn disconnect(session_id: &str) -> Result<(), CatermError> {
    require_non_empty("session_id", session_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connect_rejects_empty_host_id() {
        let result = connect(SshConnectRequest {
            host_id: "".into(),
            address: "10.0.0.1".into(),
            port: 22,
            username: "root".into(),
        });
        assert!(result.is_err());
    }

    #[test]
    fn connect_returns_session_bound_to_host() {
        let session = connect(SshConnectRequest {
            host_id: "host-1".into(),
            address: "10.0.0.1".into(),
            port: 22,
            username: "root".into(),
        })
        .expect("connect gagal");
        assert_eq!(session.host_id, "host-1");
        assert!(session.session_id.starts_with("ssh-host-1-"));
    }

    #[test]
    fn write_echoes_input_when_session_id_present() {
        let echoed = write("ssh-host-1-abc", "ls -la\n").expect("write gagal");
        assert_eq!(echoed, "ls -la\n");
    }

    #[test]
    fn write_rejects_empty_session_id() {
        assert!(write("", "ls").is_err());
    }
}
