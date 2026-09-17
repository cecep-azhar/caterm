package sync

import (
	"context"
	"os"
	"path/filepath"
	"testing"

	"caterm/internal/service/host"
	"caterm/internal/store"
	"caterm/internal/vault"
)

func TestCrossProfileRoundTrip(t *testing.T) {
	dirA := t.TempDir() + "/profA"
	dirB := t.TempDir() + "/profB"

	dbPathA := filepath.Join(dirA, "caterm.db")
	syncDirA := filepath.Join(dirA, "sync")

	dbPathB := filepath.Join(dirB, "caterm.db")
	syncDirB := filepath.Join(dirB, "sync")

	password := "QA_TEST_master_password_123"

	// 1. Setup Profile A & Create Host
	dbA, err := store.Open(dbPathA)
	if err != nil {
		t.Fatalf("Open A failed: %v", err)
	}
	ctx := context.Background()
	if err := dbA.Migrate(ctx); err != nil {
		t.Fatalf("Migrate A failed: %v", err)
	}

	vA := vault.NewVault()
	if err := vA.Init(ctx, dbA, password); err != nil {
		t.Fatalf("Init A failed: %v", err)
	}

	hsA := host.NewHostService(dbA, vA)
	hA, err := hsA.Create(ctx, host.HostInput{
		Name:       "QA_TEST_CrossHost",
		Hostname:   "1.2.3.4",
		Username:   "root",
		AuthType:   "password",
		Password:   "QA_TEST_HOST_SECRET_PASS",
		PrivateKey: "QA_TEST_HOST_SECRET_KEY",
	})
	if err != nil {
		t.Fatalf("Create Host A failed: %v", err)
	}

	// 2. Export Profile A
	expA := NewExporter(dbA, syncDirA)
	if err := expA.Export(ctx); err != nil {
		t.Fatalf("Export A failed: %v", err)
	}
	dbA.Close()

	// 3. Copy sync directory from A to B
	files, err := getFiles(syncDirA)
	if err != nil {
		t.Fatalf("getFiles syncDirA failed: %v", err)
	}
	for rel, content := range files {
		target := filepath.Join(syncDirB, rel)
		os.MkdirAll(filepath.Dir(target), 0700)
		if err := atomicWriteJSON(target, jsonRaw(content)); err != nil {
			t.Fatalf("failed to copy file to sync B: %v", err)
		}
	}

	// 4. Setup Profile B (Empty DB)
	dbB, err := store.Open(dbPathB)
	if err != nil {
		t.Fatalf("Open B failed: %v", err)
	}
	defer dbB.Close()
	if err := dbB.Migrate(ctx); err != nil {
		t.Fatalf("Migrate B failed: %v", err)
	}

	// Import sync data to B
	impB := NewImporter(dbB, syncDirB)
	resB, err := impB.Import(ctx, false)
	if err != nil {
		t.Fatalf("Import B failed: %v", err)
	}
	if resB.Added < 1 {
		t.Errorf("expected at least 1 added record in profile B")
	}

	// 5. Unlock Profile B with correct password and verify credentials
	vB := vault.NewVault()
	if err := vB.Unlock(ctx, dbB, password); err != nil {
		t.Fatalf("Unlock B with correct password failed: %v", err)
	}

	hsB := host.NewHostService(dbB, vB)
	pwdB, pkB, err := hsB.GetCredentialsForDial(ctx, hA.ID)
	if err != nil {
		t.Fatalf("GetCredentialsForDial B failed: %v", err)
	}

	if pwdB != "QA_TEST_HOST_SECRET_PASS" {
		t.Errorf("expected password QA_TEST_HOST_SECRET_PASS, got %s", pwdB)
	}
	if pkB != "QA_TEST_HOST_SECRET_KEY" {
		t.Errorf("expected private key QA_TEST_HOST_SECRET_KEY, got %s", pkB)
	}

	// 6. Test wrong password on Profile B
	vBWrong := vault.NewVault()
	if err := vBWrong.Unlock(ctx, dbB, "QA_TEST_wrong_password_999"); err != vault.ErrWrongPassword {
		t.Errorf("expected ErrWrongPassword for wrong password, got %v", err)
	}
}

type jsonRaw string

func (j jsonRaw) MarshalJSON() ([]byte, error) {
	return []byte(j), nil
}
