package sync

import (
	"context"
	"database/sql"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"

	"caterm/internal/store"
)

type Exporter struct {
	db     *store.DB
	outDir string
}

func NewExporter(db *store.DB, outDir string) *Exporter {
	return &Exporter{
		db:     db,
		outDir: outDir,
	}
}

// Meta record structure matching schema design
type Meta struct {
	SchemaVersion int    `json:"schema_version"`
	KDFParams     string `json:"kdf_params"`
	KDFSalt       string `json:"kdf_salt"`
	WrappedDEK    string `json:"wrapped_dek"`
}

func atomicWriteJSON(path string, data interface{}) error {
	// 1. Serialize deterministically (sorted keys, 2 indent, LF newline at EOF)
	b, err := json.MarshalIndent(data, "", "  ")
	if err != nil {
		return err
	}
	// json.MarshalIndent doesn't add a trailing newline
	b = append(b, '\n')

	// 2. Write to tmp file
	tmpPath := path + ".tmp"
	if err := os.WriteFile(tmpPath, b, 0600); err != nil {
		return err
	}

	// 3. Rename (atomic on POSIX, usually fine on Windows with MoveFileEx)
	return os.Rename(tmpPath, path)
}

func (e *Exporter) Export(ctx context.Context) error {
	if err := os.MkdirAll(e.outDir, 0700); err != nil {
		return fmt.Errorf("failed to create sync dir: %w", err)
	}

	// 1. Export Meta
	var m Meta
	err := e.db.QueryRowContext(ctx, "SELECT kdf_salt, wrapped_dek FROM vault_meta LIMIT 1").Scan(&m.KDFSalt, &m.WrappedDEK)
	if err != nil && err != sql.ErrNoRows {
		return fmt.Errorf("failed to read vault_meta: %w", err)
	}
	m.SchemaVersion = 1
	m.KDFParams = "argon2id t=3 m=64MiB p=4"

	if err := atomicWriteJSON(filepath.Join(e.outDir, "meta.json"), m); err != nil {
		return err
	}

	// 2. Export Groups
	if err := os.MkdirAll(filepath.Join(e.outDir, "groups"), 0700); err != nil {
		return err
	}
	groups, err := e.queryGroups(ctx)
	if err != nil {
		return err
	}
	for _, g := range groups {
		if err := atomicWriteJSON(filepath.Join(e.outDir, "groups", g["id"].(string)+".json"), g); err != nil {
			return err
		}
	}

	// 3. Export Hosts
	if err := os.MkdirAll(filepath.Join(e.outDir, "hosts"), 0700); err != nil {
		return err
	}
	hosts, err := e.queryHosts(ctx)
	if err != nil {
		return err
	}
	for _, h := range hosts {
		if err := atomicWriteJSON(filepath.Join(e.outDir, "hosts", h["id"].(string)+".json"), h); err != nil {
			return err
		}
	}

	return nil
}

func (e *Exporter) queryGroups(ctx context.Context) ([]map[string]interface{}, error) {
	rows, err := e.db.QueryContext(ctx, "SELECT id, name, sort_order, created_at, updated_at, deleted_at FROM groups")
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var result []map[string]interface{}
	for rows.Next() {
		var id, name, created, updated string
		var sort int
		var del sql.NullString
		if err := rows.Scan(&id, &name, &sort, &created, &updated, &del); err != nil {
			return nil, err
		}

		record := map[string]interface{}{
			"id":         id,
			"name":       name,
			"sort_order": sort,
			"created_at": created,
			"updated_at": updated,
			"deleted_at": nil,
		}
		if del.Valid {
			record["deleted_at"] = del.String
		}
		result = append(result, record)
	}
	return result, nil
}

func (e *Exporter) queryHosts(ctx context.Context) ([]map[string]interface{}, error) {
	rows, err := e.db.QueryContext(ctx, `
		SELECT id, group_id, name, hostname, port, username, auth_type,
		encrypted_password, encrypted_private_key, created_at, updated_at, deleted_at 
		FROM hosts
	`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var result []map[string]interface{}
	for rows.Next() {
		var id, name, host, user, auth, created, updated string
		var port int
		var gid, pwd, pk, del sql.NullString

		if err := rows.Scan(&id, &gid, &name, &host, &port, &user, &auth, &pwd, &pk, &created, &updated, &del); err != nil {
			return nil, err
		}

		record := map[string]interface{}{
			"id":                    id,
			"group_id":              nil,
			"name":                  name,
			"hostname":              host,
			"port":                  port,
			"username":              user,
			"auth_type":             auth,
			"encrypted_password":    nil,
			"encrypted_private_key": nil,
			"created_at":            created,
			"updated_at":            updated,
			"deleted_at":            nil,
		}
		if gid.Valid {
			record["group_id"] = gid.String
		}
		if pwd.Valid {
			record["encrypted_password"] = pwd.String
		}
		if pk.Valid {
			record["encrypted_private_key"] = pk.String
		}
		if del.Valid {
			record["deleted_at"] = del.String
		}
		result = append(result, record)
	}
	return result, nil
}
