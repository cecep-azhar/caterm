package sshx

import (
	"context"
	"errors"
	"fmt"
	"net"
	"time"

	"golang.org/x/crypto/ssh"
)

type AuthType string

const (
	AuthPassword AuthType = "password"
	AuthKey      AuthType = "key"
	AuthKeyPass  AuthType = "key_passphrase"
)

type DialerConfig struct {
	HostID       string
	Hostname     string
	Port         int
	User         string
	AuthType     AuthType
	Password     string
	PrivateKey   []byte
	Passphrase   []byte
	HostKeyStore HostKeyStore
	DialTimeout  time.Duration
}

func Dial(ctx context.Context, cfg DialerConfig) (*ssh.Client, error) {
	if cfg.DialTimeout == 0 {
		cfg.DialTimeout = 10 * time.Second
	}

	authMethod, err := getAuthMethod(cfg)
	if err != nil {
		return nil, fmt.Errorf("invalid auth method: %w", err)
	}

	tofu := NewTOFU(ctx, cfg.HostKeyStore, cfg.HostID)

	clientConfig := &ssh.ClientConfig{
		User: cfg.User,
		Auth: []ssh.AuthMethod{
			authMethod,
			ssh.KeyboardInteractive(func(user, instruction string, questions []string, echos []bool) (answers []string, err error) {
				return nil, errors.New("keyboard-interactive authentication is not supported")
			}),
		},
		HostKeyCallback: tofu.HostKeyCallback,
		Timeout:         cfg.DialTimeout,
	}

	addr := fmt.Sprintf("%s:%d", cfg.Hostname, cfg.Port)

	dialer := net.Dialer{Timeout: cfg.DialTimeout}
	conn, err := dialer.DialContext(ctx, "tcp", addr)
	if err != nil {
		return nil, fmt.Errorf("failed to dial %s: %w", addr, err)
	}

	sshConn, chans, reqs, err := ssh.NewClientConn(conn, addr, clientConfig)
	if err != nil {
		conn.Close()
		return nil, fmt.Errorf("ssh handshake failed: %w", err)
	}

	return ssh.NewClient(sshConn, chans, reqs), nil
}

func getAuthMethod(cfg DialerConfig) (ssh.AuthMethod, error) {
	switch cfg.AuthType {
	case AuthPassword:
		return ssh.Password(cfg.Password), nil
	case AuthKey:
		signer, err := ssh.ParsePrivateKey(cfg.PrivateKey)
		if err != nil {
			return nil, fmt.Errorf("failed to parse private key: %w", err)
		}
		return ssh.PublicKeys(signer), nil
	case AuthKeyPass:
		signer, err := ssh.ParsePrivateKeyWithPassphrase(cfg.PrivateKey, cfg.Passphrase)
		if err != nil {
			if isPassphraseError(err) {
				return nil, errors.New("incorrect passphrase for private key")
			}
			return nil, fmt.Errorf("failed to parse private key with passphrase: %w", err)
		}
		return ssh.PublicKeys(signer), nil
	default:
		return nil, fmt.Errorf("unsupported auth type: %s", cfg.AuthType)
	}
}

func isPassphraseError(err error) bool {
	if err == nil {
		return false
	}
	msg := err.Error()
	return msg == "x509: decryption password incorrect" ||
		msg == "ssh: incorrect passphrase" ||
		msg == "ssh: cannot decode encrypted private key"
}

// ValidateKey validates an SSH key before saving to catch unsupported formats early
func ValidateKey(authType AuthType, key []byte, passphrase []byte) error {
	switch authType {
	case AuthKey:
		_, err := ssh.ParsePrivateKey(key)
		return err
	case AuthKeyPass:
		_, err := ssh.ParsePrivateKeyWithPassphrase(key, passphrase)
		if err != nil {
			if isPassphraseError(err) {
				return errors.New("incorrect passphrase")
			}
			return err
		}
		return nil
	default:
		return nil
	}
}
