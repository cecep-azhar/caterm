package audit

import (
	"context"
	"path/filepath"
	"testing"

	"caterm/internal/store"
)

func setupTestDB(t *testing.T) *store.DB {
	tmpDir := t.TempDir()
	dbPath := filepath.Join(tmpDir, "test.db")
	db, err := store.Open(dbPath)
	if err != nil {
		t.Fatalf("failed to open test db: %v", err)
	}
	if err := db.Migrate(context.Background()); err != nil {
		t.Fatalf("failed to migrate db: %v", err)
	}
	return db
}

func TestAuditService(t *testing.T) {
	db := setupTestDB(t)
	defer db.Close()

	svc := NewService(db)
	ctx := context.Background()

	// Test LogCommand
	err := svc.LogCommand(ctx, "host-1", "ls -l", "total 0", 0)
	if err != nil {
		t.Fatalf("LogCommand failed: %v", err)
	}

	err = svc.LogCommand(ctx, "host-1", "cat missing.txt", "cat: missing.txt: No such file", 1)
	if err != nil {
		t.Fatalf("LogCommand failed: %v", err)
	}

	err = svc.LogCommand(ctx, "host-2", "uname -a", "Linux", 0)
	if err != nil {
		t.Fatalf("LogCommand failed: %v", err)
	}

	// Test ListLogs (All for host-1)
	logs, err := svc.ListLogs(ctx, "host-1", 0, 0)
	if err != nil {
		t.Fatalf("ListLogs failed: %v", err)
	}
	if len(logs) != 2 {
		t.Fatalf("expected 2 logs for host-1, got %d", len(logs))
	}
	if logs[0].Command != "cat missing.txt" {
		t.Fatalf("expected latest command to be 'cat missing.txt', got '%s'", logs[0].Command)
	}

	// Test ListLogs (Limit/Offset)
	logs, err = svc.ListLogs(ctx, "", 1, 1)
	if err != nil {
		t.Fatalf("ListLogs with pagination failed: %v", err)
	}
	if len(logs) != 1 {
		t.Fatalf("expected 1 log, got %d", len(logs))
	}

	// Test ClearLogs (Specific Host)
	if err := svc.ClearLogs(ctx, "host-1"); err != nil {
		t.Fatalf("ClearLogs failed: %v", err)
	}

	logs, _ = svc.ListLogs(ctx, "", 0, 0)
	if len(logs) != 1 {
		t.Fatalf("expected 1 log remaining, got %d", len(logs))
	}

	// Test ClearLogs (All)
	if err := svc.ClearLogs(ctx, ""); err != nil {
		t.Fatalf("ClearLogs all failed: %v", err)
	}

	logs, _ = svc.ListLogs(ctx, "", 0, 0)
	if len(logs) != 0 {
		t.Fatalf("expected 0 logs remaining, got %d", len(logs))
	}
}
