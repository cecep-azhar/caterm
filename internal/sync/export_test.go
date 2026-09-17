package sync

import (
	"context"
	"os"
	"path/filepath"
	"testing"
	"time"

	"caterm/internal/store"
	"caterm/internal/vault"
)

func TestDeterministicExport(t *testing.T) {
	dbPath := t.TempDir() + "/test.db"
	outDirA := t.TempDir() + "/sync_a"
	outDirB := t.TempDir() + "/sync_b"

	db, err := store.Open(dbPath)
	if err != nil {
		t.Fatalf("failed to open db: %v", err)
	}
	defer db.Close()

	ctx := context.Background()
	if err := db.Migrate(ctx); err != nil {
		t.Fatalf("failed to migrate: %v", err)
	}

	// Insert vault meta
	v := vault.NewVault()
	err = v.Init(ctx, db, "QA_TEST_pw_12345678")
	if err != nil {
		t.Fatalf("Init failed: %v", err)
	}

	// Ensure we read something from meta
	var metaCount int
	err = db.QueryRowContext(ctx, "SELECT count(*) FROM vault_meta").Scan(&metaCount)
	if err != nil || metaCount == 0 {
		t.Fatalf("No meta found")
	}

	// Insert a group
	gid := store.NewID()
	now := store.NowUTC()
	_, err = db.ExecContext(ctx, "INSERT INTO groups (id, name, sort_order, created_at, updated_at) VALUES (?, ?, ?, ?, ?)", gid, "QA_Group", 1, now, now)
	if err != nil {
		t.Fatalf("failed to insert group: %v", err)
	}

	// Insert a host
	hid := store.NewID()
	_, err = db.ExecContext(ctx, `
		INSERT INTO hosts (
			id, group_id, name, hostname, port, username, auth_type,
			encrypted_password, encrypted_private_key, created_at, updated_at
		) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`,
		hid, gid, "QA_Host", "127.0.0.1", 22, "admin", "password",
		"c1.pwd.enc", "c1.pk.enc", now, now)
	if err != nil {
		t.Fatalf("failed to insert host: %v", err)
	}

	// Export A
	expA := NewExporter(db, outDirA)
	if err := expA.Export(ctx); err != nil {
		t.Fatalf("Export A failed: %v", err)
	}

	// Wait to prove it doesn't depend on time.Now() during export
	time.Sleep(10 * time.Millisecond)

	// Export B
	expB := NewExporter(db, outDirB)
	if err := expB.Export(ctx); err != nil {
		t.Fatalf("Export B failed: %v", err)
	}

	// Compare outputs
	filesA, err := getFiles(outDirA)
	if err != nil {
		t.Fatalf("getFiles A failed: %v", err)
	}
	filesB, err := getFiles(outDirB)
	if err != nil {
		t.Fatalf("getFiles B failed: %v", err)
	}

	if len(filesA) != len(filesB) {
		t.Fatalf("file count mismatch: %d vs %d", len(filesA), len(filesB))
	}

	for pathA, dataA := range filesA {
		dataB, ok := filesB[pathA]
		if !ok {
			t.Errorf("file %s missing in B", pathA)
			continue
		}
		if dataA != dataB {
			t.Errorf("file %s differs between exports", pathA)
		}
	}
}

func TestAtomicWrite(t *testing.T) {
	outDir := t.TempDir()
	path := filepath.Join(outDir, "test.json")

	err := atomicWriteJSON(path, map[string]string{"test": "val"})
	if err != nil {
		t.Fatalf("atomicWriteJSON failed: %v", err)
	}

	b, err := os.ReadFile(path)
	if err != nil {
		t.Fatalf("ReadFile failed: %v", err)
	}

	expected := "{\n  \"test\": \"val\"\n}\n"
	if string(b) != expected {
		t.Errorf("expected %q, got %q", expected, string(b))
	}
}

func getFiles(dir string) (map[string]string, error) {
	files := make(map[string]string)
	err := filepath.Walk(dir, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		if !info.IsDir() {
			rel, _ := filepath.Rel(dir, path)
			b, err := os.ReadFile(path)
			if err != nil {
				return err
			}
			files[rel] = string(b)
		}
		return nil
	})
	return files, err
}
