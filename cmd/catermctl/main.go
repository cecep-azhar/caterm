package main

import (
	"context"
	"fmt"
	"os"

	"caterm/internal/config"
	"caterm/internal/store"
)

func main() {
	if len(os.Args) < 2 {
		fmt.Println("Usage: catermctl <command>")
		os.Exit(1)
	}

	switch os.Args[1] {
	case "env":
		fmt.Printf("DATA_PATH=%s\n", config.DataPath())

	case "db":
		if len(os.Args) < 3 {
			fmt.Println("Usage: catermctl db <schema|migrate|dump|scan>")
			os.Exit(1)
		}

		dbPath := config.DataPath()
		db, err := store.Open(dbPath)
		if err != nil {
			fmt.Printf("Error opening db: %v\n", err)
			os.Exit(1)
		}
		defer db.Close()

		ctx := context.Background()

		switch os.Args[2] {
		case "migrate":
			if err := db.Migrate(ctx); err != nil {
				fmt.Printf("Migrate error: %v\n", err)
				os.Exit(1)
			}
			fmt.Println("Migration successful")

		case "schema":
			rows, err := db.QueryContext(ctx, "SELECT sql FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'")
			if err != nil {
				fmt.Printf("Error querying schema: %v\n", err)
				os.Exit(1)
			}
			defer rows.Close()
			for rows.Next() {
				var sqlStr string
				if err := rows.Scan(&sqlStr); err != nil {
					fmt.Printf("Error scanning row: %v\n", err)
					os.Exit(1)
				}
				fmt.Println(sqlStr)
			}

		default:
			fmt.Printf("Unknown db subcommand: %s\n", os.Args[2])
			os.Exit(1)
		}

	default:
		fmt.Printf("Unknown command: %s\n", os.Args[1])
		os.Exit(1)
	}
}
