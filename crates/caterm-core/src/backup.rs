//! Encrypted JSON Vault Backup & Restore (`T2-EXIM-01`).
//! Exports all hosts, groups, snippets, and keys as an encrypted JSON archive.
//! Imports and merges encrypted backup archives with integrity validation.

use crate::ai::{self, AiSettings};
use crate::error::{CatermError, VaultError};
use crate::groups::{self, GroupInput, GroupRecord};
use crate::investigations::{self, InvestigationInput, InvestigationRecord};
use crate::keys::{self, KeyRecord};
use crate::scheduler::store as scheduler_store;
use crate::scheduler::task::{ScheduledTaskRecord, TaskInput};
use crate::snippets::{self, SnippetInput, SnippetRecord};
use crate::store::{self, HostInput, HostRecord};
use crate::teams::{self, TeamInput, TeamRecord};
use crate::totp_store::{self, TotpBackupRecord, TotpEntryInput};
use crate::tunnels::{self, TunnelRecord};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultBackupPayload {
    pub version: String,
    pub timestamp: u64,
    #[serde(default)]
    pub hosts: Vec<HostRecord>,
    #[serde(default)]
    pub groups: Vec<GroupRecord>,
    #[serde(default)]
    pub teams: Vec<TeamRecord>,
    #[serde(default)]
    pub snippets: Vec<SnippetRecord>,
    #[serde(default)]
    pub keys: Vec<KeyRecord>,
    #[serde(default)]
    pub tunnels: Vec<TunnelRecord>,
    #[serde(default)]
    pub tasks: Vec<ScheduledTaskRecord>,
    #[serde(default)]
    pub investigations: Vec<InvestigationRecord>,
    #[serde(default)]
    pub totp: Vec<TotpBackupRecord>,
    #[serde(default)]
    pub ai_settings: Option<AiSettings>,
}

pub fn build_backup_payload() -> Result<VaultBackupPayload, CatermError> {
    let hosts = store::list_hosts().unwrap_or_default();
    let groups = groups::list_groups().unwrap_or_default();
    let teams = teams::list_teams().unwrap_or_default();
    let snippets = snippets::list_snippets().unwrap_or_default();
    let keys = keys::list_keys().unwrap_or_default();
    let tunnels = tunnels::list_tunnels().unwrap_or_default();
    let tasks = scheduler_store::list_tasks().unwrap_or_default();
    let investigations = investigations::list_investigations().unwrap_or_default();
    let totp = totp_store::list_totp_for_backup().unwrap_or_default();
    let ai_settings = ai::get_ai_settings().ok();

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    Ok(VaultBackupPayload {
        version: "2.1.18".to_string(),
        timestamp: now,
        hosts,
        groups,
        teams,
        snippets,
        keys,
        tunnels,
        tasks,
        investigations,
        totp,
        ai_settings,
    })
}

pub fn apply_backup_payload(payload: VaultBackupPayload) -> Result<usize, CatermError> {
    let mut imported_count = 0;

    for host in payload.hosts {
        store::save_host(HostInput {
            id: Some(host.id),
            label: host.label,
            address: host.address,
            port: host.port,
            username: host.username,
            auth_method: host.auth_method,
            tags: host.tags,
            os: host.os,
            protocol: Some(host.protocol),
            secret: None,
        })?;
        imported_count += 1;
    }

    for group in payload.groups {
        groups::save_group(GroupInput {
            id: Some(group.id),
            name: group.name,
            color: group.color,
            host_ids: group.host_ids,
            categories: Some(group.categories),
        })?;
        imported_count += 1;
    }

    for team in payload.teams {
        let _ = teams::save_team(TeamInput {
            id: Some(team.id),
            name: team.name,
            color: team.color,
            avatar: team.avatar,
            members: team.members,
            host_ids: team.host_ids,
            group_ids: team.group_ids,
        });
        imported_count += 1;
    }

    for snippet in payload.snippets {
        snippets::save_snippet(SnippetInput {
            id: Some(snippet.id),
            label: snippet.label,
            description: snippet.description,
            command: snippet.command,
            tags: snippet.tags,
        })?;
        imported_count += 1;
    }

    for key in payload.keys {
        let _ = keys::import_key(&key.name, &key.public_key, None);
        imported_count += 1;
    }

    for tunnel in payload.tunnels {
        let _ = tunnels::save_tunnel_record(&tunnel);
        imported_count += 1;
    }

    for task in payload.tasks {
        let _ = scheduler_store::save_task(TaskInput {
            id: Some(task.id),
            name: task.name,
            description: task.description,
            task_type: task.task_type,
            host_id: task.host_id,
            schedule_expr: task.schedule_expr,
            is_enabled: task.is_enabled,
            command_script: task.command_script,
            timeout_seconds: task.timeout_seconds,
            remote_src_path: task.remote_src_path,
            local_dest_dir: task.local_dest_dir,
            remote_pre_cmd: task.remote_pre_cmd,
            remote_post_cmd: task.remote_post_cmd,
            retention_count: task.retention_count,
            notify_on_success: task.notify_on_success,
            notify_on_failure: task.notify_on_failure,
        });
        imported_count += 1;
    }

    for inv in payload.investigations {
        let _ = investigations::save_investigation(InvestigationInput {
            id: Some(inv.id),
            title: inv.title,
            host_id: inv.host_id,
            status: inv.status,
            notes: inv.notes,
            evidence: inv.evidence,
        });
        imported_count += 1;
    }

    for item in payload.totp {
        let _ = totp_store::save_totp_entry(TotpEntryInput {
            id: Some(item.id),
            label: item.label,
            issuer: item.issuer,
            secret: Some(item.secret),
        });
        imported_count += 1;
    }

    if let Some(settings) = payload.ai_settings {
        let _ = ai::save_ai_settings(settings);
    }

    Ok(imported_count)
}

pub fn export_encrypted_backup(passphrase: &str) -> Result<String, CatermError> {
    if passphrase.len() < crate::vault::MIN_PASSWORD_LEN {
        return Err(CatermError::Vault(VaultError::Generic(format!(
            "Backup passphrase must be at least {} characters",
            crate::vault::MIN_PASSWORD_LEN
        ))));
    }

    let payload = build_backup_payload()?;

    let json_bytes = serde_json::to_vec(&payload)
        .map_err(|e| CatermError::Vault(VaultError::Generic(e.to_string())))?;

    // Encrypt with passphrase derived key using Argon2id + AES-256-GCM
    let mut derived_key = [0u8; 32];
    let params = argon2::Params::new(64 * 1024, 3, 4, Some(32))
        .map_err(|e| CatermError::Vault(VaultError::Generic(e.to_string())))?;
    let argon2 = argon2::Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
    let salt = b"caterm.vault.backup.salt.2026";
    argon2
        .hash_password_into(passphrase.as_bytes(), salt, &mut derived_key)
        .map_err(|e| CatermError::Vault(VaultError::Generic(e.to_string())))?;

    let encrypted_b64 = crate::secret::encrypt_bytes(&derived_key, &json_bytes)?;
    Ok(encrypted_b64)
}

pub fn export_encrypted_backup_with_dek(dek: &[u8; 32]) -> Result<String, CatermError> {
    let payload = build_backup_payload()?;

    let json_bytes = serde_json::to_vec(&payload)
        .map_err(|e| CatermError::Vault(VaultError::Generic(e.to_string())))?;

    crate::secret::encrypt_bytes(dek, &json_bytes)
}

pub fn import_encrypted_backup_with_dek(
    encrypted_b64: &str,
    dek: &[u8; 32],
) -> Result<usize, CatermError> {
    let decrypted_bytes = crate::secret::decrypt_bytes(dek, encrypted_b64)?;
    let payload: VaultBackupPayload = serde_json::from_slice(&decrypted_bytes).map_err(|e| {
        CatermError::Vault(VaultError::Generic(format!("Corrupt backup file: {}", e)))
    })?;

    apply_backup_payload(payload)
}

pub fn import_encrypted_backup(
    encrypted_b64: &str,
    passphrase: &str,
) -> Result<usize, CatermError> {
    let mut derived_key = [0u8; 32];
    let params = argon2::Params::new(64 * 1024, 3, 4, Some(32))
        .map_err(|e| CatermError::Vault(VaultError::Generic(e.to_string())))?;
    let argon2 = argon2::Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
    let salt = b"caterm.vault.backup.salt.2026";
    argon2
        .hash_password_into(passphrase.as_bytes(), salt, &mut derived_key)
        .map_err(|e| CatermError::Vault(VaultError::Generic(e.to_string())))?;

    let decrypted_bytes = crate::secret::decrypt_bytes(&derived_key, encrypted_b64)?;
    let payload: VaultBackupPayload = serde_json::from_slice(&decrypted_bytes).map_err(|e| {
        CatermError::Vault(VaultError::Generic(format!("Corrupt backup file: {}", e)))
    })?;

    apply_backup_payload(payload)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backup_roundtrip() {
        // We can test this by mocking the store, but here we can at least test that
        // encryption/decryption works and doesn't leak plaintext.
        let passphrase = "correct-horse-battery";
        let payload = VaultBackupPayload {
            version: "2.0.10".to_string(),
            timestamp: 1234567890,
            hosts: vec![HostRecord {
                id: "h1".into(),
                label: "Test".into(),
                address: "127.0.0.1".into(),
                port: 22,
                username: "root".into(),
                auth_method: crate::store::AuthMethod::Password,
                tags: vec![],
                os: None,
                protocol: crate::store::ConnectionProtocol::Ssh,
                created_at: 1,
                updated_at: 1,
                has_secret: false,
            }],
            groups: vec![],
            teams: vec![],
            snippets: vec![],
            keys: vec![],
            tunnels: vec![],
            tasks: vec![],
            investigations: vec![],
            totp: vec![],
            ai_settings: None,
        };
        let json_bytes = serde_json::to_vec(&payload).unwrap();

        // Encrypt with passphrase derived key using Argon2id + AES-256-GCM
        let mut derived_key = [0u8; 32];
        let params = argon2::Params::new(64 * 1024, 3, 4, Some(32)).unwrap();
        let argon2 =
            argon2::Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
        let salt = b"caterm.vault.backup.salt.2026";
        argon2
            .hash_password_into(passphrase.as_bytes(), salt, &mut derived_key)
            .unwrap();

        let encrypted_b64 = crate::secret::encrypt_bytes(&derived_key, &json_bytes).unwrap();

        // Ensure no plaintext leakage
        assert!(!encrypted_b64.contains("Test"));
        assert!(!encrypted_b64.contains("127.0.0.1"));

        // Decrypt
        let decrypted_bytes = crate::secret::decrypt_bytes(&derived_key, &encrypted_b64).unwrap();
        let decrypted_payload: VaultBackupPayload =
            serde_json::from_slice(&decrypted_bytes).unwrap();

        assert_eq!(decrypted_payload.hosts[0].label, "Test");
    }
}
