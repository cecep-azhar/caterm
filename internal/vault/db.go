package vault

import (
	"context"
	"database/sql"
	"encoding/base64"
	"errors"
	"time"
)

func NowUTC() string {
	return time.Now().UTC().Format("2006-01-02T15:04:05.000Z")
}

type DB interface {
	QueryRowContext(ctx context.Context, query string, args ...any) *sql.Row
	ExecContext(ctx context.Context, query string, args ...any) (sql.Result, error)
}

func (v *Vault) Init(ctx context.Context, db DB, password string) error {
	var count int
	if err := db.QueryRowContext(ctx, "SELECT COUNT(*) FROM vault_meta").Scan(&count); err != nil {
		return err
	}
	if count > 0 {
		return ErrAlreadyInit
	}

	if len(password) < 12 {
		return errors.New("password must be at least 12 characters")
	}

	salt, err := GenerateSalt()
	if err != nil {
		return err
	}

	kek := DeriveKEK(password, salt)
	dek, err := GenerateDEK()
	if err != nil {
		return err
	}

	wrappedDEK, err := WrapDEK(dek, kek)
	if err != nil {
		return err
	}

	now := NowUTC()
	saltStr := base64.RawURLEncoding.EncodeToString(salt)
	_, err = db.ExecContext(ctx, `INSERT INTO vault_meta (
		id, schema_version, kdf, kdf_salt, kdf_time, kdf_memory, kdf_threads, wrapped_dek, created_at, updated_at
	) VALUES (1, 1, 'argon2id', ?, ?, ?, ?, ?, ?, ?)`,
		saltStr, ArgonTime, ArgonMemory, ArgonThreads, wrappedDEK, now, now)
	if err != nil {
		return err
	}

	v.SetDEK(dek)
	return nil
}

func (v *Vault) Unlock(ctx context.Context, db DB, password string) error {
	var saltStr, wrappedDEK string
	err := db.QueryRowContext(ctx, "SELECT kdf_salt, wrapped_dek FROM vault_meta LIMIT 1").Scan(&saltStr, &wrappedDEK)
	if err != nil {
		if errors.Is(err, sql.ErrNoRows) {
			return errors.New("vault is not initialized")
		}
		return err
	}

	salt, err := base64.RawURLEncoding.DecodeString(saltStr)
	if err != nil {
		return err
	}

	kek := DeriveKEK(password, salt)
	dek, err := UnwrapDEK(wrappedDEK, kek)
	if err != nil {
		return err
	}

	v.SetDEK(dek)
	return nil
}

func (v *Vault) ChangePassword(ctx context.Context, db DB, oldPassword, newPassword string) error {
	if len(newPassword) < 12 {
		return errors.New("new password must be at least 12 characters")
	}

	var saltStr, wrappedDEK string
	err := db.QueryRowContext(ctx, "SELECT kdf_salt, wrapped_dek FROM vault_meta LIMIT 1").Scan(&saltStr, &wrappedDEK)
	if err != nil {
		if errors.Is(err, sql.ErrNoRows) {
			return errors.New("vault is not initialized")
		}
		return err
	}

	salt, err := base64.RawURLEncoding.DecodeString(saltStr)
	if err != nil {
		return err
	}

	oldKEK := DeriveKEK(oldPassword, salt)
	dek, err := UnwrapDEK(wrappedDEK, oldKEK)
	if err != nil {
		return err
	}

	newSalt, err := GenerateSalt()
	if err != nil {
		return err
	}

	newKEK := DeriveKEK(newPassword, newSalt)
	newWrappedDEK, err := WrapDEK(dek, newKEK)
	if err != nil {
		return err
	}

	newSaltStr := base64.RawURLEncoding.EncodeToString(newSalt)
	now := NowUTC()
	_, err = db.ExecContext(ctx, "UPDATE vault_meta SET kdf_salt = ?, wrapped_dek = ?, updated_at = ?", newSaltStr, newWrappedDEK, now)
	if err != nil {
		return err
	}

	v.SetDEK(dek)
	return nil
}
