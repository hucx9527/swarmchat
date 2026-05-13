// Package client provides the HTTP client for communicating with SCP relay nodes.
package client

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"time"
)

// RelayClient communicates with an SCP relay node HTTP API.
type RelayClient struct {
	baseURL    string
	httpClient *http.Client
}

// PrekeyUpload is the request body for uploading a prekey bundle.
type PrekeyUpload struct {
	DID             string   `json:"did"`
	IdentityKey     string   `json:"identity_key"`
	SignedPrekey    PrekeyEntry `json:"signed_prekey"`
	OneTimePrekeys  []string `json:"one_time_prekeys"`
}

// PrekeyEntry represents a prekey with its signature.
type PrekeyEntry struct {
	Key       string `json:"key"`
	Signature string `json:"signature"`
}

// PrekeyBundle is the response for a prekey query.
type PrekeyBundle struct {
	DID              string      `json:"did"`
	IdentityKey      string      `json:"identity_key"`
	SignedPrekey     PrekeyEntry `json:"signed_prekey"`
	OneTimePrekey    string      `json:"one_time_prekey,omitempty"`
	PrekeysRemaining int         `json:"prekeys_remaining"`
}

// OfflineMessage is a message to store on the relay.
type OfflineMessage struct {
	ToDID     string `json:"to_did"`
	FromDID   string `json:"from_did"`
	MessageID string `json:"message_id"`
	Envelope  string `json:"envelope"`
	TTL       uint32 `json:"ttl"`
}

// SyncMessage is a message returned from the sync endpoint.
type SyncMessage struct {
	ID        string `json:"id"`
	FromDID   string `json:"from_did"`
	ToDID     string `json:"to_did"`
	Envelope  string `json:"envelope"`
	Timestamp int64  `json:"timestamp"`
	TTL       uint32 `json:"ttl"`
}

// SyncResponse is the response from the sync endpoint.
type SyncResponse struct {
	Messages  []SyncMessage `json:"messages"`
	HasMore   bool          `json:"has_more"`
	SyncToken string        `json:"sync_token,omitempty"`
}

// NodeInfo contains information about the relay node.
type NodeInfo struct {
	PeerID    string   `json:"peer_id"`
	Addresses []string `json:"addresses"`
	Protocols []string `json:"protocols"`
}

// NewRelayClient creates a new relay client.
func NewRelayClient(baseURL string) *RelayClient {
	return &RelayClient{
		baseURL: baseURL,
		httpClient: &http.Client{
			Timeout: 30 * time.Second,
		},
	}
}

// HealthCheck checks if the relay node is healthy.
func (c *RelayClient) HealthCheck(ctx context.Context) (map[string]interface{}, error) {
	return c.doGet(ctx, "/health")
}

// GetNodeInfo returns relay node information.
func (c *RelayClient) GetNodeInfo(ctx context.Context) (*NodeInfo, error) {
	var info NodeInfo
	err := c.doGetInto(ctx, "/node/info", &info)
	if err != nil {
		return nil, err
	}
	return &info, nil
}

// UploadPrekey uploads a prekey bundle to the relay.
func (c *RelayClient) UploadPrekey(ctx context.Context, upload *PrekeyUpload) error {
	_, err := c.doPost(ctx, "/prekey", upload)
	return err
}

// GetPrekey fetches a prekey bundle for a DID.
func (c *RelayClient) GetPrekey(ctx context.Context, did string) (*PrekeyBundle, error) {
	var bundle PrekeyBundle
	err := c.doGetInto(ctx, "/prekey/"+did, &bundle)
	if err != nil {
		return nil, err
	}
	return &bundle, nil
}

// StoreMessage stores an offline message on the relay.
func (c *RelayClient) StoreMessage(ctx context.Context, msg *OfflineMessage) error {
	_, err := c.doPost(ctx, "/message", msg)
	return err
}

// SyncMessages fetches offline messages for a DID.
func (c *RelayClient) SyncMessages(ctx context.Context, did string, since int64, limit int) (*SyncResponse, error) {
	path := fmt.Sprintf("/sync/%s?since=%d&limit=%d", did, since, limit)
	var resp SyncResponse
	err := c.doGetInto(ctx, path, &resp)
	if err != nil {
		return nil, err
	}
	return &resp, nil
}

// doGet performs a GET request and returns the raw response.
func (c *RelayClient) doGet(ctx context.Context, path string) (map[string]interface{}, error) {
	req, err := http.NewRequestWithContext(ctx, "GET", c.baseURL+path, nil)
	if err != nil {
		return nil, fmt.Errorf("create request: %w", err)
	}
	req.Header.Set("Accept", "application/json")

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return nil, fmt.Errorf("http get: %w", err)
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("read body: %w", err)
	}

	if resp.StatusCode >= 400 {
		return nil, fmt.Errorf("HTTP %d: %s", resp.StatusCode, string(body))
	}

	var result map[string]interface{}
	if err := json.Unmarshal(body, &result); err != nil {
		return nil, fmt.Errorf("parse response: %w", err)
	}
	return result, nil
}

// doGetInto performs a GET request and unmarshals into v.
func (c *RelayClient) doGetInto(ctx context.Context, path string, v interface{}) error {
	req, err := http.NewRequestWithContext(ctx, "GET", c.baseURL+path, nil)
	if err != nil {
		return fmt.Errorf("create request: %w", err)
	}
	req.Header.Set("Accept", "application/json")

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return fmt.Errorf("http get: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode >= 400 {
		body, _ := io.ReadAll(resp.Body)
		return fmt.Errorf("HTTP %d: %s", resp.StatusCode, string(body))
	}

	return json.NewDecoder(resp.Body).Decode(v)
}

// doPost performs a POST request with a JSON body.
func (c *RelayClient) doPost(ctx context.Context, path string, body interface{}) (map[string]interface{}, error) {
	data, err := json.Marshal(body)
	if err != nil {
		return nil, fmt.Errorf("marshal body: %w", err)
	}

	req, err := http.NewRequestWithContext(ctx, "POST", c.baseURL+path, bytes.NewReader(data))
	if err != nil {
		return nil, fmt.Errorf("create request: %w", err)
	}
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Accept", "application/json")

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return nil, fmt.Errorf("http post: %w", err)
	}
	defer resp.Body.Close()

	respBody, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("read body: %w", err)
	}

	if resp.StatusCode >= 400 {
		return nil, fmt.Errorf("HTTP %d: %s", resp.StatusCode, string(respBody))
	}

	var result map[string]interface{}
	if err := json.Unmarshal(respBody, &result); err != nil {
		return nil, fmt.Errorf("parse response: %w", err)
	}
	return result, nil
}
