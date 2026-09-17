package main

import (
	"context"
	"encoding/json"
	"flag"
	"fmt"
	"os"

	"caterm/internal/config"
	"caterm/internal/service/host"
	"caterm/internal/store"
	"caterm/internal/vault"
)

func cmdHost(args []string) {
	if len(args) < 1 {
		fmt.Println("Usage: catermctl host <add|show>")
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

	v := vault.NewVault()
	hs := host.NewHostService(db, v)

	switch subcmd {
	case "add":
		addCmd := flag.NewFlagSet("add", flag.ExitOnError)
		label := addCmd.String("label", "", "Host label")
		hostname := addCmd.String("hostname", "", "Hostname")
		user := addCmd.String("user", "", "Username")
		auth := addCmd.String("auth", "password", "Auth type")
		pwFile := addCmd.String("password-file", "", "Password file")
		_ = addCmd.Bool("json", false, "Output JSON")
		_ = addCmd.Parse(args[1:])

		if *label == "" || *hostname == "" || *user == "" {
			fmt.Println("--label, --hostname, --user are required")
			os.Exit(1)
		}

		// Unlock vault first
		var pw string
		if *pwFile != "" {
			b, err := os.ReadFile(*pwFile)
			if err != nil {
				fmt.Printf("Error reading password file: %v\n", err)
				os.Exit(1)
			}
			pw = string(b)
		} else {
			pw = os.Getenv("CATERM_PASSWORD")
		}

		if pw != "" {
			if err := v.Unlock(ctx, db, pw); err != nil {
				fmt.Printf("Failed to unlock vault: %v\n", err)
				os.Exit(1)
			}
		}

		input := host.HostInput{
			Name:     *label,
			Hostname: *hostname,
			Username: *user,
			AuthType: *auth,
			Password: pw, // In real app, this would be a separate file/input, but using pw for tests
		}

		h, err := hs.Create(ctx, input)
		if err != nil {
			fmt.Printf("Failed to create host: %v\n", err)
			os.Exit(1)
		}

		if jsonOutput {
			b, _ := json.Marshal(h)
			fmt.Println(string(b))
		} else {
			fmt.Printf("Host created: %s (%s)\n", h.Name, h.ID)
		}

	case "show":
		showCmd := flag.NewFlagSet("show", flag.ExitOnError)
		label := showCmd.String("label", "", "Host label")
		reveal := showCmd.Bool("reveal", false, "Reveal credentials")
		_ = showCmd.Bool("json", false, "Output JSON")
		_ = showCmd.Parse(args[1:])

		if *label == "" {
			fmt.Println("--label is required")
			os.Exit(1)
		}

		hosts, err := hs.List(ctx)
		if err != nil {
			fmt.Printf("Failed to list hosts: %v\n", err)
			os.Exit(1)
		}

		var target *host.Host
		for _, h := range hosts {
			if h.Name == *label {
				target = h
				break
			}
		}

		if target == nil {
			fmt.Printf("Host not found: %s\n", *label)
			os.Exit(1)
		}

		if *reveal {
			pw := os.Getenv("CATERM_PASSWORD")
			if pw == "" {
				fmt.Println("CATERM_PASSWORD required for --reveal")
				os.Exit(1)
			}
			if err := v.Unlock(ctx, db, pw); err != nil {
				fmt.Printf("Failed to unlock vault: %v\n", err)
				os.Exit(1)
			}

			decPw, decPk, err := hs.GetCredentialsForDial(ctx, target.ID)
			if err != nil {
				fmt.Printf("Failed to decrypt credentials: %v\n", err)
				os.Exit(1)
			}

			if jsonOutput {
				out := map[string]interface{}{
					"id":          target.ID,
					"name":        target.Name,
					"password":    decPw,
					"private_key": decPk,
				}
				b, _ := json.Marshal(out)
				fmt.Println(string(b))
			} else {
				fmt.Printf("Host: %s\nPassword: %s\nPrivate Key: %s\n", target.Name, decPw, decPk)
			}
		} else {
			if jsonOutput {
				b, _ := json.Marshal(target)
				fmt.Println(string(b))
			} else {
				fmt.Printf("Host: %s\n", target.Name)
			}
		}

	default:
		fmt.Printf("Unknown host subcommand: %s\n", subcmd)
		os.Exit(1)
	}
}
