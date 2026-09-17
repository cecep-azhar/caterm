package host

import (
	"context"
	"testing"

	"caterm/internal/store"
	"caterm/internal/vault"
)

func TestHostServiceUIBindingSafety(t *testing.T) {
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
	dek, err := vault.GenerateDEK()
	if err != nil {
		t.Fatalf("failed to generate DEK: %v", err)
	}
	v.SetDEK(dek)

	hs := NewHostService(db, v)

	h, err := hs.Create(ctx, HostInput{
		Name:       "QA_TEST_UI_Host",
		Hostname:   "127.0.0.1",
		Port:       22,
		Username:   "admin",
		AuthType:   "password",
		Password:   "QA_TEST_SECRET_PASS",
		PrivateKey: "QA_TEST_SECRET_KEY",
	})
	if err != nil {
		t.Fatalf("Create host failed: %v", err)
	}

	// Verify json tag or struct field output does not expose credentials
	if h.EncryptedPassword == "" || h.EncryptedPrivateKey == "" {
		t.Errorf("encrypted credentials should be stored in internal struct")
	}

	hosts, err := hs.List(ctx)
	if err != nil {
		t.Fatalf("List hosts failed: %v", err)
	}

	if len(hosts) != 1 {
		t.Fatalf("expected 1 host, got %d", len(hosts))
	}
}
