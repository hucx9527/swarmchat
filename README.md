# SwarmChat

A swarm intelligence based decentralized communication platform built on the
**Swarm Communication Protocol (SCP)**.

## Project Status

| Phase | Status | Description |
|-------|--------|-------------|
| Phase 0 | ✅ Complete | Core crypto (X3DH, Double Ratchet, Sender Key) |
| Phase 1 | ✅ Complete | Identity & DID system + Transport Layer |
| Phase 2 | ✅ Complete | Relay Node (Go), CLI Tool (Go), Python SDK, Dev Portal |
| Phase 3 | ✅ Complete | React Native Mobile App (TypeScript), JSI/FFI Rust bridge |

## Components

| Component | Language | Description |
|-----------|----------|-------------|
| [scp-core](scp-core/) | Rust | Core protocol: crypto, identity, transport, messaging |
| [scp-relay](scp-relay/) | Go | Relay node: NAT traversal, prekey storage, offline messages |
| [scp-cli](scp-cli/) | Go | CLI tool: identity, messaging, groups, agents |
| [scp-sdk-python](scp-sdk-python/) | Python | Agent SDK: webhooks, decorators, relay client |
| [docs-site](docs-site/) | HTML | Developer portal with API reference & quickstart |
| [swarmchat](swarmchat/) | TypeScript | React Native mobile app (iOS/Android) with JSI/FFI Rust bridge |
| [scp-spec](scp-spec/) | Markdown | Protocol specification v0.1.0 |

## Quick Start

### 1. Start a Relay Node
```bash
cd scp-relay && go build -o scp-relay . && ./scp-relay --http :8080
```

### 2. Create an Identity
```bash
cd scp-cli && go build -o scp-cli .
./scp-cli identity create --label personal --nickname "Alice"
```

### 3. Build an Agent (Python)
```python
from scp_sdk import SCPAgent, on_message, MessageType

class EchoBot(SCPAgent):
    @on_message(MessageType.TEXT)
    def echo(self, envelope, payload):
        self.reply(envelope, f"Echo: {payload['body']}")

bot = EchoBot(client=client, did="did:key:z6Mk...", name="EchoBot")
bot.run_webhook(port=5000)
```

### 4. Run the Mobile App
```bash
cd swarmchat && npm install && npx react-native start
```

## Mobile App Features (Phase 3)

| Feature | Status | Description |
|---------|--------|-------------|
| Identity Create/Recover | ✅ | BIP39 mnemonic + did:key + PeerId |
| Contact Management | ✅ | Add by DID or QR code scan |
| 1-on-1 Chat | ✅ | Double Ratchet per session |
| Group Chat | ✅ | Sender Key encryption, member list |
| File Transfer | ✅ | Image, video, audio, documents |
| QR Code | ✅ | Share DID, scan to add friends |
| Settings | ✅ | Relay URL, discovery, theme, biometric lock |
| JSI/FFI Bridge | ✅ | Rust scp-core via JNI (Android) + C FFI (iOS) |

## Architecture

```
┌──────────────────────────────────────────────────────┐
│              SwarmChat Mobile App                     │
│           (React Native · iOS/Android)                │
│  ┌─────────────────────────────────────────────────┐ │
│  │ JSI/FFI Bridge ── scp-core-bridge (Rust)        │ │
│  └─────────────────────────────────────────────────┘ │
├──────────────────────────────────────────────────────┤
│  scp-cli (Go)    │  scp-sdk-python                   │
│  CLI tool        │  Agent framework                  │
├──────────────────────────────────────────────────────┤
│              scp-core (Rust)                          │
│  X3DH | Double Ratchet | Sender Key                  │
│  DID | PeerId | Identity Manager                     │
│  libp2p | QUIC | DHT | GossipSub                     │
├──────────────────────────────────────────────────────┤
│              scp-relay (Go)                           │
│  Circuit Relay v2 | Prekey | Offline Msg             │
├──────────────────────────────────────────────────────┤
│          P2P Network (libp2p)                         │
│  QUIC · TCP · Noise · Yamux · Kademlia               │
└──────────────────────────────────────────────────────┘
```

## Documentation

- [Protocol Specification](scp-spec/Swarm%20Communication%20Protocol%20(SCP)%20Specification%20v0.1.0.md)
- [Developer Portal](docs-site/index.html)
- [Phase 1 Progress](PHASE_1_PROGRESS.md)
- [Phase 2 Progress](PHASE_2_PROGRESS.md)
- [Phase 3 Progress](PHASE_3_PROGRESS.md)
- [Project Plan](SwarmChat-Project-Plan-For-AI-Agent.md.md)
- [Windows Agent Handoff](HANDOFF_WINDOWS_AGENT.md)

## License

Apache 2.0
