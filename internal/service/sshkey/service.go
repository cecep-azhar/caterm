package sshkey

import (
	"context"
	"crypto/rand"
	"crypto/rsa"
	"crypto/x509"
	"encoding/pem"
	"errors"
	"time"

	"github.com/google/uuid"
	"golang.org/x/crypto/ssh"

	"caterm/internal/store"
)

type SSHKey struct {
	ID          string    `json:"id"`
	Name        string    `json:"name"`
	PrivateKey  string    `json:"private_key,omitempty"`
	PublicKey   string    `json:"public_key"`
	Fingerprint string    `json:"fingerprint"`
	CreatedAt   time.Time `json:"created_at"`
	UpdatedAt   time.Time `json:"updated_at"`
}

type Service struct {
	db *store.DB
}

func NewService(db *store.DB) *Service {
	return &Service{db: db}
}

func (s *Service) GenerateKey(ctx context.Context, name string, bits int) (*SSHKey, error) {
	if bits == 0 {
		bits = 4096
	}

	privateKey, err := rsa.GenerateKey(rand.Reader, bits)
	if err != nil {
		return nil, err
	}

	if err := privateKey.Validate(); err != nil {
		return nil, err
	}

	privDER := x509.MarshalPKCS1PrivateKey(privateKey)
	privBlock := pem.Block{
		Type:    "RSA PRIVATE KEY",
		Headers: nil,
		Bytes:   privDER,
	}
	privatePEM := string(pem.EncodeToMemory(&privBlock))

	publicRsaKey, err := ssh.NewPublicKey(&privateKey.PublicKey)
	if err != nil {
		return nil, err
	}

	publicBytes := ssh.MarshalAuthorizedKey(publicRsaKey)
	publicStr := string(publicBytes)
	fingerprint := ssh.FingerprintSHA256(publicRsaKey)

	now := time.Now().UTC()
	key := &SSHKey{
		ID:          uuid.NewString(),
		Name:        name,
		PrivateKey:  privatePEM,
		PublicKey:   publicStr,
		Fingerprint: fingerprint,
		CreatedAt:   now,
		UpdatedAt:   now,
	}

	_, err = s.db.ExecContext(ctx, `
		INSERT INTO ssh_keys (id, name, private_key, public_key, fingerprint, created_at, updated_at)
		VALUES (?, ?, ?, ?, ?, ?, ?)
	`, key.ID, key.Name, key.PrivateKey, key.PublicKey, key.Fingerprint, key.CreatedAt.Format(time.RFC3339), key.UpdatedAt.Format(time.RFC3339))

	if err != nil {
		return nil, err
	}

	return key, nil
}

func (s *Service) AddKey(ctx context.Context, name, privateKeyStr string) (*SSHKey, error) {
	block, _ := pem.Decode([]byte(privateKeyStr))
	if block == nil {
		return nil, errors.New("failed to parse PEM block containing the private key")
	}

	priv, err := x509.ParsePKCS1PrivateKey(block.Bytes)
	if err != nil {
		return nil, err
	}

	publicRsaKey, err := ssh.NewPublicKey(&priv.PublicKey)
	if err != nil {
		return nil, err
	}

	publicBytes := ssh.MarshalAuthorizedKey(publicRsaKey)
	fingerprint := ssh.FingerprintSHA256(publicRsaKey)

	now := time.Now().UTC()
	key := &SSHKey{
		ID:          uuid.NewString(),
		Name:        name,
		PrivateKey:  privateKeyStr,
		PublicKey:   string(publicBytes),
		Fingerprint: fingerprint,
		CreatedAt:   now,
		UpdatedAt:   now,
	}

	_, err = s.db.ExecContext(ctx, `
		INSERT INTO ssh_keys (id, name, private_key, public_key, fingerprint, created_at, updated_at)
		VALUES (?, ?, ?, ?, ?, ?, ?)
	`, key.ID, key.Name, key.PrivateKey, key.PublicKey, key.Fingerprint, key.CreatedAt.Format(time.RFC3339), key.UpdatedAt.Format(time.RFC3339))

	if err != nil {
		return nil, err
	}

	return key, nil
}

func (s *Service) ListKeys(ctx context.Context) ([]SSHKey, error) {
	rows, err := s.db.QueryContext(ctx, `
		SELECT id, name, public_key, fingerprint, created_at, updated_at
		FROM ssh_keys
		ORDER BY name ASC
	`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var keys []SSHKey
	for rows.Next() {
		var k SSHKey
		var createdStr, updatedStr string
		if err := rows.Scan(&k.ID, &k.Name, &k.PublicKey, &k.Fingerprint, &createdStr, &updatedStr); err != nil {
			return nil, err
		}
		if t, err := time.Parse(time.RFC3339, createdStr); err == nil {
			k.CreatedAt = t
		}
		if t, err := time.Parse(time.RFC3339, updatedStr); err == nil {
			k.UpdatedAt = t
		}
		keys = append(keys, k)
	}
	return keys, rows.Err()
}

func (s *Service) DeleteKey(ctx context.Context, id string) error {
	_, err := s.db.ExecContext(ctx, `DELETE FROM ssh_keys WHERE id = ?`, id)
	return err
}

func (s *Service) GetKey(ctx context.Context, id string) (*SSHKey, error) {
	row := s.db.QueryRowContext(ctx, `
		SELECT id, name, private_key, public_key, fingerprint, created_at, updated_at
		FROM ssh_keys
		WHERE id = ?
	`, id)

	var k SSHKey
	var createdStr, updatedStr string
	if err := row.Scan(&k.ID, &k.Name, &k.PrivateKey, &k.PublicKey, &k.Fingerprint, &createdStr, &updatedStr); err != nil {
		return nil, err
	}
	if t, err := time.Parse(time.RFC3339, createdStr); err == nil {
		k.CreatedAt = t
	}
	if t, err := time.Parse(time.RFC3339, updatedStr); err == nil {
		k.UpdatedAt = t
	}
	return &k, nil
}
