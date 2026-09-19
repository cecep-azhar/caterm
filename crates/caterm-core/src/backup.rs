//! Encrypted JSON Vault Backup & Restore (`T2-EXIM-01`).
//! Exports all hosts, groups, snippets, and keys as an encrypted JSON archive.
//! Imports and merges encrypted backup archives with integrity validation.

use crate::db::open_db;
use crate::error::{CatermError, ValidationError, VaultError};
use crate::keys::{self, KeyRecord};
use crate::snippets::{self, Snippet};
use crate::store::{self, Host};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultBackupPayload {
    pub version: String,
    pub timestamp: u64,
    pub hosts: Vec<Host>,
    pub groups: Vec<crate::groups::Group>,
    pub snippets: Vec<Snippet>,
    pub keys: Vec<KeyRecord>,
}

pub fn export_encrypted_backup(passphrase: &str) -> Result<String, CatermError> {
    if passphrase.len() < crate::vault::MIN_PASSWORD_LEN {
        return Err(CatermError::Vault(VaultError::Generic(format!(
            "Backup passphrase must be at least {} characters",
            crate::vault::MIN_PASSWORD_LEN
        ))));
    }

    let hosts = store::list_hosts()?;
    let groups = crate::groups::list_groups()?;
    let snippets = snippets::list_snippets()?;
    let keys = keys::list_keys()?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let payload = VaultBackupPayload {
        version: "2.0.9".to_string(),
        timestamp: now,
        hosts,
        groups,
        snippets,
        keys,
    };

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

    let encrypted_b64 = crate::secret::encrypt_secret(&json_bytes, &derived_key)?;
    Ok(encrypted_b64)
}

pub fn import_encrypted_backup(encrypted_b64: &str, passphrase: &str) -> Result<usize, CatermError> {
    let mut derived_key = [0u8; 32];
    let params = argon2::Params::new(64 * 1024, 3, 4, Some(32))
        .map_err(|e| CatermError::Vault(VaultError::Generic(e.to_string())))?;
    let argon2 = argon2::Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
    let salt = b"caterm.vault.backup.salt.2026";
    argon2
        .hash_password_into(passphrase.as_bytes(), salt, &mut derived_key)
        .map_err(|e| CatermError::Vault(VaultError::Generic(e.to_string())))?;

    let decrypted_bytes = crate::secret::decrypt_secret(encrypted_b64, &derived_key)?;
    let payload: VaultBackupPayload = serde_json::from_slice(&decrypted_bytes)
        .map_err(|e| CatermError::Vault(VaultError::Generic(format!("Corrupt backup file: {}", e))))?;

    let mut imported_count = 0;

    for host in payload.hosts {
        store::save_host(host)?;
        imported_count += 1;
    }

    for group in payload.groups {
        crate::groups::save_group(group)?;
        imported_count += 1;
    }

    for snippet in payload.snippets {
        snippets::save_snippet(snippet)?;
        imported_count += 1;
    }

    for key in payload.keys {
        // save imported keys directly
        keys::import_key(&key.name, &key.public_key, "")?;
        imported_count += 1;
    }

    Ok(imported_count)
}
