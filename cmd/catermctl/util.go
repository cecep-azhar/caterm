package main

import (
	"crypto/sha256"
	"fmt"
	"io"
	"os"
)

func cmdUtil(args []string) {
	if len(args) < 1 {
		fmt.Println("Usage: catermctl util <hash>")
		os.Exit(1)
	}

	switch args[0] {
	case "hash":
		file := ""
		for i, arg := range args {
			if arg == "--file" && i+1 < len(args) {
				file = args[i+1]
				break
			}
		}
		if file == "" {
			fmt.Println("Missing --file flag")
			os.Exit(1)
		}

		f, err := os.Open(file)
		if err != nil {
			fmt.Printf("Error opening file: %v\n", err)
			os.Exit(1)
		}
		defer f.Close()

		h := sha256.New()
		if _, err := io.Copy(h, f); err != nil {
			fmt.Printf("Error hashing file: %v\n", err)
			os.Exit(1)
		}
		fmt.Printf("%x\n", h.Sum(nil))

	default:
		fmt.Printf("Unknown util command: %s\n", args[0])
		os.Exit(1)
	}
}
