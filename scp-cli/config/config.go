// Package config manages CLI configuration and identity store.
package config

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
)

// Config holds the CLI configuration.
type Config struct {
	// Relay node HTTP API URL
	RelayURL string `json:"relay_url"`

	// Default identity label
	DefaultIdentity string `json:"default_identity,omitempty"`

	// Path to identity store file
	IdentityStorePath string `json:"identity_store_path"`

	// Known peer addresses
	KnownPeers map[string]string `json:"known_peers,omitempty"`

	// Default message TTL in seconds
	DefaultTTL uint32 `json:"default_ttl"`
}

// Default returns the default configuration.
func Default() *Config {
	home, _ := os.UserHomeDir()
	return &Config{
		RelayURL:          "http://localhost:8080",
		IdentityStorePath: filepath.Join(home, ".scp", "identities.json"),
		KnownPeers:        make(map[string]string),
		DefaultTTL:        604800, // 7 days
	}
}

// Load reads configuration from the default path.
func Load() (*Config, error) {
	home, err := os.UserHomeDir()
	if err != nil {
		return Default(), nil
	}
	configPath := filepath.Join(home, ".scp", "config.json")
	return LoadFrom(configPath)
}

// LoadFrom reads configuration from a specific path.
func LoadFrom(path string) (*Config, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		if os.IsNotExist(err) {
			return Default(), nil
		}
		return nil, fmt.Errorf("read config: %w", err)
	}

	cfg := Default()
	if err := json.Unmarshal(data, cfg); err != nil {
		return nil, fmt.Errorf("parse config: %w", err)
	}
	return cfg, nil
}

// Save writes configuration to the default path.
func (c *Config) Save() error {
	home, err := os.UserHomeDir()
	if err != nil {
		return err
	}
	configPath := filepath.Join(home, ".scp", "config.json")
	return c.SaveTo(configPath)
}

// SaveTo writes configuration to a specific path.
func (c *Config) SaveTo(path string) error {
	dir := filepath.Dir(path)
	if err := os.MkdirAll(dir, 0700); err != nil {
		return fmt.Errorf("create config dir: %w", err)
	}

	data, err := json.MarshalIndent(c, "", "  ")
	if err != nil {
		return fmt.Errorf("marshal config: %w", err)
	}

	return os.WriteFile(path, data, 0600)
}
