package sshx

import (
	"context"
	"io"
	"net"
	"runtime"
	"strconv"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"caterm/internal/sshx/testfixture"
)

func TestFourPaneFlood(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping long-running flood test in short mode")
	}

	cfg := testfixture.Config{
		Password: "testpass",
	}
	server, err := testfixture.NewServer(cfg)
	if err != nil {
		t.Fatalf("Failed to create server: %v", err)
	}
	defer server.Close()

	addr := server.Addr()
	hostStr, portStr, err := net.SplitHostPort(addr)
	if err != nil {
		t.Fatalf("Failed to split addr: %v", err)
	}
	portInt, _ := strconv.Atoi(portStr)

	// Measure baseline goroutines & RSS
	time.Sleep(1 * time.Second) // wait for server to settle

	baselineGoroutines := runtime.NumGoroutine()
	var ms runtime.MemStats
	runtime.ReadMemStats(&ms)
	baselineRSS := ms.Alloc

	t.Logf("Baseline Goroutines: %d", baselineGoroutines)
	t.Logf("Baseline RSS (Alloc): %d MB", baselineRSS/1024/1024)

	var wg sync.WaitGroup
	var activePanes int32

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	runPane := func(id int) {
		defer wg.Done()

		mockStore := newMockStore()

		dialCfg := DialerConfig{
			HostID:       "host-1",
			Hostname:     hostStr,
			Port:         portInt,
			User:         "testuser",
			AuthType:     AuthPassword,
			Password:     "testpass",
			HostKeyStore: mockStore,
		}

		client, err := Dial(ctx, dialCfg)
		if err != nil {
			t.Errorf("Pane %d: dial failed: %v", id, err)
			return
		}
		defer client.Close()

		session, err := NewSession(ctx, client, 24, 80)
		if err != nil {
			t.Errorf("Pane %d: NewSession failed: %v", id, err)
			return
		}
		defer session.Close()

		atomic.AddInt32(&activePanes, 1)

		// Read output continuously
		go func() {
			err := session.ReadBatch(func(data []byte) {
				// Sink data
			})
			if err != nil && err != io.EOF {
				// OK on close
			}
		}()

		// Start flood of data
		go func() {
			floodData := []byte("yes 'FourPaneFlood test data output stream'\n")
			for {
				select {
				case <-ctx.Done():
					return
				default:
					_, err := session.Write(floodData)
					if err != nil {
						return
					}
					time.Sleep(10 * time.Millisecond) // avoid spinning too fast
				}
			}
		}()

		// Keep running until context is cancelled
		<-ctx.Done()
	}

	// Start 4 panes
	for i := 0; i < 4; i++ {
		wg.Add(1)
		go runPane(i)
	}

	// Wait for connections to establish
	time.Sleep(2 * time.Second)

	// Check latency during flood
	t.Logf("Measuring input latency during flood with %d active panes...", atomic.LoadInt32(&activePanes))

	latencyStart := time.Now()

	mockStore := newMockStore()
	pingCfg := DialerConfig{
		HostID:       "host-ping",
		Hostname:     hostStr,
		Port:         portInt,
		User:         "testuser",
		AuthType:     AuthPassword,
		Password:     "testpass",
		HostKeyStore: mockStore,
	}

	pingClient, err := Dial(ctx, pingCfg)
	if err == nil {
		pingClient.Close()
	}
	latency := time.Since(latencyStart)

	t.Logf("Input/Connection Latency: %v", latency)
	if latency > 100*time.Millisecond {
		t.Errorf("Latency %v exceeded 100ms threshold", latency)
	}

	// Run flood for 5 seconds
	t.Logf("Running flood for 5 seconds...")
	time.Sleep(5 * time.Second)

	// Measure under load
	runtime.ReadMemStats(&ms)
	loadRSS := ms.Alloc

	t.Logf("Load RSS (Alloc): %d MB", loadRSS/1024/1024)
	rssDiff := int64(loadRSS) - int64(baselineRSS)
	rssDiffMB := rssDiff / 1024 / 1024
	if rssDiffMB > 50 {
		t.Errorf("RSS increased by %d MB, expected < 50 MB", rssDiffMB)
	}

	// Stop everything
	t.Logf("Cancelling panes and waiting for cleanup...")
	cancel()
	wg.Wait()

	// Allow goroutines to clean up and GC to run
	time.Sleep(2 * time.Second)
	runtime.GC()
	time.Sleep(1 * time.Second)

	// Measure final state
	finalGoroutines := runtime.NumGoroutine()
	runtime.ReadMemStats(&ms)
	finalRSS := ms.Alloc

	t.Logf("Final Goroutines: %d", finalGoroutines)
	t.Logf("Final RSS (Alloc): %d MB", finalRSS/1024/1024)

	goroutineDiff := finalGoroutines - baselineGoroutines
	if goroutineDiff > 10 { // Allow some slack for background workers
		t.Errorf("Goroutine leak: started with %d, ended with %d (diff: %d)", baselineGoroutines, finalGoroutines, goroutineDiff)
	}
}
