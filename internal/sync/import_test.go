package sync

import (
	"context"
	"os"
	"path/filepath"
	"testing"
	"time"

	"caterm/internal/store"
)

func TestMerge(t *testing.T) {
	// 1. Setup local DB
	dbPath := t.TempDir() + "/test.db"
	db, err := store.Open(dbPath)
	if err != nil {
		t.Fatalf("failed to open db: %v", err)
	}
	defer db.Close()

	ctx := context.Background()
	if err := db.Migrate(ctx); err != nil {
		t.Fatalf("failed to migrate: %v", err)
	}

	now := time.Now().UTC()
	oldTime := now.Add(-2 * time.Hour).Format(time.RFC3339Nano)
	midTime := now.Add(-1 * time.Hour).Format(time.RFC3339Nano)
	newTime := now.Format(time.RFC3339Nano)

	// Local data
	localHostID := store.NewID()
	conflictHostID := store.NewID()

	// Insert local-only host
	_, err = db.ExecContext(ctx, `
		INSERT INTO hosts (
			id, name, hostname, port, username, auth_type,
			created_at, updated_at
		) VALUES (?, ?, ?, ?, ?, ?, ?, ?)`,
		localHostID, "Local_Only", "127.0.0.1", 22, "admin", "password", oldTime, oldTime,
	)
	if err != nil {
		t.Fatalf("failed to insert local host: %v", err)
	}

	// Insert conflict host (older in local DB)
	_, err = db.ExecContext(ctx, `
		INSERT INTO hosts (
			id, name, hostname, port, username, auth_type,
			created_at, updated_at
		) VALUES (?, ?, ?, ?, ?, ?, ?, ?)`,
		conflictHostID, "Conflict_Local_Name", "127.0.0.1", 22, "admin", "password", oldTime, midTime,
	)
	if err != nil {
		t.Fatalf("failed to insert conflict host: %v", err)
	}

	// 2. Setup sync directory
	syncDir := t.TempDir()
	os.MkdirAll(filepath.Join(syncDir, "groups"), 0700)
	os.MkdirAll(filepath.Join(syncDir, "hosts"), 0700)

	remoteHostID := store.NewID()

	// Remote-only host (Added)
	remoteOnlyJSON := `{"id":"` + remoteHostID + `","name":"Remote_Only","hostname":"10.0.0.1","port":22,"username":"root","auth_type":"password","created_at":"` + newTime + `","updated_at":"` + newTime + `"}`
	os.WriteFile(filepath.Join(syncDir, "hosts", remoteHostID+".json"), []byte(remoteOnlyJSON), 0600)

	// Conflict host (newer in remote)
	conflictJSON := `{"id":"` + conflictHostID + `","name":"Conflict_Remote_Name","hostname":"127.0.0.1","port":22,"username":"admin","auth_type":"password","created_at":"` + oldTime + `","updated_at":"` + newTime + `"}`
	os.WriteFile(filepath.Join(syncDir, "hosts", conflictHostID+".json"), []byte(conflictJSON), 0600)

	// Broken file
	brokenJSON := `<<<<<<< HEAD
{"id":"broken"}
=======
{"id":"broken2"}
>>>>>>> branch`
	os.WriteFile(filepath.Join(syncDir, "hosts", "broken.json"), []byte(brokenJSON), 0600)

	// 3. Dry-run Merge
	importer := NewImporter(db, syncDir)
	resDry, err := importer.Import(ctx, true)
	if err != nil {
		t.Fatalf("Dry run failed: %v", err)
	}

	if resDry.Added != 1 {
		t.Errorf("expected 1 added, got %d", resDry.Added)
	}
	if resDry.Updated != 1 {
		t.Errorf("expected 1 updated, got %d", resDry.Updated)
	}
	if len(resDry.Skipped) != 1 || resDry.Skipped[0] != "broken.json" {
		t.Errorf("expected broken.json skipped, got %v", resDry.Skipped)
	}

	// Verify local DB did not change
	var name string
	db.QueryRowContext(ctx, "SELECT name FROM hosts WHERE id = ?", conflictHostID).Scan(&name)
	if name != "Conflict_Local_Name" {
		t.Errorf("expected name Conflict_Local_Name after dry run, got %s", name)
	}
	var count int
	db.QueryRowContext(ctx, "SELECT count(*) FROM hosts WHERE id = ?", remoteHostID).Scan(&count)
	if count != 0 {
		t.Errorf("expected remote host to not exist after dry run")
	}

	// 4. Actual Merge
	resReal, err := importer.Import(ctx, false)
	if err != nil {
		t.Fatalf("Real merge failed: %v", err)
	}

	if resReal.Added != 1 || resReal.Updated != 1 {
		t.Errorf("unexpected real merge result: %+v", resReal)
	}

	// Verify DB changed
	db.QueryRowContext(ctx, "SELECT name FROM hosts WHERE id = ?", conflictHostID).Scan(&name)
	if name != "Conflict_Remote_Name" {
		t.Errorf("expected name Conflict_Remote_Name after merge, got %s", name)
	}
	db.QueryRowContext(ctx, "SELECT count(*) FROM hosts WHERE id = ?", remoteHostID).Scan(&count)
	if count != 1 {
		t.Errorf("expected remote host to exist after merge")
	}

	// Verify Local-Only survived
	db.QueryRowContext(ctx, "SELECT count(*) FROM hosts WHERE id = ?", localHostID).Scan(&count)
	if count != 1 {
		t.Errorf("expected local host to survive merge")
	}

	// 5. Verify Backup
	if _, err := os.Stat(dbPath + ".bak"); os.IsNotExist(err) {
		t.Errorf("expected backup file %s to exist", dbPath+".bak")
	}
}
