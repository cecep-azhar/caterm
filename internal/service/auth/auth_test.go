package auth

import (
	"context"
	"testing"

	"caterm/internal/store"
	"caterm/internal/vault"
)

func TestChangePassword(t *testing.T) {
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

	v := vault.NewVault()
	svc := New(v, db)

	oldPw := "QA_TEST_pw_old_123456"
	newPw := "QA_TEST_pw_new_123456"

	if err := svc.Init(ctx, oldPw); err != nil {
		t.Fatalf("Init failed: %v", err)
	}

	if err := svc.ChangePassword(ctx, oldPw, newPw); err != nil {
		t.Fatalf("ChangePassword failed: %v", err)
	}

	v2 := vault.NewVault()
	svc2 := New(v2, db)

	if err := svc2.Unlock(ctx, oldPw); err == nil {
		t.Errorf("expected unlock with old password to fail, but succeeded")
	}

	v3 := vault.NewVault()
	svc3 := New(v3, db)
	if err := svc3.Unlock(ctx, newPw); err != nil {
		t.Errorf("Unlock with new password failed: %v", err)
	}
}
