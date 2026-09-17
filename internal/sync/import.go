package sync

import (
	"context"
	"database/sql"
	"encoding/json"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strings"

	"caterm/internal/store"
)

type MergeResult struct {
	Added     int      `json:"added"`
	Updated   int      `json:"updated"`
	Deleted   int      `json:"deleted"`
	Skipped   []string `json:"skipped"`
	Conflicts []string `json:"conflicts"`
}

type Importer struct {
	db      *store.DB
	syncDir string
}

func NewImporter(db *store.DB, syncDir string) *Importer {
	return &Importer{
		db:      db,
		syncDir: syncDir,
	}
}

func (imp *Importer) Import(ctx context.Context, dryRun bool) (*MergeResult, error) {
	result := &MergeResult{Skipped: []string{}, Conflicts: []string{}}

	// 1. Read sync files
	imp.importMeta(ctx)
	groupsFiles, brokenGroupFiles := imp.readJSONFiles(filepath.Join(imp.syncDir, "groups"))
	hostsFiles, brokenHostFiles := imp.readJSONFiles(filepath.Join(imp.syncDir, "hosts"))

	result.Skipped = append(result.Skipped, brokenGroupFiles...)
	result.Skipped = append(result.Skipped, brokenHostFiles...)

	if dryRun {
		// Calculate what would change without modifying DB
		if err := imp.mergeGroups(ctx, nil, groupsFiles, result, true); err != nil {
			return nil, err
		}
		if err := imp.mergeHosts(ctx, nil, hostsFiles, result, true); err != nil {
			return nil, err
		}
		return result, nil
	}

	// 2. Backup DB before write
	dbPath := imp.db.Path()
	if dbPath != ":memory:" && dbPath != "" {
		bakPath := dbPath + ".bak"
		if err := copyFile(dbPath, bakPath); err != nil {
			return nil, fmt.Errorf("failed to create db backup: %w", err)
		}
	}

	// 3. Perform merge inside transaction
	tx, err := imp.db.BeginTx(ctx, nil)
	if err != nil {
		return nil, err
	}
	defer tx.Rollback()

	if err := imp.mergeGroups(ctx, tx, groupsFiles, result, false); err != nil {
		return nil, err
	}
	if err := imp.mergeHosts(ctx, tx, hostsFiles, result, false); err != nil {
		return nil, err
	}

	if err := tx.Commit(); err != nil {
		return nil, err
	}

	return result, nil
}

func (imp *Importer) importMeta(ctx context.Context) {
	metaPath := filepath.Join(imp.syncDir, "meta.json")
	b, err := os.ReadFile(metaPath)
	if err != nil {
		return
	}

	var m Meta
	if err := json.Unmarshal(b, &m); err != nil {
		return
	}

	var count int
	err = imp.db.QueryRowContext(ctx, "SELECT count(*) FROM vault_meta").Scan(&count)
	if err == nil && count == 0 {
		now := store.NowUTC()
		imp.db.ExecContext(ctx, `INSERT INTO vault_meta (
			id, schema_version, kdf, kdf_salt, kdf_time, kdf_memory, kdf_threads, wrapped_dek, created_at, updated_at
		) VALUES (1, 1, 'argon2id', ?, 3, 65536, 4, ?, ?, ?)`, m.KDFSalt, m.WrappedDEK, now, now)
	}
}

func (imp *Importer) readJSONFiles(dir string) (map[string]map[string]interface{}, []string) {
	valid := make(map[string]map[string]interface{})
	var broken []string

	entries, err := os.ReadDir(dir)
	if err != nil {
		return valid, broken
	}

	for _, entry := range entries {
		if entry.IsDir() || !strings.HasSuffix(entry.Name(), ".json") {
			continue
		}
		path := filepath.Join(dir, entry.Name())
		b, err := os.ReadFile(path)
		if err != nil {
			broken = append(broken, entry.Name())
			continue
		}

		// Check for git conflict markers or invalid JSON
		contentStr := string(b)
		if strings.Contains(contentStr, "<<<<<<<") || strings.Contains(contentStr, "=======") || strings.Contains(contentStr, ">>>>>>>") {
			broken = append(broken, entry.Name())
			continue
		}

		var data map[string]interface{}
		if err := json.Unmarshal(b, &data); err != nil {
			broken = append(broken, entry.Name())
			continue
		}

		id, ok := data["id"].(string)
		if !ok || id == "" {
			broken = append(broken, entry.Name())
			continue
		}

		valid[id] = data
	}

	return valid, broken
}

type execer interface {
	ExecContext(ctx context.Context, query string, args ...any) (sql.Result, error)
	QueryRowContext(ctx context.Context, query string, args ...any) *sql.Row
}

func (imp *Importer) mergeGroups(ctx context.Context, tx execer, remote map[string]map[string]interface{}, result *MergeResult, dryRun bool) error {
	var ex execer = imp.db
	if tx != nil {
		ex = tx
	}

	for id, r := range remote {
		var localUpdated string
		var localDeleted sql.NullString
		err := ex.QueryRowContext(ctx, "SELECT updated_at, deleted_at FROM groups WHERE id = ?", id).Scan(&localUpdated, &localDeleted)

		rUpdated, _ := r["updated_at"].(string)
		rDeleted, _ := r["deleted_at"].(string)
		name, _ := r["name"].(string)
		sort, _ := r["sort_order"].(float64)
		created, _ := r["created_at"].(string)

		if err == sql.ErrNoRows {
			// Record exists in remote but not local -> Add
			result.Added++
			if !dryRun {
				var delVal interface{}
				if rDeleted != "" {
					delVal = rDeleted
				}
				_, err := ex.ExecContext(ctx,
					"INSERT INTO groups (id, name, sort_order, created_at, updated_at, deleted_at) VALUES (?, ?, ?, ?, ?, ?)",
					id, name, int(sort), created, rUpdated, delVal,
				)
				if err != nil {
					return err
				}
			}
		} else if err == nil {
			// Record exists in both -> LWW comparison
			if rUpdated > localUpdated {
				result.Conflicts = append(result.Conflicts, fmt.Sprintf("group:%s remote newer", id))
				if rDeleted != "" && !localDeleted.Valid {
					result.Deleted++
				} else {
					result.Updated++
				}

				if !dryRun {
					var delVal interface{}
					if rDeleted != "" {
						delVal = rDeleted
					}
					_, err := ex.ExecContext(ctx,
						"UPDATE groups SET name = ?, sort_order = ?, updated_at = ?, deleted_at = ? WHERE id = ?",
						name, int(sort), rUpdated, delVal, id,
					)
					if err != nil {
						return err
					}
				}
			}
		}
	}
	return nil
}

func (imp *Importer) mergeHosts(ctx context.Context, tx execer, remote map[string]map[string]interface{}, result *MergeResult, dryRun bool) error {
	var ex execer = imp.db
	if tx != nil {
		ex = tx
	}

	for id, r := range remote {
		var localUpdated string
		var localDeleted sql.NullString
		err := ex.QueryRowContext(ctx, "SELECT updated_at, deleted_at FROM hosts WHERE id = ?", id).Scan(&localUpdated, &localDeleted)

		rUpdated, _ := r["updated_at"].(string)
		rDeleted, _ := r["deleted_at"].(string)
		name, _ := r["name"].(string)
		host, _ := r["hostname"].(string)
		port, _ := r["port"].(float64)
		user, _ := r["username"].(string)
		auth, _ := r["auth_type"].(string)
		created, _ := r["created_at"].(string)

		var gidVal, pwdVal, pkVal interface{}
		if gid, ok := r["group_id"].(string); ok && gid != "" {
			gidVal = gid
		}
		if pwd, ok := r["encrypted_password"].(string); ok && pwd != "" {
			pwdVal = pwd
		}
		if pk, ok := r["encrypted_private_key"].(string); ok && pk != "" {
			pkVal = pk
		}

		if err == sql.ErrNoRows {
			// Record exists in remote but not local -> Add
			result.Added++
			if !dryRun {
				var delVal interface{}
				if rDeleted != "" {
					delVal = rDeleted
				}
				_, err := ex.ExecContext(ctx,
					`INSERT INTO hosts (
						id, group_id, name, hostname, port, username, auth_type,
						encrypted_password, encrypted_private_key, created_at, updated_at, deleted_at
					) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`,
					id, gidVal, name, host, int(port), user, auth, pwdVal, pkVal, created, rUpdated, delVal,
				)
				if err != nil {
					return err
				}
			}
		} else if err == nil {
			// Record exists in both -> LWW comparison
			if rUpdated > localUpdated {
				result.Conflicts = append(result.Conflicts, fmt.Sprintf("host:%s remote newer", id))
				if rDeleted != "" && !localDeleted.Valid {
					result.Deleted++
				} else {
					result.Updated++
				}

				if !dryRun {
					var delVal interface{}
					if rDeleted != "" {
						delVal = rDeleted
					}
					_, err := ex.ExecContext(ctx,
						`UPDATE hosts SET 
							group_id = ?, name = ?, hostname = ?, port = ?, username = ?, auth_type = ?,
							encrypted_password = ?, encrypted_private_key = ?, updated_at = ?, deleted_at = ?
						WHERE id = ?`,
						gidVal, name, host, int(port), user, auth, pwdVal, pkVal, rUpdated, delVal, id,
					)
					if err != nil {
						return err
					}
				}
			}
		}
	}
	return nil
}

func copyFile(src, dst string) error {
	in, err := os.Open(src)
	if err != nil {
		return err
	}
	defer in.Close()

	out, err := os.Create(dst)
	if err != nil {
		return err
	}
	defer out.Close()

	_, err = io.Copy(out, in)
	return err
}
