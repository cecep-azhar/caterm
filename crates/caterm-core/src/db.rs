//! Encrypted SQLite storage engine (T9a). Every record `caterm-core` persists lives inside a
//! single SQLCipher-encrypted file at `<data_dir>/caterm.db` — see [`crate::vault`] for where
//! the passphrase that keys it comes from. This replaces the earlier plaintext `hosts.json`
//! store that `store.rs` used before the vault existed.

use crate::error::{CatermError, DbError};
use rusqlite::Connection;
use std::path::Path;

/// Opens (creating if absent) the SQLCipher-encrypted database at `<data_dir>/caterm.db`,
/// keys it with `passphrase`, and ensures the schema exists. SQLCipher only actually
/// verifies a key lazily on first real read, so this runs a canary query up front to fail
/// fast with a clear error when the passphrase doesn't match an existing file.
pub fn open_encrypted(data_dir: &Path, passphrase: &str) -> Result<Connection, CatermError> {
    std::fs::create_dir_all(data_dir)
        .map_err(|e| CatermError::Db(DbError::Generic(format!("gagal membuat data dir: {e}"))))?;

    let conn = Connection::open(crate::paths::db_path(data_dir))
        .map_err(|e| CatermError::Db(DbError::Generic(format!("gagal membuka database: {e}"))))?;

    // PRAGMA key doesn't support bound parameters; escape single quotes defensively so a
    // passphrase containing one can't break out of the string literal.
    let escaped = passphrase.replace('\'', "''");
    conn.execute_batch(&format!("PRAGMA key = '{escaped}';"))
        .map_err(|e| CatermError::Db(DbError::Generic(format!("gagal set kunci vault: {e}"))))?;

    conn.query_row("SELECT count(*) FROM sqlite_master", [], |_| Ok(()))
        .map_err(|_| {
            CatermError::Db(DbError::Generic(
                "kunci vault salah atau database rusak".into(),
            ))
        })?;

    init_schema(&conn)?;
    Ok(conn)
}

fn init_schema(conn: &Connection) -> Result<(), CatermError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS hosts (
            id TEXT PRIMARY KEY,
            label TEXT NOT NULL,
            address TEXT NOT NULL,
            port INTEGER NOT NULL,
            username TEXT NOT NULL,
            auth_method TEXT NOT NULL,
            tags TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
         );
         CREATE TABLE IF NOT EXISTS groups (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            color TEXT NOT NULL,
            host_ids TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
         );",
    )
    .map_err(|e| CatermError::Db(DbError::Generic(format!("gagal inisialisasi skema: {e}"))))
}

/// Opens the encrypted database at the resolved data dir, keyed with the local vault key.
pub fn open() -> Result<Connection, CatermError> {
    let data_dir = crate::paths::resolve_data_dir()?.path;
    let key = crate::vault::load_or_create_local_key(&data_dir)?;
    open_encrypted(&data_dir, &key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    struct TempDataDir(PathBuf);

    impl TempDataDir {
        fn new(label: &str) -> Self {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("waktu sistem sebelum epoch")
                .as_nanos();
            Self(std::env::temp_dir().join(format!("caterm_db_test_{label}_{nanos:x}")))
        }
    }

    impl Drop for TempDataDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn wrong_passphrase_is_rejected() {
        let dir = TempDataDir::new("wrong_pass");
        {
            let conn = open_encrypted(&dir.0, "correct-horse-battery").expect("gagal buat db");
            drop(conn);
        }
        let reopened = open_encrypted(&dir.0, "totally-different-key");
        assert!(reopened.is_err());
    }

    #[test]
    fn correct_passphrase_reopens_and_keeps_schema() {
        let dir = TempDataDir::new("correct_pass");
        {
            let conn = open_encrypted(&dir.0, "correct-horse-battery").expect("gagal buat db");
            conn.execute(
                "INSERT INTO hosts (id, label, address, port, username, auth_method, tags, created_at, updated_at)
                 VALUES ('h1', 'Test', '10.0.0.1', 22, 'root', '{\"type\":\"password\"}', '[]', 1, 1)",
                [],
            )
            .expect("gagal insert");
        }
        let conn = open_encrypted(&dir.0, "correct-horse-battery").expect("gagal buka ulang db");
        let count: i64 = conn
            .query_row("SELECT count(*) FROM hosts", [], |r| r.get(0))
            .expect("gagal query hosts");
        assert_eq!(count, 1);
    }

    #[test]
    fn plaintext_file_on_disk_never_contains_the_sqlite_header() {
        // SQLCipher-encrypted files never start with the standard "SQLite format 3\0" magic —
        // that header itself would be a plaintext leak. This is the actual "strong encryption"
        // assertion for REQ-9a: the bytes on disk must not be a readable SQLite file.
        let dir = TempDataDir::new("header_check");
        let conn = open_encrypted(&dir.0, "correct-horse-battery").expect("gagal buat db");
        drop(conn);
        let bytes = std::fs::read(crate::paths::db_path(&dir.0)).expect("gagal baca file db");
        assert_ne!(&bytes[..16.min(bytes.len())], b"SQLite format 3\0");
    }
}
