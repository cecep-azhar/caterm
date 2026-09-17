package host

import (
	"context"
	"database/sql"
	"errors"
	"fmt"
	"time"

	"caterm/internal/store"
	"caterm/internal/vault"
)

type Host struct {
	ID        string  `json:"id"`
	GroupID   *string `json:"group_id,omitempty"`
	Name      string  `json:"name"`
	Hostname  string  `json:"hostname"`
	Port      int     `json:"port"`
	Username  string  `json:"username"`
	AuthType  string  `json:"auth_type"`
	CreatedAt string  `json:"created_at"`
	UpdatedAt string  `json:"updated_at"`
	DeletedAt *string `json:"deleted_at,omitempty"`

	// Credentials are never sent to frontend in lists
	EncryptedPassword   string `json:"-"`
	EncryptedPrivateKey string `json:"-"`
}

// HostInput is used for creating/updating hosts
type HostInput struct {
	GroupID  *string `json:"group_id"`
	Name     string  `json:"name"`
	Hostname string  `json:"hostname"`
	Port     int     `json:"port"`
	Username string  `json:"username"`
	AuthType string  `json:"auth_type"`

	// Plain credentials from UI, encrypted before save
	Password   string `json:"password"`
	PrivateKey string `json:"private_key"`
}

type HostService struct {
	db *store.DB
	v  *vault.Vault
}

func NewHostService(db *store.DB, v *vault.Vault) *HostService {
	return &HostService{db: db, v: v}
}

func (s *HostService) Create(ctx context.Context, input HostInput) (*Host, error) {
	if input.Name == "" || input.Hostname == "" || input.Username == "" {
		return nil, errors.New("name, hostname, and username are required")
	}
	if input.Port <= 0 {
		input.Port = 22
	}

	if !s.v.IsUnlocked() {
		return nil, vault.ErrLocked
	}

	id := store.NewID()
	now := store.NowUTC()

	pwdEnc, pkEnc := "", ""
	var err error

	if input.Password != "" {
		pwdEnc, err = s.v.EncryptField(input.Password, id, "encrypted_password")
		if err != nil {
			return nil, fmt.Errorf("failed to encrypt password: %w", err)
		}
	}
	if input.PrivateKey != "" {
		pkEnc, err = s.v.EncryptField(input.PrivateKey, id, "encrypted_private_key")
		if err != nil {
			return nil, fmt.Errorf("failed to encrypt private key: %w", err)
		}
	}

	_, err = s.db.ExecContext(ctx,
		`INSERT INTO hosts (
			id, group_id, name, hostname, port, username, auth_type,
			encrypted_password, encrypted_private_key, created_at, updated_at
		) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`,
		id, input.GroupID, input.Name, input.Hostname, input.Port, input.Username, input.AuthType,
		pwdEnc, pkEnc, now, now,
	)
	if err != nil {
		return nil, fmt.Errorf("failed to insert host: %w", err)
	}

	return s.GetByID(ctx, id)
}

func (s *HostService) Update(ctx context.Context, id string, input HostInput) (*Host, error) {
	if input.Name == "" || input.Hostname == "" || input.Username == "" {
		return nil, errors.New("name, hostname, and username are required")
	}
	if input.Port <= 0 {
		input.Port = 22
	}

	if !s.v.IsUnlocked() {
		return nil, vault.ErrLocked
	}

	time.Sleep(2 * time.Millisecond)
	now := store.NowUTC()

	existing, err := s.GetByID(ctx, id)
	if err != nil {
		return nil, err
	}

	pwdEnc, pkEnc := existing.EncryptedPassword, existing.EncryptedPrivateKey

	if input.Password != "" {
		pwdEnc, err = s.v.EncryptField(input.Password, id, "encrypted_password")
		if err != nil {
			return nil, fmt.Errorf("failed to encrypt password: %w", err)
		}
	}
	if input.PrivateKey != "" {
		pkEnc, err = s.v.EncryptField(input.PrivateKey, id, "encrypted_private_key")
		if err != nil {
			return nil, fmt.Errorf("failed to encrypt private key: %w", err)
		}
	}

	res, err := s.db.ExecContext(ctx,
		`UPDATE hosts SET 
			group_id = ?, name = ?, hostname = ?, port = ?, username = ?, auth_type = ?,
			encrypted_password = ?, encrypted_private_key = ?, updated_at = ? 
		WHERE id = ? AND deleted_at IS NULL`,
		input.GroupID, input.Name, input.Hostname, input.Port, input.Username, input.AuthType,
		pwdEnc, pkEnc, now, id,
	)
	if err != nil {
		return nil, err
	}
	rows, err := res.RowsAffected()
	if err != nil || rows == 0 {
		return nil, errors.New("host not found or deleted")
	}

	return s.GetByID(ctx, id)
}

func (s *HostService) Delete(ctx context.Context, id string) error {
	now := store.NowUTC()
	res, err := s.db.ExecContext(ctx,
		"UPDATE hosts SET deleted_at = ?, updated_at = ? WHERE id = ? AND deleted_at IS NULL",
		now, now, id,
	)
	if err != nil {
		return err
	}
	rows, err := res.RowsAffected()
	if err != nil || rows == 0 {
		return errors.New("host not found or already deleted")
	}
	return nil
}

func (s *HostService) GetByID(ctx context.Context, id string) (*Host, error) {
	row := s.db.QueryRowContext(ctx, `
		SELECT id, group_id, name, hostname, port, username, auth_type,
		encrypted_password, encrypted_private_key, created_at, updated_at, deleted_at 
		FROM hosts WHERE id = ?`, id)
	return scanHost(row)
}

func (s *HostService) List(ctx context.Context) ([]*Host, error) {
	rows, err := s.db.QueryContext(ctx, `
		SELECT id, group_id, name, hostname, port, username, auth_type,
		encrypted_password, encrypted_private_key, created_at, updated_at, deleted_at 
		FROM hosts WHERE deleted_at IS NULL ORDER BY name ASC`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var hosts []*Host
	for rows.Next() {
		h, err := scanHost(rows)
		if err != nil {
			return nil, err
		}
		hosts = append(hosts, h)
	}
	return hosts, nil
}

// Internal helper for Auth dialing
func (s *HostService) GetCredentialsForDial(ctx context.Context, id string) (password, privateKey string, err error) {
	h, err := s.GetByID(ctx, id)
	if err != nil {
		return "", "", err
	}

	if !s.v.IsUnlocked() {
		return "", "", vault.ErrLocked
	}

	if h.EncryptedPassword != "" {
		password, err = s.v.DecryptField(h.EncryptedPassword, id, "encrypted_password")
		if err != nil {
			return "", "", err
		}
	}
	if h.EncryptedPrivateKey != "" {
		privateKey, err = s.v.DecryptField(h.EncryptedPrivateKey, id, "encrypted_private_key")
		if err != nil {
			return "", "", err
		}
	}

	return password, privateKey, nil
}

type scanner interface {
	Scan(dest ...any) error
}

func scanHost(s scanner) (*Host, error) {
	var h Host
	var grpID, pwdEnc, pkEnc, delAt sql.NullString
	if err := s.Scan(
		&h.ID, &grpID, &h.Name, &h.Hostname, &h.Port, &h.Username, &h.AuthType,
		&pwdEnc, &pkEnc, &h.CreatedAt, &h.UpdatedAt, &delAt,
	); err != nil {
		return nil, err
	}
	if grpID.Valid {
		h.GroupID = &grpID.String
	}
	if pwdEnc.Valid {
		h.EncryptedPassword = pwdEnc.String
	}
	if pkEnc.Valid {
		h.EncryptedPrivateKey = pkEnc.String
	}
	if delAt.Valid {
		h.DeletedAt = &delAt.String
	}
	return &h, nil
}
