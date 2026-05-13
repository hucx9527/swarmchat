"""SCP message types and payload structures per SCP specification 5.3."""

from enum import Enum
from typing import Optional, Any, Dict, List
from pydantic import BaseModel, Field


class MessageType(str, Enum):
    """SCP message types following the naming convention: scp.{category}.v{version}.{action}"""

    # Basic Messages (§5.3.1)
    TEXT = "scp.message.v1.text"
    IMAGE = "scp.message.v1.image"
    FILE = "scp.message.v1.file"
    VIDEO = "scp.message.v1.video"
    AUDIO = "scp.message.v1.audio"

    # Group Management (§5.3.2)
    GROUP_CREATE = "scp.group.v1.create"
    GROUP_INVITE = "scp.group.v1.invite"
    GROUP_JOIN = "scp.group.v1.join"
    GROUP_LEAVE = "scp.group.v1.leave"
    GROUP_KICK = "scp.group.v1.kick"
    GROUP_UPDATE_META = "scp.group.v1.update_meta"
    GROUP_SYNC_STATE = "scp.group.v1.sync_state"

    # Agent Semantics (§5.3.3)
    AGENT_CARD = "scp.agent.v1.card"
    AGENT_TASK = "scp.agent.v1.task"
    AGENT_CLAIM = "scp.agent.v1.claim"
    AGENT_RESULT = "scp.agent.v1.result"
    AGENT_APPROVE = "scp.agent.v1.approve"
    AGENT_QUERY = "scp.agent.v1.query"
    AGENT_RESPONSE = "scp.agent.v1.response"

    # Swarm Coordination (§5.3.4)
    SWARM_FORMATION = "scp.swarm.v1.formation"
    SWARM_COORDINATE = "scp.swarm.v1.coordinate"
    SWARM_CONSENSUS = "scp.swarm.v1.consensus"

    # System & Control (§5.3.5)
    SYSTEM_TYPING = "scp.system.v1.typing"
    SYSTEM_READ = "scp.system.v1.read"
    SYSTEM_PRESENCE = "scp.system.v1.presence"
    CONTROL_ACK = "scp.control.v1.ack"
    CONTROL_ERROR = "scp.control.v1.error"

    @property
    def category(self) -> str:
        return self.value.split(".")[1]

    @property
    def action(self) -> str:
        return self.value.split(".")[-1]


# ---- Payload Models ----

class TextPayload(BaseModel):
    body: str


class ImagePayload(BaseModel):
    cid: str
    mime: str = "image/png"
    width: int = 0
    height: int = 0
    thumbnail_cid: Optional[str] = None
    size: int = 0


class FilePayload(BaseModel):
    cid: str
    name: str
    mime: str = "application/octet-stream"
    size: int = 0


class AgentCardPayload(BaseModel):
    agent_id: str = Field(alias="agent_id")
    capabilities: List[str]
    owner: Optional[str] = None
    callback_url: Optional[str] = Field(None, alias="callback_url")
    rate_limit: int = Field(10, alias="rate_limit")
    models: List[str] = Field(default_factory=list)

    class Config:
        populate_by_name = True


class TaskPayload(BaseModel):
    task_id: str = Field(alias="task_id")
    description: str
    context: Dict[str, Any] = Field(default_factory=dict)
    deadline: Optional[int] = None
    assignee: Optional[str] = None

    class Config:
        populate_by_name = True


class AgentResultPayload(BaseModel):
    task_id: str = Field(alias="task_id")
    status: str  # "success" or "failure"
    output: str
    evidence: List[str] = Field(default_factory=list)

    class Config:
        populate_by_name = True


class AgentApprovePayload(BaseModel):
    task_id: str = Field(alias="task_id")
    approved: bool
    feedback: Optional[str] = None

    class Config:
        populate_by_name = True


class GroupCreatePayload(BaseModel):
    name: str
    avatar_cid: Optional[str] = None
    settings: Dict[str, str] = Field(default_factory=dict)


class GroupInvitePayload(BaseModel):
    group_id: str
    invitees: List[str]


class MessagePayload(BaseModel):
    """Wrapper for a typed message payload."""
    type: MessageType = Field(alias="type")
    content: Dict[str, Any] = Field(default_factory=dict)

    class Config:
        populate_by_name = True


# ---- Envelope ----

class EncryptionMeta(BaseModel):
    scheme: str = "double-ratchet"
    session_id: str
    nonce: str


class Envelope(BaseModel):
    """SCP message envelope per §5.1."""
    id: str
    protocol: str = "scp/1.0"
    type: MessageType = Field(alias="type")
    from_did: str = Field(alias="from")
    to: List[str] = Field(default_factory=list)
    group_id: Optional[str] = Field(None, alias="group_id")
    timestamp: int = 0
    ttl: int = 604800
    encryption: EncryptionMeta
    signature: Optional[str] = None
    metadata: Dict[str, str] = Field(default_factory=dict)

    class Config:
        populate_by_name = True
