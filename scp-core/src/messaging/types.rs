//! SCP Message Type System
//!
//! Defines all message types, payload structures, and content enums
//! as specified in SCP §5.3. Message types follow the naming convention:
//! `scp.{category}.v{version}.{action}`
//!
//! ## Categories:
//! - **message**: Basic chat messages (text, image, file, video, audio)
//! - **group**: Group management (create, invite, join, leave, kick, etc.)
//! - **agent**: Agent semantics (card, task, claim, result, approve, query, response)
//! - **swarm**: Swarm coordination (formation, coordinate, consensus)
//! - **system**: System status (typing, read, presence)
//! - **control**: Control messages (ack, error)

use serde::{Deserialize, Serialize};

// ============================================================================
// Message Type Enum
// ============================================================================

/// Top-level message type following SCP naming convention.
///
/// ```text
/// scp.{category}.v{version}.{action}
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageType {
    // ---- Basic Messages (§5.3.1) ----
    /// Plain text message
    #[serde(rename = "scp.message.v1.text")]
    Text,

    /// Image message with CID reference
    #[serde(rename = "scp.message.v1.image")]
    Image,

    /// File message with CID reference
    #[serde(rename = "scp.message.v1.file")]
    File,

    /// Video message with CID reference
    #[serde(rename = "scp.message.v1.video")]
    Video,

    /// Audio message with CID reference
    #[serde(rename = "scp.message.v1.audio")]
    Audio,

    // ---- Group Management (§5.3.2) ----
    /// Create a new group
    #[serde(rename = "scp.group.v1.create")]
    GroupCreate,

    /// Invite members to a group
    #[serde(rename = "scp.group.v1.invite")]
    GroupInvite,

    /// Join a public group
    #[serde(rename = "scp.group.v1.join")]
    GroupJoin,

    /// Leave a group
    #[serde(rename = "scp.group.v1.leave")]
    GroupLeave,

    /// Remove a member from a group
    #[serde(rename = "scp.group.v1.kick")]
    GroupKick,

    /// Update group metadata
    #[serde(rename = "scp.group.v1.update_meta")]
    GroupUpdateMeta,

    /// Sync group state events
    #[serde(rename = "scp.group.v1.sync_state")]
    GroupSyncState,

    // ---- Agent Semantics (§5.3.3) ----
    /// Agent capability card
    #[serde(rename = "scp.agent.v1.card")]
    AgentCard,

    /// Publish/assign a task
    #[serde(rename = "scp.agent.v1.task")]
    AgentTask,

    /// Claim a task
    #[serde(rename = "scp.agent.v1.claim")]
    AgentClaim,

    /// Submit task result
    #[serde(rename = "scp.agent.v1.result")]
    AgentResult,

    /// Approve/reject task result
    #[serde(rename = "scp.agent.v1.approve")]
    AgentApprove,

    /// Agent-to-agent query
    #[serde(rename = "scp.agent.v1.query")]
    AgentQuery,

    /// Agent query response
    #[serde(rename = "scp.agent.v1.response")]
    AgentResponse,

    // ---- Swarm Coordination (§5.3.4) ----
    /// Form a sub-swarm from a parent group
    #[serde(rename = "scp.swarm.v1.formation")]
    SwarmFormation,

    /// Swarm coordination action
    #[serde(rename = "scp.swarm.v1.coordinate")]
    SwarmCoordinate,

    /// Swarm consensus proposal
    #[serde(rename = "scp.swarm.v1.consensus")]
    SwarmConsensus,

    // ---- System & Control (§5.3.5) ----
    /// Typing indicator
    #[serde(rename = "scp.system.v1.typing")]
    SystemTyping,

    /// Read receipt
    #[serde(rename = "scp.system.v1.read")]
    SystemRead,

    /// Presence status (online/away/offline)
    #[serde(rename = "scp.system.v1.presence")]
    SystemPresence,

    /// Acknowledgement
    #[serde(rename = "scp.control.v1.ack")]
    ControlAck,

    /// Error report
    #[serde(rename = "scp.control.v1.error")]
    ControlError,
}

impl MessageType {
    /// Get the category from a message type string
    pub fn category(&self) -> &'static str {
        match self {
            Self::Text | Self::Image | Self::File | Self::Video | Self::Audio => "message",
            Self::GroupCreate | Self::GroupInvite | Self::GroupJoin
            | Self::GroupLeave | Self::GroupKick | Self::GroupUpdateMeta
            | Self::GroupSyncState => "group",
            Self::AgentCard | Self::AgentTask | Self::AgentClaim
            | Self::AgentResult | Self::AgentApprove | Self::AgentQuery
            | Self::AgentResponse => "agent",
            Self::SwarmFormation | Self::SwarmCoordinate | Self::SwarmConsensus => "swarm",
            Self::SystemTyping | Self::SystemRead | Self::SystemPresence => "system",
            Self::ControlAck | Self::ControlError => "control",
        }
    }

    /// Get the version string
    pub fn version(&self) -> &'static str {
        "v1"
    }

    /// Get the action name
    pub fn action(&self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Image => "image",
            Self::File => "file",
            Self::Video => "video",
            Self::Audio => "audio",
            Self::GroupCreate => "create",
            Self::GroupInvite => "invite",
            Self::GroupJoin => "join",
            Self::GroupLeave => "leave",
            Self::GroupKick => "kick",
            Self::GroupUpdateMeta => "update_meta",
            Self::GroupSyncState => "sync_state",
            Self::AgentCard => "card",
            Self::AgentTask => "task",
            Self::AgentClaim => "claim",
            Self::AgentResult => "result",
            Self::AgentApprove => "approve",
            Self::AgentQuery => "query",
            Self::AgentResponse => "response",
            Self::SwarmFormation => "formation",
            Self::SwarmCoordinate => "coordinate",
            Self::SwarmConsensus => "consensus",
            Self::SystemTyping => "typing",
            Self::SystemRead => "read",
            Self::SystemPresence => "presence",
            Self::ControlAck => "ack",
            Self::ControlError => "error",
        }
    }

    /// Full string representation (e.g., "scp.message.v1.text")
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Text => "scp.message.v1.text",
            Self::Image => "scp.message.v1.image",
            Self::File => "scp.message.v1.file",
            Self::Video => "scp.message.v1.video",
            Self::Audio => "scp.message.v1.audio",
            Self::GroupCreate => "scp.group.v1.create",
            Self::GroupInvite => "scp.group.v1.invite",
            Self::GroupJoin => "scp.group.v1.join",
            Self::GroupLeave => "scp.group.v1.leave",
            Self::GroupKick => "scp.group.v1.kick",
            Self::GroupUpdateMeta => "scp.group.v1.update_meta",
            Self::GroupSyncState => "scp.group.v1.sync_state",
            Self::AgentCard => "scp.agent.v1.card",
            Self::AgentTask => "scp.agent.v1.task",
            Self::AgentClaim => "scp.agent.v1.claim",
            Self::AgentResult => "scp.agent.v1.result",
            Self::AgentApprove => "scp.agent.v1.approve",
            Self::AgentQuery => "scp.agent.v1.query",
            Self::AgentResponse => "scp.agent.v1.response",
            Self::SwarmFormation => "scp.swarm.v1.formation",
            Self::SwarmCoordinate => "scp.swarm.v1.coordinate",
            Self::SwarmConsensus => "scp.swarm.v1.consensus",
            Self::SystemTyping => "scp.system.v1.typing",
            Self::SystemRead => "scp.system.v1.read",
            Self::SystemPresence => "scp.system.v1.presence",
            Self::ControlAck => "scp.control.v1.ack",
            Self::ControlError => "scp.control.v1.error",
        }
    }

    /// Parse from a type string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "scp.message.v1.text" => Some(Self::Text),
            "scp.message.v1.image" => Some(Self::Image),
            "scp.message.v1.file" => Some(Self::File),
            "scp.message.v1.video" => Some(Self::Video),
            "scp.message.v1.audio" => Some(Self::Audio),
            "scp.group.v1.create" => Some(Self::GroupCreate),
            "scp.group.v1.invite" => Some(Self::GroupInvite),
            "scp.group.v1.join" => Some(Self::GroupJoin),
            "scp.group.v1.leave" => Some(Self::GroupLeave),
            "scp.group.v1.kick" => Some(Self::GroupKick),
            "scp.group.v1.update_meta" => Some(Self::GroupUpdateMeta),
            "scp.group.v1.sync_state" => Some(Self::GroupSyncState),
            "scp.agent.v1.card" => Some(Self::AgentCard),
            "scp.agent.v1.task" => Some(Self::AgentTask),
            "scp.agent.v1.claim" => Some(Self::AgentClaim),
            "scp.agent.v1.result" => Some(Self::AgentResult),
            "scp.agent.v1.approve" => Some(Self::AgentApprove),
            "scp.agent.v1.query" => Some(Self::AgentQuery),
            "scp.agent.v1.response" => Some(Self::AgentResponse),
            "scp.swarm.v1.formation" => Some(Self::SwarmFormation),
            "scp.swarm.v1.coordinate" => Some(Self::SwarmCoordinate),
            "scp.swarm.v1.consensus" => Some(Self::SwarmConsensus),
            "scp.system.v1.typing" => Some(Self::SystemTyping),
            "scp.system.v1.read" => Some(Self::SystemRead),
            "scp.system.v1.presence" => Some(Self::SystemPresence),
            "scp.control.v1.ack" => Some(Self::ControlAck),
            "scp.control.v1.error" => Some(Self::ControlError),
            _ => None,
        }
    }

    /// Check if this is a basic message type (text, image, file, video, audio)
    pub fn is_basic_message(&self) -> bool {
        matches!(self, Self::Text | Self::Image | Self::File | Self::Video | Self::Audio)
    }

    /// Check if this is a group management type
    pub fn is_group_management(&self) -> bool {
        matches!(self, Self::GroupCreate | Self::GroupInvite | Self::GroupJoin
            | Self::GroupLeave | Self::GroupKick | Self::GroupUpdateMeta
            | Self::GroupSyncState)
    }

    /// Check if this is an agent type
    pub fn is_agent(&self) -> bool {
        matches!(self, Self::AgentCard | Self::AgentTask | Self::AgentClaim
            | Self::AgentResult | Self::AgentApprove | Self::AgentQuery
            | Self::AgentResponse)
    }

    /// Check if this is a swarm coordination type
    pub fn is_swarm(&self) -> bool {
        matches!(self, Self::SwarmFormation | Self::SwarmCoordinate | Self::SwarmConsensus)
    }

    /// Check if this is a system/control type
    pub fn is_system(&self) -> bool {
        matches!(self, Self::SystemTyping | Self::SystemRead | Self::SystemPresence
            | Self::ControlAck | Self::ControlError)
    }
}

impl std::fmt::Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ============================================================================
// Payload Content Structures
// ============================================================================

/// All possible payload content types for SCP messages.
/// Maps each MessageType to its corresponding payload structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PayloadContent {
    // ---- Basic Messages ----
    Text(TextPayload),
    Image(ImagePayload),
    File(FilePayload),
    Video(VideoPayload),
    Audio(AudioPayload),

    // ---- Group Management ----
    GroupCreate(GroupCreatePayload),
    GroupInvite(GroupInvitePayload),
    GroupJoin(GroupJoinPayload),
    GroupLeave(GroupLeavePayload),
    GroupKick(GroupKickPayload),
    GroupUpdateMeta(GroupUpdateMetaPayload),
    GroupSyncState(GroupSyncStatePayload),

    // ---- Agent ----
    AgentCard(AgentCardPayload),
    AgentTask(AgentTaskPayload),
    AgentClaim(AgentClaimPayload),
    AgentResult(AgentResultPayload),
    AgentApprove(AgentApprovePayload),
    AgentQuery(AgentQueryPayload),
    AgentResponse(AgentResponsePayload),

    // ---- Swarm ----
    SwarmFormation(SwarmFormationPayload),
    SwarmCoordinate(SwarmCoordinatePayload),
    SwarmConsensus(SwarmConsensusPayload),

    // ---- System/Control ----
    SystemTyping(SystemTypingPayload),
    SystemRead(SystemReadPayload),
    SystemPresence(SystemPresencePayload),
    ControlAck(ControlAckPayload),
    ControlError(ControlErrorPayload),
}

/// Wrapper for a typed message payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagePayload {
    /// The message type
    #[serde(rename = "type")]
    pub msg_type: MessageType,

    /// The payload content
    pub content: PayloadContent,
}

// ---- Basic Message Payloads (§5.3.1) ----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextPayload {
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImagePayload {
    pub cid: String,
    pub mime: String,
    pub width: u32,
    pub height: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_cid: Option<String>,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePayload {
    pub cid: String,
    pub name: String,
    pub mime: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoPayload {
    pub cid: String,
    pub mime: String,
    pub duration: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_cid: Option<String>,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioPayload {
    pub cid: String,
    pub mime: String,
    pub duration: u32,
    pub size: u64,
}

// ---- Group Management Payloads (§5.3.2) ----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupCreatePayload {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_cid: Option<String>,
    pub settings: GroupSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupSettings {
    pub join_policy: JoinPolicy,
    pub who_can_send: WhoCanSend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JoinPolicy {
    Invite,
    Open,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WhoCanSend {
    All,
    Admins,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupInvitePayload {
    pub group_id: String,
    pub invitees: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupJoinPayload {
    pub group_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupLeavePayload {
    pub group_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupKickPayload {
    pub group_id: String,
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupUpdateMetaPayload {
    pub group_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_cid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupSyncStatePayload {
    pub group_id: String,
    pub state_hash: String,
    pub events: Vec<serde_json::Value>,
}

// ---- Agent Payloads (§5.3.3) ----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCardPayload {
    pub agent_id: String,
    pub capabilities: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_url: Option<String>,
    pub rate_limit: u32,
    pub models: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTaskPayload {
    pub task_id: String,
    pub description: String,
    pub context: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentClaimPayload {
    pub task_id: String,
    pub claimer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResultPayload {
    pub task_id: String,
    pub status: TaskStatus,
    pub output: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Success,
    Failure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentApprovePayload {
    pub task_id: String,
    pub approved: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentQueryPayload {
    pub query_id: String,
    pub question: String,
    pub context: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponsePayload {
    pub query_id: String,
    pub answer: String,
}

// ---- Swarm Payloads (§5.3.4) ----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmFormationPayload {
    pub swarm_id: String,
    pub parent_group: String,
    pub members: Vec<String>,
    pub purpose: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmCoordinatePayload {
    pub swarm_id: String,
    pub action: SwarmAction,
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SwarmAction {
    ElectLeader,
    AssignRole,
    Dissolve,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmConsensusPayload {
    pub swarm_id: String,
    pub proposal: String,
    pub votes: std::collections::HashMap<String, Vote>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Vote {
    Approve,
    Reject,
}

// ---- System/Control Payloads (§5.3.5) ----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemTypingPayload {
    pub room_id: String,
    pub is_typing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemReadPayload {
    pub room_id: String,
    pub last_read_event_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PresenceStatus {
    Online,
    Away,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPresencePayload {
    pub status: PresenceStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AckStatus {
    Delivered,
    Read,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlAckPayload {
    pub message_id: String,
    pub status: AckStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlErrorPayload {
    pub code: u32,
    pub message: String,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type_roundtrip() {
        let all_types = vec![
            MessageType::Text,
            MessageType::Image,
            MessageType::File,
            MessageType::Video,
            MessageType::Audio,
            MessageType::GroupCreate,
            MessageType::GroupInvite,
            MessageType::GroupJoin,
            MessageType::GroupLeave,
            MessageType::GroupKick,
            MessageType::GroupUpdateMeta,
            MessageType::GroupSyncState,
            MessageType::AgentCard,
            MessageType::AgentTask,
            MessageType::AgentClaim,
            MessageType::AgentResult,
            MessageType::AgentApprove,
            MessageType::AgentQuery,
            MessageType::AgentResponse,
            MessageType::SwarmFormation,
            MessageType::SwarmCoordinate,
            MessageType::SwarmConsensus,
            MessageType::SystemTyping,
            MessageType::SystemRead,
            MessageType::SystemPresence,
            MessageType::ControlAck,
            MessageType::ControlError,
        ];

        for msg_type in all_types {
            let s = msg_type.as_str();
            let parsed = MessageType::from_str(s).unwrap();
            assert_eq!(msg_type, parsed, "Roundtrip failed for {}", s);
        }
    }

    #[test]
    fn test_message_type_categories() {
        assert!(MessageType::Text.is_basic_message());
        assert!(MessageType::Image.is_basic_message());
        assert!(MessageType::File.is_basic_message());

        assert!(MessageType::GroupCreate.is_group_management());
        assert!(MessageType::GroupInvite.is_group_management());

        assert!(MessageType::AgentCard.is_agent());
        assert!(MessageType::AgentTask.is_agent());

        assert!(MessageType::SwarmFormation.is_swarm());

        assert!(MessageType::SystemTyping.is_system());
        assert!(MessageType::ControlAck.is_system());
    }

    #[test]
    fn test_invalid_message_type() {
        assert_eq!(MessageType::from_str("scp.invalid.v1.fake"), None);
        assert_eq!(MessageType::from_str("garbage"), None);
    }

    #[test]
    fn test_text_payload_serialization() {
        let payload = TextPayload {
            body: "Hello, Swarm!".to_string(),
        };
        let json = serde_json::to_string(&payload).unwrap();
        let parsed: TextPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.body, "Hello, Swarm!");
    }

    #[test]
    fn test_agent_card_payload() {
        let payload = AgentCardPayload {
            agent_id: "did:key:z6Mk...".to_string(),
            capabilities: vec!["code_review".to_string(), "security_scan".to_string()],
            owner: Some("did:key:z6Mk...owner".to_string()),
            callback_url: Some("https://agent.example.com/webhook".to_string()),
            rate_limit: 10,
            models: vec!["gpt-4".to_string()],
        };
        let json = serde_json::to_string(&payload).unwrap();
        let parsed: AgentCardPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.capabilities.len(), 2);
        assert_eq!(parsed.rate_limit, 10);
    }

    #[test]
    fn test_group_create_payload() {
        let payload = GroupCreatePayload {
            name: "Swarm Alpha".to_string(),
            avatar_cid: None,
            settings: GroupSettings {
                join_policy: JoinPolicy::Invite,
                who_can_send: WhoCanSend::All,
            },
        };
        let json = serde_json::to_string(&payload).unwrap();
        let parsed: GroupCreatePayload = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "Swarm Alpha");
    }
}
