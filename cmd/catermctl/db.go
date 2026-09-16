package main

import (
	"context"
	"encoding/json"
	"fmt"
	"os"

	"caterm/internal/store"
)

func cmdDb(args []string, dbPath string) {
	if len(args) < 1 {
		fmt.Println("Usage: catermctl db <schema|migrate|dump|scan>")
		os.Exit(1)
	}

	db, err := store.Open(dbPath)
	if err != nil {
		fmt.Printf("Error opening db: %v\n", err)
		os.Exit(1)
	}
	defer db.Close()

	ctx := context.Background()

	subcmd := args[0]
	jsonOutput := false
	for _, arg := range args {
		if arg == "--json" {
			jsonOutput = true
			break
		}
	}

	switch subcmd {
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

	case "dump":
		table := ""
		for i, arg := range args {
			if arg == "--table" && i+1 < len(args) {
				table = args[i+1]
				break
			}
		}
		if table == "" {
			fmt.Println("Missing --table <name>")
			os.Exit(1)
		}

		rows, err := db.QueryContext(ctx, fmt.Sprintf("SELECT * FROM %s", table)) // Unsafe for generic use, but acceptable for internal dump tool
		if err != nil {
			fmt.Printf("Error querying table: %v\n", err)
			os.Exit(1)
		}
		defer rows.Close()

		cols, err := rows.Columns()
		if err != nil {
			fmt.Printf("Error getting columns: %v\n", err)
			os.Exit(1)
		}

		var result []map[string]interface{}
		for rows.Next() {
			vals := make([]interface{}, len(cols))
			valPtrs := make([]interface{}, len(cols))
			for i := range cols {
				valPtrs[i] = &vals[i]
			}
			if err := rows.Scan(valPtrs...); err != nil {
				fmt.Printf("Error scanning: %v\n", err)
				os.Exit(1)
			}

			rowMap := make(map[string]interface{})
			for i, col := range cols {
				val := vals[i]
				b, ok := val.([]byte)
				if ok {
					rowMap[col] = string(b)
				} else {
					rowMap[col] = val
				}
			}
			result = append(result, rowMap)
		}

		if jsonOutput {
			b, _ := json.MarshalIndent(result, "", "  ")
			fmt.Println(string(b))
		} else {
			for _, r := range result {
				fmt.Printf("%+v\n", r)
			}
		}

	default:
		fmt.Printf("Unknown db subcommand: %s\n", subcmd)
		os.Exit(1)
	}
}
