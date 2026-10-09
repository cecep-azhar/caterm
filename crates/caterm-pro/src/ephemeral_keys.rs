use chrono::Utc;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use ssh_key::private::Ed25519Keypair;
use ssh_key::{LineEnding, PrivateKey};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EphemeralKeyRecord {
    pub id: String,
    pub name: String,
    pub public_key: String,
    pub private_key_pem: String,
    pub created_at: i64,
    pub expires_at: i64,
    pub ttl_seconds: u64,
    pub comment_signature: String,
    pub target_host: Option<String>,
    pub auto_revoke: bool,
    pub is_revoked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEphemeralKeyRequest {
    pub name: String,
    pub ttl_seconds: u64,
    pub target_host: Option<String>,
    pub auto_revoke: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EphemeralKeyStatus {
    pub id: String,
    pub name: String,
    pub public_key: String,
    pub remaining_seconds: i64,
    pub is_expired: bool,
    pub is_revoked: bool,
    pub comment_signature: String,
    pub target_host: Option<String>,
    pub created_at_iso: String,
    pub expires_at_iso: String,
}

static EPHEMERAL_KEY_STORE: Lazy<Mutex<HashMap<String, EphemeralKeyRecord>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

fn generate_ephemeral_id() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("eph-{nanos:x}")
}

/// Generates a temporary Ed25519 keypair with specified TTL (in seconds)
pub fn generate_ephemeral_key(
    req: CreateEphemeralKeyRequest,
) -> Result<EphemeralKeyRecord, String> {
    let now = Utc::now().timestamp();
    let ttl = if req.ttl_seconds == 0 {
        3600 * 2 // default 2 hours
    } else {
        req.ttl_seconds
    };
    let expires_at = now + (ttl as i64);
    let id = generate_ephemeral_id();
    let comment_signature = format!("caterm-ephemeral-{id}-{expires_at}");

    let mut rng = OsRng;
    let keypair = Ed25519Keypair::random(&mut rng);
    let private_key = PrivateKey::from(keypair);

    let private_key_pem = private_key
        .to_openssh(LineEnding::LF)
        .map_err(|e| format!("Failed to export private key: {e}"))?
        .to_string();

    let raw_pubkey = private_key
        .public_key()
        .to_openssh()
        .map_err(|e| format!("Failed to export public key: {e}"))?;

    // Append comment signature
    let public_key = format!("{} {}", raw_pubkey.trim(), comment_signature);

    let record = EphemeralKeyRecord {
        id: id.clone(),
        name: req.name,
        public_key,
        private_key_pem,
        created_at: now,
        expires_at,
        ttl_seconds: ttl,
        comment_signature,
        target_host: req.target_host,
        auto_revoke: req.auto_revoke.unwrap_or(true),
        is_revoked: false,
    };

    EPHEMERAL_KEY_STORE.lock().insert(id, record.clone());
    Ok(record)
}

/// Lists all ephemeral keys with calculated remaining lifetimes
pub fn list_ephemeral_keys() -> Vec<EphemeralKeyStatus> {
    let now = Utc::now().timestamp();
    let store = EPHEMERAL_KEY_STORE.lock();

    let mut results: Vec<EphemeralKeyStatus> = store
        .values()
        .map(|k| {
            let remaining = k.expires_at - now;
            let is_expired = remaining <= 0;
            let created_dt =
                chrono::DateTime::from_timestamp(k.created_at, 0).unwrap_or_else(|| Utc::now());
            let expires_dt =
                chrono::DateTime::from_timestamp(k.expires_at, 0).unwrap_or_else(|| Utc::now());

            EphemeralKeyStatus {
                id: k.id.clone(),
                name: k.name.clone(),
                public_key: k.public_key.clone(),
                remaining_seconds: remaining.max(0),
                is_expired,
                is_revoked: k.is_revoked,
                comment_signature: k.comment_signature.clone(),
                target_host: k.target_host.clone(),
                created_at_iso: created_dt.to_rfc3339(),
                expires_at_iso: expires_dt.to_rfc3339(),
            }
        })
        .collect();

    results.sort_by(|a, b| b.created_at_iso.cmp(&a.created_at_iso));
    results
}

/// Manually revokes or marks an ephemeral key as deleted
pub fn revoke_ephemeral_key(id: &str) -> Result<bool, String> {
    let mut store = EPHEMERAL_KEY_STORE.lock();
    if let Some(key) = store.get_mut(id) {
        key.is_revoked = true;
        Ok(true)
    } else {
        Err("Ephemeral key not found".into())
    }
}

/// Formats the shell payload / authorized_keys line for injection
pub fn render_authorized_keys_line(key: &EphemeralKeyRecord) -> String {
    key.public_key.trim().to_string()
}

/// Cleans authorized_keys file content by removing expired or specific ephemeral signatures
pub fn sanitize_authorized_keys(content: &str, current_time: i64) -> (String, usize) {
    let mut cleaned_lines = Vec::new();
    let mut removed_count = 0;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(pos) = trimmed.find("caterm-ephemeral-") {
            let marker = &trimmed[pos..];
            let token = marker.split_whitespace().next().unwrap_or("");
            if let Some(last_part) = token.split('-').last() {
                if let Ok(exp) = last_part.parse::<i64>() {
                    if current_time >= exp {
                        removed_count += 1;
                        continue;
                    }
                }
            }
        }
        cleaned_lines.push(line);
    }

    let mut result = cleaned_lines.join("\n");
    if !result.is_empty() {
        result.push('\n');
    }
    (result, removed_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ephemeral_key_lifecycle() {
        let record = generate_ephemeral_key(CreateEphemeralKeyRequest {
            name: "Temp Deploy".into(),
            ttl_seconds: 7200,
            target_host: Some("192.168.1.100".into()),
            auto_revoke: Some(true),
        })
        .expect("Failed to generate ephemeral key");

        println!("Generated Public Key: {}", record.public_key);
        println!("Comment sig: {}", record.comment_signature);

        assert!(record.public_key.contains("ssh-ed25519"));
        assert!(record.public_key.contains("caterm-ephemeral-"));
        assert_eq!(record.ttl_seconds, 7200);

        let list = list_ephemeral_keys();
        assert!(!list.is_empty());

        let test_auth_keys = format!(
            "ssh-rsa AAAAB3NzaC1yc2E admin@prod\n{}\n",
            record.public_key
        );
        let (cleaned, removed) = sanitize_authorized_keys(&test_auth_keys, record.expires_at + 10);
        println!("Cleaned:\n{cleaned}\nRemoved: {removed}");
        assert_eq!(removed, 1);
        assert!(!cleaned.contains("caterm-ephemeral-"));
        assert!(cleaned.contains("admin@prod"));
    }
}
