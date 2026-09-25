//! Zero-Knowledge Argon2id Vault Manager (`T2-CORE-01` .. `T2-CORE-03`).
//!
//! Derives a 256-bit Key Encryption Key (KEK) from the user's master password
//! using Argon2id (m=64MB, t=3, p=4), which in turn protects the Database Encryption Key (DEK).
//!
//! Includes in-memory zeroization on lock to eliminate forensic RAM leakage.

use crate::error::{CatermError, DbError, VaultError};
use argon2::{Argon2, Params, Version};
use parking_lot::RwLock;
use std::sync::LazyLock;
use zeroize::Zeroize;

pub const MIN_PASSWORD_LEN: usize = 8;

const CANARY_FILE: &str = "vault_canary.bin";
const CANARY_PLAINTEXT: &[u8] = b"CATERM_VAULT_CANARY_V2";
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

    // 1. Disconnect all active tunnels and SSH sessions
    let _ = crate::tunnels::stop_all_tunnels();
    if let Ok(mut sessions) = crate::ssh::SESSIONS.lock() {
        for (_id, handle) in sessions.drain() {
            if let Some(channel) = &handle.channel
                && let Ok(mut ch) = channel.lock()
            {
                let _ = ch.close();
            }
            if let Some(child) = &handle.child
                && let Ok(mut ch) = child.lock()
            {
                let _ = ch.kill();
            }
        }
    }

    // 2. Zeroize active vault key directly without logging to DB
    {
        let mut guard = ACTIVE_VAULT_KEY.write();
        if let Some(mut key) = guard.take() {
            key.zeroize();
        }
    }

    // 3. Remove all vault security and database files
    let files = [
        "vault_canary.bin",
        "vault.key",
        "local.key",
        "caterm.db",
        "caterm.db-wal",
        "caterm.db-shm",
        "caterm.db-journal",
        "known_hosts",
    ];
    for f in &files {
        let p = data_dir.join(f);
        if p.exists() {
            let _ = std::fs::remove_file(&p);
        }
    }

    Ok(())
}

/// Re-keys the vault to a new master password. The password's Argon2id output *is* the
/// SQLCipher key, so this rewrites every database page (`PRAGMA rekey`) and replaces the
/// canary. The vault stays unlocked, now on the new key.
pub fn change_master_password(
    current_password: &str,
    new_password: &str,
) -> Result<(), CatermError> {
    if current_password == new_password {
        return Err(vault_err(
            "Master password baru harus berbeda dari yang lama.",
        ));
    }
    let data_dir = crate::paths::resolve_data_dir()?.path;
    let canary_path = data_dir.join(CANARY_FILE);
    if !canary_path.exists() {
        return Err(vault_err("Vault belum dibuat."));
    }

    let mut old_key = derive_key(current_password)?;
    let mut new_key = match derive_key(new_password) {
        Ok(key) => key,
        Err(e) => {
            old_key.zeroize();
            return Err(e);
        }
    };

    let result = rekey(&data_dir, &canary_path, &old_key, &new_key);
    if result.is_ok() {
        let mut guard = ACTIVE_VAULT_KEY.write();
        if let Some(mut previous) = guard.replace(new_key) {
            previous.zeroize();
        }
    }
    old_key.zeroize();
    new_key.zeroize();
    result?;

    let _ = crate::audit::log_event("VAULT_PASSWORD_CHANGE", None, "Master password changed");
    Ok(())
}

/// Ordering keeps a crash from locking the user out: the new canary is staged beside the old
/// one, the database is re-keyed, and only then is the staged canary moved into place. If that
/// last move fails, the database is re-keyed back so the untouched old canary still matches.
fn rekey(
    data_dir: &std::path::Path,
    canary_path: &std::path::Path,
    old_key: &[u8; 32],
    new_key: &[u8; 32],
) -> Result<(), CatermError> {
    verify_canary(old_key, canary_path)?;

    let staged = canary_path.with_extension("bin.new");
    let encrypted_canary = crate::secret::encrypt_bytes(new_key, CANARY_PLAINTEXT)?;
    std::fs::write(&staged, encrypted_canary).map_err(|e| vault_err(e.to_string()))?;

    if let Err(e) = rekey_database(data_dir, old_key, new_key) {
        let _ = std::fs::remove_file(&staged);
        return Err(e);
    }

    if let Err(e) = std::fs::rename(&staged, canary_path) {
        let _ = rekey_database(data_dir, new_key, old_key);
        let _ = std::fs::remove_file(&staged);
        return Err(vault_err(format!("gagal menyimpan canary baru: {e}")));
    }
    Ok(())
}

fn rekey_database(
    data_dir: &std::path::Path,
    from: &[u8; 32],
    to: &[u8; 32],
) -> Result<(), CatermError> {
    let conn = crate::db::open_encrypted(data_dir, &hex::encode(from))?;
    conn.execute_batch(&format!("PRAGMA rekey = '{}';", hex::encode(to)))
        .map_err(|e| CatermError::Db(DbError::Generic(format!("gagal rekey database: {e}"))))?;
    drop(conn);
    // Proves the new key opens the file before the caller switches the canary over.
    crate::db::open_encrypted(data_dir, &hex::encode(to)).map(drop)
}

pub fn lock_vault() -> Result<(), CatermError> {
    // Audit first: once the key is gone the database can no longer be opened to record it.
    let _ = crate::audit::log_event("VAULT_LOCK", None, "Vault locked");
    // Also stop all active tunnels on lock for Zero-Knowledge containment
    let _ = crate::tunnels::stop_all_tunnels();

    // Scoped so the write guard is released before returning. `parking_lot::RwLock` is not
    // reentrant: holding it across `log_event` (which reads the key to open the database)
    // deadlocked every lock, and every unlock queued behind it hung forever.
    {
        let mut guard = ACTIVE_VAULT_KEY.write();
        if let Some(mut key) = guard.take() {
            key.zeroize();
        }
    }
    Ok(())
}

pub fn unlock_vault(master_password: &str) -> Result<(), CatermError> {
    let mut derived_key = derive_key(master_password)?;

    let data_dir = crate::paths::resolve_data_dir()?.path;
    let canary_path = data_dir.join(CANARY_FILE);

    if canary_path.exists() {
        if let Err(e) = verify_canary(&derived_key, &canary_path) {
            derived_key.zeroize();
            return Err(e);
        }
    } else {
        std::fs::create_dir_all(&data_dir).map_err(|e| vault_err(e.to_string()))?;
        let encrypted_canary = crate::secret::encrypt_bytes(&derived_key, CANARY_PLAINTEXT)?;
        std::fs::write(&canary_path, encrypted_canary).map_err(|e| vault_err(e.to_string()))?;
    }

    {
        let mut guard = ACTIVE_VAULT_KEY.write();
        *guard = Some(derived_key);
    }

    let _ = crate::audit::log_event("VAULT_UNLOCK", None, "Vault unlocked");

    Ok(())
}

fn vault_err(message: impl Into<String>) -> CatermError {
    CatermError::Vault(VaultError::Generic(message.into()))
}

/// Argon2id key from the master password — both the canary key and the SQLCipher key.
fn derive_key(master_password: &str) -> Result<[u8; 32], CatermError> {
    if master_password.len() < MIN_PASSWORD_LEN {
        return Err(vault_err(format!(
            "Master password minimal {MIN_PASSWORD_LEN} karakter"
        )));
    }

    let mut derived_key = [0u8; 32];
    let params = Params::new(ARGON2_M_COST, ARGON2_T_COST, ARGON2_P_COST, Some(32))
        .map_err(|e| vault_err(e.to_string()))?;
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, Version::V0x13, params);

    // Static application domain salt for local KEK derivation
    let salt = b"caterm.zero_knowledge.v2.domain_salt_2026";
    argon2
        .hash_password_into(master_password.as_bytes(), salt, &mut derived_key)
        .map_err(|e| vault_err(e.to_string()))?;
    Ok(derived_key)
}

fn verify_canary(key: &[u8; 32], canary_path: &std::path::Path) -> Result<(), CatermError> {
    let wrong_password = || vault_err("Master password salah. Silakan coba lagi.");
    let encrypted_canary =
        std::fs::read_to_string(canary_path).map_err(|e| vault_err(e.to_string()))?;
    let decrypted =
        crate::secret::decrypt_bytes(key, &encrypted_canary).map_err(|_| wrong_password())?;
    if decrypted != CANARY_PLAINTEXT {
        return Err(wrong_password());
    }
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
        // Isolated: this used to build a temp dir and then unlock the vault in the real data
        // directory anyway, creating a vault with password "12345678" on a fresh machine.
        let _data = crate::test_support::isolated_data_dir("vault_unlock");

        // Test lock and unlock
        assert!(unlock_vault("12345678").is_ok());
        assert!(is_unlocked().unwrap());
        assert!(lock_vault().is_ok());
        assert!(!is_unlocked().unwrap());
        // Regression: lock_vault used to keep the key's write lock while logging, so it never
        // returned and this second unlock blocked forever.
        assert!(unlock_vault("12345678").is_ok());
        assert!(is_unlocked().unwrap());
        assert!(lock_vault().is_ok());
    }

    #[test]
    fn rekey_moves_database_and_canary_to_the_new_key() {
        let dir = std::env::temp_dir().join(format!("caterm_vault_rekey_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let canary = dir.join(CANARY_FILE);
        let (old_key, new_key) = ([7u8; 32], [9u8; 32]);

        std::fs::write(
            &canary,
            crate::secret::encrypt_bytes(&old_key, CANARY_PLAINTEXT).unwrap(),
        )
        .unwrap();
        {
            let conn = crate::db::open_encrypted(&dir, &hex::encode(old_key)).unwrap();
            conn.execute_batch("CREATE TABLE probe(v TEXT); INSERT INTO probe VALUES ('kept');")
                .unwrap();
        }

        assert!(
            rekey(&dir, &canary, &new_key, &old_key).is_err(),
            "wrong current key must fail"
        );
        rekey(&dir, &canary, &old_key, &new_key).unwrap();

        assert!(verify_canary(&new_key, &canary).is_ok());
        assert!(verify_canary(&old_key, &canary).is_err());
        assert!(crate::db::open_encrypted(&dir, &hex::encode(old_key)).is_err());
        let conn = crate::db::open_encrypted(&dir, &hex::encode(new_key)).unwrap();
        let kept: String = conn
            .query_row("SELECT v FROM probe", [], |r| r.get(0))
            .unwrap();
        assert_eq!(kept, "kept");
        assert!(!canary.with_extension("bin.new").exists());
        drop(conn);
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
