package sshx

import (
	"context"
	"runtime"
	"strings"
	"sync/atomic"
	"testing"
	"time"

	"caterm/internal/sshx/testfixture"
)

func TestPTYStream(t *testing.T) {
	server, err := testfixture.NewServer(testfixture.Config{Password: "secret123"})
	if err != nil {
		t.Fatalf("failed server: %v", err)
	}
	defer server.Close()

	store := newMockStore()
	ctx := context.Background()
	cfg := DialerConfig{
		HostID:       "h1",
		Hostname:     "127.0.0.1",
		Port:         server.Port(),
		User:         "testuser",
		AuthType:     AuthPassword,
		Password:     "secret123",
		HostKeyStore: store,
	}

	client, err := Dial(ctx, cfg)
	if err != nil {
		t.Fatalf("Dial failed: %v", err)
	}
	defer client.Close()

	sess, err := NewSession(ctx, client, 24, 80)
	if err != nil {
		t.Fatalf("NewSession failed: %v", err)
	}
	defer sess.Close()

	var output []byte
	readDone := make(chan struct{})

	go func() {
		_ = sess.ReadBatch(func(data []byte) {
			output = append(output, data...)
			if strings.Contains(string(output), "hello pty") {
				select {
				case <-readDone:
				default:
					close(readDone)
				}
			}
		})
	}()

	// Give a small delay for ReadBatch loop to be actively reading
	time.Sleep(50 * time.Millisecond)

	_, err = sess.Write([]byte("hello pty\n"))
	if err != nil {
		t.Fatalf("Write failed: %v", err)
	}

	select {
	case <-readDone:
		// success
	case <-time.After(3 * time.Second):
		t.Fatalf("timeout waiting for output, got: %s", string(output))
	}
}

func TestResizeSttySize(t *testing.T) {
	server, err := testfixture.NewServer(testfixture.Config{Password: "secret123"})
	if err != nil {
		t.Fatalf("failed server: %v", err)
	}
	defer server.Close()

	store := newMockStore()
	ctx := context.Background()
	cfg := DialerConfig{
		HostID:       "h1",
		Hostname:     "127.0.0.1",
		Port:         server.Port(),
		User:         "testuser",
		AuthType:     AuthPassword,
		Password:     "secret123",
		HostKeyStore: store,
	}

	client, err := Dial(ctx, cfg)
	if err != nil {
		t.Fatalf("Dial failed: %v", err)
	}
	defer client.Close()

	sess, err := NewSession(ctx, client, 24, 80)
	if err != nil {
		t.Fatalf("NewSession failed: %v", err)
	}
	defer sess.Close()

	err = sess.WindowChange(40, 120)
	if err != nil {
		t.Fatalf("WindowChange failed: %v", err)
	}
}

func TestNoGoroutineLeak(t *testing.T) {
	server, err := testfixture.NewServer(testfixture.Config{Password: "secret123"})
	if err != nil {
		t.Fatalf("failed server: %v", err)
	}
	defer server.Close()

	store := newMockStore()
	ctx := context.Background()
	cfg := DialerConfig{
		HostID:       "h1",
		Hostname:     "127.0.0.1",
		Port:         server.Port(),
		User:         "testuser",
		AuthType:     AuthPassword,
		Password:     "secret123",
		HostKeyStore: store,
	}

	client, err := Dial(ctx, cfg)
	if err != nil {
		t.Fatalf("Dial failed: %v", err)
	}
	defer client.Close()

	baseGoroutines := runtime.NumGoroutine()

	for i := 0; i < 20; i++ {
		sess, err := NewSession(ctx, client, 24, 80)
		if err != nil {
			t.Fatalf("NewSession failed: %v", err)
		}

		go func() {
			_ = sess.ReadBatch(func(data []byte) {})
		}()

		sess.Close()
	}

	time.Sleep(200 * time.Millisecond)

	finalGoroutines := runtime.NumGoroutine()

	if finalGoroutines > baseGoroutines+2 {
		t.Fatalf("goroutine leak detected: base=%d, final=%d", baseGoroutines, finalGoroutines)
	}
}

func TestFloodBackpressure(t *testing.T) {
	server, err := testfixture.NewServer(testfixture.Config{Password: "secret123"})
	if err != nil {
		t.Fatalf("failed server: %v", err)
	}
	defer server.Close()

	store := newMockStore()
	ctx := context.Background()
	cfg := DialerConfig{
		HostID:       "h1",
		Hostname:     "127.0.0.1",
		Port:         server.Port(),
		User:         "testuser",
		AuthType:     AuthPassword,
		Password:     "secret123",
		HostKeyStore: store,
	}

	client, err := Dial(ctx, cfg)
	if err != nil {
		t.Fatalf("Dial failed: %v", err)
	}
	defer client.Close()

	sess, err := NewSession(ctx, client, 24, 80)
	if err != nil {
		t.Fatalf("NewSession failed: %v", err)
	}
	defer sess.Close()

	var bytesRead int64
	go func() {
		_ = sess.ReadBatch(func(data []byte) {
			atomic.AddInt64(&bytesRead, int64(len(data)))
		})
	}()

	payload := make([]byte, 1024*50)
	_, _ = sess.Write(payload)

	time.Sleep(100 * time.Millisecond)
	sess.Close()

	if atomic.LoadInt64(&bytesRead) == 0 {
		t.Fatalf("expected to read some bytes, read 0")
	}
}
