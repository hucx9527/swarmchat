package cmd

import (
	"fmt"

	"github.com/google/uuid"
	"github.com/urfave/cli/v2"

	"github.com/swarmchat/scp-cli/config"
)

// AgentCommand returns the agent management subcommands.
func AgentCommand(cfg *config.Config) *cli.Command {
	return &cli.Command{
		Name:  "agent",
		Usage: "Manage AI agents and publish agent cards",
		Subcommands: []*cli.Command{
			{
				Name:  "card",
				Usage: "Agent card management",
				Subcommands: []*cli.Command{
					{
						Name:  "publish",
						Usage: "Publish an agent capability card",
						Flags: []cli.Flag{
							&cli.StringFlag{Name: "name", Aliases: []string{"n"}, Required: true, Usage: "Agent name"},
							&cli.StringFlag{Name: "description", Aliases: []string{"desc"}, Usage: "Agent description"},
							&cli.StringSliceFlag{Name: "capability", Aliases: []string{"c"}, Required: true, Usage: "Capabilities (e.g., code_review, security_scan)"},
							&cli.StringSliceFlag{Name: "model", Aliases: []string{"m"}, Usage: "AI models used (e.g., gpt-4, claude-3)"},
							&cli.StringFlag{Name: "callback-url", Aliases: []string{"url"}, Usage: "Webhook callback URL"},
							&cli.UintFlag{Name: "rate-limit", Value: 10, Usage: "Requests per minute limit"},
						},
						Action: func(c *cli.Context) error {
							return publishAgentCard(cfg, c.String("name"), c.String("description"),
								c.StringSlice("capability"), c.StringSlice("model"),
								c.String("callback-url"), c.Uint("rate-limit"))
						},
					},
					{
						Name:  "show",
						Usage: "Show agent card",
						Flags: []cli.Flag{
							&cli.StringFlag{Name: "agent", Aliases: []string{"a"}, Required: true, Usage: "Agent DID"},
						},
						Action: func(c *cli.Context) error {
							return showAgentCard(cfg, c.String("agent"))
						},
					},
				},
			},
			{
				Name:  "task",
				Usage: "Task management for agent collaboration",
				Subcommands: []*cli.Command{
					{
						Name:  "create",
						Usage: "Create a task for agents",
						Flags: []cli.Flag{
							&cli.StringFlag{Name: "group", Aliases: []string{"g"}, Required: true, Usage: "Group ID"},
							&cli.StringFlag{Name: "description", Aliases: []string{"desc"}, Required: true, Usage: "Task description"},
							&cli.StringFlag{Name: "assignee", Aliases: []string{"a"}, Usage: "Specific agent DID"},
							&cli.UintFlag{Name: "deadline", Usage: "Deadline in hours from now"},
						},
						Action: func(c *cli.Context) error {
							return createTask(cfg, c.String("group"), c.String("description"),
								c.String("assignee"), c.Uint("deadline"))
						},
					},
					{
						Name:  "list",
						Usage: "List tasks in a group",
						Flags: []cli.Flag{
							&cli.StringFlag{Name: "group", Aliases: []string{"g"}, Required: true, Usage: "Group ID"},
						},
						Action: func(c *cli.Context) error {
							return listTasks(cfg, c.String("group"))
						},
					},
					{
						Name:  "approve",
						Usage: "Approve or reject a task result",
						Flags: []cli.Flag{
							&cli.StringFlag{Name: "task", Aliases: []string{"t"}, Required: true, Usage: "Task ID"},
							&cli.BoolFlag{Name: "approved", Aliases: []string{"a"}, Value: true, Usage: "Approve (true) or reject (false)"},
							&cli.StringFlag{Name: "feedback", Usage: "Feedback for the agent"},
						},
						Action: func(c *cli.Context) error {
							return approveTask(cfg, c.String("task"), c.Bool("approved"), c.String("feedback"))
						},
					},
				},
			},
		},
	}
}

func publishAgentCard(cfg *config.Config, name, description string, capabilities, models []string, callbackURL string, rateLimit uint) error {
	fromDID, err := getActiveDID(cfg)
	if err != nil {
		return err
	}

	agentID := "agent:" + uuid.New().String()

	fmt.Println("=== Agent Card Published ===")
	fmt.Printf("Agent ID:     %s\n", agentID)
	fmt.Printf("Name:         %s\n", name)
	if description != "" {
		fmt.Printf("Description:  %s\n", description)
	}
	fmt.Printf("Owner:        %s\n", fromDID[:32]+"...")
	fmt.Printf("Capabilities: %v\n", capabilities)
	if len(models) > 0 {
		fmt.Printf("Models:       %v\n", models)
	}
	if callbackURL != "" {
		fmt.Printf("Callback URL: %s\n", callbackURL)
	}
	fmt.Printf("Rate Limit:   %d req/min\n", rateLimit)
	fmt.Println()
	fmt.Println("Scan this agent DID to add it to your groups:")
	fmt.Println(agentID)
	return nil
}

func showAgentCard(cfg *config.Config, agentDID string) error {
	fmt.Println("=== Agent Card ===")
	fmt.Printf("Agent: %s\n", agentDID)
	fmt.Println("(Fetching full agent card from DHT...)")
	return nil
}

func createTask(cfg *config.Config, groupID, description, assignee string, deadline uint) error {
	fromDID, err := getActiveDID(cfg)
	if err != nil {
		return err
	}

	taskID := "task:" + uuid.New().String()

	fmt.Println("=== Task Created ===")
	fmt.Printf("Task ID:     %s\n", taskID)
	fmt.Printf("Group:       %s\n", groupID)
	fmt.Printf("Description: %s\n", description)
	fmt.Printf("Created by:  %s\n", fromDID[:32]+"...")
	if assignee != "" {
		fmt.Printf("Assignee:    %s\n", assignee[:32]+"...")
	} else {
		fmt.Println("Assignee:    (open - any agent can claim)")
	}
	if deadline > 0 {
		fmt.Printf("Deadline:    %d hours\n", deadline)
	}
	return nil
}

func listTasks(cfg *config.Config, groupID string) error {
	fmt.Printf("=== Tasks in Group %s ===\n", groupID)
	fmt.Println("(Task list will be fetched from group state)")
	fmt.Println()
	fmt.Println("Create a task: scp-cli agent task create --group <id> --description \"Review code\"")
	return nil
}

func approveTask(cfg *config.Config, taskID string, approved bool, feedback string) error {
	status := "approved"
	if !approved {
		status = "rejected"
	}

	fmt.Printf("Task %s has been %s.\n", taskID, status)
	if feedback != "" {
		fmt.Printf("Feedback: %s\n", feedback)
	}
	return nil
}
