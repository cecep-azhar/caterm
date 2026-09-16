package main

import (
	"fmt"
	"os"

	"caterm/internal/config"
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
		cmdDb(os.Args[2:], config.DataPath())
	case "vault":
		cmdVault(os.Args[2:])
	case "group":
		cmdGroup(os.Args[2:])
	case "util":
		cmdUtil(os.Args[2:])
	default:
		fmt.Printf("Unknown command: %s\n", os.Args[1])
		os.Exit(1)
	}
}
