//! Managed SSH Keys module (`T2-TOOL-05`).
//! Provides key generation (Ed25519, RSA-4096), import (OpenSSH, PKCS#8),
//! fingerprinting (SHA-256), and safe storage inside the encrypted SQLite vault.

use crate::error::{CatermError, ValidationError};
use serde::{Deserialize, Serialize};
use ssh_key::{Algorithm, HashAlg, LineEnding, PrivateKey};
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyRecord {
    pub id: String,
    pub name: String,
    pub algorithm: String,
    pub fingerprint: String,
    pub public_key: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyInput {
    pub name: String,
    pub algorithm: String, // "Ed25519" or "RSA-4096"
}

fn require_non_empty(field: &str, value: &str) -> Result<(), CatermError> {
    if value.trim().is_empty() {
        return Err(CatermError::Validation(ValidationError::Generic(format!(
            "{field} tidak boleh kosong"
        ))));
    }
    Ok(())
}

fn generate_id() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("key-{nanos:x}")
}

/// Create the `ssh_keys` database table if it doesn't already exist.
pub fn init_table(conn: &rusqlite::Connection) -> Result<(), CatermError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS ssh_keys (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            algorithm TEXT NOT NULL,
            fingerprint TEXT NOT NULL,
            public_key TEXT NOT NULL,
            secret_enc BLOB NOT NULL,
            created_at TEXT NOT NULL
        );",
    )
    .map_err(|e| {
        CatermError::Validation(ValidationError::Generic(format!(
            "Failed to create ssh_keys table: {e}"
        )))
    })?;
    Ok(())
}

/// List all stored public keys.
pub fn list_keys() -> Result<Vec<KeyRecord>, CatermError> {
    let data_info = crate::paths::resolve_data_dir()?;
    let key = crate::vault::load_or_create_local_key(&data_info.path)?;
    let conn = crate::db::open()?;
    init_table(&conn)?;

    let mut stmt = conn
        .prepare("SELECT id, name, algorithm, fingerprint, public_key, created_at FROM ssh_keys ORDER BY name ASC")
        .map_err(|e| CatermError::Validation(ValidationError::Generic(e.to_string())))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(KeyRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                algorithm: row.get(2)?,
                fingerprint: row.get(3)?,
                public_key: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .map_err(|e| CatermError::Validation(ValidationError::Generic(e.to_string())))?;

    let mut result = Vec::new();
    for r in rows {
        result
            .push(r.map_err(|e| CatermError::Validation(ValidationError::Generic(e.to_string())))?);
    }
    Ok(result)
}

/// Generate a new Ed25519 or RSA-4096 SSH key pair.
pub fn generate_key(input: KeyInput) -> Result<KeyRecord, CatermError> {
    require_non_empty("name", &input.name)?;

    let mut rng = rand::thread_rng();
    let priv_key = match input.algorithm.as_str() {
        "Ed25519" => PrivateKey::random(&mut rng, Algorithm::Ed25519).map_err(|e| {
            CatermError::Validation(ValidationError::Generic(format!(
                "Failed to generate Ed25519 key: {e}"
            )))
        })?,
        "RSA-4096" | "RSA" => {
            PrivateKey::random(&mut rng, Algorithm::Rsa { hash: None }).map_err(|e| {
                CatermError::Validation(ValidationError::Generic(format!(
                    "Failed to generate RSA key: {e}"
                )))
            })?
        }
        other => {
            return Err(CatermError::Validation(ValidationError::Generic(format!(
                "Algoritma '{other}' tidak didukung. Gunakan 'Ed25519' atau 'RSA-4096'."
            ))));
        }
    };

    let fingerprint = priv_key.fingerprint(HashAlg::Sha256).to_string();
    let public_key = priv_key.public_key().to_openssh().map_err(|e| {
        CatermError::Validation(ValidationError::Generic(format!(
            "Failed to encode public key: {e}"
        )))
    })?;

    let private_key_pem = priv_key.to_openssh(LineEnding::LF).map_err(|e| {
        CatermError::Validation(ValidationError::Generic(format!(
            "Failed to encode private key: {e}"
        )))
    })?;

    let data_info = crate::paths::resolve_data_dir()?;
    let key = crate::vault::load_or_create_local_key(&data_info.path)?;
    let encrypted_secret = crate::secret::encrypt(&key, private_key_pem.as_str())?;

    let id = generate_id();
    let created_at = chrono::Utc::now().to_rfc3339();

    let conn = crate::db::open()?;
    init_table(&conn)?;

    conn.execute(
        "INSERT INTO ssh_keys (id, name, algorithm, fingerprint, public_key, secret_enc, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            id,
            input.name,
            input.algorithm,
            fingerprint,
            public_key,
            encrypted_secret,
            created_at
        ],
    )
    .map_err(|e| CatermError::Validation(ValidationError::Generic(format!("Failed to save key: {e}"))))?;

    Ok(KeyRecord {
        id,
        name: input.name,
        algorithm: input.algorithm,
        fingerprint,
        public_key,
        created_at,
    })
}

/// Import an existing OpenSSH or PKCS#8 private key.
pub fn import_key(
    name: &str,
    private_key_pem: &str,
    passphrase: Option<&str>,
) -> Result<KeyRecord, CatermError> {
    require_non_empty("name", name)?;
    require_non_empty("private_key_pem", private_key_pem)?;

    let parsed_key = if let Some(pass) = passphrase {
        PrivateKey::from_openssh(private_key_pem)
            .and_then(|k| k.decrypt(pass))
            .map_err(|e| {
                CatermError::Validation(ValidationError::Generic(format!(
                    "Gagal mendeskripsi private key dengan passphrase: {e}"
                )))
            })?
    } else {
        PrivateKey::from_openssh(private_key_pem).map_err(|e| {
            CatermError::Validation(ValidationError::Generic(format!(
                "Invalid OpenSSH private key: {e}"
            )))
        })?
    };

    let algorithm = match parsed_key.algorithm() {
        Algorithm::Ed25519 => "Ed25519".to_string(),
        Algorithm::Rsa { .. } => "RSA".to_string(),
        other => format!("{other:?}"),
    };

    let fingerprint = parsed_key.fingerprint(HashAlg::Sha256).to_string();
    let public_key = parsed_key.public_key().to_openssh().map_err(|e| {
        CatermError::Validation(ValidationError::Generic(format!(
            "Failed to get public key: {e}"
        )))
    })?;

    let clean_pem = parsed_key.to_openssh(LineEnding::LF).map_err(|e| {
        CatermError::Validation(ValidationError::Generic(format!(
            "Failed to encode key: {e}"
        )))
    })?;

    let data_info = crate::paths::resolve_data_dir()?;
    let key = crate::vault::load_or_create_local_key(&data_info.path)?;
    let encrypted_secret = crate::secret::encrypt(&key, clean_pem.as_str())?;

    let id = generate_id();
    let created_at = chrono::Utc::now().to_rfc3339();

    let conn = crate::db::open()?;
    init_table(&conn)?;

    conn.execute(
        "INSERT INTO ssh_keys (id, name, algorithm, fingerprint, public_key, secret_enc, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            id,
            name,
            algorithm,
            fingerprint,
            public_key,
            encrypted_secret,
            created_at
        ],
    )
    .map_err(|e| CatermError::Validation(ValidationError::Generic(format!("Failed to save key: {e}"))))?;

    Ok(KeyRecord {
        id,
        name: name.to_string(),
        algorithm,
        fingerprint,
        public_key,
        created_at,
    })
}

/// Delete an SSH key, ensuring it is not currently assigned to any saved Host.
pub fn delete_key(id: &str) -> Result<(), CatermError> {
    require_non_empty("id", id)?;

    // Check if key is in use by any Host
    let hosts = crate::store::list_hosts()?;
    for h in hosts {
        if let crate::store::AuthMethod::KeyId { id: ref k_id } = h.auth_method
            && k_id == id
        {
            return Err(CatermError::Validation(ValidationError::Generic(format!(
                "Gagal menghapus: Key ini sedang digunakan oleh Host '{}'. Ubah auth method host terlebih dahulu.",
                h.label
            ))));
        }
    }

    let conn = crate::db::open()?;
    init_table(&conn)?;

    let affected = conn
        .execute("DELETE FROM ssh_keys WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| CatermError::Validation(ValidationError::Generic(e.to_string())))?;

    if affected == 0 {
        return Err(CatermError::Validation(ValidationError::Generic(format!(
            "Key with id '{id}' not found"
        ))));
    }

    Ok(())
}

/// Retrieve internal decrypted private key PEM for SSH authentication.
pub fn get_private_key(id: &str) -> Result<String, CatermError> {
    require_non_empty("id", id)?;

    let data_info = crate::paths::resolve_data_dir()?;
    let key = crate::vault::load_or_create_local_key(&data_info.path)?;
    let conn = crate::db::open()?;
    init_table(&conn)?;

    let mut stmt = conn
        .prepare("SELECT secret_enc FROM ssh_keys WHERE id = ?1")
        .map_err(|e| CatermError::Validation(ValidationError::Generic(e.to_string())))?;

    let secret_enc: String = stmt
        .query_row(rusqlite::params![id], |row| row.get(0))
        .map_err(|_| {
            CatermError::Validation(ValidationError::Generic(format!("Key '{id}' not found")))
        })?;

    let decrypted = crate::secret::decrypt(&key, &secret_enc)?;
    Ok(decrypted)
}

/// Deploy a public key from the local vault to a remote host's `~/.ssh/authorized_keys`.
pub fn deploy_public_key(host_id: &str, key_id: &str) -> Result<(), CatermError> {
    require_non_empty("host_id", host_id)?;
    require_non_empty("key_id", key_id)?;

    // Get public key content
    let keys = list_keys()?;
    let target_key = keys.iter().find(|k| k.id == key_id).ok_or_else(|| {
        CatermError::Validation(ValidationError::Generic(format!(
            "Key '{key_id}' not found"
        )))
    })?;

    // Connect to target host
    let (host, secret) = crate::store::load_host_for_connect(host_id)?;
    let port = if host.port == 0 { 22 } else { host.port };
    let addr = format!("{}:{port}", host.address);

    let tcp = std::net::TcpStream::connect_timeout(
        &addr.parse().map_err(|e| {
            CatermError::Validation(ValidationError::Generic(format!(
                "Invalid address {addr}: {e}"
            )))
        })?,
        Duration::from_secs(10),
    )
    .map_err(|e| {
        CatermError::Validation(ValidationError::Generic(format!(
            "Connection failed to {addr}: {e}"
        )))
    })?;

    let mut sess = ssh2::Session::new().map_err(|e| {
        CatermError::Validation(ValidationError::Generic(format!(
            "SSH session creation failed: {e}"
        )))
    })?;

    sess.set_tcp_stream(tcp);
    sess.handshake().map_err(|e| {
        CatermError::Validation(ValidationError::Generic(format!(
            "SSH handshake failed: {e}"
        )))
    })?;

    match &host.auth_method {
        crate::store::AuthMethod::Password => {
            let password = secret.ok_or_else(|| {
                CatermError::Validation(ValidationError::Generic(format!(
                    "Password host '{}' belum diset.",
                    host.label
                )))
            })?;
            sess.userauth_password(&host.username, &password)
                .map_err(|e| {
                    CatermError::Validation(ValidationError::Generic(format!("Auth failed: {e}")))
                })?;
        }
        crate::store::AuthMethod::Key { path } => {
            let expanded = crate::paths::expand_tilde(path);
            sess.userauth_pubkey_file(
                &host.username,
                None,
                Path::new(&expanded),
                secret.as_deref(),
            )
            .map_err(|e| {
                CatermError::Validation(ValidationError::Generic(format!(
                    "Auth via key failed: {e}"
                )))
            })?;
        }
        crate::store::AuthMethod::KeyId { id } => {
            
            let priv_pem = get_private_key(id)?;
            let temp_path = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
            std::fs::write(&temp_path, priv_pem.as_bytes()).unwrap();
            let auth_res = sess.userauth_pubkey_file(&host.username, None, &temp_path, None);
            std::fs::remove_file(&temp_path).ok();
            auth_res.map_err(|e| {
                CatermError::Validation(ValidationError::Generic(format!(
                    "Auth via key id failed: {e}"
                )))
            })?;

        }
    }

    if !sess.authenticated() {
        return Err(CatermError::Validation(ValidationError::Generic(
            "Authentication failed".into(),
        )));
    }

    // Deploy public key idempotently & fix permissions
    let mut channel = sess.channel_session().map_err(|e| {
        CatermError::Validation(ValidationError::Generic(format!(
            "Channel creation failed: {e}"
        )))
    })?;

    let pub_key_line = target_key.public_key.trim();
    let cmd = format!(
        "mkdir -p ~/.ssh && chmod 700 ~/.ssh && touch ~/.ssh/authorized_keys && chmod 600 ~/.ssh/authorized_keys && \
         grep -qF '{pub_key_line}' ~/.ssh/authorized_keys || echo '{pub_key_line}' >> ~/.ssh/authorized_keys"
    );

    channel.exec(&cmd).map_err(|e| {
        CatermError::Validation(ValidationError::Generic(format!(
            "Exec deploy key failed: {e}"
        )))
    })?;

    channel.wait_close().ok();

    let _ = crate::audit::log_event(
        "KEY_DEPLOY",
        Some(&host.id),
        &format!("Deployed key '{}' to host {}", target_key.name, host.label),
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_and_list_ed25519_key() {
        let dir = std::env::temp_dir().join(format!("caterm_keys_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        unsafe { std::env::set_var("CATERM_DATA_DIR", &dir); }
crate::vault::unlock_vault("12345678").unwrap();

        
        
                let input = KeyInput {
            name: "Test Key".to_string(),
            algorithm: "Ed25519".to_string(),
        };
        let key = generate_key(input).expect("Key generation failed");
        assert!(key.fingerprint.starts_with("SHA256:"));
        assert!(key.public_key.starts_with("ssh-ed25519"));

        let keys = list_keys().expect("List keys failed");
        assert!(keys.iter().any(|k| k.id == key.id));

        let priv_key = get_private_key(&key.id).expect("Get private key failed");
        assert!(priv_key.contains("BEGIN OPENSSH PRIVATE KEY"));

        delete_key(&key.id).expect("Delete key failed");
    }
}
