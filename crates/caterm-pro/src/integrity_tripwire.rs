use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum FileIntegrityStatus {
    Unchanged,
    Modified,
    Missing,
    PermissionChanged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaselineFileRecord {
    pub path: String,
    pub sha256_hash: String,
    pub mode: u32,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrityCheckItem {
    pub path: String,
    pub status: FileIntegrityStatus,
    pub baseline_hash: String,
    pub current_hash: Option<String>,
    pub baseline_mode: u32,
    pub current_mode: Option<u32>,
    pub alert_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrityTripwireReport {
    pub target_name: String,
    pub timestamp: String,
    pub total_files_monitored: usize,
    pub tampered_count: usize,
    pub items: Vec<IntegrityCheckItem>,
}

/// Standard critical configuration files tracked for server/system integrity
pub const MONITORED_PATHS: &[&str] = &[
    "/etc/passwd",
    "/etc/shadow",
    "/etc/sudoers",
    "/etc/ssh/sshd_config",
    "/etc/crontab",
    "/etc/resolv.conf",
    "/etc/hosts",
    "/etc/nginx/nginx.conf",
];

pub fn hash_file_sha256(path_str: &str) -> Result<(String, u32, u64), String> {
    let path = Path::new(path_str);
    if !path.exists() {
        return Err("File does not exist".to_string());
    }

    let bytes = fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let hash = hex::encode(hasher.finalize());

    let meta = fs::metadata(path).map_err(|e| format!("Failed to stat metadata: {}", e))?;
    let size = meta.len();

    #[cfg(unix)]
    let mode = {
        use std::os::unix::fs::PermissionsExt;
        meta.permissions().mode() & 0o777
    };
    #[cfg(not(unix))]
    let mode = 0o644;

    Ok((hash, mode, size))
}

/// Create a baseline snapshot of local or standard paths
pub fn create_baseline_snapshot(
    _target_name: &str,
    paths: Option<Vec<String>>,
) -> Vec<BaselineFileRecord> {
    let check_paths =
        paths.unwrap_or_else(|| MONITORED_PATHS.iter().map(|s| s.to_string()).collect());

    let mut records = Vec::new();
    for p in check_paths {
        if let Ok((hash, mode, size)) = hash_file_sha256(&p) {
            records.push(BaselineFileRecord {
                path: p,
                sha256_hash: hash,
                mode,
                size,
            });
        }
    }
    records
}

/// Compare current system files against a baseline snapshot
pub fn verify_integrity_against_baseline(
    target_name: &str,
    baseline: &[BaselineFileRecord],
) -> IntegrityTripwireReport {
    let mut items = Vec::new();
    let mut tampered = 0;

    for b in baseline {
        match hash_file_sha256(&b.path) {
            Ok((cur_hash, cur_mode, _size)) => {
                if cur_hash != b.sha256_hash {
                    tampered += 1;
                    items.push(IntegrityCheckItem {
                        path: b.path.clone(),
                        status: FileIntegrityStatus::Modified,
                        baseline_hash: b.sha256_hash.clone(),
                        current_hash: Some(cur_hash),
                        baseline_mode: b.mode,
                        current_mode: Some(cur_mode),
                        alert_message: Some(format!(
                            "TAMPER ALERT: SHA256 checksum mismatch on `{}`. File content has been modified!",
                            b.path
                        )),
                    });
                } else if cur_mode != b.mode {
                    tampered += 1;
                    items.push(IntegrityCheckItem {
                        path: b.path.clone(),
                        status: FileIntegrityStatus::PermissionChanged,
                        baseline_hash: b.sha256_hash.clone(),
                        current_hash: Some(cur_hash),
                        baseline_mode: b.mode,
                        current_mode: Some(cur_mode),
                        alert_message: Some(format!(
                            "PERMISSION ALERT: Permissions on `{}` changed from {:o} to {:o}.",
                            b.path, b.mode, cur_mode
                        )),
                    });
                } else {
                    items.push(IntegrityCheckItem {
                        path: b.path.clone(),
                        status: FileIntegrityStatus::Unchanged,
                        baseline_hash: b.sha256_hash.clone(),
                        current_hash: Some(cur_hash),
                        baseline_mode: b.mode,
                        current_mode: Some(cur_mode),
                        alert_message: None,
                    });
                }
            }
            Err(_) => {
                tampered += 1;
                items.push(IntegrityCheckItem {
                    path: b.path.clone(),
                    status: FileIntegrityStatus::Missing,
                    baseline_hash: b.sha256_hash.clone(),
                    current_hash: None,
                    baseline_mode: b.mode,
                    current_mode: None,
                    alert_message: Some(format!(
                        "MISSING ALERT: Monitored file `{}` no longer exists or is unreadable!",
                        b.path
                    )),
                });
            }
        }
    }

    IntegrityTripwireReport {
        target_name: target_name.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        total_files_monitored: baseline.len(),
        tampered_count: tampered,
        items,
    }
}
