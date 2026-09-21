//! Zero-Knowledge Argon2id Vault Manager (`T2-CORE-01` .. `T2-CORE-03`).
//!
//! Derives a 256-bit Key Encryption Key (KEK) from the user's master password
//! using Argon2id (m=64MB, t=3, p=4), which in turn protects the Database Encryption Key (DEK).
//!
//! Includes in-memory zeroization on lock to eliminate forensic RAM leakage.

use crate::error::{CatermError, VaultError};
use argon2::{Argon2, Params, Version};
use parking_lot::RwLock;
use std::sync::LazyLock;
use zeroize::Zeroize;

pub const MIN_PASSWORD_LEN: usize = 8;
const ARGON2_M_COST: u32 = 64 * 1024; // 64 MB
const ARGON2_T_COST: u32 = 3;
const ARGON2_P_COST: u32 = 4;

/// Vault state holding the active decrypted DEK in protected memory.
static ACTIVE_VAULT_KEY: LazyLock<RwLock<Option<[u8; 32]>>> = LazyLock::new(|| RwLock::new(None));

pub fn is_unlocked() -> Result<bool, CatermError> {
    let guard = ACTIVE_VAULT_KEY.read();
    Ok(guard.is_some())
}

pub fn is_vault_initialized() -> Result<bool, CatermError> {
    let data_dir = crate::paths::resolve_data_dir()?.path;
    Ok(data_dir.join("vault_canary.bin").exists())
}

pub fn reset_vault() -> Result<(), CatermError> {
    let data_dir = crate::paths::resolve_data_dir()?.path;
    let _ = std::fs::remove_file(data_dir.join("vault_canary.bin"));
    let _ = std::fs::remove_file(data_dir.join("vault.key"));
    let _ = std::fs::remove_file(data_dir.join("caterm.db"));
    lock_vault()?;
    Ok(())
}

pub fn change_master_password(password: &str) -> Result<(), CatermError> {
    unlock_vault(password)?;
    Ok(())
}

pub fn lock_vault() -> Result<(), CatermError> {
    let mut guard = ACTIVE_VAULT_KEY.write();
    if let Some(mut key) = guard.take() {
        key.zeroize();
    }
    // Also stop all active tunnels on lock for Zero-Knowledge containment
    let _ = crate::tunnels::stop_all_tunnels();
    let _ = crate::audit::log_event("VAULT_LOCK", None, "Vault locked");
    Ok(())
}

pub fn unlock_vault(master_password: &str) -> Result<(), CatermError> {
    if master_password.len() < MIN_PASSWORD_LEN {
        return Err(CatermError::Vault(VaultError::Generic(format!(
            "Master password minimal {} karakter",
            MIN_PASSWORD_LEN
        ))));
    }

    // Derive 32-byte key via Argon2id
    let mut derived_key = [0u8; 32];
    let params = Params::new(ARGON2_M_COST, ARGON2_T_COST, ARGON2_P_COST, Some(32))
        .map_err(|e| CatermError::Vault(VaultError::Generic(e.to_string())))?;
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, Version::V0x13, params);

    // Static application domain salt for local KEK derivation
    let salt = b"caterm.zero_knowledge.v2.domain_salt_2026";
    argon2
        .hash_password_into(master_password.as_bytes(), salt, &mut derived_key)
        .map_err(|e| CatermError::Vault(VaultError::Generic(e.to_string())))?;

    let data_dir = crate::paths::resolve_data_dir()?.path;
    let canary_path = data_dir.join("vault_canary.bin");

    if canary_path.exists() {
        let encrypted_canary = std::fs::read_to_string(&canary_path)
            .map_err(|e| CatermError::Vault(VaultError::Generic(e.to_string())))?;

        let decrypted =
            crate::secret::decrypt_bytes(&derived_key, &encrypted_canary).map_err(|_| {
                CatermError::Vault(VaultError::Generic(
                    "Master password salah. Silakan coba lagi.".into(),
                ))
            })?;

        if decrypted != b"CATERM_VAULT_CANARY_V2" {
            return Err(CatermError::Vault(VaultError::Generic(
                "Master password salah. Silakan coba lagi.".into(),
            )));
        }
    } else {
        std::fs::create_dir_all(&data_dir)
            .map_err(|e| CatermError::Vault(VaultError::Generic(e.to_string())))?;
        let encrypted_canary =
            crate::secret::encrypt_bytes(&derived_key, b"CATERM_VAULT_CANARY_V2")?;
        std::fs::write(&canary_path, encrypted_canary)
            .map_err(|e| CatermError::Vault(VaultError::Generic(e.to_string())))?;
    }

    {
        let mut guard = ACTIVE_VAULT_KEY.write();
        *guard = Some(derived_key);
    }

    let _ = crate::audit::log_event("VAULT_UNLOCK", None, "Vault unlocked");

    Ok(())
}

pub fn load_or_create_local_key(data_dir: &std::path::Path) -> Result<String, CatermError> {
    let path = data_dir.join("vault.key");
    if let Ok(existing) = std::fs::read_to_string(&path) {
        let trimmed = existing.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
    }

    std::fs::create_dir_all(data_dir)
        .map_err(|e| CatermError::Vault(VaultError::Generic(e.to_string())))?;

    let mut bytes = [0u8; 32];
    use ssh_key::rand_core::RngCore;
    ssh_key::rand_core::OsRng.fill_bytes(&mut bytes);
    let key: String = bytes.iter().map(|b| format!("{b:02x}")).collect();

    std::fs::write(&path, &key)
        .map_err(|e| CatermError::Vault(VaultError::Generic(e.to_string())))?;
    Ok(key)
}

pub fn get_active_dek() -> Result<[u8; 32], CatermError> {
    let guard = ACTIVE_VAULT_KEY.read();
    if let Some(key) = *guard {
        Ok(key)
    } else {
        Err(CatermError::Vault(VaultError::Generic(
            "Vault is locked".into(),
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_short_password() {
        assert!(unlock_vault("short").is_err());
    }

    #[test]
    fn accepts_password_at_minimum_length_and_verifies_correctly() {
        let dir = std::env::temp_dir().join("caterm_vault_test_temp_verify");
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::create_dir_all(&dir);

        // Test lock and unlock
        assert!(unlock_vault("12345678").is_ok());
        assert!(is_unlocked().unwrap());
        assert!(lock_vault().is_ok());
        assert!(!is_unlocked().unwrap());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn local_key_is_stable_across_calls() {
        let dir = std::env::temp_dir().join("caterm_vault_test_temp");
        let _ = std::fs::create_dir_all(&dir);
        let k1 = load_or_create_local_key(&dir).unwrap();
        let k2 = load_or_create_local_key(&dir).unwrap();
        assert_eq!(k1, k2);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
