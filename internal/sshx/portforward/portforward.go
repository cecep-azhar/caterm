package portforward

import (
	"context"
	"fmt"
	"io"
	"net"
	"sync"
	"golang.org/x/crypto/ssh"
)

type PortForward struct {
	client     *ssh.Client
	localAddr  string
	remoteAddr string
	listener   net.Listener
	mu         sync.Mutex
	closed     bool
	wg         sync.WaitGroup
	ctx        context.Context
	cancel     context.CancelFunc
}

func New(ctx context.Context, client *ssh.Client, localAddr, remoteAddr string) *PortForward {
	pfCtx, cancel := context.WithCancel(ctx)
	return &PortForward{
		client:     client,
		localAddr:  localAddr,
		remoteAddr: remoteAddr,
		ctx:        pfCtx,
		cancel:     cancel,
	}
}

func (pf *PortForward) Start() error {
	pf.mu.Lock()
	defer pf.mu.Unlock()

	if pf.closed {
		return fmt.Errorf("port forward is closed")
	}

	listener, err := net.Listen("tcp", pf.localAddr)
	if err != nil {
		return fmt.Errorf("failed to listen on %s: %w", pf.localAddr, err)
	}
	pf.listener = listener

	pf.wg.Add(1)
	go pf.acceptLoop()

	return nil
}

func (pf *PortForward) acceptLoop() {
	defer pf.wg.Done()
	for {
		localConn, err := pf.listener.Accept()
		if err != nil {
			select {
			case <-pf.ctx.Done():
				return
			default:
				// Only print or log in a real app, ignoring for now.
				return
			}
		}

		pf.wg.Add(1)
		go pf.handleConn(localConn)
	}
}

func (pf *PortForward) handleConn(localConn net.Conn) {
	defer pf.wg.Done()
	defer localConn.Close()

	remoteConn, err := pf.client.Dial("tcp", pf.remoteAddr)
	if err != nil {
		return
	}
	defer remoteConn.Close()

	var wg sync.WaitGroup
	wg.Add(2)

	go func() {
		defer wg.Done()
		io.Copy(localConn, remoteConn)
		if cw, ok := localConn.(interface{ CloseWrite() error }); ok {
			cw.CloseWrite()
		}
	}()

	go func() {
		defer wg.Done()
		io.Copy(remoteConn, localConn)
		if cw, ok := remoteConn.(interface{ CloseWrite() error }); ok {
			cw.CloseWrite()
		}
	}()

	wg.Wait()
}

func (pf *PortForward) Stop() error {
	pf.mu.Lock()
	defer pf.mu.Unlock()

	if pf.closed {
		return nil
	}
	pf.closed = true
	pf.cancel()

	if pf.listener != nil {
		pf.listener.Close()
	}

	pf.wg.Wait()
	return nil
}