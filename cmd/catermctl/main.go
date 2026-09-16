package main

import (
	"fmt"
	"os"

	"caterm/internal/config"
)

func main() {
	if len(os.Args) > 1 && os.Args[1] == "env" {
		fmt.Printf("DATA_PATH=%s\n", config.DataPath())
		return
	}
	fmt.Println("catermctl harness")
}
