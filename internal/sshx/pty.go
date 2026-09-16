package sshx

import (
	"context"
	"errors"
	"fmt"
	"io"
	"sync"
	"time"

	"golang.org/x/crypto/ssh"
)

type Session struct {
	client  *ssh.Client
	session *ssh.Session
	stdin   io.WriteCloser
	stdout  io.Reader
	cancel  context.CancelFunc
	wg      sync.WaitGroup
	mu      sync.Mutex
	closed  bool
}

func NewSession(ctx context.Context, client *ssh.Client, rows, cols int) (*Session, error) {
	sess, err := client.NewSession()
	if err != nil {
		return nil, fmt.Errorf("failed to create session: %w", err)
	}

	modes := ssh.TerminalModes{
		ssh.ECHO:          1,
		ssh.TTY_OP_ISPEED: 14400,
		ssh.TTY_OP_OSPEED: 14400,
	}

	if err := sess.RequestPty("xterm-256color", rows, cols, modes); err != nil {
		sess.Close()
		return nil, fmt.Errorf("failed to request pty: %w", err)
	}

	stdin, err := sess.StdinPipe()
	if err != nil {
		sess.Close()
		return nil, fmt.Errorf("failed to get stdin pipe: %w", err)
	}

	stdout, err := sess.StdoutPipe()
	if err != nil {
		sess.Close()
		return nil, fmt.Errorf("failed to get stdout pipe: %w", err)
	}

	if err := sess.Shell(); err != nil {
		sess.Close()
		return nil, fmt.Errorf("failed to start shell: %w", err)
	}

	reqCtx, cancel := context.WithCancel(ctx)

	s := &Session{
		client:  client,
		session: sess,
		stdin:   stdin,
		stdout:  stdout,
		cancel:  cancel,
	}

	// Keepalive loop
	s.wg.Add(1)
	go func() {
		defer s.wg.Done()
		s.keepaliveLoop(reqCtx)
	}()

	return s, nil
}

func (s *Session) Write(p []byte) (int, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	if s.closed {
		return 0, io.EOF
	}
	return s.stdin.Write(p)
}

func (s *Session) ReadBatch(onData func(data []byte)) error {
	buf := make([]byte, 4096)

	for {
		n, err := s.stdout.Read(buf)
		if n > 0 {
			onData(buf[:n])
		}
		if err != nil {
			if err == io.EOF {
				return nil
			}
			return err
		}
	}
}

func (s *Session) WindowChange(rows, cols int) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	if s.closed {
		return errors.New("session closed")
	}
	return s.session.WindowChange(rows, cols)
}

func (s *Session) keepaliveLoop(ctx context.Context) {
	ticker := time.NewTicker(30 * time.Second)
	defer ticker.Stop()

	fails := 0
	for {
		select {
		case <-ctx.Done():
			return
		case <-ticker.C:
			_, _, err := s.client.SendRequest("keepalive@openssh.com", true, nil)
			if err != nil {
				fails++
				if fails >= 3 {
					s.Close()
					return
				}
			} else {
				fails = 0
			}
		}
	}
}

func (s *Session) Close() error {
	s.mu.Lock()
	if s.closed {
		s.mu.Unlock()
		return nil
	}
	s.closed = true
	s.mu.Unlock()

	s.cancel()
	s.session.Close()
	s.wg.Wait()
	return nil
}
