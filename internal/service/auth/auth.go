package auth

import (
	"context"
	"errors"
	"strings"
	"sync"
	"time"

	"caterm/internal/vault"
)

var (
	ErrLocked      = errors.New("vault is locked")
	ErrWrongAuth   = errors.New("wrong password") // Generic error
	ErrRateLimited = errors.New("rate limited")
)

type Service struct {
	v  *vault.Vault
	db vault.DB

	mu       sync.Mutex
	fails    int
	lastFail time.Time
}

func New(v *vault.Vault, db vault.DB) *Service {
	return &Service{
		v:  v,
		db: db,
	}
}

func (s *Service) Init(ctx context.Context, password string) error {
	if len(password) < 12 {
		return errors.New("password must be at least 12 characters")
	}
	return s.v.Init(ctx, s.db, password)
}

func (s *Service) Unlock(ctx context.Context, password string) error {
	s.mu.Lock()
	if s.fails >= 5 {
		if time.Since(s.lastFail) < 5*time.Second {
			s.mu.Unlock()
			return ErrRateLimited
		}
	}
	s.mu.Unlock()

	err := s.v.Unlock(ctx, s.db, password)

	s.mu.Lock()
	defer s.mu.Unlock()

	if err != nil {
		s.fails++
		s.lastFail = time.Now()
		// Return generic error
		if err == vault.ErrWrongPassword || strings.Contains(err.Error(), "not initialized") {
			return ErrWrongAuth
		}
		return err
	}

	s.fails = 0
	return nil
}
