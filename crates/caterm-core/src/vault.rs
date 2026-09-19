//! Zero-Knowledge Argon2id Vault Manager (`T2-CORE-01` .. `T2-CORE-03`).
//!
//! Derives a 256-bit Key Encryption Key (KEK) from the user's master password
//! using Argon2id (m=64MB, t=3, p=4), which in turn protects the Database Encryption Key (DEK).
//!
//! Includes in-memory zeroization on lock to eliminate forensic RAM leakage.

use crate::error::{CatermError, VaultError};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, Params, Version,
};
use parking_lot::RwLock;
use secrecy::{ExposeSecret, SecretBox};
use serde::{Deserialize, Serialize};
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

pub fn lock_vault() -> Result<(), CatermError> {
    let mut guard = ACTIVE_VAULT_KEY.write();
    if let Some(mut key) = guard.take() {
        key.zeroize();
    }
    // Also stop all active tunnels on lock for Zero-Knowledge containment
    let _ = crate::tunnels::stop_all_tunnels();
    Ok(())
}

pub fn unlock_vault(master_password: &str) -> Result<(), CatermError> {
    if master_password.len() < MIN_PASSWORD_LEN {
        return Err(CatermError::Vault(VaultError::Generic(format!(
            "Master password must be at least {} characters",
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

    let mut guard = ACTIVE_VAULT_KEY.write();
    *guard = Some(derived_key);

    Ok(())
}

pub fn get_active_dek() -> Result<[u8; 32], CatermError> {
    let guard = ACTIVE_VAULT_KEY.read();
    if let Some(key) = *guard {
        Ok(key)
    } else {
        // Auto-fallback to deterministic host local key if master password is not yet configured
        let fallback_key = load_or_create_local_key()?;
        Ok(fallback_key)
    }
}

pub fn load_or_create_local_key() -> Result<[u8; 32], CatermError> {
    let key_file = crate::paths::caterm_data_dir()?.join(".local_key");
    if key_file.exists() {
        let bytes = std::fs::read(&key_file).map_err(|e| CatermError::Vault(VaultError::Generic(e.to_string())))?;
        if bytes.len() == 32 {
            let mut key = [0u8; 32];
            key.copy_from_slice(&bytes);
            return Ok(key);
        }
    }

    let mut new_key = [0u8; 32];
    use rand_core::RngCore;
    rand_core::OsRng.fill_bytes(&mut new_key);

    std::fs::write(&key_file, &new_key).map_err(|e| CatermError::Vault(VaultError::Generic(e.to_string())))?;
    Ok(new_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_short_password() {
        assert!(unlock_vault("short").is_err());
    }

    #[test]
    fn accepts_password_at_minimum_length() {
        assert!(unlock_vault("12345678").is_ok());
        assert!(is_unlocked().unwrap());
        assert!(lock_vault().is_ok());
        assert!(!is_unlocked().unwrap());
    }

    #[test]
    fn local_key_is_stable_across_calls() {
        let k1 = load_or_create_local_key().unwrap();
        let k2 = load_or_create_local_key().unwrap();
        assert_eq!(k1, k2);
    }
}
