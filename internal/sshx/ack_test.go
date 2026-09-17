package sshx

import (
	"sync/atomic"
	"testing"
	"time"
)

func TestAckFlowControl(t *testing.T) {
	fc := NewFlowControl(1024)

	var waitStarted int32
	var added int32

	go func() {
		atomic.StoreInt32(&waitStarted, 1)
		// Should block when adding beyond 1024
		fc.AddUnacked(1000)
		fc.AddUnacked(50) // Now unacked is 1050 > 1024, it will block
		atomic.StoreInt32(&added, 1)
	}()

	// Give goroutine time to run and block
	time.Sleep(100 * time.Millisecond)

	if atomic.LoadInt32(&added) == 1 {
		t.Fatalf("AddUnacked did not block when exceeding max limit")
	}

	if fc.UnackedBytes() != 1050 {
		t.Fatalf("expected unacked bytes to be 1050, got %d", fc.UnackedBytes())
	}

	// Release block by acknowledging bytes
	fc.Ack(100)

	// Give goroutine time to wake up and finish
	time.Sleep(100 * time.Millisecond)

	if atomic.LoadInt32(&added) == 0 {
		t.Fatalf("AddUnacked did not unblock after Ack")
	}

	if fc.UnackedBytes() != 950 {
		t.Fatalf("expected unacked bytes to be 950, got %d", fc.UnackedBytes())
	}
}
