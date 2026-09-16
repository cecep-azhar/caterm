package auth

import (
	"context"
	"path/filepath"
	"testing"
	"time"

	"caterm/internal/store"
	"caterm/internal/vault"
)

func TestAuthService_InitAndUnlock(t *testing.T) {
	tmpDir := t.TempDir()
	dbPath := filepath.Join(tmpDir, "test.db")
	db, err := store.Open(dbPath)
	if err != nil {
		t.Fatalf("Open failed: %v", err)
	}
	defer db.Close()

	ctx := context.Background()
	if err := db.Migrate(ctx); err != nil {
		t.Fatalf("Migrate failed: %v", err)
	}

	v := vault.NewVault()
	svc := New(v, db)

	// Password short -> error
	if err := svc.Init(ctx, "short"); err == nil {
		t.Errorf("expected error for short password")
	}

	pw := "QA_TEST_pw_12345678"
	if err := svc.Init(ctx, pw); err != nil {
		t.Fatalf("Init failed: %v", err)
	}

	// Unlock wrong -> generic error
	if err := svc.Unlock(ctx, "bad_password_12345"); err != ErrWrongAuth {
		t.Errorf("expected ErrWrongAuth, got: %v", err)
	}

	// Rate limiting test: 5 bad attempts -> 6th fails with rate limit or delay
	for i := 0; i < 4; i++ {
		_ = svc.Unlock(ctx, "bad_password_12345")
	}

	start := time.Now()
	err = svc.Unlock(ctx, "bad_password_12345")
	if err != ErrRateLimited {
		t.Errorf("expected ErrRateLimited on 6th attempt, got: %v", err)
	}
	_ = start

	// Correct password after lock/reset or wait
	// In test, simulate time pass
	svc.lastFail = time.Now().Add(-6 * time.Second)
	if err := svc.Unlock(ctx, pw); err != nil {
		t.Errorf("Unlock failed with correct password: %v", err)
	}
}
