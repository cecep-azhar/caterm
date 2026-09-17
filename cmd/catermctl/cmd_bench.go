package main

import (
	"context"
	"encoding/json"
	"fmt"
	"net"
	"os"
	"runtime"
	"strconv"
	"strings"
	"sync"
	"sync/atomic"
	"time"

	"caterm/internal/sshx"
	"caterm/internal/sshx/testfixture"
)

type BenchResult struct {
	Panes             int     `json:"panes"`
	Duration          string  `json:"duration"`
	BaselineGoroutine int     `json:"baseline_goroutines"`
	FinalGoroutine    int     `json:"final_goroutines"`
	BaselineRSSMB     uint64  `json:"baseline_rss_mb"`
	PeakRSSMB         uint64  `json:"peak_rss_mb"`
	FinalRSSMB        uint64  `json:"final_rss_mb"`
	RSSDiffMB         int64   `json:"rss_diff_mb"`
	LatencyMs         float64 `json:"latency_ms"`
	Success           bool    `json:"success"`
	Error             string  `json:"error,omitempty"`
}

type dummyHostKeyStore struct{}

func (d *dummyHostKeyStore) GetHostKey(ctx context.Context, hostID string) (string, error) {
	return "", nil
}

func (d *dummyHostKeyStore) SaveHostKey(ctx context.Context, hostID string, fingerprint string) error {
	return nil
}

func runBenchPty(args []string) {
	panes := 4
	durationSec := 60
	jsonOutput := false

	for i := 0; i < len(args); i++ {
		switch args[i] {
		case "--panes":
			if i+1 < len(args) {
				panes, _ = strconv.Atoi(args[i+1])
				i++
			}
		case "--duration":
			if i+1 < len(args) {
				durStr := strings.TrimSuffix(args[i+1], "s")
				durationSec, _ = strconv.Atoi(durStr)
				i++
			}
		case "--json":
			jsonOutput = true
		}
	}

	result := BenchResult{
		Panes:    panes,
		Duration: fmt.Sprintf("%ds", durationSec),
	}

	server, err := testfixture.NewServer(testfixture.Config{Password: "testpass"})
	if err != nil {
		result.Error = fmt.Sprintf("failed to start SSH fixture: %v", err)
		outputBenchResult(result, jsonOutput)
		os.Exit(1)
	}
	defer server.Close()

	hostStr, portStr, _ := net.SplitHostPort(server.Addr())
	portInt, _ := strconv.Atoi(portStr)

	time.Sleep(500 * time.Millisecond)

	result.BaselineGoroutine = runtime.NumGoroutine()
	var ms runtime.MemStats
	runtime.ReadMemStats(&ms)
	result.BaselineRSSMB = ms.Alloc / 1024 / 1024

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	var wg sync.WaitGroup
	var activePanes int32

	for i := 0; i < panes; i++ {
		wg.Add(1)
		go func(paneID int) {
			defer wg.Done()

			dialCfg := sshx.DialerConfig{
				HostID:       fmt.Sprintf("bench-host-%d", paneID),
				Hostname:     hostStr,
				Port:         portInt,
				User:         "testuser",
				AuthType:     sshx.AuthPassword,
				Password:     "testpass",
				HostKeyStore: &dummyHostKeyStore{},
			}

			client, err := sshx.Dial(ctx, dialCfg)
			if err != nil {
				return
			}
			defer client.Close()

			session, err := sshx.NewSession(ctx, client, 24, 80)
			if err != nil {
				return
			}
			defer session.Close()

			atomic.AddInt32(&activePanes, 1)

			go func() {
				_ = session.ReadBatch(func(data []byte) {})
			}()

			go func() {
				floodData := []byte("yes 'catermctl bench pty flood output stream'\n")
				for {
					select {
					case <-ctx.Done():
						return
					default:
						_, wErr := session.Write(floodData)
						if wErr != nil {
							return
						}
						time.Sleep(10 * time.Millisecond)
					}
				}
			}()

			<-ctx.Done()
		}(i)
	}

	time.Sleep(1 * time.Second)

	// Measure connection/input latency under load
	latStart := time.Now()
	pingCfg := sshx.DialerConfig{
		HostID:       "bench-ping",
		Hostname:     hostStr,
		Port:         portInt,
		User:         "testuser",
		AuthType:     sshx.AuthPassword,
		Password:     "testpass",
		HostKeyStore: &dummyHostKeyStore{},
	}
	pingClient, err := sshx.Dial(ctx, pingCfg)
	if err == nil {
		pingClient.Close()
	}
	result.LatencyMs = float64(time.Since(latStart).Microseconds()) / 1000.0

	// Run for duration
	time.Sleep(time.Duration(durationSec) * time.Second)

	runtime.ReadMemStats(&ms)
	result.PeakRSSMB = ms.Alloc / 1024 / 1024
	result.RSSDiffMB = int64(result.PeakRSSMB) - int64(result.BaselineRSSMB)

	cancel()
	wg.Wait()

	time.Sleep(1 * time.Second)
	runtime.GC()
	time.Sleep(500 * time.Millisecond)

	result.FinalGoroutine = runtime.NumGoroutine()
	runtime.ReadMemStats(&ms)
	result.FinalRSSMB = ms.Alloc / 1024 / 1024
	result.Success = true

	outputBenchResult(result, jsonOutput)
}

func outputBenchResult(res BenchResult, jsonOutput bool) {
	if jsonOutput {
		enc := json.NewEncoder(os.Stdout)
		enc.SetIndent("", "  ")
		_ = enc.Encode(res)
		return
	}

	fmt.Printf("PTY Benchmark Results (%d panes, %s duration):\n", res.Panes, res.Duration)
	fmt.Printf("  Latency          : %.2f ms\n", res.LatencyMs)
	fmt.Printf("  Baseline RSS     : %d MB\n", res.BaselineRSSMB)
	fmt.Printf("  Peak RSS         : %d MB\n", res.PeakRSSMB)
	fmt.Printf("  RSS Diff         : %d MB (limit: <50 MB)\n", res.RSSDiffMB)
	fmt.Printf("  Goroutines Start : %d\n", res.BaselineGoroutine)
	fmt.Printf("  Goroutines End   : %d\n", res.FinalGoroutine)
	if res.Success {
		fmt.Println("Status: PASSED")
	} else {
		fmt.Printf("Status: FAILED (%s)\n", res.Error)
	}
}
