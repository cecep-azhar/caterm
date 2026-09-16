package main

import (
	"encoding/json"
	"fmt"
	"io"
	"os"
)

func cmdUtil(args []string) {
	if len(args) < 1 {
		fmt.Println("Usage: catermctl util <json-validate|diff-field|diff-shape>")
		os.Exit(1)
	}

	switch args[0] {
	case "json-validate":
		b, err := io.ReadAll(os.Stdin)
		if err != nil {
			fmt.Printf("Error reading stdin: %v\n", err)
			os.Exit(1)
		}
		var js interface{}
		if err := json.Unmarshal(b, &js); err != nil {
			fmt.Printf("Invalid JSON: %v\n", err)
			os.Exit(1)
		}
		fmt.Println("Valid JSON")
	default:
		fmt.Printf("Unknown util subcommand: %s\n", args[0])
		os.Exit(1)
	}
}
