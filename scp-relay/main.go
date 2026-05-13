// scp-relay: SwarmChat Relay Node
//
// Implements SCP Relay Node functionality per SCP Specification SS3.3:
// - libp2p Circuit Relay v2 for NAT traversal
// - Prekey Bundle storage and distribution (SS4.6)
// - Offline message storage with sync endpoint (SS5.5)
// - HTTP API for client interaction
//
// Usage:
//   scp-relay --listen /ip4/0.0.0.0/udp/9090/quic-v1 --http :8080 --db ./relay.db

package main

import (
	"context"
	"crypto/ed25519"
	"crypto/rand"
	"flag"
	"fmt"
	"log"
	"os"
	"os/signal"
	"strings"
	"sync"
	"syscall"
	"time"

	"github.com/gorilla/mux"
	"github.com/libp2p/go-libp2p"
	dht "github.com/libp2p/go-libp2p-kad-dht"
	"github.com/libp2p/go-libp2p/core/crypto"
	"github.com/libp2p/go-libp2p/core/host"
	"github.com/libp2p/go-libp2p/core/peer"
	"github.com/libp2p/go-libp2p/p2p/protocol/circuitv2/relay"
	"github.com/multiformats/go-multiaddr"

	"github.com/swarmchat/scp-relay/internal/prekey"
	"github.com/swarmchat/scp-relay/internal/storage"
)

var (
	listenAddr  = flag.String("listen", "/ip4/0.0.0.0/udp/9090/quic-v1", "libp2p listen address")
	httpAddr    = flag.String("http", ":8080", "HTTP API listen address")
	dbPath      = flag.String("db", "./relay.db", "SQLite database path")
	bootstrap   = flag.String("bootstrap", "", "Comma-separated bootstrap multiaddrs")
	maxMsgAge   = flag.Duration("max-msg-age", 30*24*time.Hour, "Maximum offline message age (TTL cap)")
	enableDebug = flag.Bool("debug", false, "Enable debug logging")
)

func main() {
	flag.Parse()

	log.SetFlags(log.Ldate | log.Ltime | log.Lshortfile)
	log.Printf("=== SwarmChat Relay Node ===")
	log.Printf("Starting SCP Relay v0.1.0")
	log.Printf("libp2p listen: %s", *listenAddr)
	log.Printf("HTTP API: %s", *httpAddr)
	log.Printf("Database: %s", *dbPath)

	// ---- Initialize Database ----
	db, err := storage.NewSQLiteStore(*dbPath)
	if err != nil {
		log.Fatalf("Failed to open database: %v", err)
	}
	defer db.Close()

	if err := db.Migrate(); err != nil {
		log.Fatalf("Failed to run migrations: %v", err)
	}
	log.Println("Database initialized")

	// ---- Create libp2p Host ----
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	host, err := createLibp2pHost(ctx)
	if err != nil {
		log.Fatalf("Failed to create libp2p host: %v", err)
	}
	defer host.Close()

	log.Printf("libp2p host created: PeerId=%s", host.ID().String())

	// ---- Start Circuit Relay v2 ----
	_, err = relay.New(host)
	if err != nil {
		log.Fatalf("Failed to create relay: %v", err)
	}
	log.Println("Circuit Relay v2 enabled")

	// ---- Connect to Bootstrap Nodes ----
	if *bootstrap != "" {
		connectToBootstrap(ctx, host, *bootstrap)
	}

	// ---- Start Kademlia DHT ----
	kadDHT, err := dht.New(ctx, host, dht.Mode(dht.ModeServer))
	if err != nil {
		log.Fatalf("Failed to create DHT: %v", err)
	}
	if err := kadDHT.Bootstrap(ctx); err != nil {
		log.Printf("Warning: DHT bootstrap failed: %v", err)
	}
	log.Println("Kademlia DHT started (server mode)")

	// ---- Start HTTP API Server ----
	prekeyStore := prekey.NewStore(db.DB())
	offlineStore := storage.NewOfflineStore(db, *maxMsgAge)

	api := NewAPI(prekeyStore, offlineStore, host)
	router := mux.NewRouter()
	api.RegisterRoutes(router)

	go func() {
		log.Printf("HTTP API listening on %s", *httpAddr)
		if err := api.Listen(*httpAddr); err != nil {
			log.Fatalf("HTTP server error: %v", err)
		}
	}()

	// ---- Print Node Info ----
	log.Println("")
	log.Println("=== Relay Node Ready ===")
	log.Printf("PeerId: %s", host.ID().String())
	for _, addr := range host.Addrs() {
		fullAddr, _ := multiaddr.NewMultiaddr(fmt.Sprintf("%s/p2p/%s", addr.String(), host.ID().String()))
		log.Printf("Address: %s", fullAddr.String())
	}
	log.Printf("HTTP API: http://localhost%s", *httpAddr)
	log.Printf("Endpoints:")
	log.Printf("  POST /prekey          - Upload prekey bundle")
	log.Printf("  GET  /prekey/{did}    - Get prekey bundle for a DID")
	log.Printf("  POST /message         - Store offline message")
	log.Printf("  GET  /sync/{did}      - Sync offline messages")
	log.Printf("  GET  /health          - Health check")
	log.Println("Press Ctrl+C to stop")

	// ---- Wait for Shutdown ----
	sigCh := make(chan os.Signal, 1)
	signal.Notify(sigCh, syscall.SIGINT, syscall.SIGTERM)
	<-sigCh

	log.Println("Shutting down...")
	cancel()

	// Graceful shutdown
	shutdownCtx, shutdownCancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer shutdownCancel()

	if err := kadDHT.Close(); err != nil {
		log.Printf("Error closing DHT: %v", err)
	}

	<-shutdownCtx.Done()
	log.Println("Shutdown complete")
}

// createLibp2pHost creates a libp2p host with relay capabilities
func createLibp2pHost(ctx context.Context) (host.Host, error) {
	// Generate Ed25519 keypair
	pub, priv, err := ed25519.GenerateKey(rand.Reader)
	if err != nil {
		return nil, fmt.Errorf("generate key: %w", err)
	}

	libp2pPriv, err := crypto.UnmarshalEd25519PrivateKey(priv)
	if err != nil {
		return nil, fmt.Errorf("unmarshal private key: %w", err)
	}

	// Validate public key
	_ = pub // used for PeerId generation via libp2p

	// Parse listen address
	addr, err := multiaddr.NewMultiaddr(*listenAddr)
	if err != nil {
		return nil, fmt.Errorf("parse listen addr: %w", err)
	}

	// Create libp2p host
	h, err := libp2p.New(
		libp2p.Identity(libp2pPriv),
		libp2p.ListenAddrs(addr),
		libp2p.EnableRelay(),
		libp2p.EnableHolePunching(),
		libp2p.ForceReachabilityPrivate(),
		libp2p.NATPortMap(),
		libp2p.Ping(true),
		libp2p.EnableNATService(),
	)
	if err != nil {
		return nil, fmt.Errorf("create host: %w", err)
	}

	return h, nil
}

// connectToBootstrap dials bootstrap nodes
func connectToBootstrap(ctx context.Context, h host.Host, bootstrapStr string) {
	var wg sync.WaitGroup
	for _, addrStr := range splitAndTrim(bootstrapStr, ",") {
		addr, err := multiaddr.NewMultiaddr(addrStr)
		if err != nil {
			log.Printf("Invalid bootstrap addr %s: %v", addrStr, err)
			continue
		}

		peerInfo, err := peer.AddrInfoFromP2pAddr(addr)
		if err != nil {
			log.Printf("Failed to parse peer info from %s: %v", addrStr, err)
			continue
		}

		wg.Add(1)
		go func() {
			defer wg.Done()
			timeoutCtx, cancel := context.WithTimeout(ctx, 30*time.Second)
			defer cancel()

			if err := h.Connect(timeoutCtx, *peerInfo); err != nil {
				log.Printf("Failed to connect to bootstrap %s: %v", addrStr, err)
			} else {
				log.Printf("Connected to bootstrap: %s", peerInfo.ID)
			}
		}()
	}
	wg.Wait()
}

func splitAndTrim(s, sep string) []string {
	if s == "" {
		return nil
	}
	var result []string
	for _, part := range strings.Split(s, sep) {
		if trimmed := strings.TrimSpace(part); trimmed != "" {
			result = append(result, trimmed)
		}
	}
	return result
}
