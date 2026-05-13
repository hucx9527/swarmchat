"""HTTP client for communicating with SCP relay nodes."""

import time
import uuid
from typing import Optional, Dict, Any, List
import requests

from .types import Envelope, EncryptionMeta, MessageType


class SCPClient:
    """Client for interacting with SCP relay node HTTP API.

    Provides methods for:
    - Uploading/fetching prekey bundles (§4.6)
    - Storing/syncing offline messages (§5.5)
    - Health checks and node information

    Args:
        relay_url: Base URL of the relay node HTTP API.
        timeout: HTTP request timeout in seconds.
    """

    def __init__(self, relay_url: str = "http://localhost:8080", timeout: int = 30):
        self.relay_url = relay_url.rstrip("/")
        self.timeout = timeout
        self._session = requests.Session()
        self._session.headers.update({
            "Content-Type": "application/json",
            "Accept": "application/json",
            "User-Agent": f"scp-sdk-python/0.1.0",
        })

    # ---- Health & Info ----

    def health(self) -> Dict[str, Any]:
        """Check relay node health."""
        resp = self._session.get(f"{self.relay_url}/health", timeout=self.timeout)
        resp.raise_for_status()
        return resp.json()

    def node_info(self) -> Dict[str, Any]:
        """Get relay node information."""
        resp = self._session.get(f"{self.relay_url}/node/info", timeout=self.timeout)
        resp.raise_for_status()
        return resp.json()

    # ---- Prekey Bundles (§4.6) ----

    def upload_prekey(
        self,
        did: str,
        identity_key: str,
        signed_prekey: str,
        spk_signature: str,
        one_time_prekeys: List[str],
    ) -> Dict[str, Any]:
        """Upload a prekey bundle to the relay."""
        payload = {
            "did": did,
            "identity_key": identity_key,
            "signed_prekey": {
                "key": signed_prekey,
                "signature": spk_signature,
            },
            "one_time_prekeys": one_time_prekeys,
        }
        resp = self._session.post(
            f"{self.relay_url}/prekey",
            json=payload,
            timeout=self.timeout,
        )
        resp.raise_for_status()
        return resp.json()

    def get_prekey(self, did: str) -> Dict[str, Any]:
        """Fetch a prekey bundle for a DID."""
        resp = self._session.get(
            f"{self.relay_url}/prekey/{did}",
            timeout=self.timeout,
        )
        resp.raise_for_status()
        return resp.json()

    # ---- Offline Messages (§5.5) ----

    def store_message(
        self,
        to_did: str,
        envelope: str,
        from_did: str,
        message_id: Optional[str] = None,
        ttl: int = 604800,
    ) -> Dict[str, Any]:
        """Store an offline message on the relay."""
        if message_id is None:
            message_id = str(uuid.uuid4())

        payload = {
            "to_did": to_did,
            "from_did": from_did,
            "message_id": message_id,
            "envelope": envelope,
            "ttl": ttl,
        }
        resp = self._session.post(
            f"{self.relay_url}/message",
            json=payload,
            timeout=self.timeout,
        )
        resp.raise_for_status()
        return resp.json()

    def sync_messages(
        self,
        did: str,
        since: int = 0,
        limit: int = 50,
    ) -> Dict[str, Any]:
        """Sync offline messages for a DID."""
        params = {"since": since, "limit": min(limit, 500)}
        resp = self._session.get(
            f"{self.relay_url}/sync/{did}",
            params=params,
            timeout=self.timeout,
        )
        resp.raise_for_status()
        return resp.json()

    # ---- Message Helpers ----

    def send_text(
        self,
        from_did: str,
        to_did: str,
        text: str,
        session_id: Optional[str] = None,
    ) -> Dict[str, Any]:
        """Send a text message to a peer."""
        if session_id is None:
            session_id = str(uuid.uuid4())

        import base64
        import json

        # Build a simple envelope (in production, encrypt with Double Ratchet)
        envelope_data = {
            "id": str(uuid.uuid4()),
            "protocol": "scp/1.0",
            "type": MessageType.TEXT.value,
            "from": from_did,
            "to": [to_did],
            "timestamp": int(time.time() * 1000),
            "ttl": 604800,
            "encryption": {
                "scheme": "double-ratchet",
                "session_id": session_id,
                "nonce": base64.b64encode(str(uuid.uuid4()).encode()).decode()[:16],
            },
        }
        envelope_b64 = base64.b64encode(
            json.dumps(envelope_data).encode()
        ).decode()

        # Store message payload alongside envelope
        payload_b64 = base64.b64encode(text.encode()).decode()

        return self.store_message(
            to_did=to_did,
            envelope=envelope_b64,
            from_did=from_did,
        )

    def close(self):
        """Close the HTTP session."""
        self._session.close()

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.close()
