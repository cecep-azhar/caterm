package main

import (
	"context"
	"encoding/json"
	"flag"
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
		importCmd := flag.NewFlagSet("import", flag.ExitOnError)
		dryRun := importCmd.Bool("dry-run", false, "Perform dry-run without modifying database")
		jsonOutput := importCmd.Bool("json", false, "Output in JSON format")
		_ = importCmd.Parse(args[1:])

		db, err := store.Open(dbPath)
		if err != nil {
			fmt.Printf("failed to open db: %v\n", err)
			os.Exit(1)
		}
		defer db.Close()

		if err := db.Migrate(context.Background()); err != nil {
			fmt.Printf("Migration failed: %v\n", err)
			os.Exit(1)
		}

		importer := sync.NewImporter(db, syncPath)
		res, err := importer.Import(context.Background(), *dryRun)
		if err != nil {
			fmt.Printf("Import failed: %v\n", err)
			os.Exit(1)
		}

		if *jsonOutput {
			b, _ := json.Marshal(res)
			fmt.Println(string(b))
		} else {
			fmt.Printf("Import completed: Added=%d, Updated=%d, Deleted=%d, Skipped=%d\n",
				res.Added, res.Updated, res.Deleted, len(res.Skipped))
		}

	default:
		fmt.Printf("Unknown sync command: %s\n", args[0])
		os.Exit(1)
	}
}
