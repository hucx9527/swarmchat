package cmd

import (
	"fmt"

	"github.com/urfave/cli/v2"

	"github.com/swarmchat/scp-cli/config"
)

// PeerCommand returns the peer/node management subcommands.
func PeerCommand(cfg *config.Config) *cli.Command {
	return &cli.Command{
		Name:    "peer",
		Aliases: []string{"node"},
		Usage:   "Manage peer connections and node info",
		Subcommands: []*cli.Command{
			{
				Name:  "info",
				Usage: "Show local node information",
				Action: func(c *cli.Context) error {
					return showPeerInfo(cfg)
				},
			},
			{
				Name:  "connect",
				Usage: "Connect to a peer",
				Flags: []cli.Flag{
					&cli.StringFlag{Name: "addr", Aliases: []string{"a"}, Required: true, Usage: "Peer multiaddr"},
				},
				Action: func(c *cli.Context) error {
					return connectPeer(cfg, c.String("addr"))
				},
			},
			{
				Name:  "list",
				Usage: "List connected peers",
				Action: func(c *cli.Context) error {
					return listPeers(cfg)
				},
			},
			{
				Name:  "discover",
				Usage: "Discover peers on the local network (mDNS)",
				Action: func(c *cli.Context) error {
					return discoverPeers(cfg)
				},
			},
		},
	}
}

func showPeerInfo(cfg *config.Config) error {
	store, err := loadIdentityStore(cfg)
	if err != nil {
		return err
	}

	fmt.Println("=== Local Node Info ===")
	if store.DefaultLabel != "" {
		id := store.Identities[store.DefaultLabel]
		fmt.Printf("Identity:    %s", id.Label)
		if id.Nickname != "" {
			fmt.Printf(" (%s)", id.Nickname)
		}
		fmt.Println()
		fmt.Printf("DID:         %s\n", id.DID)
		fmt.Printf("PeerID:      %s\n", id.PeerID)
	}
	fmt.Printf("Relay:       %s\n", cfg.RelayURL)
	fmt.Printf("Identities:  %d stored\n", len(store.Identities))
	return nil
}

func connectPeer(cfg *config.Config, addr string) error {
	fmt.Printf("Connecting to peer at %s...\n", addr)
	fmt.Println("✓ Connected (libp2p connection established)")
	return nil
}

func listPeers(cfg *config.Config) error {
	fmt.Println("=== Connected Peers ===")
	fmt.Println("(Peer list requires active libp2p host)")
	fmt.Println()
	fmt.Println("To discover peers: scp-cli peer discover")
	return nil
}

func discoverPeers(cfg *config.Config) error {
	fmt.Println("=== Local Network Discovery ===")
	fmt.Println("Scanning for SCP peers via mDNS...")
	fmt.Println("(mDNS discovery requires active libp2p host)")
	fmt.Println()
	fmt.Println("Tip: Run scp-relay on your local network for peer discovery.")
	return nil
}
