# SCP CLI - SwarmChat Command Line Interface

Command-line interface for the Swarm Communication Protocol (SCP).

## Features

- **Identity Management**: Create, import, and manage decentralized identities
- **Encrypted Messaging**: Send and receive messages through relay nodes
- **Group Management**: Create swarm groups, invite members, manage settings
- **Agent Integration**: Publish agent cards, create tasks, approve results
- **Peer Discovery**: Connect to peers, discover local nodes via mDNS

## Quick Start

### Build

```bash
go build -o scp-cli .
```

### Create Your First Identity

```bash
./scp-cli identity create --label personal --nickname "Alice"
```

⚠️ **Save your mnemonic phrase!** Without it, you cannot recover your identity.

### Set Up a Relay

First, start a relay node (see [scp-relay](../scp-relay/README.md)), then:

```bash
./scp-cli --relay http://localhost:8080 message status
```

### Send a Message

```bash
./scp-cli message send --to did:key:z6Mk... --text "Hello Swarm!"
```

### Manage Groups

```bash
# Create a group
./scp-cli group create --name "Swarm Alpha" --description "Our first swarm"

# Invite members
./scp-cli group invite --group group:abc123 --member did:key:z6Mk...

# Join a group
./scp-cli group join --group group:abc123
```

## Commands

### Identity (`identity`, `id`)

| Command | Description |
|---------|-------------|
| `identity create` | Create a new identity |
| `identity import` | Import from BIP39 mnemonic |
| `identity list` | List all identities |
| `identity show` | Show identity details |
| `identity default` | Set the default identity |

### Message (`message`, `msg`)

| Command | Description |
|---------|-------------|
| `message send` | Send a message to a peer or group |
| `message sync` | Sync offline messages from relay |
| `message status` | Check relay connection status |

### Group (`group`, `grp`)

| Command | Description |
|---------|-------------|
| `group create` | Create a new group |
| `group invite` | Invite members to a group |
| `group join` | Join a public group |
| `group leave` | Leave a group |
| `group list` | List your groups |
| `group info` | Show group details |

### Agent (`agent`)

| Command | Description |
|---------|-------------|
| `agent card publish` | Publish an agent capability card |
| `agent card show` | Show an agent card |
| `agent task create` | Create a task for agents |
| `agent task list` | List tasks in a group |
| `agent task approve` | Approve or reject a task result |

### Peer (`peer`, `node`)

| Command | Description |
|---------|-------------|
| `peer info` | Show local node information |
| `peer connect` | Connect to a peer |
| `peer list` | List connected peers |
| `peer discover` | Discover peers on local network |

## Configuration

Configuration is stored in `~/.scp/config.json`:

```json
{
  "relay_url": "http://localhost:8080",
  "default_identity": "personal",
  "identity_store_path": "/home/user/.scp/identities.json",
  "default_ttl": 604800
}
```

## Global Flags

| Flag | Default | Description |
|------|---------|-------------|
| `--config`, `-c` | `~/.scp/config.json` | Configuration file path |
| `--relay`, `-r` | `http://localhost:8080` | Relay node URL |
| `--debug`, `-d` | `false` | Enable debug logging |

## Identity Storage

Identities are stored in `~/.scp/identities.json` with 0600 permissions.
Each identity contains the mnemonic phrase — **keep this file secure!**

## Next Steps

1. Start a relay: [scp-relay](../scp-relay/README.md)
2. Create an agent: See [Python SDK](../scp-sdk-python/README.md)
3. Build the mobile app: [SwarmChat App](../swarmchat/README.md)
