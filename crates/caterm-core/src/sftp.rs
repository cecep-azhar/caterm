//! SFTP Remote File Manager (`T2-SFTP-01`).
//! Provides remote directory listing, reading, writing, and deletion for a saved host.
//!
//! Every operation runs on a pooled non-interactive session obtained from
//! [`crate::ssh::with_exec_session`], never on the `ssh2::Session` behind a terminal pane.
//! The file manager therefore works whether or not a terminal tab for that host is open, and
//! a slow transfer can never stall the PTY reader thread (the previous implementation shared
//! the PTY session and flipped it to blocking mode, which froze the terminal).

use crate::error::{CatermError, SftpError};
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

fn sftp_err(message: String) -> CatermError {
    CatermError::Sftp(SftpError::Generic(message))
}

fn open_sftp(sess: &ssh2::Session) -> Result<ssh2::Sftp, CatermError> {
    sess.sftp()
        .map_err(|e| sftp_err(format!("Failed to initialize SFTP subsystem: {e}")))
}

/// Joins a directory with one of its entries, keeping exactly one separator and never
/// producing a leading `//`.
fn join_remote(dir: &str, name: &str) -> String {
    if dir == "/" || dir == "." || dir.is_empty() {
        format!("/{name}")
    } else {
        format!("{}/{name}", dir.trim_end_matches('/'))
    }
}

pub fn list_remote_dir(
    host_id: &str,
    remote_path: &str,
) -> Result<Vec<SftpFileEntry>, CatermError> {
    let path = if remote_path.trim().is_empty() {
        ".".to_string()
    } else {
        remote_path.to_string()
    };

    crate::ssh::with_exec_session(host_id, move |sess| {
        let sftp = open_sftp(sess)?;
        let mut dir = sftp
            .opendir(Path::new(&path))
            .map_err(|e| sftp_err(format!("Failed to open remote dir {path}: {e}")))?;

        let mut entries = Vec::new();
        while let Ok((p, stat)) = dir.readdir() {
            let file_name = match p.file_name() {
                Some(n) => n.to_string_lossy().to_string(),
                None => continue,
            };

            if file_name == "." || file_name == ".." {
                continue;
            }

            entries.push(SftpFileEntry {
                path: join_remote(&path, &file_name),
                name: file_name,
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
    })
}

pub fn read_remote_file(host_id: &str, remote_path: &str) -> Result<Vec<u8>, CatermError> {
    let path = remote_path.to_string();

    crate::ssh::with_exec_session(host_id, move |sess| {
        let sftp = open_sftp(sess)?;
        let mut file = sftp
            .open(Path::new(&path))
            .map_err(|e| sftp_err(format!("Failed to open remote file {path}: {e}")))?;

        let mut buf = Vec::new();
        file.read_to_end(&mut buf)
            .map_err(|e| sftp_err(format!("Failed to read remote file: {e}")))?;
        Ok(buf)
    })
}

pub fn write_remote_file(host_id: &str, remote_path: &str, data: &[u8]) -> Result<(), CatermError> {
    let path = remote_path.to_string();

    crate::ssh::with_exec_session(host_id, move |sess| {
        let sftp = open_sftp(sess)?;
        let mut file = sftp
            .create(Path::new(&path))
            .map_err(|e| sftp_err(format!("Failed to create remote file {path}: {e}")))?;

        file.write_all(data)
            .map_err(|e| sftp_err(format!("Failed to write to remote file: {e}")))?;
        Ok(())
    })
}

pub fn delete_remote_file(host_id: &str, remote_path: &str) -> Result<(), CatermError> {
    let path = remote_path.to_string();

    crate::ssh::with_exec_session(host_id, move |sess| {
        let sftp = open_sftp(sess)?;
        sftp.unlink(Path::new(&path))
            .map_err(|e| sftp_err(format!("Failed to delete remote file {path}: {e}")))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operations_reject_empty_host_id() {
        assert!(list_remote_dir("", "/").is_err());
        assert!(read_remote_file("", "/etc/hostname").is_err());
        assert!(write_remote_file("", "/tmp/x", b"x").is_err());
        assert!(delete_remote_file("", "/tmp/x").is_err());
    }

    #[test]
    fn join_remote_keeps_exactly_one_separator() {
        assert_eq!(join_remote("/", "etc"), "/etc");
        assert_eq!(join_remote(".", "etc"), "/etc");
        assert_eq!(join_remote("/var/log", "syslog"), "/var/log/syslog");
        assert_eq!(join_remote("/var/log/", "syslog"), "/var/log/syslog");
    }
}
