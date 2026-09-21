//! SFTP Remote File Manager (`T2-SFTP-01`).
//! Provides remote directory listing, reading, writing, and deletion over active SSH sessions.

use crate::error::CatermError;
use crate::ssh::SESSIONS;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SftpFileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub mtime: u64,
}

pub fn list_remote_dir(
    host_id: &str,
    remote_path: &str,
) -> Result<Vec<SftpFileEntry>, CatermError> {
    let sess_arc = {
        let sessions = SESSIONS.lock().map_err(|_| {
            CatermError::Ssh(crate::error::SshError::Generic("Lock poisoned".to_string()))
        })?;
        if let Some(session_handle) = sessions.values().find(|h| h.host_id == host_id) {
            session_handle.session.clone()
        } else {
            return Err(CatermError::Ssh(crate::error::SshError::Generic(format!(
                "No active SSH session for host {}",
                host_id
            ))));
        }
    };

    let sess_inner = sess_arc.lock().map_err(|_| {
        CatermError::Ssh(crate::error::SshError::Generic("Lock poisoned".to_string()))
    })?;
    let sftp = sess_inner.sftp().map_err(|e| {
        CatermError::Sftp(crate::error::SftpError::Generic(format!(
            "Failed to initialize SFTP subsystem: {}",
            e
        )))
    })?;

    let path = if remote_path.trim().is_empty() {
        "."
    } else {
        remote_path
    };
    let mut dir = sftp.opendir(Path::new(path)).map_err(|e| {
        CatermError::Sftp(crate::error::SftpError::Generic(format!(
            "Failed to open remote dir {}: {}",
            path, e
        )))
    })?;

    let mut entries = Vec::new();
    while let Ok((p, stat)) = dir.readdir() {
        let file_name = match p.file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => continue,
        };

        if file_name == "." || file_name == ".." {
            continue;
        }

        let full_path = if path == "/" || path == "." {
            format!("/{}", file_name)
        } else {
            format!("{}/{}", path.trim_end_matches('/'), file_name)
        };

        entries.push(SftpFileEntry {
            name: file_name,
            path: full_path,
            is_dir: stat.is_dir(),
            size: stat.size.unwrap_or(0),
            mtime: stat.mtime.unwrap_or(0),
        });
    }

    entries.sort_by(|a, b| {
        if a.is_dir == b.is_dir {
            a.name.cmp(&b.name)
        } else if a.is_dir {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });

    Ok(entries)
}

pub fn read_remote_file(host_id: &str, remote_path: &str) -> Result<Vec<u8>, CatermError> {
    let sess_arc = {
        let sessions = SESSIONS.lock().map_err(|_| {
            CatermError::Ssh(crate::error::SshError::Generic("Lock poisoned".to_string()))
        })?;
        if let Some(session_handle) = sessions.values().find(|h| h.host_id == host_id) {
            session_handle.session.clone()
        } else {
            return Err(CatermError::Ssh(crate::error::SshError::Generic(format!(
                "No active SSH session for host {}",
                host_id
            ))));
        }
    };

    let sess_inner = sess_arc.lock().map_err(|_| {
        CatermError::Ssh(crate::error::SshError::Generic("Lock poisoned".to_string()))
    })?;
    let sftp = sess_inner.sftp().map_err(|e| {
        CatermError::Sftp(crate::error::SftpError::Generic(format!(
            "Failed to initialize SFTP subsystem: {}",
            e
        )))
    })?;

    let mut file = sftp.open(Path::new(remote_path)).map_err(|e| {
        CatermError::Sftp(crate::error::SftpError::Generic(format!(
            "Failed to open remote file {}: {}",
            remote_path, e
        )))
    })?;

    let mut buf = Vec::new();
    file.read_to_end(&mut buf).map_err(|e| {
        CatermError::Sftp(crate::error::SftpError::Generic(format!(
            "Failed to read remote file: {}",
            e
        )))
    })?;

    Ok(buf)
}

pub fn write_remote_file(host_id: &str, remote_path: &str, data: &[u8]) -> Result<(), CatermError> {
    let sess_arc = {
        let sessions = SESSIONS.lock().map_err(|_| {
            CatermError::Ssh(crate::error::SshError::Generic("Lock poisoned".to_string()))
        })?;
        if let Some(session_handle) = sessions.values().find(|h| h.host_id == host_id) {
            session_handle.session.clone()
        } else {
            return Err(CatermError::Ssh(crate::error::SshError::Generic(format!(
                "No active SSH session for host {}",
                host_id
            ))));
        }
    };

    let sess_inner = sess_arc.lock().map_err(|_| {
        CatermError::Ssh(crate::error::SshError::Generic("Lock poisoned".to_string()))
    })?;
    let sftp = sess_inner.sftp().map_err(|e| {
        CatermError::Sftp(crate::error::SftpError::Generic(format!(
            "Failed to initialize SFTP subsystem: {}",
            e
        )))
    })?;

    let mut file = sftp.create(Path::new(remote_path)).map_err(|e| {
        CatermError::Sftp(crate::error::SftpError::Generic(format!(
            "Failed to create remote file {}: {}",
            remote_path, e
        )))
    })?;

    file.write_all(data).map_err(|e| {
        CatermError::Sftp(crate::error::SftpError::Generic(format!(
            "Failed to write to remote file: {}",
            e
        )))
    })?;

    Ok(())
}

pub fn delete_remote_file(host_id: &str, remote_path: &str) -> Result<(), CatermError> {
    let sess_arc = {
        let sessions = SESSIONS.lock().map_err(|_| {
            CatermError::Ssh(crate::error::SshError::Generic("Lock poisoned".to_string()))
        })?;
        if let Some(session_handle) = sessions.values().find(|h| h.host_id == host_id) {
            session_handle.session.clone()
        } else {
            return Err(CatermError::Ssh(crate::error::SshError::Generic(format!(
                "No active SSH session for host {}",
                host_id
            ))));
        }
    };

    let sess_inner = sess_arc.lock().map_err(|_| {
        CatermError::Ssh(crate::error::SshError::Generic("Lock poisoned".to_string()))
    })?;
    let sftp = sess_inner.sftp().map_err(|e| {
        CatermError::Sftp(crate::error::SftpError::Generic(format!(
            "Failed to initialize SFTP subsystem: {}",
            e
        )))
    })?;

    sftp.unlink(Path::new(remote_path)).map_err(|e| {
        CatermError::Sftp(crate::error::SftpError::Generic(format!(
            "Failed to delete remote file {}: {}",
            remote_path, e
        )))
    })?;

    Ok(())
}
