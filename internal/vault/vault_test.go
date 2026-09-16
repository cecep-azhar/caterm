package vault

import (
	"bytes"
	"crypto/rand"
	"encoding/base64"
	"io"
	"strings"
	"testing"
)

type errReader struct{}

func (errReader) Read(p []byte) (n int, err error) {
	return 0, io.ErrUnexpectedEOF
}

func TestGenerateSaltError(t *testing.T) {
	oldRand := rand.Reader
	rand.Reader = errReader{}
	defer func() { rand.Reader = oldRand }()

	_, err := GenerateSalt()
	if err == nil {
		t.Errorf("expected error when generate salt fails")
	}
}

func TestGenerateDEKError(t *testing.T) {
	oldRand := rand.Reader
	rand.Reader = errReader{}
	defer func() { rand.Reader = oldRand }()

	_, err := GenerateDEK()
	if err == nil {
		t.Errorf("expected error when generate DEK fails")
	}
}

func TestWrapDEKError(t *testing.T) {
	oldRand := rand.Reader
	rand.Reader = errReader{}
	defer func() { rand.Reader = oldRand }()

	_, err := WrapDEK([]byte("12345678901234567890123456789012"), []byte("12345678901234567890123456789012"))
	if err == nil {
		t.Errorf("expected error when WrapDEK rand fails")
	}

	// Invalid key length for aes.NewCipher
	_, err = WrapDEK([]byte("1234"), []byte("invalid_key_len"))
	if err == nil {
		t.Errorf("expected error for invalid KEK length")
	}
}

func TestUnwrapDEKInvalidKeyLength(t *testing.T) {
	// Invalid key length for aes.NewCipher
	validWrappedStr := "c1.123456789012.123456789012345678901234"
	_, err := UnwrapDEK(validWrappedStr, []byte("invalid_key_len"))
	if err == nil {
		t.Errorf("expected error for invalid KEK length")
	}
}

func TestEncryptFieldError(t *testing.T) {
	oldRand := rand.Reader
	rand.Reader = errReader{}
	defer func() { rand.Reader = oldRand }()

	v := NewVault()
	v.SetDEK([]byte("12345678901234567890123456789012"))

	_, err := v.EncryptField("plain", "rec", "field")
	if err == nil {
		t.Errorf("expected error when EncryptField rand fails")
	}
}

func TestUnwrapDEKInvalidNonceAndCipher(t *testing.T) {
	kek := []byte("12345678901234567890123456789012")
	// Invalid b64
	_, err := UnwrapDEK("c1.invalid_b64!.c29tZQ==", kek)
	if err != ErrInvalidCipher {
		t.Errorf("expected ErrInvalidCipher, got %v", err)
	}
	_, err = UnwrapDEK("c1.c29tZQ==.invalid_b64!", kek)
	if err != ErrInvalidCipher {
		t.Errorf("expected ErrInvalidCipher, got %v", err)
	}
	// Wrong nonce size
	shortNonce := base64.RawURLEncoding.EncodeToString([]byte("short"))
	_, err = UnwrapDEK("c1."+shortNonce+".c29tZQ==", kek)
	if err != ErrInvalidCipher {
		t.Errorf("expected ErrInvalidCipher for short nonce, got %v", err)
	}
}

func TestDecryptFieldInvalidNonce(t *testing.T) {
	v := NewVault()
	v.SetDEK([]byte("12345678901234567890123456789012"))

	shortNonce := base64.RawURLEncoding.EncodeToString([]byte("short"))
	_, err := v.DecryptField("c1."+shortNonce+".c29tZQ==", "rec", "field")
	if err != ErrInvalidCipher {
		t.Errorf("expected ErrInvalidCipher for short nonce, got %v", err)
	}

	// Invalid b64 cipher
	_, err = v.DecryptField("c1.123456789012.invalid_b64!", "rec", "field")
	if err != ErrInvalidCipher {
		t.Errorf("expected ErrInvalidCipher, got %v", err)
	}
}

func TestConstantTimeCompare(t *testing.T) {
	if !ConstantTimeCompare([]byte("a"), []byte("a")) {
		t.Errorf("expected true")
	}
	if ConstantTimeCompare([]byte("a"), []byte("b")) {
		t.Errorf("expected false")
	}
}

func TestRoundtripEncryptDecrypt(t *testing.T) {
	v := NewVault()
	dek, err := GenerateDEK()
	if err != nil {
		t.Fatalf("failed to generate DEK: %v", err)
	}
	v.SetDEK(dek)

	plain := "QA_TEST_secret_password_123"
	recordID := "rec-uuid-001"
	fieldName := "password"

	cipherStr, err := v.EncryptField(plain, recordID, fieldName)
	if err != nil {
		t.Fatalf("failed to encrypt: %v", err)
	}

	if !strings.HasPrefix(cipherStr, "c1.") {
		t.Errorf("expected cipher prefix c1., got %s", cipherStr)
	}

	decrypted, err := v.DecryptField(cipherStr, recordID, fieldName)
	if err != nil {
		t.Fatalf("failed to decrypt: %v", err)
	}

	if decrypted != plain {
		t.Errorf("expected %s, got %s", plain, decrypted)
	}
}

func TestWrongPassword(t *testing.T) {
	password := "QA_TEST_correct_pass_123"
	wrongPass := "QA_TEST_wrong_pass_456"

	salt, err := GenerateSalt()
	if err != nil {
		t.Fatalf("failed to generate salt: %v", err)
	}

	dek, err := GenerateDEK()
	if err != nil {
		t.Fatalf("failed to generate DEK: %v", err)
	}

	kek := DeriveKEK(password, salt)
	wrappedDEK, err := WrapDEK(dek, kek)
	if err != nil {
		t.Fatalf("failed to wrap DEK: %v", err)
	}

	wrongKEK := DeriveKEK(wrongPass, salt)
	_, err = UnwrapDEK(wrappedDEK, wrongKEK)
	if err != ErrWrongPassword {
		t.Errorf("expected ErrWrongPassword, got %v", err)
	}
}

func TestTamper(t *testing.T) {
	v := NewVault()
	dek, err := GenerateDEK()
	if err != nil {
		t.Fatalf("failed to generate DEK: %v", err)
	}
	v.SetDEK(dek)

	plain := "QA_TEST_secret_value"
	recordID := "rec-uuid-002"
	fieldName := "password"

	cipherStr, err := v.EncryptField(plain, recordID, fieldName)
	if err != nil {
		t.Fatalf("failed to encrypt: %v", err)
	}

	parts := strings.Split(cipherStr, ".")
	rawCipher, _ := base64.RawURLEncoding.DecodeString(parts[2])
	rawCipher[0] ^= 0xff // tamper 1 byte
	tamperedParts2 := base64.RawURLEncoding.EncodeToString(rawCipher)
	tamperedCipherStr := parts[0] + "." + parts[1] + "." + tamperedParts2

	_, err = v.DecryptField(tamperedCipherStr, recordID, fieldName)
	if err != ErrTamperedData {
		t.Errorf("expected ErrTamperedData, got %v", err)
	}
}

func TestAAD(t *testing.T) {
	v := NewVault()
	dek, err := GenerateDEK()
	if err != nil {
		t.Fatalf("failed to generate DEK: %v", err)
	}
	v.SetDEK(dek)

	plain := "QA_TEST_secret_value"
	recordA := "rec-uuid-A"
	recordB := "rec-uuid-B"
	fieldName := "password"

	cipherStr, err := v.EncryptField(plain, recordA, fieldName)
	if err != nil {
		t.Fatalf("failed to encrypt: %v", err)
	}

	// Try decrypting with recordB instead of recordA
	_, err = v.DecryptField(cipherStr, recordB, fieldName)
	if err != ErrTamperedData {
		t.Errorf("expected ErrTamperedData for mismatched AAD, got %v", err)
	}

	// Try decrypting with fieldName "private_key" instead of "password"
	_, err = v.DecryptField(cipherStr, recordA, "private_key")
	if err != ErrTamperedData {
		t.Errorf("expected ErrTamperedData for mismatched field AAD, got %v", err)
	}
}

func TestChangePassword(t *testing.T) {
	oldPassword := "QA_TEST_old_pass_123"
	newPassword := "QA_TEST_new_pass_456"

	salt, err := GenerateSalt()
	if err != nil {
		t.Fatalf("failed to generate salt: %v", err)
	}

	dek, err := GenerateDEK()
	if err != nil {
		t.Fatalf("failed to generate DEK: %v", err)
	}

	oldKEK := DeriveKEK(oldPassword, salt)
	oldWrappedDEK, err := WrapDEK(dek, oldKEK)
	if err != nil {
		t.Fatalf("failed to wrap DEK with old KEK: %v", err)
	}

	v := NewVault()
	v.SetDEK(dek)

	plain := "QA_TEST_my_secret_key"
	recordID := "rec-001"
	fieldName := "password"

	cipherTextBefore, err := v.EncryptField(plain, recordID, fieldName)
	if err != nil {
		t.Fatalf("failed to encrypt field: %v", err)
	}

	// Simulasikan Ganti Password: DEK tidak berubah, KEK & salt baru dibuat, wrappedDEK diperbarui
	newSalt, err := GenerateSalt()
	if err != nil {
		t.Fatalf("failed to generate new salt: %v", err)
	}
	newKEK := DeriveKEK(newPassword, newSalt)
	newWrappedDEK, err := WrapDEK(dek, newKEK)
	if err != nil {
		t.Fatalf("failed to wrap DEK with new KEK: %v", err)
	}

	if oldWrappedDEK == newWrappedDEK {
		t.Errorf("wrapped_dek must change after password change")
	}

	// Pastikan ciphertext field lama tetap terbaca dengan DEK yang sama
	decryptedBefore, err := v.DecryptField(cipherTextBefore, recordID, fieldName)
	if err != nil {
		t.Fatalf("failed to decrypt old ciphertext after password change: %v", err)
	}
	if decryptedBefore != plain {
		t.Errorf("expected %s, got %s", plain, decryptedBefore)
	}

	// Check unwrap with new password
	unwrappedDEK, err := UnwrapDEK(newWrappedDEK, newKEK)
	if err != nil {
		t.Fatalf("failed to unwrap DEK with new KEK: %v", err)
	}
	if !bytes.Equal(unwrappedDEK, dek) {
		t.Errorf("unwrapped DEK does not match original DEK")
	}
}

func TestLockedState(t *testing.T) {
	v := NewVault()

	if v.IsUnlocked() {
		t.Errorf("new vault should be locked")
	}

	_, err := v.EncryptField("plain", "rec", "field")
	if err != ErrLocked {
		t.Errorf("expected ErrLocked, got %v", err)
	}

	_, err = v.DecryptField("c1.nonce.cipher", "rec", "field")
	if err != ErrLocked {
		t.Errorf("expected ErrLocked, got %v", err)
	}
}

func TestLockZeroize(t *testing.T) {
	v := NewVault()
	dek, err := GenerateDEK()
	if err != nil {
		t.Fatalf("failed to generate DEK: %v", err)
	}
	v.SetDEK(dek)

	if !v.IsUnlocked() {
		t.Errorf("vault should be unlocked")
	}

	v.Lock()

	if v.IsUnlocked() {
		t.Errorf("vault should be locked after Lock()")
	}
}

func TestInvalidCipherFormat(t *testing.T) {
	v := NewVault()
	dek, _ := GenerateDEK()
	v.SetDEK(dek)

	invalidFormatStrs := []string{
		"c2.nonce.cipher",
		"c1.nonce",
		"c1.invalid_b64!.cipher",
		"c1.nonce.invalid_b64!",
	}

	for _, str := range invalidFormatStrs {
		_, err := v.DecryptField(str, "rec", "field")
		if err != ErrInvalidCipher {
			t.Errorf("expected ErrInvalidCipher for string %s, got %v", str, err)
		}
	}
}
