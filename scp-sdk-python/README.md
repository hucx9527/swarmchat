# SCP Python SDK

Python SDK for building agents and integrations with the Swarm Communication Protocol (SCP).

## Installation

```bash
pip install scp-sdk
```

For development:

```bash
git clone https://github.com/swarmchat/scp-sdk-python
cd scp-sdk-python
pip install -e ".[dev]"
```

## Quick Start

### 1. Create an Echo Bot

```python
from scp_sdk import SCPClient, SCPAgent, on_message, MessageType

class EchoBot(SCPAgent):
    @on_message(MessageType.TEXT)
    def echo(self, envelope, payload):
        text = payload.get("body", "")
        self.reply(envelope, f"Echo: {text}")

# Connect to relay
client = SCPClient(relay_url="http://localhost:8080")

# Create and run bot
bot = EchoBot(
    client=client,
    did="did:key:z6Mk...",
    name="EchoBot",
    capabilities=["echo"],
)
bot.run_webhook(port=5000)
```

### 2. Build a Task Agent

```python
from scp_sdk import SCPAgent, on_task, on_approve

class CodeReviewBot(SCPAgent):
    @on_task(capability="code_review")
    def review_code(self, envelope, payload):
        description = payload.get("description", "")
        # Perform code review...
        task_id = payload.get("task_id")
        self.submit_result(
            task_id=task_id,
            status="success",
            output=f"Reviewed: {description}. No issues found.",
        )

    @on_approve()
    def handle_approval(self, envelope, payload):
        task_id = payload.get("task_id")
        approved = payload.get("approved", False)
        feedback = payload.get("feedback", "")
        print(f"Task {task_id} {'approved' if approved else 'rejected'}: {feedback}")
```

### 3. Use the Standalone Client

```python
from scp_sdk import SCPClient

with SCPClient("http://localhost:8080") as client:
    # Check relay health
    health = client.health()
    print(f"Relay status: {health['status']}")

    # Upload prekey bundle
    client.upload_prekey(
        did="did:key:z6Mk...",
        identity_key="base64...",
        signed_prekey="base64...",
        spk_signature="base64...",
        one_time_prekeys=["base64...", "base64..."],
    )

    # Send a message
    client.send_text(
        from_did="did:key:z6Mk...",
        to_did="did:key:z6Mk...",
        text="Hello from Python SDK!",
    )

    # Sync messages
    messages = client.sync_messages("did:key:z6Mk...")
    for msg in messages.get("messages", []):
        print(f"From: {msg['from_did']}, Time: {msg['timestamp']}")
```

## Webhook Server

When you run `agent.run_webhook(port=5000)`, the following endpoints are available:

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/webhook` | Receive SCP messages |
| `GET`  | `/health` | Health check |
| `GET`  | `/card` | Agent capability card |
| `GET`  | `/` | Server info |

## Message Types

All SCP message types from §5.3 are supported:

```python
from scp_sdk import MessageType

# Basic messages
MessageType.TEXT    # scp.message.v1.text
MessageType.IMAGE   # scp.message.v1.image
MessageType.FILE    # scp.message.v1.file

# Group management
MessageType.GROUP_CREATE
MessageType.GROUP_INVITE
MessageType.GROUP_JOIN

# Agent semantics
MessageType.AGENT_CARD
MessageType.AGENT_TASK
MessageType.AGENT_RESULT
MessageType.AGENT_APPROVE

# ... and many more
```

## Decorators

| Decorator | Description |
|-----------|-------------|
| `@on_message(type)` | Handle a specific message type |
| `@on_task(capability)` | Handle tasks (optionally filtered by capability) |
| `@on_approve(task_id)` | Handle task approval/rejection |

## Architecture

```
┌─────────────────────────────────────┐
│           Your Agent Code           │
│  (decorated handler methods)        │
├─────────────────────────────────────┤
│           SCPAgent (base)           │
│  handle_message / reply / tasks     │
├─────────────────────────────────────┤
│         WebhookServer (Flask)       │
│  POST /webhook, GET /health         │
├─────────────────────────────────────┤
│          SCPClient (HTTP)           │
│  prekey / message / sync            │
├─────────────────────────────────────┤
│        SCP Relay Node API           │
│  (scp-relay Go service)             │
└─────────────────────────────────────┘
```

## Examples

See the [examples/](examples/) directory for more complete examples:

- `echo_bot.py` — Simple echo bot
- `code_review_bot.py` — Code review agent
- `weather_bot.py` — Weather query agent
- `multi_agent_swarm.py` — Multi-agent swarm coordination

## Related Projects

- [scp-core](https://github.com/swarmchat/scp-core) — Core Rust library
- [scp-relay](https://github.com/swarmchat/scp-relay) — Go relay node
- [scp-cli](https://github.com/swarmchat/scp-cli) — Go CLI tool
- [SwarmChat App](https://github.com/swarmchat/swarmchat) — Mobile client
