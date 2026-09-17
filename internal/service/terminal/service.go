package terminal

import (
	"context"
	"fmt"
	"sync"

	"caterm/internal/sshx"
)

type Service struct {
	emitter  EventEmitter
	ctx      context.Context
	sessions map[string]*SessionState
	mu       sync.Mutex
	isTest   bool // Added to allow testing without Wails runtime context
}

type SessionState struct {
	PaneID      string
	Session     *sshx.Session
	Client      interface{} // *ssh.Client interface abstraction
	FlowControl *sshx.FlowControl
	Cancel      context.CancelFunc
}

func New() *Service {
	return &Service{
		sessions: make(map[string]*SessionState),
	}
}

func (s *Service) Startup(ctx context.Context) {
	s.ctx = ctx
}

// Ack receives an acknowledgment from the frontend that N bytes were processed.
func (s *Service) Ack(paneID string, bytes int64) {
	s.mu.Lock()
	state, exists := s.sessions[paneID]
	s.mu.Unlock()

	if exists && state.FlowControl != nil {
		state.FlowControl.Ack(bytes)
	}
}

// StartSession initializes flow control for a given pane ID
func (s *Service) StartSession(paneID string, sess *sshx.Session) {
	ctx, cancel := context.WithCancel(context.Background())
	if s.ctx != nil {
		ctx, cancel = context.WithCancel(s.ctx)
	}

	fc := sshx.NewFlowControl(256 * 1024) // 256KB max unacked

	state := &SessionState{
		PaneID:      paneID,
		Session:     sess,
		FlowControl: fc,
		Cancel:      cancel,
	}

	s.mu.Lock()
	s.sessions[paneID] = state
	s.mu.Unlock()

	go func() {
		defer cancel()
		defer s.ClosePane(paneID)

		_ = sess.ReadBatch(func(data []byte) {
			fc.AddUnacked(int64(len(data)))

			// Wails events are used to send output to frontend, skip in tests
			if s.ctx != nil && !s.isTest {
				s.emitEvent(ctx, fmt.Sprintf("terminal:output:%s", paneID), data)
			}
		})
	}()
}

// Write passes input to the active session
func (s *Service) Write(paneID string, data []byte) error {
	s.mu.Lock()
	state, exists := s.sessions[paneID]
	s.mu.Unlock()

	if !exists {
		return fmt.Errorf("session not found: %s", paneID)
	}

	_, err := state.Session.Write(data)
	return err
}

// Resize handles frontend resize events and passes them to the SSH session
func (s *Service) Resize(paneID string, rows, cols int) error {
	s.mu.Lock()
	state, exists := s.sessions[paneID]
	s.mu.Unlock()

	if !exists {
		return fmt.Errorf("session not found: %s", paneID)
	}

	return state.Session.WindowChange(rows, cols)
}

// ClosePane terminates a specific pane's session
func (s *Service) ClosePane(paneID string) {
	s.mu.Lock()
	state, exists := s.sessions[paneID]
	if exists {
		delete(s.sessions, paneID)
	}
	s.mu.Unlock()

	if exists {
		state.Session.Close()
		state.Cancel()
		if s.ctx != nil && !s.isTest {
			s.emitEvent(s.ctx, fmt.Sprintf("terminal:closed:%s", paneID), nil)
		}
	}
}

type EventEmitter func(ctx context.Context, eventName string, optionalData ...interface{})

func (s *Service) SetEventEmitter(emitter EventEmitter) {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.emitter = emitter
}

func (s *Service) emitEvent(ctx context.Context, eventName string, data interface{}) {
	s.mu.Lock()
	emitter := s.emitter
	s.mu.Unlock()

	if emitter != nil && ctx != nil {
		emitter(ctx, eventName, data)
	}
}
