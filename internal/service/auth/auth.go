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

	mu          sync.Mutex
	fails       int
	lastFail    time.Time
	idleTimeout time.Duration
	idleTimer   *time.Timer
	timerStopCh chan struct{}
	onAutoLock  func()
}

func New(v *vault.Vault, db vault.DB) *Service {
	return &Service{
		v:           v,
		db:          db,
		idleTimeout: 15 * time.Minute, // Default 15 minutes
	}
}

func (s *Service) SetIdleTimeout(d time.Duration) {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.idleTimeout = d
	s.resetIdleTimerLocked()
}

func (s *Service) SetOnAutoLock(cb func()) {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.onAutoLock = cb
}

func (s *Service) Activity() {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.resetIdleTimerLocked()
}

func (s *Service) StartIdleTimer() {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.resetIdleTimerLocked()
}

func (s *Service) StopIdleTimer() {
	s.mu.Lock()
	defer s.mu.Unlock()
	if s.idleTimer != nil {
		s.idleTimer.Stop()
		s.idleTimer = nil
	}
}

func (s *Service) resetIdleTimerLocked() {
	if s.idleTimeout <= 0 {
		if s.idleTimer != nil {
			s.idleTimer.Stop()
			s.idleTimer = nil
		}
		return
	}
	if s.idleTimer != nil {
		s.idleTimer.Stop()
	}
	s.idleTimer = time.AfterFunc(s.idleTimeout, func() {
		s.Lock()
	})
}

func (s *Service) Lock() {
	s.mu.Lock()
	onLock := s.onAutoLock
	if s.idleTimer != nil {
		s.idleTimer.Stop()
		s.idleTimer = nil
	}
	s.mu.Unlock()

	s.v.Lock()
	if onLock != nil {
		onLock()
	}
}

func (s *Service) IsInitialized(ctx context.Context) (bool, error) {
	var count int
	err := s.db.QueryRowContext(ctx, "SELECT COUNT(*) FROM vault_meta").Scan(&count)
	if err != nil {
		return false, err
	}
	return count > 0, nil
}

func (s *Service) Init(ctx context.Context, password string) error {
	if len(password) < 12 {
		return errors.New("password must be at least 12 characters")
	}
	err := s.v.Init(ctx, s.db, password)
	if err == nil {
		s.Activity()
	}
	return err
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
	s.resetIdleTimerLocked()
	return nil
}

func (s *Service) ChangePassword(ctx context.Context, oldPassword, newPassword string) error {
	if len(newPassword) < 12 {
		return errors.New("new password must be at least 12 characters")
	}
	return s.v.ChangePassword(ctx, s.db, oldPassword, newPassword)
}
