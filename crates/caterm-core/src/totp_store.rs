//! Persistent Vault Storage for Dedicated 2FA / TOTP Authenticator (REQ-37).

use crate::db;
use crate::error::{CatermError, DbError, ValidationError};
use crate::totp::{generate_totp_at_timestamp, parse_otpauth_uri};
use crate::vault::get_active_dek;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TotpEntryRecord {
    pub id: String,
    pub label: String,
    pub issuer: Option<String>,
    pub has_secret: bool,
    pub token: String,
    pub remaining_seconds: u64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TotpEntryInput {
    pub id: Option<String>,
    pub label: String,
    pub issuer: Option<String>,
    pub secret: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TotpGeneratedToken {
    pub token: String,
    pub remaining_seconds: u64,
}

fn now_sec() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn list_totp_entries() -> Result<Vec<TotpEntryRecord>, CatermError> {
    let key = get_active_dek()?;
    let key_hex = hex::encode(key);
    let conn = db::open()?;
    let mut stmt = conn
        .prepare("SELECT id, label, issuer, secret_enc, created_at, updated_at FROM totp_entries ORDER BY label COLLATE NOCASE ASC;")
        .map_err(|e| CatermError::Db(DbError::Generic(e.to_string())))?;

    let now = now_sec();
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
            ))
        })
        .map_err(|e| CatermError::Db(DbError::Generic(e.to_string())))?;

    let mut entries = Vec::new();
    for r in rows {
        let (id, label, issuer, secret_enc, created_at, updated_at) =
            r.map_err(|e| CatermError::Db(DbError::Generic(e.to_string())))?;

        let (token, remaining_seconds) = match crate::secret::decrypt(&key_hex, &secret_enc) {
            Ok(secret) => generate_totp_at_timestamp(&secret, now, 30, 6)
                .unwrap_or_else(|_| ("------".to_string(), 0)),
            Err(_) => ("------".to_string(), 0),
        };

        entries.push(TotpEntryRecord {
            id,
            label,
            issuer,
            has_secret: !secret_enc.is_empty(),
            token,
            remaining_seconds,
            created_at,
            updated_at,
        });
    }

    Ok(entries)
}

pub fn save_totp_entry(input: TotpEntryInput) -> Result<TotpEntryRecord, CatermError> {
    let mut label = input.label.trim().to_string();
    let mut issuer = input
        .issuer
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let mut raw_secret = input.secret.unwrap_or_default().trim().to_string();

    // Support pasting raw otpauth:// URI directly
    if raw_secret.starts_with("otpauth://") {
        if let Ok(parsed) = parse_otpauth_uri(&raw_secret) {
            if label.is_empty() {
                label = parsed.label;
            }
            if issuer.is_none() {
                issuer = parsed.issuer;
            }
            raw_secret = parsed.secret;
        }
    }

    if label.is_empty() {
        return Err(CatermError::Validation(ValidationError::Generic(
            "Label 2FA tidak boleh kosong".to_string(),
        )));
    }

    let key = get_active_dek()?;
    let key_hex = hex::encode(key);
    let conn = db::open()?;
    let now = now_sec() as i64;
    let id = input
        .id
        .unwrap_or_else(|| format!("totp-{}", &uuid::Uuid::new_v4().simple().to_string()[..12]));

    let secret_enc = if !raw_secret.is_empty() {
        // Validate base32 secret before encrypting
        crate::totp::decode_base32(&raw_secret)?;
        crate::secret::encrypt(&key_hex, &raw_secret)?
    } else {
        // If editing without changing secret, preserve existing
        let existing: Option<String> = conn
            .query_row(
                "SELECT secret_enc FROM totp_entries WHERE id = ?1;",
                params![id],
                |row| row.get(0),
            )
            .ok();
        existing.ok_or_else(|| {
            CatermError::Validation(ValidationError::Generic(
                "Secret 2FA wajib diisi".to_string(),
            ))
        })?
    };

    conn.execute(
        "INSERT INTO totp_entries (id, label, issuer, secret_enc, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?5)
         ON CONFLICT(id) DO UPDATE SET
            label = excluded.label,
            issuer = excluded.issuer,
            secret_enc = excluded.secret_enc,
            updated_at = excluded.updated_at;",
        params![id, label, issuer, secret_enc, now],
    )
    .map_err(|e| CatermError::Db(DbError::Generic(e.to_string())))?;

    let (token, remaining_seconds) = match crate::secret::decrypt(&key_hex, &secret_enc) {
        Ok(secret) => generate_totp_at_timestamp(&secret, now as u64, 30, 6)
            .unwrap_or_else(|_| ("------".to_string(), 0)),
        Err(_) => ("------".to_string(), 0),
    };

    Ok(TotpEntryRecord {
        id,
        label,
        issuer,
        has_secret: true,
        token,
        remaining_seconds,
        created_at: now,
        updated_at: now,
    })
}

pub fn delete_totp_entry(id: &str) -> Result<(), CatermError> {
    let conn = db::open()?;
    conn.execute("DELETE FROM totp_entries WHERE id = ?1;", params![id])
        .map_err(|e| CatermError::Db(DbError::Generic(e.to_string())))?;
    Ok(())
}

pub fn generate_current_totp(secret_or_id: &str) -> Result<TotpGeneratedToken, CatermError> {
    let now = now_sec();
    let secret = if secret_or_id.starts_with("totp-") || secret_or_id.len() <= 20 {
        // Treat as ID
        let key = get_active_dek()?;
        let key_hex = hex::encode(key);
        let conn = db::open()?;
        let enc: String = conn
            .query_row(
                "SELECT secret_enc FROM totp_entries WHERE id = ?1;",
                params![secret_or_id],
                |row| row.get(0),
            )
            .map_err(|e| CatermError::Db(DbError::Generic(e.to_string())))?;
        crate::secret::decrypt(&key_hex, &enc)?
    } else if secret_or_id.starts_with("otpauth://") {
        parse_otpauth_uri(secret_or_id)?.secret
    } else {
        secret_or_id.to_string()
    };

    let (token, remaining_seconds) = generate_totp_at_timestamp(&secret, now, 30, 6)?;
    Ok(TotpGeneratedToken {
        token,
        remaining_seconds,
    })
}
