package testfixture

import (
	"crypto/rand"
	"crypto/rsa"
	"fmt"
	"net"
	"sync"

	gliderssh "github.com/gliderlabs/ssh"
	"golang.org/x/crypto/ssh"
)

// Server represents an in-process SSH test server fixture.
type Server struct {
	listener  net.Listener
	sshServer *gliderssh.Server
	mu        sync.Mutex
	addr      string

	// Auth options / state
	Password     string
	PublicKey    ssh.PublicKey
	Passphrase   string
	AuthAttempts int

	// Host key options
	CurrentHostKey ssh.Signer
}

// Config holds options for setting up the SSH fixture.
type Config struct {
	Password   string
	PublicKey  ssh.PublicKey
	Passphrase string
	HostKey    ssh.Signer
}

// GenerateHostKey creates a new RSA host key.
func GenerateHostKey() (ssh.Signer, error) {
	key, err := rsa.GenerateKey(rand.Reader, 2048)
	if err != nil {
		return nil, err
	}
	return ssh.NewSignerFromKey(key)
}

// NewServer initializes and starts an SSH test server on an ephemeral port (127.0.0.1:0).
func NewServer(cfg Config) (*Server, error) {
	l, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		return nil, fmt.Errorf("failed to listen on ephemeral port: %w", err)
	}

	hostKey := cfg.HostKey
	if hostKey == nil {
		hostKey, err = GenerateHostKey()
		if err != nil {
			l.Close()
			return nil, fmt.Errorf("failed to generate host key: %w", err)
		}
	}

	s := &Server{
		listener:       l,
		addr:           l.Addr().String(),
		Password:       cfg.Password,
		PublicKey:      cfg.PublicKey,
		Passphrase:     cfg.Passphrase,
		CurrentHostKey: hostKey,
	}

	glider := &gliderssh.Server{
		Handler: func(sess gliderssh.Session) {
			buf := make([]byte, 1024)
			for {
				n, err := sess.Read(buf)
				if n > 0 {
					_, _ = sess.Write(buf[:n])
				}
				if err != nil {
					break
				}
			}
		},
		PasswordHandler: func(ctx gliderssh.Context, password string) bool {
			s.mu.Lock()
			s.AuthAttempts++
			s.mu.Unlock()
			if s.Password != "" && password == s.Password {
				return true
			}
			return false
		},
		PublicKeyHandler: func(ctx gliderssh.Context, key gliderssh.PublicKey) bool {
			s.mu.Lock()
			s.AuthAttempts++
			s.mu.Unlock()
			if s.PublicKey != nil && ssh.FingerprintSHA256(key) == ssh.FingerprintSHA256(s.PublicKey) {
				return true
			}
			return false
		},
	}

	glider.AddHostKey(hostKey)
	s.sshServer = glider

	go func() {
		_ = glider.Serve(l)
	}()

	return s, nil
}

// Addr returns the listening host:port of the server.
func (s *Server) Addr() string {
	return s.addr
}

// Port returns the port number.
func (s *Server) Port() int {
	_, portStr, _ := net.SplitHostPort(s.addr)
	var p int
	fmt.Sscanf(portStr, "%d", &p)
	return p
}

// ChangeHostKey replaces the current host key with a new one for testing TOFU / MITM detection.
func (s *Server) ChangeHostKey(newKey ssh.Signer) {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.CurrentHostKey = newKey
	s.sshServer.HostSigners = []gliderssh.Signer{newKey}
}

// GetAuthAttempts returns the total number of authentication attempts recorded.
func (s *Server) GetAuthAttempts() int {
	s.mu.Lock()
	defer s.mu.Unlock()
	return s.AuthAttempts
}

// Close shuts down the server listener cleanly.
func (s *Server) Close() error {
	return s.sshServer.Close()
}
