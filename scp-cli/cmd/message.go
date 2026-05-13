package cmd

import (
	"context"
	"encoding/base64"
	"fmt"
	"time"

	"github.com/google/uuid"
	"github.com/urfave/cli/v2"

	"github.com/swarmchat/scp-cli/client"
	"github.com/swarmchat/scp-cli/config"
)

// MessageCommand returns the messaging subcommands.
func MessageCommand(cfg *config.Config) *cli.Command {
	return &cli.Command{
		Name:    "message",
		Aliases: []string{"msg"},
		Usage:   "Send and receive encrypted messages",
		Subcommands: []*cli.Command{
			{
				Name:  "send",
				Usage: "Send a message to a peer or group",
				Flags: []cli.Flag{
					&cli.StringFlag{Name: "to", Aliases: []string{"t"}, Usage: "Recipient DID"},
					&cli.StringFlag{Name: "group", Aliases: []string{"g"}, Usage: "Group ID"},
					&cli.StringFlag{Name: "text", Aliases: []string{"m"}, Usage: "Message text"},
					&cli.StringFlag{Name: "file", Usage: "File path to send"},
					&cli.UintFlag{Name: "ttl", Value: 604800, Usage: "Message TTL in seconds"},
				},
				Action: func(c *cli.Context) error {
					return sendMessage(cfg, c.String("to"), c.String("group"), c.String("text"), c.String("file"), c.Uint("ttl"))
				},
			},
			{
				Name:  "sync",
				Usage: "Sync offline messages from relay",
				Flags: []cli.Flag{
					&cli.Int64Flag{Name: "since", Value: 0, Usage: "Sync messages since timestamp"},
					&cli.IntFlag{Name: "limit", Value: 50, Usage: "Maximum messages to fetch"},
				},
				Action: func(c *cli.Context) error {
					return syncMessages(cfg, c.Int64("since"), c.Int("limit"))
				},
			},
			{
				Name:  "status",
				Usage: "Check relay connection status",
				Action: func(c *cli.Context) error {
					return checkRelayStatus(cfg)
				},
			},
		},
	}
}

func getActiveDID(cfg *config.Config) (string, error) {
	store, err := loadIdentityStore(cfg)
	if err != nil {
		return "", err
	}
	if store.DefaultLabel == "" {
		return "", fmt.Errorf("no default identity. Set one with: scp-cli identity default --label <name>")
	}
	id, ok := store.Identities[store.DefaultLabel]
	if !ok {
		return "", fmt.Errorf("default identity not found")
	}
	return id.DID, nil
}

func sendMessage(cfg *config.Config, to, group, text, filePath string, ttl uint) error {
	if to == "" && group == "" {
		return fmt.Errorf("either --to or --group is required")
	}
	if text == "" && filePath == "" {
		return fmt.Errorf("either --text or --file is required")
	}

	fromDID, err := getActiveDID(cfg)
	if err != nil {
		return err
	}

	msgID := uuid.New().String()

	// Build message payload
	payload := text
	if filePath != "" {
		payload = fmt.Sprintf("[File: %s]", filePath)
	}

	// In production, this would encrypt the payload with Double Ratchet/Sender Key
	// For now we base64-encode as a placeholder for the encrypted envelope
	envelope := base64.StdEncoding.EncodeToString([]byte(payload))

	relayClient := client.NewRelayClient(cfg.RelayURL)
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	targetDID := to
	if group != "" {
		targetDID = group
	}

	msg := &client.OfflineMessage{
		ToDID:     targetDID,
		FromDID:   fromDID,
		MessageID: msgID,
		Envelope:  envelope,
		TTL:       uint32(ttl),
	}

	if err := relayClient.StoreMessage(ctx, msg); err != nil {
		return fmt.Errorf("store message: %w", err)
	}

	fmt.Println("=== Message Sent ===")
	fmt.Printf("ID:      %s\n", msgID)
	fmt.Printf("From:    %s\n", fromDID)
	fmt.Printf("To:      %s\n", targetDID)
	fmt.Printf("TTL:     %ds\n", ttl)
	return nil
}

func syncMessages(cfg *config.Config, since int64, limit int) error {
	did, err := getActiveDID(cfg)
	if err != nil {
		return err
	}

	relayClient := client.NewRelayClient(cfg.RelayURL)
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	resp, err := relayClient.SyncMessages(ctx, did, since, limit)
	if err != nil {
		return fmt.Errorf("sync messages: %w", err)
	}

	if len(resp.Messages) == 0 {
		fmt.Println("No new messages.")
		return nil
	}

	fmt.Printf("=== %d New Message(s) ===\n", len(resp.Messages))
	for _, msg := range resp.Messages {
		// Decode the envelope (placeholder - in production, decrypt with ratchet)
		decoded, _ := base64.StdEncoding.DecodeString(msg.Envelope)
		fmt.Printf("[%s] %s: %s\n",
			time.Unix(msg.Timestamp, 0).Format("15:04:05"),
			msg.FromDID[:20]+"...",
			string(decoded),
		)
	}
	if resp.HasMore {
		fmt.Println("(more messages available, use --since and --limit)")
	}
	return nil
}

func checkRelayStatus(cfg *config.Config) error {
	relayClient := client.NewRelayClient(cfg.RelayURL)
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	health, err := relayClient.HealthCheck(ctx)
	if err != nil {
		return fmt.Errorf("relay unreachable: %w", err)
	}

	fmt.Println("=== Relay Status ===")
	fmt.Printf("URL:     %s\n", cfg.RelayURL)
	fmt.Printf("Status:  %v\n", health["status"])
	fmt.Printf("Version: %v\n", health["version"])
	if peerID, ok := health["peer_id"]; ok {
		fmt.Printf("PeerID:  %v\n", peerID)
	}

	info, err := relayClient.GetNodeInfo(ctx)
	if err == nil {
		fmt.Println("Protocols:", info.Protocols)
	}
	return nil
}
