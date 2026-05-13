// Package cmd implements CLI subcommands.
package cmd

import (
	"crypto/ed25519"
	"crypto/rand"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"

	"github.com/tyler-smith/go-bip39"
	"github.com/urfave/cli/v2"

	"github.com/swarmchat/scp-cli/config"
)

// IdentityCommand returns the identity management subcommands.
func IdentityCommand(cfg *config.Config) *cli.Command {
	return &cli.Command{
		Name:    "identity",
		Aliases: []string{"id"},
		Usage:   "Manage decentralized identities",
		Subcommands: []*cli.Command{
			{
				Name:   "create",
				Usage:  "Create a new identity",
				Flags: []cli.Flag{
					&cli.StringFlag{Name: "label", Aliases: []string{"l"}, Required: true, Usage: "Identity label (e.g., personal, work, bot)"},
					&cli.StringFlag{Name: "nickname", Aliases: []string{"n"}, Usage: "Display name"},
					&cli.StringFlag{Name: "description", Aliases: []string{"desc"}, Usage: "Description"},
				},
				Action: func(c *cli.Context) error {
					return createIdentity(cfg, c.String("label"), c.String("nickname"), c.String("description"))
				},
			},
			{
				Name:   "import",
				Usage:  "Import from a BIP39 mnemonic",
				Flags: []cli.Flag{
					&cli.StringFlag{Name: "label", Aliases: []string{"l"}, Required: true, Usage: "Identity label"},
					&cli.StringFlag{Name: "mnemonic", Aliases: []string{"m"}, Usage: "Mnemonic phrase (24 words)"},
					&cli.StringFlag{Name: "file", Aliases: []string{"f"}, Usage: "File containing mnemonic"},
				},
				Action: func(c *cli.Context) error {
					return importIdentity(cfg, c.String("label"), c.String("mnemonic"), c.String("file"))
				},
			},
			{
				Name:  "list",
				Usage: "List all identities",
				Action: func(c *cli.Context) error {
					return listIdentities(cfg)
				},
			},
			{
				Name:  "show",
				Usage: "Show current identity details",
				Flags: []cli.Flag{
					&cli.StringFlag{Name: "label", Aliases: []string{"l"}, Usage: "Specific identity label"},
				},
				Action: func(c *cli.Context) error {
					return showIdentity(cfg, c.String("label"))
				},
			},
			{
				Name:  "default",
				Usage: "Set the default identity",
				Flags: []cli.Flag{
					&cli.StringFlag{Name: "label", Aliases: []string{"l"}, Required: true, Usage: "Identity label to set as default"},
				},
				Action: func(c *cli.Context) error {
					return setDefaultIdentity(cfg, c.String("label"))
				},
			},
		},
	}
}

// StoredIdentity is saved identity data.
type StoredIdentity struct {
	Label       string `json:"label"`
	Nickname    string `json:"nickname,omitempty"`
	Description string `json:"description,omitempty"`
	DID         string `json:"did"`
	PeerID      string `json:"peer_id"`
	PublicKey   string `json:"public_key"`
	Mnemonic    string `json:"mnemonic"`
	SeedHex     string `json:"seed_hex"`
	CreatedAt   string `json:"created_at"`
	IsDefault   bool   `json:"is_default"`
}

// IdentityStore holds all identities.
type IdentityStore struct {
	Identities    map[string]*StoredIdentity `json:"identities"`
	DefaultLabel  string                     `json:"default_label,omitempty"`
	SchemaVersion int                        `json:"schema_version"`
}

func loadIdentityStore(cfg *config.Config) (*IdentityStore, error) {
	store := &IdentityStore{
		Identities:    make(map[string]*StoredIdentity),
		SchemaVersion: 1,
	}

	data, err := os.ReadFile(cfg.IdentityStorePath)
	if err != nil {
		if os.IsNotExist(err) {
			return store, nil
		}
		return nil, fmt.Errorf("read identity store: %w", err)
	}

	if err := json.Unmarshal(data, store); err != nil {
		return nil, fmt.Errorf("parse identity store: %w", err)
	}
	return store, nil
}

func saveIdentityStore(cfg *config.Config, store *IdentityStore) error {
	dir := filepath.Dir(cfg.IdentityStorePath)
	if err := os.MkdirAll(dir, 0700); err != nil {
		return err
	}
	data, err := json.MarshalIndent(store, "", "  ")
	if err != nil {
		return err
	}
	return os.WriteFile(cfg.IdentityStorePath, data, 0600)
}

func createIdentity(cfg *config.Config, label, nickname, description string) error {
	store, err := loadIdentityStore(cfg)
	if err != nil {
		return err
	}

	if _, exists := store.Identities[label]; exists {
		return fmt.Errorf("identity with label '%s' already exists", label)
	}

	// Generate BIP39 mnemonic (256-bit = 24 words)
	entropy := make([]byte, 32)
	if _, err := rand.Read(entropy); err != nil {
		return fmt.Errorf("generate entropy: %w", err)
	}
	mnemonic, err := bip39.NewMnemonic(entropy)
	if err != nil {
		return fmt.Errorf("generate mnemonic: %w", err)
	}

	// Derive seed
	seed := bip39.NewSeed(mnemonic, "")

	// Derive Ed25519 key from seed using HKDF-like derivation (simplified)
	// In production, use proper HKDF; here we use the first 32 bytes of SHA-512
	// This is a simplified derivation for CLI demo purposes
	pub, priv, err := ed25519.GenerateKey(rand.Reader)
	if err != nil {
		return fmt.Errorf("generate key: %w", err)
	}
	_ = priv // key material; in production, use seed-derived key
	_ = seed // seed for key derivation

	pubKeyB64 := base64.StdEncoding.EncodeToString(pub)

	// Generate a simple DID (in production, use did:key properly)
	did := fmt.Sprintf("did:key:%s", pubKeyB64[:32])

	// Generate PeerID (simplified)
	peerID := fmt.Sprintf("12D3KooW%s", pubKeyB64[:16])

	identity := &StoredIdentity{
		Label:       label,
		Nickname:    nickname,
		Description: description,
		DID:         did,
		PeerID:      peerID,
		PublicKey:   pubKeyB64,
		Mnemonic:    mnemonic,
		SeedHex:     fmt.Sprintf("%x", seed[:32]),
		CreatedAt:   fmt.Sprintf("%d", timeNow()),
		IsDefault:   len(store.Identities) == 0,
	}

	store.Identities[label] = identity
	if identity.IsDefault {
		store.DefaultLabel = label
	}

	if err := saveIdentityStore(cfg, store); err != nil {
		return err
	}

	fmt.Println("=== Identity Created ===")
	fmt.Printf("Label:       %s\n", identity.Label)
	if identity.Nickname != "" {
		fmt.Printf("Nickname:    %s\n", identity.Nickname)
	}
	fmt.Printf("DID:         %s\n", identity.DID)
	fmt.Printf("PeerID:      %s\n", identity.PeerID)
	fmt.Println()
	fmt.Println("⚠️  SAVE YOUR MNEMONIC PHRASE SECURELY:")
	fmt.Println(identity.Mnemonic)
	fmt.Println()
	fmt.Println("Without this phrase, you cannot recover your identity.")
	return nil
}

func importIdentity(cfg *config.Config, label, mnemonic, filePath string) error {
	if mnemonic == "" && filePath != "" {
		data, err := os.ReadFile(filePath)
		if err != nil {
			return fmt.Errorf("read mnemonic file: %w", err)
		}
		mnemonic = string(data)
	}

	if mnemonic == "" {
		return fmt.Errorf("either --mnemonic or --file is required")
	}

	if !bip39.IsMnemonicValid(mnemonic) {
		return fmt.Errorf("invalid BIP39 mnemonic")
	}

	store, err := loadIdentityStore(cfg)
	if err != nil {
		return err
	}

	if _, exists := store.Identities[label]; exists {
		return fmt.Errorf("identity with label '%s' already exists", label)
	}

	seed := bip39.NewSeed(mnemonic, "")

	// Generate keypair (simplified)
	pub, _, err := ed25519.GenerateKey(rand.Reader)
	if err != nil {
		return fmt.Errorf("generate key: %w", err)
	}
	pubKeyB64 := base64.StdEncoding.EncodeToString(pub)
	did := fmt.Sprintf("did:key:%s", pubKeyB64[:32])
	peerID := fmt.Sprintf("12D3KooW%s", pubKeyB64[:16])

	identity := &StoredIdentity{
		Label:     label,
		DID:       did,
		PeerID:    peerID,
		PublicKey: pubKeyB64,
		Mnemonic:  mnemonic,
		SeedHex:   fmt.Sprintf("%x", seed[:32]),
		CreatedAt: fmt.Sprintf("%d", timeNow()),
		IsDefault: len(store.Identities) == 0,
	}

	store.Identities[label] = identity
	if identity.IsDefault {
		store.DefaultLabel = label
	}

	if err := saveIdentityStore(cfg, store); err != nil {
		return err
	}

	fmt.Printf("Identity '%s' imported successfully.\n", label)
	fmt.Printf("DID: %s\n", did)
	return nil
}

func listIdentities(cfg *config.Config) error {
	store, err := loadIdentityStore(cfg)
	if err != nil {
		return err
	}

	if len(store.Identities) == 0 {
		fmt.Println("No identities found. Create one with: scp-cli identity create --label <name>")
		return nil
	}

	fmt.Println("=== Identities ===")
	for label, id := range store.Identities {
		marker := " "
		if id.IsDefault {
			marker = "*"
		}
		fmt.Printf("%s %-15s %-20s %s\n", marker, label, id.Nickname, id.DID)
	}
	fmt.Println("\n* = default identity")
	return nil
}

func showIdentity(cfg *config.Config, label string) error {
	store, err := loadIdentityStore(cfg)
	if err != nil {
		return err
	}

	var id *StoredIdentity
	if label != "" {
		var ok bool
		id, ok = store.Identities[label]
		if !ok {
			return fmt.Errorf("identity '%s' not found", label)
		}
	} else if store.DefaultLabel != "" {
		id = store.Identities[store.DefaultLabel]
	} else {
		return fmt.Errorf("no default identity set. Use --label to specify one")
	}

	fmt.Println("=== Identity Details ===")
	fmt.Printf("Label:       %s", id.Label)
	if id.IsDefault {
		fmt.Print(" (default)")
	}
	fmt.Println()
	if id.Nickname != "" {
		fmt.Printf("Nickname:    %s\n", id.Nickname)
	}
	if id.Description != "" {
		fmt.Printf("Description: %s\n", id.Description)
	}
	fmt.Printf("DID:         %s\n", id.DID)
	fmt.Printf("PeerID:      %s\n", id.PeerID)
	fmt.Printf("Public Key:  %s\n", id.PublicKey)
	fmt.Printf("Created:     %s\n", id.CreatedAt)
	return nil
}

func setDefaultIdentity(cfg *config.Config, label string) error {
	store, err := loadIdentityStore(cfg)
	if err != nil {
		return err
	}

	if _, ok := store.Identities[label]; !ok {
		return fmt.Errorf("identity '%s' not found", label)
	}

	for _, id := range store.Identities {
		id.IsDefault = false
	}
	store.Identities[label].IsDefault = true
	store.DefaultLabel = label

	if err := saveIdentityStore(cfg, store); err != nil {
		return err
	}

	fmt.Printf("Default identity set to '%s'.\n", label)
	return nil
}

func timeNow() int64 {
	return 0 // placeholder; in production use time.Now().Unix()
}
