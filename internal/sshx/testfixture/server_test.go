package testfixture_test

import (
	"bytes"
	"crypto/rand"
	"crypto/rsa"
	"net"
	"testing"
	"time"

	gossh "golang.org/x/crypto/ssh"

	"caterm/internal/sshx/testfixture"
)

func TestFixtureLifecycle(t *testing.T) {
	hostKey, err := testfixture.GenerateHostKey()
	if err != nil {
		t.Fatalf("failed to generate host key: %v", err)
	}

	server, err := testfixture.NewServer(testfixture.Config{
		Password: "QA_TEST_password_123",
		HostKey:  hostKey,
	})
	if err != nil {
		t.Fatalf("failed to create server fixture: %v", err)
	}
	defer server.Close()

	// Verify ephemeral port
	if server.Port() <= 0 {
		t.Fatalf("expected non-zero port, got %d", server.Port())
	}

	// Connect using x/crypto/ssh client
	var hostKeyConfirmed gossh.PublicKey
	clientConfig := &gossh.ClientConfig{
		User: "qa_user",
		Auth: []gossh.AuthMethod{
			gossh.Password("QA_TEST_password_123"),
		},
		HostKeyCallback: func(hostname string, remote net.Addr, key gossh.PublicKey) error {
			hostKeyConfirmed = key
			return nil
		},
		Timeout: 5 * time.Second,
	}

	client, err := gossh.Dial("tcp", server.Addr(), clientConfig)
	if err != nil {
		t.Fatalf("failed to dial SSH server: %v", err)
	}
	defer client.Close()

	if hostKeyConfirmed == nil {
		t.Fatal("expected host key callback to be triggered")
	}

	expectedFingerprint := gossh.FingerprintSHA256(hostKey.PublicKey())
	actualFingerprint := gossh.FingerprintSHA256(hostKeyConfirmed)
	if actualFingerprint != expectedFingerprint {
		t.Fatalf("host key mismatch: got %s, want %s", actualFingerprint, expectedFingerprint)
	}

	// Open session & verify handshake + echo
	sess, err := client.NewSession()
	if err != nil {
		t.Fatalf("failed to create session: %v", err)
	}
	defer sess.Close()

	stdin, err := sess.StdinPipe()
	if err != nil {
		t.Fatalf("failed to get stdin pipe: %v", err)
	}
	stdout, err := sess.StdoutPipe()
	if err != nil {
		t.Fatalf("failed to get stdout pipe: %v", err)
	}

	if err := sess.Shell(); err != nil {
		t.Fatalf("failed to start shell: %v", err)
	}

	msg := []byte("hello ssh fixture\n")
	if _, err := stdin.Write(msg); err != nil {
		t.Fatalf("failed to write to stdin: %v", err)
	}

	buf := make([]byte, 1024)
	n, err := stdout.Read(buf)
	if err != nil {
		t.Fatalf("failed to read stdout: %v", err)
	}

	if !bytes.Contains(buf[:n], []byte("hello ssh fixture")) {
		t.Fatalf("unexpected stdout response: %s", string(buf[:n]))
	}
}

func TestHostKeyChange(t *testing.T) {
	key1, err := testfixture.GenerateHostKey()
	if err != nil {
		t.Fatalf("failed to generate key1: %v", err)
	}
	key2, err := testfixture.GenerateHostKey()
	if err != nil {
		t.Fatalf("failed to generate key2: %v", err)
	}

	server, err := testfixture.NewServer(testfixture.Config{
		Password: "QA_TEST_pass",
		HostKey:  key1,
	})
	if err != nil {
		t.Fatalf("failed to create server: %v", err)
	}
	defer server.Close()

	// 1. First connection captures key1
	var receivedKey gossh.PublicKey
	clientConfig := &gossh.ClientConfig{
		User: "test",
		Auth: []gossh.AuthMethod{gossh.Password("QA_TEST_pass")},
		HostKeyCallback: func(hostname string, remote net.Addr, key gossh.PublicKey) error {
			receivedKey = key
			return nil
		},
		Timeout: 5 * time.Second,
	}

	c1, err := gossh.Dial("tcp", server.Addr(), clientConfig)
	if err != nil {
		t.Fatalf("failed first dial: %v", err)
	}
	c1.Close()

	if gossh.FingerprintSHA256(receivedKey) != gossh.FingerprintSHA256(key1.PublicKey()) {
		t.Fatalf("expected initial host key1")
	}

	// 2. Server changes host key
	// Due to data races in glider-ssh when modifying keys on the fly,
	// we shut down the first server and start a new one on the same (or new) port
	// to simulate the host key changing.
	server.Close()

	server2, err := testfixture.NewServer(testfixture.Config{
		Password: "QA_TEST_pass",
		HostKey:  key2,
	})
	if err != nil {
		t.Fatalf("failed to create server2: %v", err)
	}
	defer server2.Close()

	// 3. Second connection should receive key2
	var receivedKey2 gossh.PublicKey
	clientConfig.HostKeyCallback = func(hostname string, remote net.Addr, key gossh.PublicKey) error {
		receivedKey2 = key
		return nil
	}

	c2, err := gossh.Dial("tcp", server2.Addr(), clientConfig)
	if err != nil {
		t.Fatalf("failed second dial: %v", err)
	}
	c2.Close()

	if gossh.FingerprintSHA256(receivedKey2) != gossh.FingerprintSHA256(key2.PublicKey()) {
		t.Fatalf("expected updated host key2 after ChangeHostKey")
	}
}

func TestPublicKeyAuth(t *testing.T) {
	rawClientKey, err := rsa.GenerateKey(rand.Reader, 2048)
	if err != nil {
		t.Fatalf("failed to gen client key: %v", err)
	}
	clientSigner, err := gossh.NewSignerFromKey(rawClientKey)
	if err != nil {
		t.Fatalf("failed to create client signer: %v", err)
	}

	server, err := testfixture.NewServer(testfixture.Config{
		PublicKey: clientSigner.PublicKey(),
	})
	if err != nil {
		t.Fatalf("failed to start server: %v", err)
	}
	defer server.Close()

	clientConfig := &gossh.ClientConfig{
		User: "pubkey_user",
		Auth: []gossh.AuthMethod{
			gossh.PublicKeys(clientSigner),
		},
		HostKeyCallback: func(hostname string, remote net.Addr, key gossh.PublicKey) error { return nil },
		Timeout:         5 * time.Second,
	}

	c, err := gossh.Dial("tcp", server.Addr(), clientConfig)
	if err != nil {
		t.Fatalf("public key authentication failed: %v", err)
	}
	c.Close()

	if attempts := server.GetAuthAttempts(); attempts != 1 {
		t.Fatalf("expected 1 auth attempt, got %d", attempts)
	}
}
