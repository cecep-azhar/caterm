package main

import (
	"context"
	"os"
	"path/filepath"
	"testing"

	"caterm/internal/service/audit"
	"caterm/internal/service/auth"
	"caterm/internal/service/host"
	"caterm/internal/service/snippet"
	"caterm/internal/service/sshkey"
	"caterm/internal/service/team"
	"caterm/internal/service/terminal"
	"caterm/internal/store"
	"caterm/internal/vault"
)

func TestSettingsHardDelete(t *testing.T) {
	tmpDataDir := t.TempDir()
	os.Setenv("CATERM_DATA_DIR", tmpDataDir)
	defer os.Unsetenv("CATERM_DATA_DIR")

	dbPath := filepath.Join(tmpDataDir, "caterm.db")
	db, err := store.Open(dbPath)
	if err != nil {
		t.Fatalf("failed to open test db: %v", err)
	}
	defer db.Close()

	ctx := context.Background()
	_ = db.Migrate(ctx)

	v := vault.NewVault()
	authService := auth.New(v, db)
	groupService := host.NewGroupService(db)
	hostService := host.NewHostService(db, v)
	snippetService := snippet.NewSnippetService(db)
	teamService := team.NewTeamService(db)
	sshKeyService := sshkey.NewService(db)
	auditService := audit.NewService(db)

	terminalService := terminal.NewService()
	app := NewApp(authService, groupService, hostService, snippetService, teamService, sshKeyService, auditService, terminalService, db)
	app.startup(ctx)

	// Insert a test group
	grp, err := groupService.Create(ctx, "QA_TEST_Grp", 1)
	if err != nil {
		t.Fatalf("failed to create group: %v", err)
	}

	// Soft delete first
	err = groupService.Delete(ctx, grp.ID)
	if err != nil {
		t.Fatalf("failed to soft delete group: %v", err)
	}

	// Hard delete via settings binding
	err = app.HardDeleteRecord("groups", grp.ID)
	if err != nil {
		t.Fatalf("failed to hard delete group: %v", err)
	}

	// Verify completely gone
	var count int
	err = db.QueryRow("SELECT COUNT(*) FROM groups WHERE id = ?", grp.ID).Scan(&count)
	if err != nil {
		t.Fatalf("query failed: %v", err)
	}
	if count != 0 {
		t.Fatalf("record was not hard deleted, count=%d", count)
	}
}
