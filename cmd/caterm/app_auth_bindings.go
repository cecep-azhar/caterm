package main

import (
	"errors"
)

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
