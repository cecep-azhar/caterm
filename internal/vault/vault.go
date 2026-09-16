package vault

import (
	"crypto/aes"
	"crypto/cipher"
	"crypto/rand"
	"crypto/subtle"
	"encoding/base64"
	"errors"
	"fmt"
	"io"
	"strings"
	"sync"

	"golang.org/x/crypto/argon2"
)

var (
	ErrLocked        = errors.New("vault is locked")
	ErrAlreadyInit   = errors.New("vault is already initialized")
	ErrWrongPassword = errors.New("wrong password")
	ErrInvalidCipher = errors.New("invalid ciphertext format")
	ErrTamperedData  = errors.New("cipher data authentication failed or tampered")
)

const (
	ArgonTime    = 3
	ArgonMemory  = 64 * 1024
	ArgonThreads = 4
	KeyLen       = 32
	SaltLen      = 16
	NonceLen     = 12
)

type Vault struct {
	mu  sync.RWMutex
	dek []byte
}

func NewVault() *Vault {
	return &Vault{}
}

func (v *Vault) IsUnlocked() bool {
	v.mu.RLock()
	defer v.mu.RUnlock()
	return v.dek != nil
}

func (v *Vault) Lock() {
	v.mu.Lock()
	defer v.mu.Unlock()
	if v.dek != nil {
		for i := range v.dek {
			v.dek[i] = 0
		}
		v.dek = nil
	}
}

func DeriveKEK(password string, salt []byte) []byte {
	return argon2.IDKey([]byte(password), salt, ArgonTime, ArgonMemory, ArgonThreads, KeyLen)
}

func GenerateSalt() ([]byte, error) {
	salt := make([]byte, SaltLen)
	if _, err := io.ReadFull(rand.Reader, salt); err != nil {
		return nil, fmt.Errorf("failed to generate salt: %w", err)
	}
	return salt, nil
}

func GenerateDEK() ([]byte, error) {
	dek := make([]byte, KeyLen)
	if _, err := io.ReadFull(rand.Reader, dek); err != nil {
		return nil, fmt.Errorf("failed to generate DEK: %w", err)
	}
	return dek, nil
}

func WrapDEK(dek, kek []byte) (string, error) {
	block, err := aes.NewCipher(kek)
	if err != nil {
		return "", err
	}
	gcm, err := cipher.NewGCM(block)
	if err != nil {
		return "", err
	}
	nonce := make([]byte, gcm.NonceSize())
	if _, err := io.ReadFull(rand.Reader, nonce); err != nil {
		return "", err
	}
	ciphertext := gcm.Seal(nil, nonce, dek, nil)
	return fmt.Sprintf("c1.%s.%s", base64.RawURLEncoding.EncodeToString(nonce), base64.RawURLEncoding.EncodeToString(ciphertext)), nil
}

func UnwrapDEK(wrappedDEK string, kek []byte) ([]byte, error) {
	parts := strings.Split(wrappedDEK, ".")
	if len(parts) != 3 || parts[0] != "c1" {
		return nil, ErrInvalidCipher
	}
	nonce, err := base64.RawURLEncoding.DecodeString(parts[1])
	if err != nil {
		return nil, ErrInvalidCipher
	}
	ciphertext, err := base64.RawURLEncoding.DecodeString(parts[2])
	if err != nil {
		return nil, ErrInvalidCipher
	}
	block, err := aes.NewCipher(kek)
	if err != nil {
		return nil, err
	}
	gcm, err := cipher.NewGCM(block)
	if err != nil {
		return nil, err
	}
	if len(nonce) != gcm.NonceSize() {
		return nil, ErrInvalidCipher
	}
	dek, err := gcm.Open(nil, nonce, ciphertext, nil)
	if err != nil {
		return nil, ErrWrongPassword
	}
	return dek, nil
}

func (v *Vault) SetDEK(dek []byte) {
	v.mu.Lock()
	defer v.mu.Unlock()
	v.dek = make([]byte, len(dek))
	copy(v.dek, dek)
}

func (v *Vault) EncryptField(plain string, recordID, fieldName string) (string, error) {
	v.mu.RLock()
	defer v.mu.RUnlock()
	if v.dek == nil {
		return "", ErrLocked
	}
	block, err := aes.NewCipher(v.dek)
	if err != nil {
		return "", err
	}
	gcm, err := cipher.NewGCM(block)
	if err != nil {
		return "", err
	}
	nonce := make([]byte, gcm.NonceSize())
	if _, err := io.ReadFull(rand.Reader, nonce); err != nil {
		return "", err
	}
	aad := []byte(recordID + "|" + fieldName)
	ciphertext := gcm.Seal(nil, nonce, []byte(plain), aad)
	return fmt.Sprintf("c1.%s.%s", base64.RawURLEncoding.EncodeToString(nonce), base64.RawURLEncoding.EncodeToString(ciphertext)), nil
}

func (v *Vault) DecryptField(cipherStr string, recordID, fieldName string) (string, error) {
	v.mu.RLock()
	defer v.mu.RUnlock()
	if v.dek == nil {
		return "", ErrLocked
	}
	parts := strings.Split(cipherStr, ".")
	if len(parts) != 3 || parts[0] != "c1" {
		return "", ErrInvalidCipher
	}
	nonce, err := base64.RawURLEncoding.DecodeString(parts[1])
	if err != nil {
		return "", ErrInvalidCipher
	}
	ciphertext, err := base64.RawURLEncoding.DecodeString(parts[2])
	if err != nil {
		return "", ErrInvalidCipher
	}
	block, err := aes.NewCipher(v.dek)
	if err != nil {
		return "", err
	}
	gcm, err := cipher.NewGCM(block)
	if err != nil {
		return "", err
	}
	if len(nonce) != gcm.NonceSize() {
		return "", ErrInvalidCipher
	}
	aad := []byte(recordID + "|" + fieldName)
	plain, err := gcm.Open(nil, nonce, ciphertext, aad)
	if err != nil {
		return "", ErrTamperedData
	}
	return string(plain), nil
}

func ConstantTimeCompare(a, b []byte) bool {
	return subtle.ConstantTimeCompare(a, b) == 1
}
