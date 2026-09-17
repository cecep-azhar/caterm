package main

import (
	"context"
	"fmt"
	"os"
	"path/filepath"

	"caterm/internal/config"
	"caterm/internal/store"
	"caterm/internal/sync"
)

func cmdSync(args []string) {
	if len(args) < 1 {
		fmt.Println("Usage: catermctl sync <export|import>")
		os.Exit(1)
	}

	dbPath := config.DataPath()
	syncPath := filepath.Join(filepath.Dir(dbPath), "sync")

	switch args[0] {
	case "export":
		db, err := store.Open(dbPath)
		if err != nil {
			fmt.Printf("failed to open db: %v\n", err)
			os.Exit(1)
		}
		defer db.Close()

		exporter := sync.NewExporter(db, syncPath)
		if err := exporter.Export(context.Background()); err != nil {
			fmt.Printf("Export failed: %v\n", err)
			os.Exit(1)
		}
		fmt.Printf("Exported successfully to %s\n", syncPath)

	case "import":
		fmt.Println("Not implemented yet")
		os.Exit(1)

	default:
		fmt.Printf("Unknown sync command: %s\n", args[0])
		os.Exit(1)
	}
}
