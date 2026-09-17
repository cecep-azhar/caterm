package main

import (
	"errors"

	"caterm/internal/service/snippet"
	"caterm/internal/service/team"
)

// Snippet Bindings

func (a *App) ListSnippets() ([]*snippet.Snippet, error) {
	if a.ctx == nil {
		return nil, errors.New("context not initialized")
	}
	return a.snippetService.List(a.ctx)
}

func (a *App) CreateSnippet(input snippet.SnippetInput) (*snippet.Snippet, error) {
	if a.ctx == nil {
		return nil, errors.New("context not initialized")
	}
	return a.snippetService.Create(a.ctx, input)
}

func (a *App) UpdateSnippet(id string, input snippet.SnippetInput) (*snippet.Snippet, error) {
	if a.ctx == nil {
		return nil, errors.New("context not initialized")
	}
	return a.snippetService.Update(a.ctx, id, input)
}

func (a *App) DeleteSnippet(id string) error {
	if a.ctx == nil {
		return errors.New("context not initialized")
	}
	return a.snippetService.Delete(a.ctx, id)
}

// Team Bindings

func (a *App) ListTeams() ([]*team.Team, error) {
	if a.ctx == nil {
		return nil, errors.New("context not initialized")
	}
	return a.teamService.List(a.ctx)
}

func (a *App) CreateTeam(input team.TeamInput) (*team.Team, error) {
	if a.ctx == nil {
		return nil, errors.New("context not initialized")
	}
	return a.teamService.Create(a.ctx, input)
}

func (a *App) UpdateTeam(id string, input team.TeamInput) (*team.Team, error) {
	if a.ctx == nil {
		return nil, errors.New("context not initialized")
	}
	return a.teamService.Update(a.ctx, id, input)
}

func (a *App) DeleteTeam(id string) error {
	if a.ctx == nil {
		return errors.New("context not initialized")
	}
	return a.teamService.Delete(a.ctx, id)
}
