# SCP Relay Node

SwarmChat Relay Node - implements relay services for the Swarm Communication Protocol.

## Features

- **Circuit Relay v2** (SCP §3.3): NAT traversal via libp2p circuit relay
- **Prekey Bundle Storage** (SCP §4.6): HTTP API for uploading and fetching prekey bundles
- **Offline Message Storage** (SCP §5.5): Store encrypted messages for offline recipients
- **Kademlia DHT**: Peer discovery and routing
- **SQLite Persistence**: All data stored locally in SQLite

## Quick Start

### Build

```bash
go build -o scp-relay .
```

### Run

```bash
./scp-relay \
  --listen /ip4/0.0.0.0/udp/9090/quic-v1 \
  --http :8080 \
  --db ./relay.db
```

### With bootstrap nodes

```bash
./scp-relay \
  --listen /ip4/0.0.0.0/udp/9090/quic-v1 \
  --http :8080 \
  --bootstrap "/dnsaddr/bootstrap.scp.io/udp/9090/quic-v1/p2p/12D3KooW..." \
  --max-msg-age 720h
```

## HTTP API

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/health` | Health check |
| `GET` | `/node/info` | Node information |
| `POST` | `/prekey` | Upload prekey bundle |
| `GET` | `/prekey/{did}` | Get prekey bundle for a DID |
| `POST` | `/message` | Store offline message |
| `GET` | `/sync/{did}` | Sync offline messages |
| `GET` | `/sync/{did}?since=1700000000&limit=50` | Sync with pagination |

### Upload Prekey Bundle

```bash
curl -X POST http://localhost:8080/prekey \
  -H "Content-Type: application/json" \
  -d '{
    "did": "did:key:z6Mk...",
    "identity_key": "base64...",
    "signed_prekey": {
      "key": "base64...",
      "signature": "base64..."
    },
    "one_time_prekeys": ["base64...", "base64..."]
  }'
```

### Store Offline Message

```bash
curl -X POST http://localhost:8080/message \
  -H "Content-Type: application/json" \
  -d '{
    "to_did": "did:key:z6Mk...",
    "from_did": "did:key:z6Mk...",
    "message_id": "01HXVBYZ3M7P8Q2R5S9T0U1V2W",
    "envelope": "base64_encoded_envelope...",
    "ttl": 604800
  }'
```

### Sync Offline Messages

```bash
curl "http://localhost:8080/sync/did:key:z6Mk...?since=1700000000&limit=50"
```

## Database Schema

### prekey_bundles
Stores identity keys and signed prekeys per DID.

### one_time_prekeys
Stores one-time prekeys (marked as used after serving).

### offline_messages
Stores encrypted message envelopes for offline delivery.

### message_acks
Tracks message acknowledgement status.

## Flags

| Flag | Default | Description |
|------|---------|-------------|
| `--listen` | `/ip4/0.0.0.0/udp/9090/quic-v1` | libp2p listen address |
| `--http` | `:8080` | HTTP API listen address |
| `--db` | `./relay.db` | SQLite database path |
| `--bootstrap` | `""` | Comma-separated bootstrap multiaddrs |
| `--max-msg-age` | `720h` | Maximum offline message age |
| `--debug` | `false` | Enable debug logging |

## Production Deployment

For production deployment, it is recommended to:

1. Use a reverse proxy (nginx/Caddy) with TLS for the HTTP API
2. Configure firewall rules for libp2p ports (UDP 9090)
3. Set up systemd service for automatic restart
4. Monitor database size and run periodic cleanup
5. Use a dedicated bootstrap node list

## SCP Specification References

- §3.3: Node Types (Relay Node)
- §4.6: Prekey Management
- §5.5: Offline Messages
