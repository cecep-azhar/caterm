package store

import (
	"context"
	"fmt"
)

var schema = []string{
	`CREATE TABLE IF NOT EXISTS schema_migrations (version INTEGER PRIMARY KEY);`,
	`CREATE TABLE IF NOT EXISTS vault_meta (
		id INTEGER PRIMARY KEY CHECK (id = 1),
		schema_version INTEGER NOT NULL,
		kdf TEXT NOT NULL,
		kdf_salt TEXT NOT NULL,
		kdf_time INTEGER NOT NULL,
		kdf_memory INTEGER NOT NULL,
		kdf_threads INTEGER NOT NULL,
		wrapped_dek TEXT NOT NULL,
		created_at TEXT NOT NULL,
		updated_at TEXT NOT NULL
	);`,
	`CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);`,
	`CREATE TABLE IF NOT EXISTS groups (
		id TEXT PRIMARY KEY,
		name TEXT NOT NULL,
		sort_order INTEGER NOT NULL DEFAULT 0,
		created_at TEXT NOT NULL,
		updated_at TEXT NOT NULL,
		deleted_at TEXT
	);`,
	`CREATE TABLE IF NOT EXISTS hosts (
		id TEXT PRIMARY KEY,
		group_id TEXT REFERENCES groups(id) ON DELETE SET NULL,
		name TEXT NOT NULL,
		hostname TEXT NOT NULL,
		port INTEGER NOT NULL DEFAULT 22,
		username TEXT NOT NULL,
		auth_type TEXT NOT NULL,
		encrypted_password TEXT,
		encrypted_private_key TEXT,
		created_at TEXT NOT NULL,
		updated_at TEXT NOT NULL,
		deleted_at TEXT
	);
	CREATE INDEX IF NOT EXISTS idx_hosts_group ON hosts(group_id);
	CREATE INDEX IF NOT EXISTS idx_hosts_updated ON hosts(updated_at);`,
	`CREATE TABLE IF NOT EXISTS snippets (
		id TEXT PRIMARY KEY,
		name TEXT NOT NULL,
		body TEXT NOT NULL,
		auto_enter INTEGER NOT NULL DEFAULT 0,
		scope_type TEXT NOT NULL DEFAULT 'global',
		scope_id TEXT,
		created_at TEXT NOT NULL,
		updated_at TEXT NOT NULL,
		deleted_at TEXT
	);`,
	`CREATE TABLE IF NOT EXISTS host_keys (
		id TEXT PRIMARY KEY,
		hostname TEXT NOT NULL,
		port INTEGER NOT NULL,
		key_type TEXT NOT NULL,
		fingerprint TEXT NOT NULL,
		created_at TEXT NOT NULL,
		updated_at TEXT NOT NULL,
		deleted_at TEXT,
		UNIQUE(hostname, port, key_type)
	);`,
	`CREATE TABLE IF NOT EXISTS ssh_keys (
		id TEXT PRIMARY KEY,
		name TEXT NOT NULL UNIQUE,
		private_key TEXT NOT NULL,
		public_key TEXT NOT NULL,
		fingerprint TEXT NOT NULL,
		created_at TEXT NOT NULL,
		updated_at TEXT NOT NULL
	);`,
	`CREATE TABLE IF NOT EXISTS audit_logs (
		id TEXT PRIMARY KEY,
		host_id TEXT NOT NULL,
		command TEXT NOT NULL,
		output TEXT,
		exit_code INTEGER NOT NULL DEFAULT 0,
		created_at TEXT NOT NULL
	);
	CREATE INDEX IF NOT EXISTS idx_audit_logs_host ON audit_logs(host_id);
	CREATE INDEX IF NOT EXISTS idx_audit_logs_created ON audit_logs(created_at);`,
	`CREATE TABLE IF NOT EXISTS teams (
		id TEXT PRIMARY KEY,
		name TEXT NOT NULL,
		created_at TEXT NOT NULL,
		updated_at TEXT NOT NULL,
		deleted_at TEXT
	);`,
}

func (db *DB) Migrate(ctx context.Context) error {
	tx, err := db.BeginTx(ctx, nil)
	if err != nil {
		return fmt.Errorf("begin tx: %w", err)
	}
	defer tx.Rollback()

	for i, q := range schema {
		if _, err := tx.ExecContext(ctx, q); err != nil {
			return fmt.Errorf("migration step %d failed: %w", i, err)
		}
	}

	// Record version 1 if not exists
	_, err = tx.ExecContext(ctx, `INSERT OR IGNORE INTO schema_migrations (version) VALUES (1)`)
	if err != nil {
		return fmt.Errorf("failed to record schema version: %w", err)
	}

	return tx.Commit()
}
