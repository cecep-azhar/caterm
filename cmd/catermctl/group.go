package main

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"strconv"

	"caterm/internal/config"
	"caterm/internal/service/host"
	"caterm/internal/store"
)

func cmdGroup(args []string) {
	if len(args) < 1 {
		fmt.Println("Usage: catermctl group <add|edit|delete|list>")
		os.Exit(1)
	}

	subcmd := args[0]
	jsonOutput := false
	for _, arg := range args {
		if arg == "--json" {
			jsonOutput = true
			break
		}
	}

	dbPath := config.DataPath()
	db, err := store.Open(dbPath)
	if err != nil {
		fmt.Printf("Error opening db: %v\n", err)
		os.Exit(1)
	}
	defer db.Close()
	ctx := context.Background()

	svc := host.NewGroupService(db)

	switch subcmd {
	case "add":
		name := ""
		sortOrder := 0
		for i, arg := range args {
			if arg == "--name" && i+1 < len(args) {
				name = args[i+1]
			}
			if arg == "--sort" && i+1 < len(args) {
				sortOrder, _ = strconv.Atoi(args[i+1])
			}
		}

		g, err := svc.Create(ctx, name, sortOrder)
		if err != nil {
			fmt.Printf("Error: %v\n", err)
			os.Exit(1)
		}
		if jsonOutput {
			b, _ := json.Marshal(g)
			fmt.Println(string(b))
		} else {
			fmt.Printf("Group created: %s (%s)\n", g.Name, g.ID)
		}

	case "edit":
		id := ""
		name := ""
		sortOrder := -1
		for i, arg := range args {
			if arg == "--id" && i+1 < len(args) {
				id = args[i+1]
			}
			if arg == "--name" && i+1 < len(args) {
				name = args[i+1]
			}
			if arg == "--sort" && i+1 < len(args) {
				sortOrder, _ = strconv.Atoi(args[i+1])
			}
		}

		if id == "" {
			fmt.Println("Missing --id")
			os.Exit(1)
		}

		// Fallback to existing if not provided
		existing, err := svc.GetByID(ctx, id)
		if err != nil {
			fmt.Printf("Error: %v\n", err)
			os.Exit(1)
		}
		if name == "" {
			name = existing.Name
		}
		if sortOrder == -1 {
			sortOrder = existing.SortOrder
		}

		g, err := svc.Update(ctx, id, name, sortOrder)
		if err != nil {
			fmt.Printf("Error: %v\n", err)
			os.Exit(1)
		}
		if jsonOutput {
			b, _ := json.Marshal(g)
			fmt.Println(string(b))
		} else {
			fmt.Printf("Group updated: %s (%s)\n", g.Name, g.ID)
		}

	case "delete":
		id := ""
		for i, arg := range args {
			if arg == "--id" && i+1 < len(args) {
				id = args[i+1]
			}
		}
		if id == "" {
			fmt.Println("Missing --id")
			os.Exit(1)
		}

		if err := svc.Delete(ctx, id); err != nil {
			fmt.Printf("Error: %v\n", err)
			os.Exit(1)
		}
		if jsonOutput {
			fmt.Println(`{"status":"deleted"}`)
		} else {
			fmt.Println("Group deleted")
		}

	case "list":
		groups, err := svc.List(ctx)
		if err != nil {
			fmt.Printf("Error: %v\n", err)
			os.Exit(1)
		}
		if jsonOutput {
			b, _ := json.Marshal(groups)
			fmt.Println(string(b))
		} else {
			for _, g := range groups {
				fmt.Printf("%s - %s\n", g.ID, g.Name)
			}
		}

	default:
		fmt.Printf("Unknown group subcommand: %s\n", subcmd)
		os.Exit(1)
	}
}
