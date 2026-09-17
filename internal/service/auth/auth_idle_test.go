package auth

import (
	"context"
	"testing"
	"time"

	"caterm/internal/store"
	"caterm/internal/vault"
)

func TestIdleAutoLock(t *testing.T) {
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
	s := New(v, db)
	s.SetIdleTimeout(100 * time.Millisecond) // Short timeout for testing

	// Init and unlock
	err = s.Init(ctx, "QA_TEST_pw_12345678")
	if err != nil {
		t.Fatalf("Init failed: %v", err)
	}

	s.StartIdleTimer() // Should start the background timer
	defer s.StopIdleTimer()

	if !v.IsUnlocked() {
		t.Fatalf("Vault should be unlocked after Init")
	}

	// Wait longer than idle timeout
	time.Sleep(150 * time.Millisecond)

	if v.IsUnlocked() {
		t.Errorf("Vault should be locked after idle timeout")
	}

	// Unlock again and simulate activity
	err = s.Unlock(ctx, "QA_TEST_pw_12345678")
	if err != nil {
		t.Fatalf("Unlock failed: %v", err)
	}

	time.Sleep(50 * time.Millisecond)
	s.Activity()                     // Reset timer
	time.Sleep(80 * time.Millisecond) // Total 130ms, but timer was reset at 50ms

	if !v.IsUnlocked() {
		t.Errorf("Vault should still be unlocked because of activity")
	}

	time.Sleep(150 * time.Millisecond) // Now let it expire
	if v.IsUnlocked() {
		t.Errorf("Vault should be locked after second idle period")
	}
}
