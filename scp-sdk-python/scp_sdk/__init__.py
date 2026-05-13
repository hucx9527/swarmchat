"""
SCP Python SDK — Swarm Communication Protocol client for Python.

Provides a complete toolkit for building SCP agents, bots, and
integrations. Includes a relay HTTP client, webhook server,
message decorators, and agent base classes.

Quick Start
-----------
```python
from scp_sdk import SCPClient, SCPAgent, on_message, MessageType

# Create a client
client = SCPClient(relay_url="http://localhost:8080")

# Create an agent
class MyBot(SCPAgent):
    @on_message(MessageType.TEXT)
    def handle_text(self, message):
        self.reply(message, f"Echo: {message.content['body']}")

agent = MyBot(client=client, did="did:key:z6Mk...")
agent.run_webhook(port=5000)
```
"""

from .client import SCPClient
from .types import (
    MessageType,
    MessagePayload,
    TextPayload,
    AgentCardPayload,
    TaskPayload,
    AgentResultPayload,
    Envelope,
)
from .decorators import on_message, on_task, on_approve
from .agent import SCPAgent
from .webhook import WebhookServer

__version__ = "0.1.0"
__all__ = [
    "SCPClient",
    "SCPAgent",
    "WebhookServer",
    "MessageType",
    "MessagePayload",
    "TextPayload",
    "AgentCardPayload",
    "TaskPayload",
    "AgentResultPayload",
    "Envelope",
    "on_message",
    "on_task",
    "on_approve",
]
