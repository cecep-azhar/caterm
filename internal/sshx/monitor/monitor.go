package monitor

import (
	"bytes"
	"context"
	"fmt"
	"strconv"
	"strings"
	"time"

	"golang.org/x/crypto/ssh"
)

type Stats struct {
	CPUUsage  float64 `json:"cpu_usage"`
	RAMTotal  uint64  `json:"ram_total"`
	RAMUsed   uint64  `json:"ram_used"`
	DiskTotal uint64  `json:"disk_total"`
	DiskUsed  uint64  `json:"disk_used"`
}

type Monitor struct {
	client *ssh.Client
	ctx    context.Context
	cancel context.CancelFunc
}

func New(ctx context.Context, client *ssh.Client) *Monitor {
	ctx, cancel := context.WithCancel(ctx)
	return &Monitor{
		client: client,
		ctx:    ctx,
		cancel: cancel,
	}
}

func (m *Monitor) GetStats() (Stats, error) {
	var stats Stats

	session, err := m.client.NewSession()
	if err != nil {
		return stats, fmt.Errorf("failed to create session: %w", err)
	}
	defer session.Close()

	cmd := `
# CPU Usage
cat /proc/stat | grep '^cpu ' | awk '{print $2" "$4" "$5}' &&
# Memory
free -b | grep Mem | awk '{print $2" "$3}' &&
# Disk
df -B1 / | tail -n 1 | awk '{print $2" "$3}'
`
	var stdout bytes.Buffer
	session.Stdout = &stdout
	
	if err := session.Run(cmd); err != nil {
		return stats, fmt.Errorf("failed to run monitor command: %w", err)
	}

	lines := strings.Split(strings.TrimSpace(stdout.String()), "\n")
	if len(lines) < 3 {
		return stats, fmt.Errorf("unexpected output format")
	}

	// CPU
	cpuParts := strings.Fields(lines[0])
	if len(cpuParts) == 3 {
		user, _ := strconv.ParseFloat(cpuParts[0], 64)
		sys, _ := strconv.ParseFloat(cpuParts[1], 64)
		idle, _ := strconv.ParseFloat(cpuParts[2], 64)
		total := user + sys + idle
		if total > 0 {
			stats.CPUUsage = ((user + sys) / total) * 100
		}
	}

	// RAM
	ramParts := strings.Fields(lines[1])
	if len(ramParts) == 2 {
		stats.RAMTotal, _ = strconv.ParseUint(ramParts[0], 10, 64)
		stats.RAMUsed, _ = strconv.ParseUint(ramParts[1], 10, 64)
	}

	// Disk
	diskParts := strings.Fields(lines[2])
	if len(diskParts) == 2 {
		stats.DiskTotal, _ = strconv.ParseUint(diskParts[0], 10, 64)
		stats.DiskUsed, _ = strconv.ParseUint(diskParts[1], 10, 64)
	}

	return stats, nil
}

func (m *Monitor) Watch(interval time.Duration, callback func(Stats, error)) {
	go func() {
		ticker := time.NewTicker(interval)
		defer ticker.Stop()

		for {
			select {
			case <-m.ctx.Done():
				return
			case <-ticker.C:
				stats, err := m.GetStats()
				callback(stats, err)
			}
		}
	}()
}

func (m *Monitor) Stop() {
	m.cancel()
}