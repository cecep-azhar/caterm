package vault

import (
	"testing"
)

func TestZeroizeOnLock(t *testing.T) {
	v := NewVault()
	dek, err := GenerateDEK()
	if err != nil {
		t.Fatalf("failed to generate DEK: %v", err)
	}

	// Create a copy of DEK to verify against zeroed memory
	originalDEK := make([]byte, len(dek))
	copy(originalDEK, dek)

	v.SetDEK(dek)

	if !v.IsUnlocked() {
		t.Errorf("vault should be unlocked")
	}

	// Lock should zeroize the DEK memory slice
	v.Lock()

	if v.IsUnlocked() {
		t.Errorf("vault should be locked after Lock()")
	}

	// The underlying slice memory (that we passed to SetDEK) is not modified directly,
	// but the slice referenced by v.dek should be all zeros.
	// We need to inspect the internal v.dek slice via reflection or package-internal access.
	// Since we are in package vault, we can access v.dek directly.

	for i, b := range v.dek {
		if b != 0 {
			t.Errorf("expected DEK byte at index %d to be 0, got %x", i, b)
		}
	}
}
