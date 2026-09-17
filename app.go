package main

import (
	"context"
	"errors"
	"path/filepath"

	"caterm/internal/config"
	"caterm/internal/service/audit"
	"caterm/internal/service/auth"
	"caterm/internal/service/host"
	"caterm/internal/service/snippet"
	"caterm/internal/service/sshkey"
	"caterm/internal/service/team"
	"caterm/internal/store"
	"caterm/internal/sync"
)

type App struct {
	ctx            context.Context
	authService    *auth.Service
	groupService   *host.GroupService
	hostService    *host.HostService
	snippetService *snippet.SnippetService
	teamService    *team.TeamService
	sshKeyService  *sshkey.Service
	auditService   *audit.Service
	db             *store.DB
}

func NewApp(
	authService *auth.Service,
	groupService *host.GroupService,
	hostService *host.HostService,
	snippetService *snippet.SnippetService,
	teamService *team.TeamService,
	sshKeyService *sshkey.Service,
	auditService *audit.Service,
	db *store.DB,
) *App {
	return &App{
		authService:    authService,
		groupService:   groupService,
		hostService:    hostService,
		snippetService: snippetService,
		teamService:    teamService,
		sshKeyService:  sshKeyService,
		auditService:   auditService,
		db:             db,
	}
}

func (a *App) startup(ctx context.Context) {
	a.ctx = ctx
}

func (a *App) Greet(name string) string {
	return "Hello " + name
}

func (a *App) IsInitialized() (bool, error) {
	if a.ctx == nil {
		return false, errors.New("context not initialized")
	}
	return a.authService.IsInitialized(a.ctx)
}

func (a *App) Setup(password string) error {
	if a.ctx == nil {
		return errors.New("context not initialized")
	}
	return a.authService.Init(a.ctx, password)
}

func (a *App) Unlock(password string) error {
	if a.ctx == nil {
		return errors.New("context not initialized")
	}
	return a.authService.Unlock(a.ctx, password)
}

func (a *App) ChangePassword(oldPassword, newPassword string) error {
	if a.ctx == nil {
		return errors.New("context not initialized")
	}
	return a.authService.ChangePassword(a.ctx, oldPassword, newPassword)
}

func (a *App) CheckSyncPending() (*sync.MergeResult, error) {
	syncDir := filepath.Join(filepath.Dir(config.DataPath()), "sync")
	importer := sync.NewImporter(a.db, syncDir)
	return importer.Import(a.ctx, true)
}

func (a *App) ApplySync() (*sync.MergeResult, error) {
	syncDir := filepath.Join(filepath.Dir(config.DataPath()), "sync")
	importer := sync.NewImporter(a.db, syncDir)
	return importer.Import(a.ctx, false)
}

func (a *App) ResetVault() error {
	if a.ctx == nil {
		return errors.New("context not initialized")
	}
	
	_, err := a.db.ExecContext(a.ctx, "DELETE FROM vault_meta; DELETE FROM groups; DELETE FROM hosts;")
	return err
}

// SSH Key Bindings
func (a *App) GenerateSSHKey(name string, bits int) (*sshkey.SSHKey, error) {
	return a.sshKeyService.GenerateKey(a.ctx, name, bits)
}

func (a *App) AddSSHKey(name, privateKeyStr string) (*sshkey.SSHKey, error) {
	return a.sshKeyService.AddKey(a.ctx, name, privateKeyStr)
}

func (a *App) ListSSHKeys() ([]sshkey.SSHKey, error) {
	return a.sshKeyService.ListKeys(a.ctx)
}

func (a *App) DeleteSSHKey(id string) error {
	return a.sshKeyService.DeleteKey(a.ctx, id)
}

func (a *App) GetSSHKey(id string) (*sshkey.SSHKey, error) {
	return a.sshKeyService.GetKey(a.ctx, id)
}

// Audit Log Bindings
func (a *App) LogCommand(hostID, command, output string, exitCode int) error {
	return a.auditService.LogCommand(a.ctx, hostID, command, output, exitCode)
}

func (a *App) ListAuditLogs(hostID string, limit, offset int) ([]audit.AuditLog, error) {
	return a.auditService.ListLogs(a.ctx, hostID, limit, offset)
}

func (a *App) ClearAuditLogs(hostID string) error {
	return a.auditService.ClearLogs(a.ctx, hostID)
}
