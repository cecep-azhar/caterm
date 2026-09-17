package main

import (
	"errors"

	"caterm/internal/service/ai"
)

func (a *App) GetAIConfig() (*ai.Config, error) {
	if a.ctx == nil {
		return nil, errors.New("context not initialized")
	}
	return a.aiService.GetConfig(a.ctx)
}

func (a *App) SaveAIConfig(cfg *ai.Config) error {
	if a.ctx == nil {
		return errors.New("context not initialized")
	}
	return a.aiService.SaveConfig(a.ctx, cfg)
}

func (a *App) AskAI(prompt string) (string, error) {
	if a.ctx == nil {
		return "", errors.New("context not initialized")
	}
	return a.aiService.Ask(a.ctx, prompt)
}
