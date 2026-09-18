package hostkey

import (
	"context"
	"database/sql"
	"time"

	"caterm/internal/store"
	"github.com/google/uuid"
)

type Store struct {
	db *store.DB
}

func NewStore(db *store.DB) *Store {
	return &Store{db: db}
}

func (s *Store) GetHostKey(ctx context.Context, hostname string, port int, keyType string) (string, error) {
	var fingerprint string
	err := s.db.QueryRowContext(ctx, `
		SELECT fingerprint FROM host_keys 
		WHERE hostname = ? AND port = ? AND key_type = ? AND deleted_at IS NULL
	`, hostname, port, keyType).Scan(&fingerprint)

	if err == sql.ErrNoRows {
		return "", nil
	}
	if err != nil {
		return "", err
	}
	return fingerprint, nil
}

func (s *Store) SaveHostKey(ctx context.Context, hostname string, port int, keyType string, fingerprint string) error {
	now := time.Now().UTC().Format(time.RFC3339Nano)
	id := uuid.New().String()

	_, err := s.db.ExecContext(ctx, `
		INSERT INTO host_keys (id, hostname, port, key_type, fingerprint, created_at, updated_at)
		VALUES (?, ?, ?, ?, ?, ?, ?)
		ON CONFLICT(hostname, port, key_type) DO UPDATE SET
			fingerprint = excluded.fingerprint,
			updated_at = excluded.updated_at,
			deleted_at = NULL
	`, id, hostname, port, keyType, fingerprint, now, now)

	return err
}

func (s *Store) DeleteHostKey(ctx context.Context, hostname string, port int, keyType string) error {
	now := time.Now().UTC().Format(time.RFC3339Nano)
	_, err := s.db.ExecContext(ctx, `
		UPDATE host_keys SET deleted_at = ? WHERE hostname = ? AND port = ? AND key_type = ?
	`, now, hostname, port, keyType)
	return err
}
