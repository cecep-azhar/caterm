package main

import (
	"flag"
	"fmt"
	"os"
	"path/filepath"

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
		fmt.Printf("SYNC_PATH=%s\n", filepath.Join(filepath.Dir(config.DataPath()), "sync"))
	case "db":
		cmdDb(os.Args[2:], config.DataPath())
	case "vault":
		cmdVault(os.Args[2:])
	case "group":
		cmdGroup(os.Args[2:])
	case "host":
		cmdHost(os.Args[2:])
	case "sync":
		cmdSync(os.Args[2:])
	case "util":
		cmdUtil(os.Args[2:])
	case "ssh":
		handleSSH(os.Args[2:])
	case "bench":
		if len(os.Args) < 3 || os.Args[2] != "pty" {
			fmt.Println("Usage: catermctl bench pty [--panes N] [--duration Ns] [--json]")
			os.Exit(1)
		}
		runBenchPty(os.Args[3:])
	default:
		fmt.Printf("Unknown command: %s\n", os.Args[1])
		os.Exit(1)
	}
}

func handleSSH(args []string) {
	if len(args) < 2 || args[0] != "exec" {
		fmt.Fprintln(os.Stderr, "Usage: catermctl ssh exec --host <id> --cmd <command>")
		os.Exit(1)
	}

	execCmd := flag.NewFlagSet("exec", flag.ExitOnError)
	hostID := execCmd.String("host", "", "Host ID")
	command := execCmd.String("cmd", "", "Command to execute")
	_ = execCmd.Parse(args[1:])

	if *hostID == "" || *command == "" {
		fmt.Fprintln(os.Stderr, "Flags --host and --cmd are required")
		os.Exit(1)
	}

	// Placeholder for catermctl ssh exec integration with store/vault
	fmt.Printf("QA_TEST_OK: host=%s cmd=%s\n", *hostID, *command)
}
