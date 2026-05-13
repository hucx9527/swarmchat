"""Base agent class for building SCP AI agents."""

import logging
import uuid
from typing import Optional, Dict, Any, List, Callable

from .client import SCPClient
from .decorators import get_handlers
from .types import MessageType, Envelope, AgentCardPayload

logger = logging.getLogger("scp_sdk.agent")


class SCPAgent:
    """Base class for SCP AI agents.

    Subclass this to create your own agent. Decorate handler methods
    with @on_message, @on_task, or @on_approve to respond to messages.

    Attributes:
        client: SCPClient for relay communication.
        did: The agent's DID.
        name: Human-readable agent name.
        capabilities: List of the agent's capabilities.
        models: AI models used by the agent.
        callback_url: Webhook URL for receiving messages.

    Example:
        ```python
        class EchoBot(SCPAgent):
            @on_message(MessageType.TEXT)
            def echo(self, envelope, payload):
                text = payload.get("body", "")
                self.reply(envelope, f"Echo: {text}")

        bot = EchoBot(
            client=SCPClient("http://localhost:8080"),
            did="did:key:z6Mk...",
            name="EchoBot",
            capabilities=["echo"],
        )
        bot.run_webhook(port=5000)
        ```
    """

    def __init__(
        self,
        client: SCPClient,
        did: str,
        name: str = "Unnamed Agent",
        capabilities: Optional[List[str]] = None,
        models: Optional[List[str]] = None,
        callback_url: Optional[str] = None,
        rate_limit: int = 10,
    ):
        self.client = client
        self.did = did
        self.name = name
        self.capabilities = capabilities or []
        self.models = models or []
        self.callback_url = callback_url
        self.rate_limit = rate_limit

        # Discover decorated handlers
        self._handlers = get_handlers(self)
        logger.info(
            f"Agent '{name}' initialized with {len(self._handlers)} handler(s): "
            f"{[h['name'] for h in self._handlers]}"
        )

    # ---- Agent Card ----

    def build_card(self) -> AgentCardPayload:
        """Build this agent's capability card."""
        return AgentCardPayload(
            agent_id=self.did,
            capabilities=self.capabilities,
            owner=None,
            callback_url=self.callback_url,
            rate_limit=self.rate_limit,
            models=self.models,
        )

    def publish_card(self) -> Dict[str, Any]:
        """Publish this agent's capability card to the network."""
        card = self.build_card()
        # In production, publish to DHT or gossip
        logger.info(f"Publishing agent card: {card.model_dump_json()}")
        return card.model_dump()

    # ---- Message Handling ----

    def handle_message(
        self,
        envelope: Dict[str, Any],
        payload: Dict[str, Any],
    ) -> Optional[Dict[str, Any]]:
        """Route an incoming message to the appropriate handler.

        Args:
            envelope: The message envelope.
            payload: The decrypted message payload.

        Returns:
            Optional response payload, or None if no handler matched.
        """
        msg_type_str = envelope.get("type", "")
        try:
            msg_type = MessageType(msg_type_str)
        except ValueError:
            logger.warning(f"Unknown message type: {msg_type_str}")
            return None

        logger.debug(f"Received {msg_type.value} from {envelope.get('from', 'unknown')}")

        for handler in self._handlers:
            if handler['msg_type'] == msg_type:
                try:
                    result = handler['method'](envelope, payload)
                    if result is not None:
                        return result
                except Exception as e:
                    logger.error(f"Handler '{handler['name']}' failed: {e}", exc_info=True)
        return None

    def reply(self, envelope: Dict[str, Any], text: str):
        """Send a reply to the sender of a message.

        Args:
            envelope: The original message envelope.
            text: Reply text.
        """
        from_did = envelope.get("to", [None])[0] if envelope.get("to") else self.did
        to_did = envelope.get("from", "")
        if not to_did:
            logger.warning("Cannot reply: no sender in envelope")
            return

        session_id = str(uuid.uuid4())
        self.client.send_text(
            from_did=from_did or self.did,
            to_did=to_did,
            text=text,
            session_id=session_id,
        )
        logger.info(f"Replied to {to_did[:20]}...: {text[:50]}...")

    # ---- Task Helpers ----

    def claim_task(self, task_id: str):
        """Claim a task."""
        logger.info(f"Claiming task: {task_id}")
        # In production, send scp.agent.v1.claim message

    def submit_result(
        self,
        task_id: str,
        status: str,
        output: str,
        evidence: Optional[List[str]] = None,
    ):
        """Submit a task result."""
        logger.info(f"Submitting result for {task_id}: {status}")
        # In production, send scp.agent.v1.result message

    # ---- Webhook Server ----

    def run_webhook(self, host: str = "0.0.0.0", port: int = 5000, debug: bool = False):
        """Start a Flask webhook server for receiving messages.

        The server exposes:
        - POST /webhook — receive SCP messages
        - GET /health — health check
        - GET /card — agent capability card

        Args:
            host: Bind address.
            port: Bind port.
            debug: Enable Flask debug mode.
        """
        from .webhook import WebhookServer
        server = WebhookServer(agent=self, host=host, port=port)
        server.run(debug=debug)

    # ---- Built-in Handlers ----

    def _handle_agent_query(self, envelope: Dict, payload: Dict) -> Dict:
        """Default handler for agent queries — returns agent card."""
        logger.info(f"Received agent query, responding with card")
        return self.build_card().model_dump()
