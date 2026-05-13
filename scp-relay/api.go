package main

import (
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"time"

	"github.com/gorilla/mux"
	"github.com/libp2p/go-libp2p/core/host"

	"github.com/swarmchat/scp-relay/internal/prekey"
	"github.com/swarmchat/scp-relay/internal/storage"
)

// API handles HTTP API requests for the relay node
type API struct {
	prekeyStore  *prekey.Store
	offlineStore *storage.OfflineStore
	host         host.Host
	server       *http.Server
}

// NewAPI creates a new API handler
func NewAPI(ps *prekey.Store, os *storage.OfflineStore, h host.Host) *API {
	return &API{
		prekeyStore:  ps,
		offlineStore: os,
		host:         h,
	}
}

// RegisterRoutes registers all HTTP API routes
func (api *API) RegisterRoutes(r *mux.Router) {
	r.HandleFunc("/health", api.handleHealth).Methods("GET")
	r.HandleFunc("/prekey", api.handleUploadPrekey).Methods("POST")
	r.HandleFunc("/prekey/{did}", api.handleGetPrekey).Methods("GET")
	r.HandleFunc("/message", api.handleStoreMessage).Methods("POST")
	r.HandleFunc("/sync/{did}", api.handleSyncMessages).Methods("GET")
	r.HandleFunc("/node/info", api.handleNodeInfo).Methods("GET")
}

// Listen starts the HTTP server
func (api *API) Listen(addr string) error {
	api.server = &http.Server{
		Addr:         addr,
		Handler:      api.withLogging(mux.NewRouter()),
		ReadTimeout:  15 * time.Second,
		WriteTimeout: 15 * time.Second,
		IdleTimeout:  60 * time.Second,
	}
	return api.server.ListenAndServe()
}

func (api *API) withLogging(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		start := time.Now()
		next.ServeHTTP(w, r)
		log.Printf("%s %s %s %v", r.Method, r.RequestURI, r.RemoteAddr, time.Since(start))
	})
}

// ---- Health Check ----

func (api *API) handleHealth(w http.ResponseWriter, r *http.Request) {
	respondJSON(w, http.StatusOK, map[string]interface{}{
		"status":    "ok",
		"timestamp": time.Now().UnixMilli(),
		"peer_id":   api.host.ID().String(),
		"version":   "0.1.0",
	})
}

// ---- Node Info ----

func (api *API) handleNodeInfo(w http.ResponseWriter, r *http.Request) {
	var addrs []string
	for _, a := range api.host.Addrs() {
		addrs = append(addrs, a.String())
	}

	respondJSON(w, http.StatusOK, map[string]interface{}{
		"peer_id":   api.host.ID().String(),
		"addresses": addrs,
		"protocols": []string{"circuit-relay-v2", "dht", "gossipsub"},
	})
}

// ---- Prekey Bundle API (SS4.6) ----

// PrekeyUploadRequest is the request to upload a prekey bundle
type PrekeyUploadRequest struct {
	// DID of the prekey owner
	DID string `json:"did"`

	// Base64-encoded identity key (IK) public key
	IdentityKey string `json:"identity_key"`

	// Signed prekey
	SignedPrekey PrekeyEntry `json:"signed_prekey"`

	// One-time prekeys
	OneTimePrekeys []string `json:"one_time_prekeys"`
}

// PrekeyEntry represents a prekey with its signature
type PrekeyEntry struct {
	// Base64-encoded public key
	Key string `json:"key"`

	// Signature over the key (Base64)
	Signature string `json:"signature"`
}

// PrekeyBundleResponse is the response for querying a prekey bundle
type PrekeyBundleResponse struct {
	DID             string      `json:"did"`
	IdentityKey     string      `json:"identity_key"`
	SignedPrekey    PrekeyEntry `json:"signed_prekey"`
	OneTimePrekey   string      `json:"one_time_prekey,omitempty"`
	PrekeysRemaining int        `json:"prekeys_remaining"`
}

func (api *API) handleUploadPrekey(w http.ResponseWriter, r *http.Request) {
	var req PrekeyUploadRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		respondError(w, http.StatusBadRequest, "Invalid request body: "+err.Error())
		return
	}

	if req.DID == "" {
		respondError(w, http.StatusBadRequest, "DID is required")
		return
	}

	if req.IdentityKey == "" {
		respondError(w, http.StatusBadRequest, "Identity key is required")
		return
	}

	// Validate and store (in production, verify SPK signature here)
	bundle := &prekey.Bundle{
		DID:             req.DID,
		IdentityKey:     req.IdentityKey,
		SignedPrekey:    req.SignedPrekey.Key,
		SPKSignature:    req.SignedPrekey.Signature,
		OneTimePrekeys:  req.OneTimePrekeys,
		UploadedAt:      time.Now(),
	}

	if err := api.prekeyStore.Upload(r.Context(), bundle); err != nil {
		respondError(w, http.StatusInternalServerError, "Failed to store prekey bundle: "+err.Error())
		return
	}

	respondJSON(w, http.StatusOK, map[string]interface{}{
		"status": "ok",
		"prekeys_stored": len(req.OneTimePrekeys),
		"message": "Prekey bundle stored successfully",
	})
}

func (api *API) handleGetPrekey(w http.ResponseWriter, r *http.Request) {
	vars := mux.Vars(r)
	did := vars["did"]

	bundle, err := api.prekeyStore.Get(r.Context(), did)
	if err != nil {
		respondError(w, http.StatusNotFound, "Prekey bundle not found for DID: "+did)
		return
	}

	// Return one OPK and remove it (one-time use)
	var opk string
	if len(bundle.OneTimePrekeys) > 0 {
		opk = bundle.OneTimePrekeys[0]
		// Mark as used
		_ = api.prekeyStore.UseOneTimePrekey(r.Context(), did, opk)
	}

	resp := PrekeyBundleResponse{
		DID:              bundle.DID,
		IdentityKey:      bundle.IdentityKey,
		SignedPrekey: PrekeyEntry{
			Key:       bundle.SignedPrekey,
			Signature: bundle.SPKSignature,
		},
		OneTimePrekey:    opk,
		PrekeysRemaining: len(bundle.OneTimePrekeys) - 1,
	}

	respondJSON(w, http.StatusOK, resp)
}

// ---- Offline Message API (SS5.5) ----

// OfflineMessageRequest is the request to store an offline message
type OfflineMessageRequest struct {
	// Target DID the message is for
	ToDID string `json:"to_did"`

	// Encrypted envelope (Base64)
	Envelope string `json:"envelope"`

	// Message ID
	MessageID string `json:"message_id"`

	// TTL in seconds
	TTL uint32 `json:"ttl"`

	// Sender DID
	FromDID string `json:"from_did"`
}

// SyncResponse is the response from the sync endpoint
type SyncResponse struct {
	Messages    []OfflineMessageResponse `json:"messages"`
	HasMore     bool                     `json:"has_more"`
	SyncToken   string                   `json:"sync_token,omitempty"`
}

// OfflineMessageResponse is a single offline message in the sync response
type OfflineMessageResponse struct {
	ID        string `json:"id"`
	FromDID   string `json:"from_did"`
	ToDID     string `json:"to_did"`
	Envelope  string `json:"envelope"`
	Timestamp int64  `json:"timestamp"`
	TTL       uint32 `json:"ttl"`
}

func (api *API) handleStoreMessage(w http.ResponseWriter, r *http.Request) {
	var req OfflineMessageRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		respondError(w, http.StatusBadRequest, "Invalid request body: "+err.Error())
		return
	}

	if req.ToDID == "" || req.Envelope == "" {
		respondError(w, http.StatusBadRequest, "to_did and envelope are required")
		return
	}

	msg := &storage.OfflineMessage{
		ID:        req.MessageID,
		FromDID:   req.FromDID,
		ToDID:     req.ToDID,
		Envelope:  req.Envelope,
		Timestamp: time.Now().Unix(),
		TTL:       req.TTL,
		Delivered: false,
	}

	if err := api.offlineStore.Store(r.Context(), msg); err != nil {
		respondError(w, http.StatusInternalServerError, "Failed to store message: "+err.Error())
		return
	}

	respondJSON(w, http.StatusOK, map[string]interface{}{
		"status":     "stored",
		"message_id": msg.ID,
	})
}

func (api *API) handleSyncMessages(w http.ResponseWriter, r *http.Request) {
	vars := mux.Vars(r)
	did := vars["did"]

	sinceStr := r.URL.Query().Get("since")
	var since int64
	if sinceStr != "" {
		fmt.Sscanf(sinceStr, "%d", &since)
	}

	limit := 100
	if limitStr := r.URL.Query().Get("limit"); limitStr != "" {
		fmt.Sscanf(limitStr, "%d", &limit)
		if limit > 500 {
			limit = 500
		}
	}

	messages, err := api.offlineStore.Sync(r.Context(), did, since, limit)
	if err != nil {
		respondError(w, http.StatusInternalServerError, "Failed to sync messages: "+err.Error())
		return
	}

	var respMessages []OfflineMessageResponse
	for _, msg := range messages {
		respMessages = append(respMessages, OfflineMessageResponse{
			ID:        msg.ID,
			FromDID:   msg.FromDID,
			ToDID:     msg.ToDID,
			Envelope:  msg.Envelope,
			Timestamp: msg.Timestamp,
			TTL:       msg.TTL,
		})
	}

	// Mark messages as delivered
	for _, msg := range messages {
		_ = api.offlineStore.MarkDelivered(r.Context(), msg.ID)
	}

	resp := SyncResponse{
		Messages:  respMessages,
		HasMore:   len(messages) == limit,
	}

	respondJSON(w, http.StatusOK, resp)
}

// ---- Helpers ----

func respondJSON(w http.ResponseWriter, status int, data interface{}) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	json.NewEncoder(w).Encode(data)
}

func respondError(w http.ResponseWriter, status int, message string) {
	respondJSON(w, status, map[string]string{
		"error": message,
	})
}
