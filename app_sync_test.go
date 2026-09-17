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

func TestAppSyncReviewPanelNoAutoApply(t *testing.T) {
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

	terminalService := terminal.New()
	app := NewApp(authService, groupService, hostService, snippetService, teamService, sshKeyService, auditService, terminalService, db)
	app.startup(ctx)

	// Hash DB before
	beforeStat, _ := os.Stat(dbPath)
	beforeSize := beforeStat.Size()

	// Check pending sync (dry run)
	res, err := app.CheckSyncPending()
	if err != nil {
		t.Fatalf("CheckSyncPending failed: %v", err)
	}
	_ = res

	// Hash DB after CheckSyncPending -> DB must remain untouched!
	afterStat, _ := os.Stat(dbPath)
	afterSize := afterStat.Size()

	if beforeSize != afterSize {
		t.Fatalf("DB size changed after CheckSyncPending! before=%d, after=%d", beforeSize, afterSize)
	}
}
