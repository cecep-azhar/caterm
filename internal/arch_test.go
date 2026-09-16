package arch_test

import (
	"go/parser"
	"go/token"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestArchNoWailsImport(t *testing.T) {
	// Find root of internal package
	rootDir := "."
	if _, err := os.Stat("internal"); err == nil {
		rootDir = "internal"
	}

	err := filepath.Walk(rootDir, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		if info.IsDir() || !strings.HasSuffix(path, ".go") {
			return nil
		}

		fset := token.NewFileSet()
		node, err := parser.ParseFile(fset, path, nil, parser.ImportsOnly)
		if err != nil {
			return err
		}

		for _, imp := range node.Imports {
			importPath := strings.Trim(imp.Path.Value, `"`)
			if strings.Contains(importPath, "github.com/wailsapp/wails") {
				t.Errorf("K-1 Violation: file %s imports Wails package %s", path, importPath)
			}
		}
		return nil
	})

	if err != nil {
		t.Fatalf("Failed to walk internal directory: %v", err)
	}
}
