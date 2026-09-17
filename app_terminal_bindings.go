package main

import (
	"errors"
	"time"

	"caterm/internal/sshx"
)

func (a *App) ConnectTerminal(paneID string, hostID string, rows, cols int) error {
	if a.ctx == nil {
		return errors.New("context not initialized")
	}

	h, err := a.hostService.GetByID(a.ctx, hostID)
	if err != nil {
		return err
	}

	pass, key, err := a.hostService.GetCredentialsForDial(a.ctx, hostID)
	if err != nil {
		return err
	}

	var authType sshx.AuthType
	if h.AuthType == "password" {
		authType = sshx.AuthPassword
	} else {
		authType = sshx.AuthKey
	}

	cfg := sshx.DialerConfig{
		HostID:      hostID,
		Hostname:    h.Hostname,
		Port:        h.Port,
		User:        h.Username,
		AuthType:    authType,
		Password:    pass,
		PrivateKey:  []byte(key),
		DialTimeout: 10 * time.Second,
	}

	client, err := sshx.Dial(a.ctx, cfg)
	if err != nil {
		return err
	}

	sess, err := sshx.NewSession(a.ctx, client, rows, cols)
	if err != nil {
		return err
	}

	a.terminalService.StartSession(paneID, sess, client)
	return nil
}

func (a *App) WriteTerminal(paneID string, data string) error {
	if a.ctx == nil {
		return errors.New("context not initialized")
	}
	return a.terminalService.Write(paneID, []byte(data))
}

func (a *App) ResizeTerminal(paneID string, rows, cols int) error {
	if a.ctx == nil {
		return errors.New("context not initialized")
	}
	return a.terminalService.Resize(paneID, rows, cols)
}

func (a *App) CloseTerminal(paneID string) error {
	if a.ctx == nil {
		return errors.New("context not initialized")
	}
	a.terminalService.ClosePane(paneID)
	return nil
}

func (a *App) AckTerminal(paneID string, bytes int) error {
	if a.ctx == nil {
		return errors.New("context not initialized")
	}
	a.terminalService.Ack(paneID, int64(bytes))
	return nil
}
