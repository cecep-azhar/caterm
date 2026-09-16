package main

import (
	"caterm/internal/service/auth"
	"context"
)

type App struct {
	ctx         context.Context
	authService *auth.Service
}

func NewApp(authService *auth.Service) *App {
	return &App{
		authService: authService,
	}
}

func (a *App) startup(ctx context.Context) {
	a.ctx = ctx
}

func (a *App) Greet(name string) string {
	return "Hello " + name
}
