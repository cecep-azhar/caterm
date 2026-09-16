package sshx

import (
	"context"
	"crypto/rand"
	"crypto/rsa"
	"crypto/x509"
	"encoding/pem"
	"net"
	"strings"
	"testing"
	"time"

	"caterm/internal/sshx/testfixture"
	"golang.org/x/crypto/ssh"
)

func generateRSAPrivateKeyPEM() ([]byte, ssh.Signer, error) {
	key, err := rsa.GenerateKey(rand.Reader, 2048)
	if err != nil {
		return nil, nil, err
	}
	signer, err := ssh.NewSignerFromKey(key)
	if err != nil {
		return nil, nil, err
	}

	pemBlock := &pem.Block{
		Type:  "RSA PRIVATE KEY",
		Bytes: x509.MarshalPKCS1PrivateKey(key),
	}
	keyPEM := pem.EncodeToMemory(pemBlock)
	return keyPEM, signer, nil
}

func generateEncryptedRSAPrivateKeyPEM(passphrase string) ([]byte, ssh.Signer, error) {
	key, err := rsa.GenerateKey(rand.Reader, 2048)
	if err != nil {
		return nil, nil, err
	}
	signer, err := ssh.NewSignerFromKey(key)
	if err != nil {
		return nil, nil, err
	}

	block, err := x509.EncryptPEMBlock(rand.Reader, "RSA PRIVATE KEY", x509.MarshalPKCS1PrivateKey(key), []byte(passphrase), x509.PEMCipherAES256)
	if err != nil {
		return nil, nil, err
	}
	keyPEM := pem.EncodeToMemory(block)
	return keyPEM, signer, nil
}

func TestAuthPassword(t *testing.T) {
	server, err := testfixture.NewServer(testfixture.Config{Password: "secret123"})
	if err != nil {
		t.Fatalf("failed server: %v", err)
	}
	defer server.Close()

	host, portStr, _ := net.SplitHostPort(server.Addr())
	var port int
	_ = strings.TrimPrefix(portStr, "")
	_, _ = net.LookupPort("tcp", portStr)
	port = server.Port()

	store := newMockStore()
	ctx := context.Background()

	cfg := DialerConfig{
		HostID:       "h1",
		Hostname:     host,
		Port:         port,
		User:         "testuser",
		AuthType:     AuthPassword,
		Password:     "secret123",
		HostKeyStore: store,
	}

	client, err := Dial(ctx, cfg)
	if err != nil {
		t.Fatalf("Dial with password failed: %v", err)
	}
	client.Close()
}

func TestAuthKey(t *testing.T) {
	pemBytes, signer, err := generateRSAPrivateKeyPEM()
	if err != nil {
		t.Fatalf("failed key gen: %v", err)
	}

	server, err := testfixture.NewServer(testfixture.Config{PublicKey: signer.PublicKey()})
	if err != nil {
		t.Fatalf("failed server: %v", err)
	}
	defer server.Close()

	store := newMockStore()
	ctx := context.Background()

	cfg := DialerConfig{
		HostID:       "h2",
		Hostname:     "127.0.0.1",
		Port:         server.Port(),
		User:         "testuser",
		AuthType:     AuthKey,
		PrivateKey:   pemBytes,
		HostKeyStore: store,
	}

	client, err := Dial(ctx, cfg)
	if err != nil {
		t.Fatalf("Dial with unencrypted key failed: %v", err)
	}
	client.Close()
}

func TestAuthKeyPassphrase(t *testing.T) {
	pemBytes, signer, err := generateEncryptedRSAPrivateKeyPEM("pass123")
	if err != nil {
		t.Fatalf("failed key gen: %v", err)
	}

	server, err := testfixture.NewServer(testfixture.Config{PublicKey: signer.PublicKey()})
	if err != nil {
		t.Fatalf("failed server: %v", err)
	}
	defer server.Close()

	store := newMockStore()
	ctx := context.Background()

	// 1. Success case
	cfg := DialerConfig{
		HostID:       "h3",
		Hostname:     "127.0.0.1",
		Port:         server.Port(),
		User:         "testuser",
		AuthType:     AuthKeyPass,
		PrivateKey:   pemBytes,
		Passphrase:   []byte("pass123"),
		HostKeyStore: store,
	}

	client, err := Dial(ctx, cfg)
	if err != nil {
		t.Fatalf("Dial with encrypted key failed: %v", err)
	}
	client.Close()

	// 2. Wrong passphrase case
	cfg.Passphrase = []byte("wrongpass")
	_, err = Dial(ctx, cfg)
	if err == nil {
		t.Fatalf("expected error for wrong passphrase, got nil")
	}
	if !strings.Contains(err.Error(), "incorrect passphrase") {
		t.Fatalf("expected 'incorrect passphrase' error, got: %v", err)
	}
}

func TestDialTimeout(t *testing.T) {
	store := newMockStore()
	ctx := context.Background()

	cfg := DialerConfig{
		HostID:       "h4",
		Hostname:     "198.51.100.1", // TEST-NET-2, reserved and blackholed
		Port:         22,
		User:         "testuser",
		AuthType:     AuthPassword,
		Password:     "pass",
		HostKeyStore: store,
		DialTimeout:  10 * time.Millisecond,
	}

	start := time.Now()
	_, err := Dial(ctx, cfg)
	elapsed := time.Since(start)

	if err == nil {
		t.Fatalf("expected timeout error")
	}
	if elapsed > 1*time.Second {
		t.Fatalf("dial timeout took too long: %v", elapsed)
	}
}

func TestKbdInteractive(t *testing.T) {
	server, err := testfixture.NewServer(testfixture.Config{Password: "secret123"})
	if err != nil {
		t.Fatalf("failed server: %v", err)
	}
	defer server.Close()

	store := newMockStore()
	ctx := context.Background()

	cfg := DialerConfig{
		HostID:       "h5",
		Hostname:     "127.0.0.1",
		Port:         server.Port(),
		User:         "testuser",
		AuthType:     AuthPassword,
		Password:     "wrongpass",
		HostKeyStore: store,
	}

	_, err = Dial(ctx, cfg)
	if err == nil {
		t.Fatalf("expected failure when auth fails or kbd-interactive is required")
	}
}
