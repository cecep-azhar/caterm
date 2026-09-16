package sshx

import (
	"context"
	"crypto/sha256"
	"encoding/base64"
	"errors"
	"fmt"
	"net"

	"golang.org/x/crypto/ssh"
)

type HostKeyStore interface {
	GetHostKey(ctx context.Context, hostID string) (string, error)
	SaveHostKey(ctx context.Context, hostID string, fingerprint string) error
}

type TOFU struct {
	store  HostKeyStore
	hostID string
	ctx    context.Context
}

func NewTOFU(ctx context.Context, store HostKeyStore, hostID string) *TOFU {
	return &TOFU{
		store:  store,
		hostID: hostID,
		ctx:    ctx,
	}
}

func (t *TOFU) HostKeyCallback(hostname string, remote net.Addr, key ssh.PublicKey) error {
	fingerprint := getFingerprint(key)

	saved, err := t.store.GetHostKey(t.ctx, t.hostID)
	if err != nil {
		return fmt.Errorf("failed to check host key: %w", err)
	}

	if saved == "" {
		err = t.store.SaveHostKey(t.ctx, t.hostID, fingerprint)
		if err != nil {
			return fmt.Errorf("failed to save host key: %w", err)
		}
		return nil
	}

	if saved != fingerprint {
		return errors.New("host key mismatch: MITM detected")
	}

	return nil
}

func getFingerprint(key ssh.PublicKey) string {
	hash := sha256.Sum256(key.Marshal())
	return "SHA256:" + base64.StdEncoding.EncodeToString(hash[:])
}
