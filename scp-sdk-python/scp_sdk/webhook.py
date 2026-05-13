"""Flask-based webhook server for SCP agents."""

import json
import logging
from typing import TYPE_CHECKING

from flask import Flask, request, jsonify

if TYPE_CHECKING:
    from .agent import SCPAgent

logger = logging.getLogger("scp_sdk.webhook")


class WebhookServer:
    """Flask webhook server for receiving SCP messages.

    Exposes endpoints for receiving messages from SCP relay nodes
    or direct peer connections.

    Endpoints:
        POST /webhook  — Receive an SCP message envelope
        GET  /health   — Health check
        GET  /card     — Agent capability card

    Args:
        agent: The SCPAgent instance to handle messages.
        host: Bind address.
        port: Bind port.
    """

    def __init__(self, agent: "SCPAgent", host: str = "0.0.0.0", port: int = 5000):
        self.agent = agent
        self.host = host
        self.port = port
        self.app = Flask(__name__)
        self._register_routes()

        # Suppress Flask's default logging in production
        log = logging.getLogger("werkzeug")
        log.setLevel(logging.WARNING)

    def _register_routes(self):
        """Register all webhook routes."""

        @self.app.route("/webhook", methods=["POST"])
        def handle_webhook():
            """Receive and process an SCP message envelope.

            Expects JSON body:
            {
                "envelope": { ... },  // SCP envelope
                "payload": "base64..."  // Encrypted payload (base64)
            }
            """
            try:
                data = request.get_json(force=True)
            except Exception as e:
                logger.error(f"Invalid JSON: {e}")
                return jsonify({"error": "Invalid JSON"}), 400

            envelope = data.get("envelope", data)
            payload = data.get("payload", {})

            # Decode payload if it's base64 encoded
            if isinstance(payload, str):
                import base64
                try:
                    decoded = base64.b64decode(payload).decode("utf-8")
                    payload = json.loads(decoded)
                except Exception:
                    payload = {"raw": payload}

            try:
                response = self.agent.handle_message(envelope, payload)
                if response:
                    return jsonify({"status": "ok", "response": response}), 200
                return jsonify({"status": "processed"}), 200
            except Exception as e:
                logger.error(f"Error handling message: {e}", exc_info=True)
                return jsonify({"error": str(e)}), 500

        @self.app.route("/health", methods=["GET"])
        def health():
            """Health check endpoint."""
            return jsonify({
                "status": "ok",
                "agent": self.agent.name,
                "did": self.agent.did,
                "handlers": len(self.agent._handlers),
            })

        @self.app.route("/card", methods=["GET"])
        def card():
            """Return the agent's capability card."""
            return jsonify(self.agent.build_card().model_dump())

        @self.app.route("/", methods=["GET"])
        def index():
            """Root endpoint with basic info."""
            return jsonify({
                "service": "SCP Agent Webhook",
                "agent": self.agent.name,
                "version": "0.1.0",
                "endpoints": {
                    "webhook": "POST /webhook",
                    "health": "GET /health",
                    "card": "GET /card",
                },
            })

    def run(self, debug: bool = False):
        """Start the webhook server.

        Args:
            debug: Enable Flask debug mode (disable in production).
        """
        logger.info(
            f"Starting webhook server for '{self.agent.name}' "
            f"on {self.host}:{self.port}"
        )
        self.app.run(host=self.host, port=self.port, debug=debug)
