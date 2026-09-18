//! Master vault password policy, and the local key that currently backs the SQLCipher
//! database (T9). A full password-derived-key unlock flow (Fase 1) still needs its own
//! session/unlock screen — until that lands, `load_or_create_local_key` below generates and
//! remembers the actual SQLCipher passphrase protecting `caterm.db` at rest, independently
//! of whatever the user sets as their master vault password in Settings.

use crate::error::{CatermError, IoError, VaultError};
use rand::RngCore;
use std::path::{Path, PathBuf};

/// REQ-9b: minimum length enforced for the vault master password.
pub const MIN_PASSWORD_LEN: usize = 8;

/// Rejects a master vault password shorter than [`MIN_PASSWORD_LEN`].
pub fn validate_master_password(password: &str) -> Result<(), CatermError> {
    if password.chars().count() < MIN_PASSWORD_LEN {
        return Err(CatermError::Vault(VaultError::Generic(format!(
            "password vault minimal {MIN_PASSWORD_LEN} karakter"
        ))));
    }
    Ok(())
}

fn key_file_path_in(data_dir: &Path) -> PathBuf {
    data_dir.join("vault.key")
}

/// Loads the local SQLCipher passphrase from `<data_dir>/vault.key`, generating a random
/// 256-bit one (hex-encoded) on first run so the encrypted database always has a real key
/// even before a password-based unlock flow exists.
pub fn load_or_create_local_key(data_dir: &Path) -> Result<String, CatermError> {
    let path = key_file_path_in(data_dir);
    if let Ok(existing) = std::fs::read_to_string(&path) {
        let trimmed = existing.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
    }

    std::fs::create_dir_all(data_dir)
        .map_err(|e| CatermError::Io(IoError::Generic(format!("gagal membuat data dir: {e}"))))?;

    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    let key: String = bytes.iter().map(|b| format!("{b:02x}")).collect();

    std::fs::write(&path, &key)
        .map_err(|e| CatermError::Io(IoError::Generic(format!("gagal menulis vault.key: {e}"))))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(&path) {
            let mut perms = meta.permissions();
            perms.set_mode(0o600);
            let _ = std::fs::set_permissions(&path, perms);
        }
    }

    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TempDataDir(PathBuf);

    impl TempDataDir {
        fn new(label: &str) -> Self {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("waktu sistem sebelum epoch")
                .as_nanos();
            Self(std::env::temp_dir().join(format!("caterm_vault_test_{label}_{nanos:x}")))
        }
    }

    impl Drop for TempDataDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn rejects_short_password() {
        assert!(validate_master_password("short1").is_err());
    }

    #[test]
    fn accepts_password_at_minimum_length() {
        assert!(validate_master_password("12345678").is_ok());
    }

    #[test]
    fn local_key_is_stable_across_calls() {
        let dir = TempDataDir::new("stable");
        let first = load_or_create_local_key(&dir.0).expect("gagal generate key");
        let second = load_or_create_local_key(&dir.0).expect("gagal baca key");
        assert_eq!(first, second);
        assert_eq!(first.len(), 64); // 32 bytes hex-encoded
    }
}
