package terminal

import (
	"context"
	"testing"
	"time"

	"caterm/internal/sshx"
	"caterm/internal/sshx/testfixture"
)

type mockHostKeyStore struct {
	keys map[string]string
}

func (m *mockHostKeyStore) GetHostKey(ctx context.Context, hostID string) (string, error) {
	return m.keys[hostID], nil
}

func (m *mockHostKeyStore) SaveHostKey(ctx context.Context, hostID string, fingerprint string) error {
	m.keys[hostID] = fingerprint
	return nil
}

func TestTerminalService(t *testing.T) {
	server, err := testfixture.NewServer(testfixture.Config{Password: "secret123"})
	if err != nil {
		t.Fatalf("failed server: %v", err)
	}
	defer server.Close()

	store := &mockHostKeyStore{keys: make(map[string]string)}
	ctx := context.Background()

	cfg := sshx.DialerConfig{
		HostID:       "h1",
		Hostname:     "127.0.0.1",
		Port:         server.Port(),
		User:         "testuser",
		AuthType:     sshx.AuthPassword,
		Password:     "secret123",
		HostKeyStore: store,
	}

	client, err := sshx.Dial(ctx, cfg)
	if err != nil {
		t.Fatalf("Dial failed: %v", err)
	}
	defer client.Close()

	sess, err := sshx.NewSession(ctx, client, 24, 80)
	if err != nil {
		t.Fatalf("NewSession failed: %v", err)
	}

	svc := New()
	// Usually provided by Wails lifecycle
	svc.isTest = true

	paneID := "pane-123"
	svc.StartSession(paneID, sess, client)

	err = svc.Write(paneID, []byte("echo QA_TEST_ECHO\n"))
	if err != nil {
		t.Fatalf("Write failed: %v", err)
	}

	err = svc.Resize(paneID, 40, 120)
	if err != nil {
		t.Fatalf("Resize failed: %v", err)
	}

	// We can't easily capture the Wails events here without mocking the runtime,
	// but we can verify the service manages the state correctly.
	svc.mu.Lock()
	state, exists := svc.sessions[paneID]
	svc.mu.Unlock()

	if !exists {
		t.Fatalf("session state not tracked")
	}

	// Since we are not acking, unacked bytes should increase
	time.Sleep(100 * time.Millisecond)

	unacked := state.FlowControl.UnackedBytes()
	if unacked == 0 {
		t.Fatalf("expected some unacked bytes from echo response")
	}

	svc.Ack(paneID, unacked)

	if state.FlowControl.UnackedBytes() != 0 {
		t.Fatalf("Ack failed to clear unacked bytes")
	}

	svc.ClosePane(paneID)

	svc.mu.Lock()
	_, exists = svc.sessions[paneID]
	svc.mu.Unlock()

	if exists {
		t.Fatalf("session state still tracked after close")
	}
}
