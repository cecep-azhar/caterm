package config

import (
	"os"
	"path/filepath"
	"runtime"
)

// DataPath returns the directory path for caterm data
func DataPath() string {
	if p := os.Getenv("CATERM_DATA_DIR"); p != "" {
		return p
	}

	if portable := os.Getenv("CATERM_PORTABLE"); portable == "1" {
		if exe, err := os.Executable(); err == nil {
			return filepath.Join(filepath.Dir(exe), "data")
		}
	}

	switch runtime.GOOS {
	case "windows":
		return filepath.Join(os.Getenv("APPDATA"), "caterm")
	default:
		home, _ := os.UserHomeDir()
		return filepath.Join(home, ".caterm")
	}
}
