//! SFTP Remote File Manager (`T2-SFTP-01`).
//! Provides remote directory listing, reading, writing, chmod, recursive deletion,
//! and high-performance chunked file transfer with progress callbacks.
//!
//! Every operation runs on a pooled non-interactive session obtained from
//! [`crate::ssh::with_exec_session`], never on the `ssh2::Session` behind a terminal pane.

use crate::error::{CatermError, SftpError};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

static CANCEL_TOKENS: once_cell::sync::Lazy<Mutex<HashSet<String>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(HashSet::new()));

/// # Infallible: inserts transfer ID to atomic cancel set.
pub fn cancel_transfer(transfer_id: &str) -> Result<(), CatermError> {
    let mut set = CANCEL_TOKENS.lock();
    set.insert(transfer_id.to_string());
    Ok(())
}

fn is_cancelled(transfer_id: &str) -> bool {
    let set = CANCEL_TOKENS.lock();
    set.contains(transfer_id)
}

fn clear_cancel_token(transfer_id: &str) {
    let mut set = CANCEL_TOKENS.lock();
    set.remove(transfer_id);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SftpFileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub size: u64,
    pub mtime: u64,
    pub mode: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SftpProgressPayload {
    pub transfer_id: String,
    pub bytes_transferred: u64,
    pub total_bytes: u64,
    pub speed_bps: u64,
}

fn sftp_err(message: String) -> CatermError {
    CatermError::Sftp(SftpError::Generic(message))
}

fn open_sftp(sess: &ssh2::Session) -> Result<ssh2::Sftp, CatermError> {
    sess.sftp()
        .map_err(|e| sftp_err(format!("Failed to initialize SFTP subsystem: {e}")))
}

/// # Infallible: pure string manipulation joining paths with one separator.
pub fn join_remote(dir: &str, name: &str) -> Result<String, CatermError> {
    if dir == "/" || dir == "." || dir.is_empty() {
        Ok(format!("/{name}"))
    } else {
        Ok(format!("{}/{name}", dir.trim_end_matches('/')))
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

            let mode = stat.perm.unwrap_or(0);
            let is_symlink = (mode & 0o120000) == 0o120000;

            let entry_p = join_remote(&path, &file_name).unwrap_or_else(|_| format!("{path}/{file_name}"));
            entries.push(SftpFileEntry {
                path: entry_p,
                name: file_name,
                is_dir: stat.is_dir(),
                is_symlink,
                size: stat.size.unwrap_or(0),
                mtime: stat.mtime.unwrap_or(0),
                mode,
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

pub fn stat_remote(host_id: &str, remote_path: &str) -> Result<SftpFileEntry, CatermError> {
    let path = remote_path.to_string();

    crate::ssh::with_exec_session(host_id, move |sess| {
        let sftp = open_sftp(sess)?;
        let stat = sftp
            .stat(Path::new(&path))
            .map_err(|e| sftp_err(format!("Failed to stat remote file {path}: {e}")))?;

        let name = Path::new(&path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.clone());

        let mode = stat.perm.unwrap_or(0);
        let is_symlink = (mode & 0o120000) == 0o120000;

        Ok(SftpFileEntry {
            name,
            path: path.clone(),
            is_dir: stat.is_dir(),
            is_symlink,
            size: stat.size.unwrap_or(0),
            mtime: stat.mtime.unwrap_or(0),
            mode,
        })
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

pub fn mkdir_remote_dir(host_id: &str, remote_path: &str) -> Result<(), CatermError> {
    let path = remote_path.to_string();

    crate::ssh::with_exec_session(host_id, move |sess| {
        let sftp = open_sftp(sess)?;
        sftp.mkdir(Path::new(&path), 0o755)
            .map_err(|e| sftp_err(format!("Failed to create remote directory {path}: {e}")))
    })
}

fn remove_remote_dir_recursive(sftp: &ssh2::Sftp, path: &Path) -> Result<(), CatermError> {
    let mut dir = match sftp.opendir(path) {
        Ok(d) => d,
        Err(_) => {
            let _ = sftp.unlink(path);
            let _ = sftp.rmdir(path);
            return Ok(());
        }
    };

    let mut children = Vec::new();
    while let Ok((child_path, stat)) = dir.readdir() {
        let name = match child_path.file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => continue,
        };
        if name == "." || name == ".." {
            continue;
        }
        children.push((child_path, stat.is_dir()));
    }

    for (child, is_dir) in children {
        if is_dir {
            remove_remote_dir_recursive(sftp, &child)?;
        } else {
            let _ = sftp.unlink(&child);
        }
    }

    sftp.rmdir(path)
        .map_err(|e| sftp_err(format!("Failed to remove remote dir {}: {e}", path.display())))?;
    Ok(())
}

pub fn delete_remote_file(
    host_id: &str,
    remote_path: &str,
    is_dir: bool,
    recursive: bool,
) -> Result<(), CatermError> {
    let path = remote_path.to_string();

    crate::ssh::with_exec_session(host_id, move |sess| {
        let sftp = open_sftp(sess)?;
        let p = Path::new(&path);
        if is_dir {
            if recursive {
                remove_remote_dir_recursive(&sftp, p)?;
            } else {
                sftp.rmdir(p)
                    .map_err(|e| sftp_err(format!("Failed to rmdir {path}: {e}")))?;
            }
        } else {
            sftp.unlink(p)
                .map_err(|e| sftp_err(format!("Failed to unlink {path}: {e}")))?;
        }
        Ok(())
    })
}

pub fn rename_remote_file(
    host_id: &str,
    old_path: &str,
    new_path: &str,
) -> Result<(), CatermError> {
    let old_p = old_path.to_string();
    let new_p = new_path.to_string();

    crate::ssh::with_exec_session(host_id, move |sess| {
        let sftp = open_sftp(sess)?;
        sftp.rename(Path::new(&old_p), Path::new(&new_p), None)
            .map_err(|e| sftp_err(format!("Failed to rename remote file {old_p} -> {new_p}: {e}")))
    })
}

pub fn chmod_remote_file(host_id: &str, remote_path: &str, mode: u32) -> Result<(), CatermError> {
    let path = remote_path.to_string();

    crate::ssh::with_exec_session(host_id, move |sess| {
        let sftp = open_sftp(sess)?;
        let mut stat = sftp
            .stat(Path::new(&path))
            .map_err(|e| sftp_err(format!("Failed to stat remote file {path}: {e}")))?;
        stat.perm = Some(mode);
        sftp.setstat(Path::new(&path), stat)
            .map_err(|e| sftp_err(format!("Failed to chmod remote file {path} to {mode:o}: {e}")))
    })
}

pub fn copy_remote_file(
    host_id: &str,
    src_path: &str,
    dst_path: &str,
) -> Result<(), CatermError> {
    let data = read_remote_file(host_id, src_path)?;
    write_remote_file(host_id, dst_path, &data)
}

/// Uploads a local file to remote SFTP with 64KB chunking and progress callbacks.
pub fn upload_file_with_progress<F>(
    host_id: &str,
    local_path: &str,
    remote_path: &str,
    transfer_id: &str,
    on_progress: Arc<Mutex<F>>,
) -> Result<(), CatermError>
where
    F: FnMut(SftpProgressPayload) + Send + 'static,
{
    clear_cancel_token(transfer_id);
    let local_file = File::open(local_path)
        .map_err(|e| sftp_err(format!("Failed to open local file {local_path}: {e}")))?;
    let total_bytes = local_file
        .metadata()
        .map(|m| m.len())
        .unwrap_or(0);

    let local_file_arc = Arc::new(Mutex::new(local_file));
    let t_id = transfer_id.to_string();
    let r_path = remote_path.to_string();

    crate::ssh::with_exec_session(host_id, move |sess| {
        let sftp = open_sftp(sess)?;
        let mut remote_file = sftp
            .create(Path::new(&r_path))
            .map_err(|e| sftp_err(format!("Failed to create remote file {r_path}: {e}")))?;

        let mut buffer = vec![0u8; 64 * 1024];
        let mut bytes_transferred = 0u64;
        let start_time = Instant::now();
        let mut last_emit = Instant::now();

        loop {
            if is_cancelled(&t_id) {
                clear_cancel_token(&t_id);
                return Err(sftp_err("Transfer cancelled by user".to_string()));
            }

            let n = {
                let mut guard = local_file_arc.lock();
                guard
                    .read(&mut buffer)
                    .map_err(|e| sftp_err(format!("Failed to read local file: {e}")))?
            };
            if n == 0 {
                break;
            }

            remote_file
                .write_all(&buffer[..n])
                .map_err(|e| sftp_err(format!("Failed to write remote chunk: {e}")))?;

            bytes_transferred += n as u64;

            if last_emit.elapsed().as_millis() >= 100 || bytes_transferred == total_bytes {
                let elapsed_secs = start_time.elapsed().as_secs_f64();
                let speed_bps = if elapsed_secs > 0.0 {
                    (bytes_transferred as f64 / elapsed_secs) as u64
                } else {
                    0
                };

                {
                    let mut cb = on_progress.lock();
                    cb(SftpProgressPayload {
                        transfer_id: t_id.clone(),
                        bytes_transferred,
                        total_bytes,
                        speed_bps,
                    });
                }
                last_emit = Instant::now();
            }
        }

        clear_cancel_token(&t_id);
        Ok(())
    })
}

/// Downloads a remote SFTP file to local path with 64KB chunking and progress callbacks.
pub fn download_file_with_progress<F>(
    host_id: &str,
    remote_path: &str,
    local_path: &str,
    transfer_id: &str,
    on_progress: Arc<Mutex<F>>,
) -> Result<(), CatermError>
where
    F: FnMut(SftpProgressPayload) + Send + 'static,
{
    clear_cancel_token(transfer_id);
    let t_id = transfer_id.to_string();
    let r_path = remote_path.to_string();
    let l_path = local_path.to_string();

    crate::ssh::with_exec_session(host_id, move |sess| {
        let sftp = open_sftp(sess)?;
        let stat = sftp
            .stat(Path::new(&r_path))
            .map_err(|e| sftp_err(format!("Failed to stat remote file {r_path}: {e}")))?;
        let total_bytes = stat.size.unwrap_or(0);

        let mut remote_file = sftp
            .open(Path::new(&r_path))
            .map_err(|e| sftp_err(format!("Failed to open remote file {r_path}: {e}")))?;

        if let Some(parent) = Path::new(&l_path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let local_file = File::create(&l_path)
            .map_err(|e| sftp_err(format!("Failed to create local destination {l_path}: {e}")))?;
        let local_file_arc = Arc::new(Mutex::new(local_file));

        let mut buffer = vec![0u8; 64 * 1024];
        let mut bytes_transferred = 0u64;
        let start_time = Instant::now();
        let mut last_emit = Instant::now();

        loop {
            if is_cancelled(&t_id) {
                clear_cancel_token(&t_id);
                return Err(sftp_err("Transfer cancelled by user".to_string()));
            }

            let n = remote_file
                .read(&mut buffer)
                .map_err(|e| sftp_err(format!("Failed to read remote chunk: {e}")))?;
            if n == 0 {
                break;
            }

            {
                let mut guard = local_file_arc.lock();
                guard
                    .write_all(&buffer[..n])
                    .map_err(|e| sftp_err(format!("Failed to write to local file: {e}")))?;
            }

            bytes_transferred += n as u64;

            if last_emit.elapsed().as_millis() >= 100 || bytes_transferred == total_bytes {
                let elapsed_secs = start_time.elapsed().as_secs_f64();
                let speed_bps = if elapsed_secs > 0.0 {
                    (bytes_transferred as f64 / elapsed_secs) as u64
                } else {
                    0
                };

                {
                    let mut cb = on_progress.lock();
                    cb(SftpProgressPayload {
                        transfer_id: t_id.clone(),
                        bytes_transferred,
                        total_bytes,
                        speed_bps,
                    });
                }
                last_emit = Instant::now();
            }
        }

        clear_cancel_token(&t_id);
        Ok(())
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
        assert!(delete_remote_file("", "/tmp/x", false, false).is_err());
        assert!(chmod_remote_file("", "/tmp/x", 0o755).is_err());
        assert!(stat_remote("", "/tmp/x").is_err());
    }

    #[test]
    fn join_remote_keeps_exactly_one_separator() {
        assert_eq!(join_remote("/", "etc").unwrap(), "/etc");
        assert_eq!(join_remote(".", "etc").unwrap(), "/etc");
        assert_eq!(join_remote("/var/log", "syslog").unwrap(), "/var/log/syslog");
        assert_eq!(join_remote("/var/log/", "syslog").unwrap(), "/var/log/syslog");
    }

    #[test]
    fn cancel_token_lifecycle() {
        let id = "test-transfer-cancel";
        assert!(!is_cancelled(id));
        cancel_transfer(id).unwrap();
        assert!(is_cancelled(id));
        clear_cancel_token(id);
        assert!(!is_cancelled(id));
    }
}
