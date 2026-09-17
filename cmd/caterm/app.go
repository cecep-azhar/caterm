package main

import (
	"caterm/internal/service/auth"
	"caterm/internal/service/host"
	"context"
)

type App struct {
	ctx          context.Context
	authService  *auth.Service
	groupService *host.GroupService
	hostService  *host.HostService
}

func NewApp(authService *auth.Service, groupService *host.GroupService, hostService *host.HostService) *App {
	return &App{
		authService:  authService,
		groupService: groupService,
		hostService:  hostService,
	}
}

func (a *App) startup(ctx context.Context) {
	a.ctx = ctx
}

func (a *App) Greet(name string) string {
	return "Hello " + name
}
