package sshx

import (
	"context"
	"testing"

	"caterm/internal/sshx/testfixture"
	"golang.org/x/crypto/ssh"
)

type mockHostKeyStore struct {
	keys map[string]string
}

func newMockStore() *mockHostKeyStore {
	return &mockHostKeyStore{keys: make(map[string]string)}
}

func (m *mockHostKeyStore) GetHostKey(ctx context.Context, hostID string) (string, error) {
	return m.keys[hostID], nil
}

func (m *mockHostKeyStore) SaveHostKey(ctx context.Context, hostID string, fingerprint string) error {
	m.keys[hostID] = fingerprint
	return nil
}

func TestTOFU(t *testing.T) {
	server, err := testfixture.NewServer(testfixture.Config{Password: "testpass"})
	if err != nil {
		t.Fatalf("failed to create fixture server: %v", err)
	}
	defer server.Close()

	store := newMockStore()
	ctx := context.Background()
	hostID := "host-1"

	tofu := NewTOFU(ctx, store, hostID)

	clientConfig := &ssh.ClientConfig{
		User:            "testuser",
		Auth:            []ssh.AuthMethod{ssh.Password("testpass")},
		HostKeyCallback: tofu.HostKeyCallback,
	}

	client, err := ssh.Dial("tcp", server.Addr(), clientConfig)
	if err != nil {
		t.Fatalf("TOFU initial connection failed: %v", err)
	}
	client.Close()

	fingerprint := store.keys[hostID]
	if fingerprint == "" {
		t.Errorf("expected host key to be saved in store after initial connection")
	}

	client, err = ssh.Dial("tcp", server.Addr(), clientConfig)
	if err != nil {
		t.Fatalf("subsequent connection with valid key failed: %v", err)
	}
	client.Close()
}

func TestHostKeyChanged(t *testing.T) {
	server, err := testfixture.NewServer(testfixture.Config{Password: "testpass"})
	if err != nil {
		t.Fatalf("failed to create fixture server: %v", err)
	}
	defer server.Close()

	store := newMockStore()
	ctx := context.Background()
	hostID := "host-1"

	tofu := NewTOFU(ctx, store, hostID)

	clientConfig := &ssh.ClientConfig{
		User:            "testuser",
		Auth:            []ssh.AuthMethod{ssh.Password("testpass")},
		HostKeyCallback: tofu.HostKeyCallback,
	}

	client, err := ssh.Dial("tcp", server.Addr(), clientConfig)
	if err != nil {
		t.Fatalf("initial connect failed: %v", err)
	}
	client.Close()

	newHostKey, _ := testfixture.GenerateHostKey()
	server.Close()

	server2, err := testfixture.NewServer(testfixture.Config{
		Password: "testpass",
		HostKey:  newHostKey,
	})
	if err != nil {
		t.Fatalf("failed to create server2: %v", err)
	}
	defer server2.Close()

	client, err = ssh.Dial("tcp", server2.Addr(), clientConfig)
	if err == nil {
		client.Close()
		t.Fatalf("expected dial error on host key mismatch, got success")
	}
}

func TestNoCredsOnMismatch(t *testing.T) {
	server, err := testfixture.NewServer(testfixture.Config{Password: "testpass"})
	if err != nil {
		t.Fatalf("failed to create fixture server: %v", err)
	}
	defer server.Close()

	store := newMockStore()
	ctx := context.Background()
	hostID := "host-1"

	store.keys[hostID] = "SHA256:fake_fingerprint"

	tofu := NewTOFU(ctx, store, hostID)

	clientConfig := &ssh.ClientConfig{
		User:            "testuser",
		Auth:            []ssh.AuthMethod{ssh.Password("testpass")},
		HostKeyCallback: tofu.HostKeyCallback,
	}

	client, err := ssh.Dial("tcp", server.Addr(), clientConfig)
	if err == nil {
		client.Close()
		t.Fatalf("expected failure on key mismatch")
	}

	if attempts := server.GetAuthAttempts(); attempts > 0 {
		t.Errorf("security violation: auth attempted despite host key mismatch! %d attempts recorded.", attempts)
	}
}
