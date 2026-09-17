package team

import (
	"context"
	"errors"
	"time"

	"caterm/internal/store"
)

type Team struct {
	ID        string    `json:"id"`
	Name      string    `json:"name"`
	CreatedAt time.Time `json:"createdAt"`
	UpdatedAt time.Time `json:"updatedAt"`
}

type TeamInput struct {
	Name string `json:"name"`
}

type TeamService struct {
	db *store.DB
}

func NewTeamService(db *store.DB) *TeamService {
	return &TeamService{db: db}
}

func (s *TeamService) List(ctx context.Context) ([]*Team, error) {
	rows, err := s.db.QueryContext(ctx, "SELECT id, name, created_at, updated_at FROM teams WHERE deleted_at IS NULL")
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var teams []*Team
	for rows.Next() {
		var t Team
		var createdAt, updatedAt string

		if err := rows.Scan(&t.ID, &t.Name, &createdAt, &updatedAt); err != nil {
			return nil, err
		}

		t.CreatedAt, _ = time.Parse(time.RFC3339Nano, createdAt)
		t.UpdatedAt, _ = time.Parse(time.RFC3339Nano, updatedAt)

		teams = append(teams, &t)
	}
	return teams, nil
}

func (s *TeamService) Create(ctx context.Context, input TeamInput) (*Team, error) {
	now := time.Now().UTC()
	t := &Team{
		ID:        store.NewID(),
		Name:      input.Name,
		CreatedAt: now,
		UpdatedAt: now,
	}

	_, err := s.db.ExecContext(ctx,
		"INSERT INTO teams (id, name, created_at, updated_at) VALUES (?, ?, ?, ?)",
		t.ID, t.Name, t.CreatedAt.Format(time.RFC3339Nano), t.UpdatedAt.Format(time.RFC3339Nano))

	if err != nil {
		return nil, err
	}

	return t, nil
}

func (s *TeamService) Update(ctx context.Context, id string, input TeamInput) (*Team, error) {
	now := time.Now().UTC()

	res, err := s.db.ExecContext(ctx,
		"UPDATE teams SET name = ?, updated_at = ? WHERE id = ? AND deleted_at IS NULL",
		input.Name, now.Format(time.RFC3339Nano), id)

	if err != nil {
		return nil, err
	}

	affected, _ := res.RowsAffected()
	if affected == 0 {
		return nil, errors.New("team not found")
	}

	var t Team
	var createdAt, updatedAt string

	err = s.db.QueryRowContext(ctx, "SELECT id, name, created_at, updated_at FROM teams WHERE id = ?", id).
		Scan(&t.ID, &t.Name, &createdAt, &updatedAt)
	if err != nil {
		return nil, err
	}

	t.CreatedAt, _ = time.Parse(time.RFC3339Nano, createdAt)
	t.UpdatedAt, _ = time.Parse(time.RFC3339Nano, updatedAt)

	return &t, nil
}

func (s *TeamService) Delete(ctx context.Context, id string) error {
	now := time.Now().UTC().Format(time.RFC3339Nano)
	res, err := s.db.ExecContext(ctx, "UPDATE teams SET deleted_at = ?, updated_at = ? WHERE id = ? AND deleted_at IS NULL", now, now, id)
	if err != nil {
		return err
	}
	affected, _ := res.RowsAffected()
	if affected == 0 {
		return errors.New("team not found")
	}
	return nil
}
