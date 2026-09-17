package main

import (
	"errors"
	"path/filepath"

	"caterm/internal/config"
	"github.com/wailsapp/wails/v2/pkg/runtime"
)

func (a *App) HardDeleteRecord(tableName, id string) error {
	if a.ctx == nil {
		return errors.New("context not initialized")
	}

	// This should be done securely through a service, but for G-01 scope
	// we'll implement it here to satisfy the requirement
	if tableName != "groups" && tableName != "hosts" && tableName != "snippets" {
		return errors.New("invalid table name")
	}

	query := "DELETE FROM " + tableName + " WHERE id = ?"
	_, err := a.db.ExecContext(a.ctx, query, id)
	return err
}

func (a *App) OpenDonationLink() {
	if a.ctx != nil {
		runtime.BrowserOpenURL(a.ctx, "https://github.com/sponsors/cecep-azhar")
	}
}

func (a *App) GetSyncPath() string {
	return filepath.Join(filepath.Dir(config.DataPath()), "sync")
}
