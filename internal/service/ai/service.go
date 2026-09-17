package ai

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"time"

	"caterm/internal/store"
)

type Config struct {
	Provider string `json:"provider"`
	Model    string `json:"model"`
	BaseURL  string `json:"base_url"`
	APIKey   string `json:"api_key"`
}

type Service struct {
	db *store.DB
}

func NewService(db *store.DB) *Service {
	return &Service{db: db}
}

func (s *Service) GetConfig(ctx context.Context) (*Config, error) {
	cfg := &Config{
		Provider: "9router",
		Model:    "ZA126_PRO",
		BaseURL:  "http://100.76.150.46:3007/v1",
		APIKey:   "",
	}

	rows, err := s.db.QueryContext(ctx, "SELECT key, value FROM settings WHERE key LIKE 'ai_%'")
	if err != nil {
		return cfg, nil
	}
	defer rows.Close()

	for rows.Next() {
		var k, v string
		if err := rows.Scan(&k, &v); err == nil {
			switch k {
			case "ai_provider":
				cfg.Provider = v
			case "ai_model":
				cfg.Model = v
			case "ai_base_url":
				cfg.BaseURL = v
			case "ai_api_key":
				cfg.APIKey = v
			}
		}
	}

	return cfg, nil
}

func (s *Service) SaveConfig(ctx context.Context, cfg *Config) error {
	queries := []struct {
		k, v string
	}{
		{"ai_provider", cfg.Provider},
		{"ai_model", cfg.Model},
		{"ai_base_url", cfg.BaseURL},
		{"ai_api_key", cfg.APIKey},
	}

	for _, q := range queries {
		_, err := s.db.ExecContext(ctx, `INSERT INTO settings (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value=excluded.value`, q.k, q.v)
		if err != nil {
			return err
		}
	}
	return nil
}

type chatMessage struct {
	Role    string `json:"role"`
	Content string `json:"content"`
}

type chatRequest struct {
	Model    string        `json:"model"`
	Messages []chatMessage `json:"messages"`
}

type chatResponse struct {
	Choices []struct {
		Message chatMessage `json:"message"`
	} `json:"choices"`
}

func (s *Service) Ask(ctx context.Context, prompt string) (string, error) {
	cfg, err := s.GetConfig(ctx)
	if err != nil {
		return "", err
	}

	if cfg.BaseURL == "" {
		cfg.BaseURL = "http://100.76.150.46:3007/v1"
	}
	if cfg.Model == "" {
		cfg.Model = "ZA126_PRO"
	}

	reqBody := chatRequest{
		Model: cfg.Model,
		Messages: []chatMessage{
			{
				Role:    "system",
				Content: "You are CATerm AI Terminal Assistant. Help sysadmins with bash/shell commands. Provide short, concise explanations and executable commands.",
			},
			{
				Role:    "user",
				Content: prompt,
			},
		},
	}

	data, err := json.Marshal(reqBody)
	if err != nil {
		return "", err
	}

	url := fmt.Sprintf("%s/chat/completions", cfg.BaseURL)
	req, err := http.NewRequestWithContext(ctx, "POST", url, bytes.NewBuffer(data))
	if err != nil {
		return "", err
	}

	req.Header.Set("Content-Type", "application/json")
	if cfg.APIKey != "" {
		req.Header.Set("Authorization", fmt.Sprintf("Bearer %s", cfg.APIKey))
	}

	client := &http.Client{Timeout: 30 * time.Second}
	resp, err := client.Do(req)
	if err != nil {
		return "", fmt.Errorf("AI Provider error: %w", err)
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return "", err
	}

	if resp.StatusCode >= 400 {
		return "", fmt.Errorf("AI API returned status %d: %s", resp.StatusCode, string(body))
	}

	var chatResp chatResponse
	if err := json.Unmarshal(body, &chatResp); err != nil {
		return "", fmt.Errorf("failed to parse AI response: %w", err)
	}

	if len(chatResp.Choices) == 0 {
		return "No response from AI provider.", nil
	}

	return chatResp.Choices[0].Message.Content, nil
}
