package store

import (
	"context"
	"os"
	"path/filepath"
	"testing"
)

func TestStore_Migrate(t *testing.T) {
	tmpDir := t.TempDir()
	dbPath := filepath.Join(tmpDir, "test.db")

	db, err := Open(dbPath)
	if err != nil {
		t.Fatalf("Open failed: %v", err)
	}
	defer db.Close()

	ctx := context.Background()

	// Test 1: First run should succeed
	if err := db.Migrate(ctx); err != nil {
		t.Fatalf("First Migrate failed: %v", err)
	}

	// Check if file exists
	if _, err := os.Stat(dbPath); os.IsNotExist(err) {
		t.Fatalf("Database file not created at %s", dbPath)
	}

	// Test 2: Idempotency - second run should also succeed without error
	if err := db.Migrate(ctx); err != nil {
		t.Fatalf("Second Migrate failed (not idempotent): %v", err)
	}

	// Verify tables are created (just checking if we can query schema_migrations)
	var count int
	if err := db.QueryRowContext(ctx, "SELECT COUNT(*) FROM schema_migrations").Scan(&count); err != nil {
		t.Fatalf("Failed to query schema_migrations: %v", err)
	}
	if count != 1 {
		t.Fatalf("Expected 1 row in schema_migrations, got %d (not idempotent)", count)
	}
}

func TestStore_NewID(t *testing.T) {
	count := 1000
	ids := make([]string, count)

	for i := 0; i < count; i++ {
		ids[i] = NewID()
	}

	// Test ordering - lexicographically sortable
	for i := 1; i < count; i++ {
		if ids[i-1] >= ids[i] {
			t.Errorf("IDs not ordered lexiographically at index %d: %s >= %s", i, ids[i-1], ids[i])
		}
	}
}

func TestStore_NowUTC(t *testing.T) {
	now := NowUTC()
	// Length should be 24 characters: YYYY-MM-DDTHH:MM:SS.mmmZ
	if len(now) != 24 {
		t.Errorf("NowUTC length expected 24, got %d: %s", len(now), now)
	}
	if now[len(now)-1] != 'Z' {
		t.Errorf("NowUTC must end with Z: %s", now)
	}
}
