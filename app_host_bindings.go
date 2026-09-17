package main

import (
	"errors"

	"caterm/internal/service/host"
)

func (a *App) ListGroups() ([]*host.Group, error) {
	if a.ctx == nil {
		return nil, errors.New("context not initialized")
	}
	return a.groupService.List(a.ctx)
}

func (a *App) CreateGroup(name string, sortOrder int) (*host.Group, error) {
	if a.ctx == nil {
		return nil, errors.New("context not initialized")
	}
	return a.groupService.Create(a.ctx, name, sortOrder)
}

func (a *App) UpdateGroup(id, name string, sortOrder int) (*host.Group, error) {
	if a.ctx == nil {
		return nil, errors.New("context not initialized")
	}
	return a.groupService.Update(a.ctx, id, name, sortOrder)
}

func (a *App) DeleteGroup(id string) error {
	if a.ctx == nil {
		return errors.New("context not initialized")
	}
	return a.groupService.Delete(a.ctx, id)
}

func (a *App) ListHosts() ([]*host.Host, error) {
	if a.ctx == nil {
		return nil, errors.New("context not initialized")
	}
	return a.hostService.List(a.ctx)
}

func (a *App) CreateHost(input host.HostInput) (*host.Host, error) {
	if a.ctx == nil {
		return nil, errors.New("context not initialized")
	}
	return a.hostService.Create(a.ctx, input)
}

func (a *App) UpdateHost(id string, input host.HostInput) (*host.Host, error) {
	if a.ctx == nil {
		return nil, errors.New("context not initialized")
	}
	return a.hostService.Update(a.ctx, id, input)
}

func (a *App) DeleteHost(id string) error {
	if a.ctx == nil {
		return errors.New("context not initialized")
	}
	return a.hostService.Delete(a.ctx, id)
}
