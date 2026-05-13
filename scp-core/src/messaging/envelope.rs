//! SCP Message Envelope
//!
//! Implements the message envelope structure as defined in SCP §5.1.
//! The envelope wraps encrypted payloads with metadata for routing,
//! encryption session management, and integrity verification.
//!
//! ## Envelope Structure (SCP §5.1.1)
//!
//! ```json
//! {
//!   "envelope": {
//!     "id": "01HXVBYZ3M7P8Q2R5S9T0U1V2W",
//!     "protocol": "scp/1.0",
//!     "type": "scp.message.v1.text",
//!     "from": "did:key:z6Mk...",
//!     "to": ["did:key:z6Mk..."],
//!     "group_id": "group:01HXVBYZ3M7P8Q2R5S9T0U1V2W",
//!     "timestamp": 1713523456789,
//!     "ttl": 604800,
//!     "encryption": {
//!       "scheme": "double-ratchet",
//!       "session_id": "sess_01HXVBYZ3M7P8Q2R5S9T0U1V2W",
//!       "nonce": "W9h0K1mP3xQ7vL2n"
//!     },
//!     "signature": "6pHv8Qr2Jk5Lm9Nd4Fg7Hj3Kp1Rt5Uw8Yb0Cd3Ea2Zs..."
//!   },
//!   "payload": "V2hhdCBhIHdvbmRlcmZ1bCB3b3JsZCE="
//! }

use std::collections::HashMap;

use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey, Signature};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

use super::types::MessageType;

/// Error types for envelope operations
#[derive(Debug, thiserror::Error)]
pub enum EnvelopeError {
    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Missing field: {0}")]
    MissingField(String),

    #[error("Invalid protocol version: {0}")]
    InvalidProtocol(String),

    #[error("Envelope expired at {0}")]
    Expired(u64),

    #[error("Signature error: {0}")]
    SignatureError(String),

    #[error("Base64 decode error: {0}")]
    Base64Error(#[from] base64::DecodeError),
}

/// Encryption metadata embedded in the envelope
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EncryptionMeta {
    /// Encryption scheme used
    pub scheme: EncryptionScheme,

    /// Session identifier for the encryption session
    pub session_id: String,

    /// Nonce used for encryption (Base64 encoded)
    pub nonce: String,
}

/// Supported encryption schemes (SCP §5.1.2)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum EncryptionScheme {
    /// Double Ratchet for 1:1 communication
    DoubleRatchet,

    /// Sender Key for group communication
    SenderKey,

    /// No encryption (control messages only)
    None,
}

impl EncryptionScheme {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DoubleRatchet => "double-ratchet",
            Self::SenderKey => "sender-key",
            Self::None => "none",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "double-ratchet" => Some(Self::DoubleRatchet),
            "sender-key" => Some(Self::SenderKey),
            "none" => Some(Self::None),
            _ => None,
        }
    }
}

/// The main SCP Message Envelope
///
/// Contains all metadata needed for message routing, encryption session
/// management, and integrity verification. The actual message content is
/// carried as an encrypted payload separate from the envelope metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    /// Unique message identifier (ULID format)
    pub id: String,

    /// Protocol version (fixed "scp/1.0")
    pub protocol: String,

    /// Message type following SCP naming convention
    #[serde(rename = "type")]
    pub msg_type: MessageType,

    /// Sender's DID
    pub from: String,

    /// Recipients' DIDs (empty for group messages)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub to: Vec<String>,

    /// Group ID (required for group messages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,

    /// Unix timestamp in milliseconds
    pub timestamp: u64,

    /// Time-to-live in seconds (0 = never expires)
    #[serde(default)]
    pub ttl: u32,

    /// Encryption metadata
    pub encryption: EncryptionMeta,

    /// Ed25519 signature over the envelope (Base64, excluding signature field)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,

    /// Additional metadata (extensible)
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub metadata: HashMap<String, String>,
}

impl Envelope {
    /// Protocol version constant
    pub const PROTOCOL_VERSION: &'static str = "scp/1.0";

    /// Create a new unsigned envelope
    pub fn new(
        id: String,
        msg_type: MessageType,
        from: String,
        to: Vec<String>,
        group_id: Option<String>,
        encryption: EncryptionMeta,
        ttl: u32,
    ) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Envelope {
            id,
            protocol: Self::PROTOCOL_VERSION.to_string(),
            msg_type,
            from,
            to,
            group_id,
            timestamp,
            ttl,
            encryption,
            signature: None,
            metadata: HashMap::new(),
        }
    }

    /// Create an envelope for a 1:1 message
    pub fn one_to_one(
        id: String,
        msg_type: MessageType,
        from: String,
        to: String,
        session_id: String,
        nonce: String,
    ) -> Self {
        Envelope::new(
            id,
            msg_type,
            from,
            vec![to],
            None,
            EncryptionMeta {
                scheme: EncryptionScheme::DoubleRatchet,
                session_id,
                nonce,
            },
            604_800, // default 7-day TTL
        )
    }

    /// Create an envelope for a group message
    pub fn group_message(
        id: String,
        msg_type: MessageType,
        from: String,
        group_id: String,
        session_id: String,
        nonce: String,
    ) -> Self {
        Envelope::new(
            id,
            msg_type,
            from,
            vec![],
            Some(group_id),
            EncryptionMeta {
                scheme: EncryptionScheme::SenderKey,
                session_id,
                nonce,
            },
            604_800,
        )
    }

    /// Create a control message envelope (no encryption)
    pub fn control(
        id: String,
        msg_type: MessageType,
        from: String,
        to: Vec<String>,
    ) -> Self {
        let nonce = generate_nonce();
        Envelope::new(
            id,
            msg_type,
            from,
            to,
            None,
            EncryptionMeta {
                scheme: EncryptionScheme::None,
                session_id: String::new(),
                nonce,
            },
            0, // control messages don't expire
        )
    }

    /// Compute the canonical JSON representation for signing.
    ///
    /// Per SCP §5.1.3: the signature is computed over all envelope fields
    /// (in sorted key order) EXCEPT the signature field itself.
    pub fn canonical_for_signing(&self) -> Result<Vec<u8>, EnvelopeError> {
        let canonical = serde_json::json!({
            "id": self.id,
            "protocol": self.protocol,
            "type": self.msg_type.as_str(),
            "from": self.from,
            "to": self.to,
            "group_id": self.group_id,
            "timestamp": self.timestamp,
            "ttl": self.ttl,
            "encryption": {
                "scheme": self.encryption.scheme.as_str(),
                "session_id": self.encryption.session_id,
                "nonce": self.encryption.nonce,
            },
            "metadata": self.metadata,
        });

        // Serialize with sorted keys (serde_json preserves insertion order,
        // so we use a BTreeMap-like approach via sorted serialization)
        let mut buf = Vec::new();
        let mut ser = serde_json::Serializer::with_formatter(
            &mut buf,
            CanonicalFormatter,
        );
        canonical.serialize(&mut ser)
            .map_err(|e| EnvelopeError::Serialization(e.to_string()))?;

        Ok(buf)
    }

    /// Sign the envelope with the sender's Ed25519 signing key.
    ///
    /// This computes the canonical JSON of the envelope (without the signature field),
    /// hashes it with SHA-256, and signs it with Ed25519.
    pub fn sign(&mut self, signing_key: &SigningKey) -> Result<(), EnvelopeError> {
        let canonical = self.canonical_for_signing()?;

        // SHA-256 hash then Ed25519 sign
        let hash = Sha256::digest(&canonical);
        let sig: Signature = signing_key.sign(&hash);

        self.signature = Some(base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            sig.to_bytes(),
        ));

        Ok(())
    }

    /// Verify the envelope's signature against a public key.
    ///
    /// Returns true if the signature is valid, false otherwise.
    /// Envelopes without a signature always fail verification.
    pub fn verify(&self, verifying_key: &VerifyingKey) -> Result<bool, EnvelopeError> {
        let signature_b64 = match &self.signature {
            Some(sig) => sig,
            None => return Ok(false),
        };

        let sig_bytes = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            signature_b64,
        )?;

        if sig_bytes.len() != 64 {
            return Err(EnvelopeError::SignatureError(
                format!("Signature must be 64 bytes, got {}", sig_bytes.len())
            ));
        }

        let mut sig_array = [0u8; 64];
        sig_array.copy_from_slice(&sig_bytes);
        let signature = Signature::from_bytes(&sig_array);

        let canonical = self.canonical_for_signing()?;
        let hash = Sha256::digest(&canonical);

        Ok(verifying_key.verify(&hash, &signature).is_ok())
    }

    /// Serialize the envelope to JSON
    pub fn to_json(&self) -> Result<String, EnvelopeError> {
        serde_json::to_string(self)
            .map_err(|e| EnvelopeError::Serialization(e.to_string()))
    }

    /// Deserialize the envelope from JSON
    pub fn from_json(json: &str) -> Result<Self, EnvelopeError> {
        serde_json::from_str(json)
            .map_err(|e| EnvelopeError::Deserialization(e.to_string()))
    }

    /// Serialize the envelope to CBOR (for efficient network transport)
    pub fn to_cbor(&self) -> Result<Vec<u8>, EnvelopeError> {
        let mut buf = Vec::new();
        ciborium::into_writer(self, &mut buf)
            .map_err(|e| EnvelopeError::Serialization(e.to_string()))?;
        Ok(buf)
    }

    /// Deserialize the envelope from CBOR
    pub fn from_cbor(data: &[u8]) -> Result<Self, EnvelopeError> {
        ciborium::from_reader(data)
            .map_err(|e| EnvelopeError::Deserialization(e.to_string()))
    }

    /// Check if the envelope has expired based on its TTL
    pub fn is_expired(&self) -> bool {
        if self.ttl == 0 {
            return false; // never expires
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        // TTL is in seconds, timestamp is in milliseconds
        let ttl_ms = (self.ttl as u64) * 1000;
        now > self.timestamp + ttl_ms
    }

    /// Check if protocol version is supported
    pub fn is_supported_protocol(&self) -> bool {
        self.protocol == Self::PROTOCOL_VERSION
    }

    /// Set custom metadata
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

// ============================================================================
// Canonical JSON Formatter for Signature
// ============================================================================

/// A custom JSON formatter that ensures canonical (sorted-key) output
/// for deterministic signing.
struct CanonicalFormatter;

impl serde_json::ser::Formatter for CanonicalFormatter {
    fn begin_object<W: ?Sized + std::io::Write>(&mut self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(b"{")
    }

    fn end_object<W: ?Sized + std::io::Write>(&mut self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(b"}")
    }

    fn begin_object_key<W: ?Sized + std::io::Write>(&mut self, _writer: &mut W, _first: bool) -> std::io::Result<()> {
        Ok(()) // caller writes the string
    }

    fn end_object_key<W: ?Sized + std::io::Write>(&mut self, _writer: &mut W) -> std::io::Result<()> {
        Ok(())
    }

    fn begin_object_value<W: ?Sized + std::io::Write>(&mut self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(b":")
    }

    fn end_object_value<W: ?Sized + std::io::Write>(&mut self, _writer: &mut W) -> std::io::Result<()> {
        Ok(())
    }

    fn write_string_fragment<W: ?Sized + std::io::Write>(&mut self, writer: &mut W, fragment: &str) -> std::io::Result<()> {
        writer.write_all(fragment.as_bytes())
    }

    fn write_number_str<W: ?Sized + std::io::Write>(&mut self, writer: &mut W, value: &str) -> std::io::Result<()> {
        writer.write_all(value.as_bytes())
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Generate a random nonce for encryption
fn generate_nonce() -> String {
    let mut nonce = [0u8; 12];
    getrandom::getrandom(&mut nonce).expect("Failed to generate nonce");
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, nonce)
}

/// Generate a ULID-like message ID
pub fn generate_message_id() -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    let mut random = [0u8; 8];
    getrandom::getrandom(&mut random).expect("Failed to generate random bytes");

    // Encode as base32-like string (simplified ULID format)
    format!("{:013x}{:016x}", timestamp, u64::from_le_bytes(random))
}

/// Create an encryption metadata with random nonce
pub fn create_encryption_meta(scheme: EncryptionScheme, session_id: String) -> EncryptionMeta {
    EncryptionMeta {
        scheme,
        session_id,
        nonce: generate_nonce(),
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;

    fn generate_keypair() -> SigningKey {
        SigningKey::generate(&mut OsRng)
    }

    #[test]
    fn test_envelope_creation() {
        let envelope = Envelope::one_to_one(
            "01HXVBYZ3M7P8Q2R5S9T0U1V2W".to_string(),
            MessageType::Text,
            "did:key:z6Mk...alice".to_string(),
            "did:key:z6Mk...bob".to_string(),
            "sess_01HXVBYZ3M7P8Q2R5S9T0U1V2W".to_string(),
            "W9h0K1mP3xQ7vL2n".to_string(),
        );

        assert_eq!(envelope.protocol, "scp/1.0");
        assert_eq!(envelope.msg_type, MessageType::Text);
        assert_eq!(envelope.to.len(), 1);
        assert!(envelope.group_id.is_none());
    }

    #[test]
    fn test_envelope_group_message() {
        let envelope = Envelope::group_message(
            "01HXVBYZ3M7P8Q2R5S9T0U1V2W".to_string(),
            MessageType::Text,
            "did:key:z6Mk...alice".to_string(),
            "group:01HXVBYZ3M7P8Q2R5S9T0U1V2W".to_string(),
            "sess_group_123".to_string(),
            "nonce123".to_string(),
        );

        assert_eq!(envelope.encryption.scheme, EncryptionScheme::SenderKey);
        assert!(envelope.group_id.is_some());
        assert!(envelope.to.is_empty());
    }

    #[test]
    fn test_sign_and_verify() {
        let signing_key = generate_keypair();
        let verifying_key = signing_key.verifying_key();

        let mut envelope = Envelope::one_to_one(
            generate_message_id(),
            MessageType::Text,
            "did:key:z6Mk...alice".to_string(),
            "did:key:z6Mk...bob".to_string(),
            "sess_123".to_string(),
            "nonce123".to_string(),
        );

        // Sign
        envelope.sign(&signing_key).unwrap();
        assert!(envelope.signature.is_some());

        // Verify with correct key
        assert!(envelope.verify(&verifying_key).unwrap());

        // Verify with wrong key
        let wrong_key = generate_keypair();
        assert!(!envelope.verify(&wrong_key.verifying_key()).unwrap());

        // Envelope without signature should fail verification
        let unsigned = Envelope::one_to_one(
            generate_message_id(),
            MessageType::Text,
            "did:key:z6Mk...alice".to_string(),
            "did:key:z6Mk...bob".to_string(),
            "sess_123".to_string(),
            "nonce456".to_string(),
        );
        assert!(!unsigned.verify(&verifying_key).unwrap());
    }

    #[test]
    fn test_signing_deterministic() {
        let signing_key = generate_keypair();
        let verifying_key = signing_key.verifying_key();

        let mut envelope1 = Envelope::one_to_one(
            "01HXVBYZ3M7P8Q2R5S9T0U1V2W".to_string(),
            MessageType::Text,
            "did:key:z6Mk...alice".to_string(),
            "did:key:z6Mk...bob".to_string(),
            "sess_123".to_string(),
            "nonce123".to_string(),
        );

        let mut envelope2 = envelope1.clone();

        envelope1.sign(&signing_key).unwrap();
        envelope2.sign(&signing_key).unwrap();

        // Same content + same key = same signature
        assert_eq!(envelope1.signature, envelope2.signature);

        // Both verify correctly
        assert!(envelope1.verify(&verifying_key).unwrap());
        assert!(envelope2.verify(&verifying_key).unwrap());
    }

    #[test]
    fn test_json_roundtrip() {
        let mut envelope = Envelope::one_to_one(
            generate_message_id(),
            MessageType::Text,
            "did:key:z6Mk...alice".to_string(),
            "did:key:z6Mk...bob".to_string(),
            "sess_123".to_string(),
            "nonce456".to_string(),
        );

        let signing_key = generate_keypair();
        envelope.sign(&signing_key).unwrap();

        let json = envelope.to_json().unwrap();
        let decoded = Envelope::from_json(&json).unwrap();

        assert_eq!(decoded.id, envelope.id);
        assert_eq!(decoded.msg_type, envelope.msg_type);
        assert_eq!(decoded.from, envelope.from);
        assert_eq!(decoded.to, envelope.to);
        assert_eq!(decoded.signature, envelope.signature);
    }

    #[test]
    fn test_cbor_roundtrip() {
        let envelope = Envelope::one_to_one(
            generate_message_id(),
            MessageType::File,
            "did:key:z6Mk...alice".to_string(),
            "did:key:z6Mk...bob".to_string(),
            "sess_123".to_string(),
            "nonce456".to_string(),
        );

        let cbor = envelope.to_cbor().unwrap();
        let decoded = Envelope::from_cbor(&cbor).unwrap();

        assert_eq!(decoded.id, envelope.id);
        assert_eq!(decoded.msg_type, envelope.msg_type);
    }

    #[test]
    fn test_expiration() {
        let envelope = Envelope::one_to_one(
            generate_message_id(),
            MessageType::Text,
            "did:key:z6Mk...alice".to_string(),
            "did:key:z6Mk...bob".to_string(),
            "sess_123".to_string(),
            "nonce456".to_string(),
        );

        // Just created with 7-day TTL, should not be expired
        assert!(!envelope.is_expired());

        // Control message with TTL=0 should never expire
        let control = Envelope::control(
            generate_message_id(),
            MessageType::ControlAck,
            "did:key:z6Mk...alice".to_string(),
            vec!["did:key:z6Mk...bob".to_string()],
        );
        assert!(!control.is_expired());
    }

    #[test]
    fn test_protocol_version() {
        let envelope = Envelope::one_to_one(
            generate_message_id(),
            MessageType::Text,
            "did:key:z6Mk...alice".to_string(),
            "did:key:z6Mk...bob".to_string(),
            "sess_123".to_string(),
            "nonce456".to_string(),
        );

        assert!(envelope.is_supported_protocol());
    }

    #[test]
    fn test_generate_message_id_unique() {
        let id1 = generate_message_id();
        let id2 = generate_message_id();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_message_id_format() {
        let id = generate_message_id();
        // Should be hex chars (timestamp + random)
        assert_eq!(id.len(), 13 + 16);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
