package main

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"strings"

	"caterm/internal/config"
	"caterm/internal/store"
	"caterm/internal/vault"
)

func getPassword(args []string) (string, error) {
	pwFile := ""
	for i, arg := range args {
		if arg == "--password-file" && i+1 < len(args) {
			pwFile = args[i+1]
			break
		}
	}

	if pwFile == "" {
		envPw := os.Getenv("CATERM_PASSWORD")
		if envPw != "" {
			return envPw, nil
		}
		return "", fmt.Errorf("password not provided. use --password-file or CATERM_PASSWORD env")
	}

	b, err := os.ReadFile(pwFile)
	if err != nil {
		return "", fmt.Errorf("error reading password file: %w", err)
	}
	return string(b), nil
}

func getOldNewPassword(args []string) (string, string, error) {
	oldFile, newFile := "", ""
	for i, arg := range args {
		if arg == "--old-file" && i+1 < len(args) {
			oldFile = args[i+1]
		}
		if arg == "--new-file" && i+1 < len(args) {
			newFile = args[i+1]
		}
	}
	if oldFile == "" || newFile == "" {
		return "", "", fmt.Errorf("missing --old-file or --new-file")
	}
	bOld, err := os.ReadFile(oldFile)
	if err != nil {
		return "", "", err
	}
	bNew, err := os.ReadFile(newFile)
	if err != nil {
		return "", "", err
	}
	return string(bOld), string(bNew), nil
}

func cmdVault(args []string) {
	if len(args) < 1 {
		fmt.Println("Usage: catermctl vault <init|unlock|status|change-password>")
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

	switch subcmd {
	case "init":
		pw, err := getPassword(args)
		if err != nil {
			fmt.Println(err)
			os.Exit(1)
		}

		v := vault.NewVault()
		if err := db.Migrate(ctx); err != nil {
			fmt.Printf("Migration error: %v\n", err)
			os.Exit(1)
		}
		if err := v.Init(ctx, db, pw); err != nil {
			fmt.Printf("Init error: %v\n", err)
			os.Exit(1)
		}
		if jsonOutput {
			fmt.Println(`{"status":"initialized"}`)
		} else {
			fmt.Println("Vault initialized successfully")
		}

	case "unlock":
		pw, err := getPassword(args)
		if err != nil {
			fmt.Println(err)
			os.Exit(1)
		}

		v := vault.NewVault()
		if err := v.Unlock(ctx, db, pw); err != nil {
			// Do not distinguish between "not exists" and "wrong password"
			if err == vault.ErrWrongPassword || strings.Contains(err.Error(), "not initialized") {
				fmt.Printf("Unlock error: wrong password\n")
			} else {
				fmt.Printf("Unlock error: %v\n", err)
			}
			os.Exit(1)
		}
		if jsonOutput {
			fmt.Println(`{"status":"unlocked"}`)
		} else {
			fmt.Println("Vault unlocked successfully")
		}

	case "status":
		status := "locked"
		var metaCount int
		err := db.QueryRowContext(ctx, "SELECT COUNT(*) FROM vault_meta").Scan(&metaCount)
		if err == nil && metaCount == 0 {
			status = "uninitialized"
		}
		if jsonOutput {
			b, _ := json.Marshal(map[string]string{"status": status})
			fmt.Println(string(b))
		} else {
			fmt.Printf("Vault status: %s\n", status)
		}

	case "change-password":
		oldPw, newPw, err := getOldNewPassword(args)
		if err != nil {
			fmt.Println(err)
			os.Exit(1)
		}
		v := vault.NewVault()
		if err := v.ChangePassword(ctx, db, oldPw, newPw); err != nil {
			fmt.Printf("Change password error: %v\n", err)
			os.Exit(1)
		}
		if jsonOutput {
			fmt.Println(`{"status":"password_changed"}`)
		} else {
			fmt.Println("Password changed successfully")
		}

	default:
		fmt.Printf("Unknown vault subcommand: %s\n", subcmd)
		os.Exit(1)
	}
}
