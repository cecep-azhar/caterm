package host

import (
	"context"
	"database/sql"
	"errors"
	"fmt"
	"time"

	"caterm/internal/store"
)

type Group struct {
	ID        string  `json:"id"`
	Name      string  `json:"name"`
	SortOrder int     `json:"sort_order"`
	CreatedAt string  `json:"created_at"`
	UpdatedAt string  `json:"updated_at"`
	DeletedAt *string `json:"deleted_at,omitempty"`
}

type GroupService struct {
	db *store.DB
}

func NewGroupService(db *store.DB) *GroupService {
	return &GroupService{db: db}
}

func (s *GroupService) Create(ctx context.Context, name string, sortOrder int) (*Group, error) {
	if name == "" {
		return nil, errors.New("group name is required")
	}

	id := store.NewID()
	now := store.NowUTC()

	_, err := s.db.ExecContext(ctx,
		"INSERT INTO groups (id, name, sort_order, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
		id, name, sortOrder, now, now,
	)
	if err != nil {
		return nil, fmt.Errorf("failed to insert group: %w", err)
	}

	return &Group{
		ID:        id,
		Name:      name,
		SortOrder: sortOrder,
		CreatedAt: now,
		UpdatedAt: now,
	}, nil
}

func (s *GroupService) Update(ctx context.Context, id, name string, sortOrder int) (*Group, error) {
	time.Sleep(2 * time.Millisecond)
	now := store.NowUTC()
	res, err := s.db.ExecContext(ctx,
		"UPDATE groups SET name = ?, sort_order = ?, updated_at = ? WHERE id = ? AND deleted_at IS NULL",
		name, sortOrder, now, id,
	)
	if err != nil {
		return nil, err
	}
	rows, err := res.RowsAffected()
	if err != nil || rows == 0 {
		return nil, errors.New("group not found or deleted")
	}

	return s.GetByID(ctx, id)
}

func (s *GroupService) Delete(ctx context.Context, id string) error {
	now := store.NowUTC()
	res, err := s.db.ExecContext(ctx,
		"UPDATE groups SET deleted_at = ?, updated_at = ? WHERE id = ? AND deleted_at IS NULL",
		now, now, id,
	)
	if err != nil {
		return err
	}
	rows, err := res.RowsAffected()
	if err != nil || rows == 0 {
		return errors.New("group not found or already deleted")
	}
	return nil
}

func (s *GroupService) GetByID(ctx context.Context, id string) (*Group, error) {
	row := s.db.QueryRowContext(ctx, "SELECT id, name, sort_order, created_at, updated_at, deleted_at FROM groups WHERE id = ?", id)
	var g Group
	var delAt sql.NullString
	if err := row.Scan(&g.ID, &g.Name, &g.SortOrder, &g.CreatedAt, &g.UpdatedAt, &delAt); err != nil {
		return nil, err
	}
	if delAt.Valid {
		g.DeletedAt = &delAt.String
	}
	return &g, nil
}

func (s *GroupService) List(ctx context.Context) ([]*Group, error) {
	rows, err := s.db.QueryContext(ctx, "SELECT id, name, sort_order, created_at, updated_at FROM groups WHERE deleted_at IS NULL ORDER BY sort_order ASC, name ASC")
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var groups []*Group
	for rows.Next() {
		var g Group
		if err := rows.Scan(&g.ID, &g.Name, &g.SortOrder, &g.CreatedAt, &g.UpdatedAt); err != nil {
			return nil, err
		}
		groups = append(groups, &g)
	}
	return groups, nil
}
