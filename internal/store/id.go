package store

import (
	"crypto/rand"
	"sync"
	"time"

	"github.com/oklog/ulid/v2"
)

var (
	entropyMu sync.Mutex
	entropy   = ulid.Monotonic(rand.Reader, 0)
)

// NewID returns a string UUIDv7 / ULID that is lexicographically sortable by creation time.
func NewID() string {
	entropyMu.Lock()
	defer entropyMu.Unlock()

	ms := ulid.Timestamp(time.Now())
	id, err := ulid.New(ms, entropy)
	if err != nil {
		// Fallback if monotonic entropy fails
		return ulid.Make().String()
	}
	return id.String()
}

// NowUTC returns the current UTC time formatted as ISO8601 with millisecond precision.
func NowUTC() string {
	return time.Now().UTC().Format("2006-01-02T15:04:05.000Z")
}
