package main

import (
	"context"
	"errors"
	"path/filepath"

	"caterm/internal/config"
	"caterm/internal/service/auth"
	"caterm/internal/service/host"
	"caterm/internal/store"
	"caterm/internal/sync"
)

type App struct {
	ctx          context.Context
	authService  *auth.Service
	groupService *host.GroupService
	hostService  *host.HostService
	db           *store.DB
}

func NewApp(authService *auth.Service, groupService *host.GroupService, hostService *host.HostService, db *store.DB) *App {
	return &App{
		authService:  authService,
		groupService: groupService,
		hostService:  hostService,
		db:           db,
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
