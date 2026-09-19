//! Host store (K2-2 / T9a). Backed by the SQLCipher-encrypted `caterm.db` (see `crate::db`)
//! — every row here lives encrypted at rest, replacing the earlier plaintext `hosts.json`
//! placeholder this module used before the vault existed.
//!
//! Host credentials (password, or an SSH key's passphrase) are encrypted independently via
//! `crate::secret` before being written to the `secret_enc` column — see that module for the
//! key derivation. Plaintext secrets only ever exist transiently: on the way in (`HostInput`)
//! and on the way back out to `crate::ssh` for an actual connection attempt
//! (`load_host_for_connect`). `HostRecord`, the shape returned over IPC to the frontend,
//! never carries the plaintext — only `has_secret`, so the UI can say "already set".
//! `AuthMethod::Key` still only carries a filesystem path, never key bytes.

use crate::db;
use crate::error::{CatermError, DbError};
use crate::groups;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AuthMethod {
    Password,
    Key { path: String },
    KeyId { id: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostRecord {
    pub id: String,
    pub label: String,
    pub address: String,
    pub port: u16,
    pub username: String,
    pub auth_method: AuthMethod,
    pub tags: Vec<String>,
    pub created_at: u64,
    pub updated_at: u64,
    /// Whether a password/passphrase is already stored for this host. Never the secret
    /// itself — just enough for the UI to render "leave blank to keep current password".
    pub has_secret: bool,
}

/// Payload for `save_host`: same shape as `HostRecord` minus the fields the
/// store itself owns (`id` is optional — omitted/empty means "create new",
/// present means "update that record"; timestamps are always server-set).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostInput {
    pub id: Option<String>,
    pub label: String,
    pub address: String,
    pub port: u16,
    pub username: String,
    pub auth_method: AuthMethod,
    pub tags: Vec<String>,
    /// Write-only. `None` (field omitted) = leave the stored secret untouched. `Some("")` =
    /// clear it. `Some(s)` = encrypt `s` and store it, replacing whatever was there.
    #[serde(default)]
    pub secret: Option<String>,
}

fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn generate_id() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("host-{nanos:x}")
}

fn row_to_host(row: &rusqlite::Row) -> rusqlite::Result<HostRecord> {
    let auth_json: String = row.get("auth_method")?;
    let tags_json: String = row.get("tags")?;
    let secret_enc: Option<String> = row.get("secret_enc")?;
    let auth_method: AuthMethod = serde_json::from_str(&auth_json).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;
    let tags: Vec<String> = serde_json::from_str(&tags_json).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;
    Ok(HostRecord {
        id: row.get("id")?,
        label: row.get("label")?,
        address: row.get("address")?,
        port: row.get::<_, i64>("port")? as u16,
        username: row.get("username")?,
        auth_method,
        tags,
        created_at: row.get::<_, i64>("created_at")? as u64,
        updated_at: row.get::<_, i64>("updated_at")? as u64,
        has_secret: secret_enc.map(|s| !s.is_empty()).unwrap_or(false),
    })
}

/// All saved hosts, ordered by creation time. Crate-visible so `crate::groups` can validate
/// that a group's `host_ids` actually reference real hosts (the Hosts<->Groups link, T6).
pub(crate) fn list_hosts_in(conn: &Connection) -> Result<Vec<HostRecord>, CatermError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, label, address, port, username, auth_method, tags, created_at, updated_at, secret_enc \
             FROM hosts ORDER BY created_at ASC",
        )
        .map_err(|e| CatermError::Db(DbError::Generic(format!("gagal query hosts: {e}"))))?;
    let rows = stmt
        .query_map([], row_to_host)
        .map_err(|e| CatermError::Db(DbError::Generic(format!("gagal query hosts: {e}"))))?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| CatermError::Db(DbError::Generic(format!("baris hosts rusak: {e}"))))?);
    }
    Ok(out)
}

pub(crate) fn save_host_in(
    conn: &Connection,
    input: HostInput,
    local_key: &str,
) -> Result<HostRecord, CatermError> {
    let now = now_unix();
    let id = input.id.filter(|id| !id.is_empty());
    let (created_at, existing_secret_enc): (u64, Option<String>) = match &id {
        Some(existing_id) => conn
            .query_row(
                "SELECT created_at, secret_enc FROM hosts WHERE id = ?1",
                params![existing_id],
                |r| Ok((r.get::<_, i64>(0)? as u64, r.get::<_, Option<String>>(1)?)),
            )
            .unwrap_or((now, None)),
        None => (now, None),
    };
    let id = id.unwrap_or_else(generate_id);

    let secret_enc: Option<String> = match input.secret.as_deref() {
        None => existing_secret_enc,
        Some("") => None,
        Some(plaintext) => Some(crate::secret::encrypt(local_key, plaintext)?),
    };

    let auth_json = serde_json::to_string(&input.auth_method)
        .map_err(|e| CatermError::Db(DbError::Generic(format!("gagal serialisasi auth_method: {e}"))))?;
    let tags_json = serde_json::to_string(&input.tags)
        .map_err(|e| CatermError::Db(DbError::Generic(format!("gagal serialisasi tags: {e}"))))?;

    conn.execute(
        "INSERT INTO hosts (id, label, address, port, username, auth_method, tags, created_at, updated_at, secret_enc)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         ON CONFLICT(id) DO UPDATE SET
            label = excluded.label,
            address = excluded.address,
            port = excluded.port,
            username = excluded.username,
            auth_method = excluded.auth_method,
            tags = excluded.tags,
            updated_at = excluded.updated_at,
            secret_enc = excluded.secret_enc",
        params![
            id,
            input.label,
            input.address,
            input.port,
            input.username,
            auth_json,
            tags_json,
            created_at as i64,
            now as i64,
            secret_enc
        ],
    )
    .map_err(|e| CatermError::Db(DbError::Generic(format!("gagal menyimpan host: {e}"))))?;

    Ok(HostRecord {
        id,
        label: input.label,
        address: input.address,
        port: input.port,
        username: input.username,
        auth_method: input.auth_method,
        tags: input.tags,
        created_at,
        updated_at: now,
        has_secret: secret_enc.map(|s| !s.is_empty()).unwrap_or(false),
    })
}

fn delete_host_in(conn: &Connection, id: &str) -> Result<(), CatermError> {
    let affected = conn
        .execute("DELETE FROM hosts WHERE id = ?1", params![id])
        .map_err(|e| CatermError::Db(DbError::Generic(format!("gagal hapus host: {e}"))))?;
    if affected == 0 {
        return Err(CatermError::Db(DbError::Generic(format!(
            "host dengan id {id} tidak ditemukan"
        ))));
    }
    // Keep the Hosts<->Groups link consistent: a deleted host can't stay referenced by any
    // group's host_ids (T6).
    groups::strip_host_from_groups(conn, id)
}

/// All saved hosts, in no particular order.
pub fn list_hosts() -> Result<Vec<HostRecord>, CatermError> {
    list_hosts_in(&db::open()?)
}

/// Insert a new host (empty/absent `id`) or update an existing one in place,
/// preserving its original `created_at`.
pub fn save_host(input: HostInput) -> Result<HostRecord, CatermError> {
    let data_dir = crate::paths::resolve_data_dir()?.path;
    let local_key = crate::vault::load_or_create_local_key(&data_dir)?;
    save_host_in(&db::open()?, input, &local_key)
}

/// Remove a host by id. Errors if no host with that id exists.
pub fn delete_host(id: &str) -> Result<(), CatermError> {
    delete_host_in(&db::open()?, id)
}

fn load_host_for_connect_in(
    conn: &Connection,
    id: &str,
    local_key: &str,
) -> Result<(HostRecord, Option<String>), CatermError> {
    let (host, secret_enc): (HostRecord, Option<String>) = conn
        .query_row(
            "SELECT id, label, address, port, username, auth_method, tags, created_at, updated_at, secret_enc \
             FROM hosts WHERE id = ?1",
            params![id],
            |row| Ok((row_to_host(row)?, row.get::<_, Option<String>>("secret_enc")?)),
        )
        .map_err(|e| CatermError::Db(DbError::Generic(format!("host {id} tidak ditemukan: {e}"))))?;

    let secret = match secret_enc {
        Some(enc) if !enc.is_empty() => Some(crate::secret::decrypt(local_key, &enc)?),
        _ => None,
    };
    Ok((host, secret))
}

/// Crate-internal: resolves a host plus its decrypted secret (password, or SSH key
/// passphrase) for `crate::ssh` to actually open a connection with. The plaintext never
/// travels any further than this — it is not part of `HostRecord` and never crosses IPC.
pub(crate) fn load_host_for_connect(id: &str) -> Result<(HostRecord, Option<String>), CatermError> {
    let data_dir = crate::paths::resolve_data_dir()?.path;
    let local_key = crate::vault::load_or_create_local_key(&data_dir)?;
    load_host_for_connect_in(&db::open()?, id, &local_key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    const TEST_KEY: &str = "test-local-key";

    struct TempDb(Connection, PathBuf);

    impl TempDb {
        fn new(label: &str) -> Self {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("waktu sistem sebelum epoch")
                .as_nanos();
            let dir = std::env::temp_dir().join(format!("caterm_store_test_{label}_{nanos:x}"));
            let conn = db::open_encrypted(&dir, "test-passphrase").expect("gagal buka db test");
            Self(conn, dir)
        }
    }

    impl Drop for TempDb {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.1);
        }
    }

    #[test]
    fn save_then_list_roundtrips() {
        let db = TempDb::new("roundtrip");
        let saved = save_host_in(
            &db.0,
            HostInput {
                id: None,
                label: "Test Host".into(),
                address: "10.0.0.5".into(),
                port: 22,
                username: "root".into(),
                auth_method: AuthMethod::Password,
                tags: vec!["test".into()],
                secret: None,
            },
            TEST_KEY,
        )
        .expect("save_host gagal");

        let all = list_hosts_in(&db.0).expect("list_hosts gagal");
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].id, saved.id);
        assert_eq!(all[0].label, "Test Host");
    }

    #[test]
    fn save_with_existing_id_updates_in_place() {
        let db = TempDb::new("update");
        let first = save_host_in(
            &db.0,
            HostInput {
                id: None,
                label: "Original".into(),
                address: "10.0.0.5".into(),
                port: 22,
                username: "root".into(),
                auth_method: AuthMethod::Password,
                tags: vec![],
                secret: Some("hunter2".into()),
            },
            TEST_KEY,
        )
        .expect("save_host gagal");
        assert!(first.has_secret);

        let updated = save_host_in(
            &db.0,
            HostInput {
                id: Some(first.id.clone()),
                label: "Updated".into(),
                address: "10.0.0.6".into(),
                port: 2222,
                username: "admin".into(),
                auth_method: AuthMethod::Key {
                    path: "/home/user/.ssh/id_ed25519".into(),
                },
                tags: vec!["prod".into()],
                secret: None,
            },
            TEST_KEY,
        )
        .expect("save_host gagal");

        assert_eq!(updated.id, first.id);
        assert_eq!(updated.created_at, first.created_at);
        // `secret: None` on the update must not wipe the password saved above.
        assert!(updated.has_secret);

        let all = list_hosts_in(&db.0).expect("list_hosts gagal");
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].label, "Updated");

        let (_, decrypted) = load_host_for_connect_in(&db.0, &first.id, TEST_KEY).expect("load gagal");
        assert_eq!(decrypted.as_deref(), Some("hunter2"));
    }

    #[test]
    fn empty_string_secret_clears_it() {
        let db = TempDb::new("clear_secret");
        let host = save_host_in(
            &db.0,
            HostInput {
                id: None,
                label: "Host".into(),
                address: "10.0.0.9".into(),
                port: 22,
                username: "root".into(),
                auth_method: AuthMethod::Password,
                tags: vec![],
                secret: Some("hunter2".into()),
            },
            TEST_KEY,
        )
        .expect("save_host gagal");
        assert!(host.has_secret);

        let cleared = save_host_in(
            &db.0,
            HostInput {
                id: Some(host.id.clone()),
                label: "Host".into(),
                address: "10.0.0.9".into(),
                port: 22,
                username: "root".into(),
                auth_method: AuthMethod::Password,
                tags: vec![],
                secret: Some("".into()),
            },
            TEST_KEY,
        )
        .expect("save_host gagal");
        assert!(!cleared.has_secret);
    }

    #[test]
    fn delete_removes_host_and_errors_when_missing() {
        let db = TempDb::new("delete");
        let saved = save_host_in(
            &db.0,
            HostInput {
                id: None,
                label: "To Delete".into(),
                address: "10.0.0.7".into(),
                port: 22,
                username: "root".into(),
                auth_method: AuthMethod::Password,
                tags: vec![],
                secret: None,
            },
            TEST_KEY,
        )
        .expect("save_host gagal");

        delete_host_in(&db.0, &saved.id).expect("delete_host gagal");
        assert!(list_hosts_in(&db.0).expect("list_hosts gagal").is_empty());
        assert!(delete_host_in(&db.0, &saved.id).is_err());
    }

    #[test]
    fn deleting_a_host_strips_it_from_groups() {
        let db = TempDb::new("delete_link");
        let host = save_host_in(
            &db.0,
            HostInput {
                id: None,
                label: "Linked Host".into(),
                address: "10.0.0.8".into(),
                port: 22,
                username: "root".into(),
                auth_method: AuthMethod::Password,
                tags: vec![],
                secret: None,
            },
            TEST_KEY,
        )
        .expect("save_host gagal");

        let group = groups::save_group_in(
            &db.0,
            groups::GroupInput {
                id: None,
                name: "Prod".into(),
                color: "#ef4444".into(),
                host_ids: vec![host.id.clone()],
            },
        )
        .expect("save_group gagal");
        assert_eq!(group.host_ids, vec![host.id.clone()]);

        delete_host_in(&db.0, &host.id).expect("delete_host gagal");

        let groups_after = groups::list_groups_in(&db.0).expect("list_groups gagal");
        assert_eq!(groups_after.len(), 1);
        assert!(groups_after[0].host_ids.is_empty());
    }
}
