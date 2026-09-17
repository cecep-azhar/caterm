package sshx

import (
	"sync"
)

type FlowControl struct {
	mu           sync.Mutex
	unackedBytes int64
	maxUnacked   int64
	cond         *sync.Cond
}

func NewFlowControl(maxUnacked int64) *FlowControl {
	fc := &FlowControl{
		maxUnacked: maxUnacked,
	}
	fc.cond = sync.NewCond(&fc.mu)
	return fc
}

func (fc *FlowControl) AddUnacked(n int64) {
	fc.mu.Lock()
	defer fc.mu.Unlock()
	fc.unackedBytes += n
	for fc.unackedBytes >= fc.maxUnacked {
		fc.cond.Wait()
	}
}

func (fc *FlowControl) Ack(n int64) {
	fc.mu.Lock()
	defer fc.mu.Unlock()
	fc.unackedBytes -= n
	if fc.unackedBytes < 0 {
		fc.unackedBytes = 0
	}
	if fc.unackedBytes < fc.maxUnacked {
		fc.cond.Broadcast()
	}
}

func (fc *FlowControl) UnackedBytes() int64 {
	fc.mu.Lock()
	defer fc.mu.Unlock()
	return fc.unackedBytes
}
