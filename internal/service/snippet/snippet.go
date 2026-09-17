package snippet

import (
	"context"
	"database/sql"
	"errors"
	"time"

	"caterm/internal/store"
)

type Snippet struct {
	ID        string    `json:"id"`
	Name      string    `json:"name"`
	Body      string    `json:"body"`
	AutoEnter int       `json:"autoEnter"`
	ScopeType string    `json:"scopeType"`
	ScopeID   string    `json:"scopeId"`
	CreatedAt time.Time `json:"createdAt"`
	UpdatedAt time.Time `json:"updatedAt"`
}

type SnippetInput struct {
	Name      string `json:"name"`
	Body      string `json:"body"`
	AutoEnter int    `json:"autoEnter"`
	ScopeType string `json:"scopeType"`
	ScopeID   string `json:"scopeId"`
}

type SnippetService struct {
	db *store.DB
}

func NewSnippetService(db *store.DB) *SnippetService {
	return &SnippetService{db: db}
}

func (s *SnippetService) List(ctx context.Context) ([]*Snippet, error) {
	rows, err := s.db.QueryContext(ctx, "SELECT id, name, body, auto_enter, scope_type, scope_id, created_at, updated_at FROM snippets WHERE deleted_at IS NULL")
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var snippets []*Snippet
	for rows.Next() {
		var snip Snippet
		var scopeID sql.NullString
		var createdAt, updatedAt string

		if err := rows.Scan(&snip.ID, &snip.Name, &snip.Body, &snip.AutoEnter, &snip.ScopeType, &scopeID, &createdAt, &updatedAt); err != nil {
			return nil, err
		}

		snip.ScopeID = scopeID.String
		snip.CreatedAt, _ = time.Parse(time.RFC3339Nano, createdAt)
		snip.UpdatedAt, _ = time.Parse(time.RFC3339Nano, updatedAt)

		snippets = append(snippets, &snip)
	}
	return snippets, nil
}

func (s *SnippetService) Create(ctx context.Context, input SnippetInput) (*Snippet, error) {
	now := time.Now().UTC()
	snip := &Snippet{
		ID:        store.NewID(),
		Name:      input.Name,
		Body:      input.Body,
		AutoEnter: input.AutoEnter,
		ScopeType: input.ScopeType,
		ScopeID:   input.ScopeID,
		CreatedAt: now,
		UpdatedAt: now,
	}

	if snip.ScopeType == "" {
		snip.ScopeType = "global"
	}

	var scopeID interface{}
	if snip.ScopeID != "" {
		scopeID = snip.ScopeID
	}

	_, err := s.db.ExecContext(ctx,
		"INSERT INTO snippets (id, name, body, auto_enter, scope_type, scope_id, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
		snip.ID, snip.Name, snip.Body, snip.AutoEnter, snip.ScopeType, scopeID, snip.CreatedAt.Format(time.RFC3339Nano), snip.UpdatedAt.Format(time.RFC3339Nano))

	if err != nil {
		return nil, err
	}

	return snip, nil
}

func (s *SnippetService) Update(ctx context.Context, id string, input SnippetInput) (*Snippet, error) {
	now := time.Now().UTC()
	var scopeID interface{}
	if input.ScopeID != "" {
		scopeID = input.ScopeID
	}

	if input.ScopeType == "" {
		input.ScopeType = "global"
	}

	res, err := s.db.ExecContext(ctx,
		"UPDATE snippets SET name = ?, body = ?, auto_enter = ?, scope_type = ?, scope_id = ?, updated_at = ? WHERE id = ? AND deleted_at IS NULL",
		input.Name, input.Body, input.AutoEnter, input.ScopeType, scopeID, now.Format(time.RFC3339Nano), id)

	if err != nil {
		return nil, err
	}

	affected, _ := res.RowsAffected()
	if affected == 0 {
		return nil, errors.New("snippet not found")
	}

	var snip Snippet
	var qScopeID sql.NullString
	var createdAt, updatedAt string

	err = s.db.QueryRowContext(ctx, "SELECT id, name, body, auto_enter, scope_type, scope_id, created_at, updated_at FROM snippets WHERE id = ?", id).
		Scan(&snip.ID, &snip.Name, &snip.Body, &snip.AutoEnter, &snip.ScopeType, &qScopeID, &createdAt, &updatedAt)
	if err != nil {
		return nil, err
	}

	snip.ScopeID = qScopeID.String
	snip.CreatedAt, _ = time.Parse(time.RFC3339Nano, createdAt)
	snip.UpdatedAt, _ = time.Parse(time.RFC3339Nano, updatedAt)

	return &snip, nil
}

func (s *SnippetService) Delete(ctx context.Context, id string) error {
	now := time.Now().UTC().Format(time.RFC3339Nano)
	res, err := s.db.ExecContext(ctx, "UPDATE snippets SET deleted_at = ?, updated_at = ? WHERE id = ? AND deleted_at IS NULL", now, now, id)
	if err != nil {
		return err
	}
	affected, _ := res.RowsAffected()
	if affected == 0 {
		return errors.New("snippet not found")
	}
	return nil
}
