// scp-cli: SwarmChat Command Line Interface
//
// Provides command-line access to all SCP protocol functionality:
// - Identity creation and management (SS4.3)
// - Encrypted messaging (SS5.1)
// - Group management (SS5.3.2)
// - Agent interaction (SS5.3.3)
//
// Usage:
//   scp-cli identity create --label personal --nickname "Alice"
//   scp-cli message send --to did:key:z6Mk... --text "Hello Swarm!"
//   scp-cli group create --name "Swarm Alpha"
//   scp-cli agent card publish --name "CodeBot" --capability code_review

package main

import (
	"fmt"
	"os"
	"sort"

	"github.com/urfave/cli/v2"

	"github.com/swarmchat/scp-cli/cmd"
	"github.com/swarmchat/scp-cli/config"
)

var (
	Version   = "0.1.0"
	Commit    = "dev"
	BuildTime = "unknown"
)

func main() {
	cfg, err := config.Load()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Warning: could not load config: %v\n", err)
		cfg = config.Default()
	}

	app := &cli.App{
		Name:    "scp-cli",
		Usage:   "SwarmChat Command Line Interface",
		Version: fmt.Sprintf("%s (commit: %s, built: %s)", Version, Commit, BuildTime),
		Authors: []*cli.Author{
			{Name: "SwarmChat Protocol Team"},
		},
		Description: `
SCP CLI provides full access to the Swarm Communication Protocol.
Manage decentralized identities, send encrypted messages, create
swarm groups, and interact with AI agents — all from the command line.`,
		Flags: []cli.Flag{
			&cli.StringFlag{
				Name:    "config",
				Aliases: []string{"c"},
				Value:   "~/.scp/config.json",
				Usage:   "Path to configuration file",
			},
			&cli.StringFlag{
				Name:    "relay",
				Aliases: []string{"r"},
				Value:   cfg.RelayURL,
				Usage:   "Relay node HTTP API URL",
			},
			&cli.BoolFlag{
				Name:    "debug",
				Aliases: []string{"d"},
				Value:   false,
				Usage:   "Enable debug logging",
			},
		},
		Commands: []*cli.Command{
			cmd.IdentityCommand(cfg),
			cmd.MessageCommand(cfg),
			cmd.GroupCommand(cfg),
			cmd.AgentCommand(cfg),
			cmd.PeerCommand(cfg),
		},
	}

	sort.Sort(cli.FlagsByName(app.Flags))
	sort.Sort(cli.CommandsByName(app.Commands))

	if err := app.Run(os.Args); err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		os.Exit(1)
	}
}
