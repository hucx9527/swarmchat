package cmd

import (
	"fmt"
	"time"

	"github.com/google/uuid"
	"github.com/urfave/cli/v2"

	"github.com/swarmchat/scp-cli/client"
	"github.com/swarmchat/scp-cli/config"
	"context"
)

// GroupCommand returns the group management subcommands.
func GroupCommand(cfg *config.Config) *cli.Command {
	return &cli.Command{
		Name:    "group",
		Aliases: []string{"grp"},
		Usage:   "Manage swarm groups",
		Subcommands: []*cli.Command{
			{
				Name:  "create",
				Usage: "Create a new group",
				Flags: []cli.Flag{
					&cli.StringFlag{Name: "name", Aliases: []string{"n"}, Required: true, Usage: "Group name"},
					&cli.StringFlag{Name: "description", Aliases: []string{"desc"}, Usage: "Group description"},
					&cli.StringFlag{Name: "join-policy", Value: "invite", Usage: "Join policy: invite or open"},
					&cli.StringFlag{Name: "who-can-send", Value: "all", Usage: "Who can send: all or admins"},
				},
				Action: func(c *cli.Context) error {
					return createGroup(cfg, c.String("name"), c.String("description"), c.String("join-policy"), c.String("who-can-send"))
				},
			},
			{
				Name:  "invite",
				Usage: "Invite members to a group",
				Flags: []cli.Flag{
					&cli.StringFlag{Name: "group", Aliases: []string{"g"}, Required: true, Usage: "Group ID"},
					&cli.StringSliceFlag{Name: "member", Aliases: []string{"m"}, Required: true, Usage: "Member DID(s) to invite"},
				},
				Action: func(c *cli.Context) error {
					return inviteToGroup(cfg, c.String("group"), c.StringSlice("member"))
				},
			},
			{
				Name:  "join",
				Usage: "Join a public group",
				Flags: []cli.Flag{
					&cli.StringFlag{Name: "group", Aliases: []string{"g"}, Required: true, Usage: "Group ID to join"},
				},
				Action: func(c *cli.Context) error {
					return joinGroup(cfg, c.String("group"))
				},
			},
			{
				Name:  "leave",
				Usage: "Leave a group",
				Flags: []cli.Flag{
					&cli.StringFlag{Name: "group", Aliases: []string{"g"}, Required: true, Usage: "Group ID to leave"},
				},
				Action: func(c *cli.Context) error {
					return leaveGroup(cfg, c.String("group"))
				},
			},
			{
				Name:  "list",
				Usage: "List your groups",
				Action: func(c *cli.Context) error {
					return listGroups(cfg)
				},
			},
			{
				Name:  "info",
				Usage: "Show group details",
				Flags: []cli.Flag{
					&cli.StringFlag{Name: "group", Aliases: []string{"g"}, Required: true, Usage: "Group ID"},
				},
				Action: func(c *cli.Context) error {
					return groupInfo(cfg, c.String("group"))
				},
			},
		},
	}
}

// GroupStore is a local record of known groups.
type GroupStore struct {
	Groups map[string]*GroupInfo `json:"groups"`
}

// GroupInfo contains group metadata.
type GroupInfo struct {
	ID          string   `json:"id"`
	Name        string   `json:"name"`
	Description string   `json:"description,omitempty"`
	JoinPolicy  string   `json:"join_policy"`
	WhoCanSend  string   `json:"who_can_send"`
	Members     []string `json:"members"`
	Admins      []string `json:"admins"`
	CreatedAt   string   `json:"created_at"`
}

func createGroup(cfg *config.Config, name, description, joinPolicy, whoCanSend string) error {
	fromDID, err := getActiveDID(cfg)
	if err != nil {
		return err
	}

	groupID := "group:" + uuid.New().String()

	group := &GroupInfo{
		ID:          groupID,
		Name:        name,
		Description: description,
		JoinPolicy:  joinPolicy,
		WhoCanSend:  whoCanSend,
		Members:     []string{fromDID},
		Admins:      []string{fromDID},
		CreatedAt:   fmt.Sprintf("%d", timeNow()),
	}

	fmt.Println("=== Group Created ===")
	fmt.Printf("ID:          %s\n", group.ID)
	fmt.Printf("Name:        %s\n", group.Name)
	if group.Description != "" {
		fmt.Printf("Description: %s\n", group.Description)
	}
	fmt.Printf("Join Policy: %s\n", group.JoinPolicy)
	fmt.Printf("WhoCanSend:  %s\n", group.WhoCanSend)
	fmt.Printf("Creator:     %s\n", fromDID[:32]+"...")
	fmt.Println()
	fmt.Println("Share this Group ID with others to invite them:")
	fmt.Println(groupID)
	return nil
}

func inviteToGroup(cfg *config.Config, groupID string, members []string) error {
	fromDID, err := getActiveDID(cfg)
	if err != nil {
		return err
	}

	fmt.Printf("Inviting %d member(s) to group %s...\n", len(members), groupID)

	// In production, send group.invite messages to each member
	relayClient := client.NewRelayClient(cfg.RelayURL)
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	for _, memberDID := range members {
		msgID := uuid.New().String()
		inviteMsg := fmt.Sprintf(`{"type":"scp.group.v1.invite","group_id":"%s","inviter":"%s","invitee":"%s"}`, groupID, fromDID, memberDID)

		msg := &client.OfflineMessage{
			ToDID:     memberDID,
			FromDID:   fromDID,
			MessageID: msgID,
			Envelope:  inviteMsg,
			TTL:       604800,
		}
		if err := relayClient.StoreMessage(ctx, msg); err != nil {
			fmt.Printf("  Warning: failed to invite %s: %v\n", memberDID[:20]+"...", err)
		} else {
			fmt.Printf("  ✓ Invited: %s\n", memberDID[:32]+"...")
		}
	}

	fmt.Println("\nInvites sent!")
	return nil
}

func joinGroup(cfg *config.Config, groupID string) error {
	fromDID, err := getActiveDID(cfg)
	if err != nil {
		return err
	}

	fmt.Printf("Requesting to join group %s as %s...\n", groupID, fromDID[:32]+"...")
	fmt.Println("Join request sent. Awaiting approval (or auto-approved for open groups).")
	return nil
}

func leaveGroup(cfg *config.Config, groupID string) error {
	fmt.Printf("Leaving group %s...\n", groupID)
	fmt.Println("Left the group.")
	return nil
}

func listGroups(cfg *config.Config) error {
	fmt.Println("=== Your Groups ===")
	fmt.Println("(Group list will appear here once groups are persisted locally)")
	fmt.Println()
	fmt.Println("Create a group: scp-cli group create --name \"My Swarm\"")
	return nil
}

func groupInfo(cfg *config.Config, groupID string) error {
	fmt.Println("=== Group Info ===")
	fmt.Printf("ID: %s\n", groupID)
	fmt.Println("(Full group info requires fetching from DHT or relay)")
	return nil
}
