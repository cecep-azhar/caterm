package sshkey

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

func TestSSHKeyService(t *testing.T) {
	db := setupTestDB(t)
	defer db.Close()

	svc := NewService(db)
	ctx := context.Background()

	// Test GenerateKey
	key, err := svc.GenerateKey(ctx, "test-key", 2048)
	if err != nil {
		t.Fatalf("GenerateKey failed: %v", err)
	}
	if key.Name != "test-key" || key.Fingerprint == "" || key.PublicKey == "" || key.PrivateKey == "" {
		t.Fatalf("Invalid generated key fields: %+v", key)
	}

	// Test ListKeys
	keys, err := svc.ListKeys(ctx)
	if err != nil {
		t.Fatalf("ListKeys failed: %v", err)
	}
	if len(keys) != 1 {
		t.Fatalf("expected 1 key, got %d", len(keys))
	}

	// Test GetKey
	gotKey, err := svc.GetKey(ctx, key.ID)
	if err != nil {
		t.Fatalf("GetKey failed: %v", err)
	}
	if gotKey.PrivateKey != key.PrivateKey {
		t.Fatalf("GetKey returned wrong private key")
	}

	// Test DeleteKey
	if err := svc.DeleteKey(ctx, key.ID); err != nil {
		t.Fatalf("DeleteKey failed: %v", err)
	}

	keys, err = svc.ListKeys(ctx)
	if err != nil {
		t.Fatalf("ListKeys failed: %v", err)
	}
	if len(keys) != 0 {
		t.Fatalf("expected 0 keys after deletion, got %d", len(keys))
	}
}
