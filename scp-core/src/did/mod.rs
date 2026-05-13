//! DID (Decentralized Identifier) module for SwarmChat
//!
//! Implements W3C Decentralized Identifiers (DIDs) with the `did:key` method.
//! Based on SCP Specification section 4.1.
//!
//! Provides:
//! - `did_from_public_key()` — generate a did:key from an Ed25519 public key
//! - `public_key_from_did()` — extract Ed25519 public key from a did:key string
//! - DID Document generation per W3C spec

use ed25519_dalek::VerifyingKey;
use base58::{FromBase58, ToBase58};
use serde::{Deserialize, Serialize};
use std::fmt;

/// DID-related errors
#[derive(Debug, thiserror::Error)]
pub enum DidError {
    #[error("Invalid DID format: {0}")]
    InvalidFormat(String),

    #[error("Unsupported DID method: {0}")]
    UnsupportedMethod(String),

    #[error("Invalid multibase encoding: {0}")]
    InvalidMultibase(String),

    #[error("Invalid multicodec prefix: {0}")]
    InvalidMulticodec(String),

    #[error("Invalid public key: {0}")]
    InvalidPublicKey(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// Multicodec prefix for Ed25519 public key (varint encoded 0xed)
const ED25519_MULTICODEC_PREFIX: [u8; 2] = [0xed, 0x01];

/// DID (Decentralized Identifier) representation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Did {
    /// DID method (e.g., "key")
    pub method: String,

    /// DID identifier (base58-encoded multicodec+public_key)
    pub identifier: String,

    /// Public key bytes (raw, without multicodec prefix)
    pub public_key: Vec<u8>,

    /// Key type (e.g., "Ed25519", "X25519")
    pub key_type: String,
}

/// DID Document as per W3C specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidDocument {
    /// JSON-LD context
    #[serde(rename = "@context")]
    pub context: Vec<String>,

    /// The DID identifier
    pub id: String,

    /// Verification methods
    pub verification_method: Vec<VerificationMethod>,

    /// Authentication methods
    pub authentication: Vec<String>,

    /// Assertion methods
    pub assertion_method: Vec<String>,

    /// Key agreement methods
    pub key_agreement: Vec<String>,

    /// Created timestamp
    pub created: String,

    /// Updated timestamp
    pub updated: String,
}

/// Verification method in DID document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationMethod {
    /// Verification method ID
    pub id: String,

    /// Type of verification method
    #[serde(rename = "type")]
    pub vm_type: String,

    /// Controller (DID)
    pub controller: String,

    /// Public key in multibase format
    pub public_key_multibase: String,
}

impl Did {
    /// Create a new DID from raw public key bytes and key type
    pub fn new(public_key: &[u8], key_type: &str) -> Result<Self, DidError> {
        if public_key.len() != 32 {
            return Err(DidError::InvalidPublicKey(
                format!("Expected 32-byte public key, got {} bytes", public_key.len())
            ));
        }

        // Build multicodec-prefixed key: prefix + raw_key
        let mut multicodec_key = Vec::with_capacity(34);
        multicodec_key.extend_from_slice(&ED25519_MULTICODEC_PREFIX);
        multicodec_key.extend_from_slice(public_key);

        let identifier = multicodec_key.to_base58();

        Ok(Did {
            method: "key".to_string(),
            identifier,
            public_key: public_key.to_vec(),
            key_type: key_type.to_string(),
        })
    }

    /// Create a DID from an Ed25519 verifying key
    pub fn from_ed25519(verifying_key: &VerifyingKey) -> Result<Self, DidError> {
        Self::new(verifying_key.as_bytes(), "Ed25519")
    }

    /// Parse a DID string into a Did struct
    pub fn parse(did_string: &str) -> Result<Self, DidError> {
        // DID format: did:method:identifier
        let parts: Vec<&str> = did_string.split(':').collect();

        if parts.len() < 3 || parts[0] != "did" {
            return Err(DidError::InvalidFormat(
                format!("Expected 'did:method:identifier', got '{}'", did_string)
            ));
        }

        let method = parts[1].to_string();

        if method != "key" {
            return Err(DidError::UnsupportedMethod(method));
        }

        // For did:key, the identifier is base58(multicodec_prefix || raw_public_key)
        let identifier = parts[2..].join(":");
        let multicodec_key = identifier.from_base58()
            .map_err(|e| DidError::InvalidMultibase(format!("Base58 decode error: {:?}", e)))?;

        if multicodec_key.len() < 2 + 32 {
            return Err(DidError::InvalidMulticodec(
                format!("Multicodec key too short: {} bytes", multicodec_key.len())
            ));
        }

        // Extract multicodec prefix and determine key type
        let prefix = &multicodec_key[0..2];
        let raw_key = &multicodec_key[2..];

        let key_type = match prefix {
            [0xed, 0x01] => "Ed25519".to_string(),
            [0xec, 0x01] => "X25519".to_string(),
            _ => "Unknown".to_string(),
        };

        Ok(Did {
            method,
            identifier,
            public_key: raw_key.to_vec(),
            key_type,
        })
    }

    /// Convert DID to string representation
    pub fn to_string(&self) -> String {
        format!("did:{}:{}", self.method, self.identifier)
    }

    /// Extract the raw public key bytes (without multicodec prefix)
    pub fn public_key_bytes(&self) -> &[u8] {
        &self.public_key
    }

    /// Generate a DID document for this DID (W3C compliant)
    pub fn to_document(&self) -> Result<DidDocument, DidError> {
        let did_string = self.to_string();
        let timestamp = chrono::Utc::now().to_rfc3339();

        let verification_method = VerificationMethod {
            id: format!("{}#{}", did_string, "key-1"),
            vm_type: match self.key_type.as_str() {
                "Ed25519" => "Ed25519VerificationKey2020".to_string(),
                "X25519" => "X25519KeyAgreementKey2020".to_string(),
                _ => "JsonWebKey2020".to_string(),
            },
            controller: did_string.clone(),
            public_key_multibase: format!("z{}", self.identifier),
        };

        Ok(DidDocument {
            context: vec![
                "https://www.w3.org/ns/did/v1".to_string(),
                "https://w3id.org/security/suites/ed25519-2020/v1".to_string(),
            ],
            id: did_string.clone(),
            verification_method: vec![verification_method],
            authentication: vec![format!("{}#key-1", did_string)],
            assertion_method: vec![format!("{}#key-1", did_string)],
            key_agreement: vec![format!("{}#key-1", did_string)],
            created: timestamp.clone(),
            updated: timestamp,
        })
    }
}

impl fmt::Display for Did {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "did:{}:{}", self.method, self.identifier)
    }
}

// ── Standalone functions (per P0-3 spec) ──

/// Generate a `did:key` identifier from an Ed25519 public key.
/// This is the primary DID creation function per SCP spec §4.1.
pub fn did_from_public_key(verifying_key: &VerifyingKey) -> Result<String, DidError> {
    let did = Did::from_ed25519(verifying_key)?;
    Ok(did.to_string())
}

/// Extract the raw Ed25519 public key bytes from a `did:key` string.
/// Returns the 32-byte public key.
pub fn public_key_from_did(did_string: &str) -> Result<Vec<u8>, DidError> {
    let did = Did::parse(did_string)?;

    if did.public_key.len() != 32 {
        return Err(DidError::InvalidPublicKey(
            format!("Expected 32-byte key, got {} bytes", did.public_key.len())
        ));
    }

    Ok(did.public_key)
}

/// Generate a Did from a mutable Identity reference (will derive keys if needed).
pub fn generate_did_from_identity(
    identity: &mut crate::identity::Identity,
) -> Result<Did, DidError> {
    let keys = identity.derive_keys()
        .map_err(|e| DidError::InvalidPublicKey(format!("Key derivation failed: {}", e)))?;

    Did::from_ed25519(&keys.verifying_key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::Identity;

    #[test]
    fn test_did_creation() {
        let public_key = vec![
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
            0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
            0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
            0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
        ];

        let did = Did::new(&public_key, "Ed25519").unwrap();
        assert_eq!(did.method, "key");
        assert_eq!(did.key_type, "Ed25519");
        assert_eq!(did.public_key, public_key);

        // Test string representation
        let did_string = did.to_string();
        assert!(did_string.starts_with("did:key:"));

        // Test round-trip parse
        let parsed = Did::parse(&did_string).unwrap();
        assert_eq!(parsed.method, "key");
        assert_eq!(parsed.public_key, public_key);
    }

    #[test]
    fn test_did_from_ed25519() {
        use ed25519_dalek::SigningKey;
        let signing_key = SigningKey::from_bytes(&[0x42u8; 32]);
        let verifying_key = signing_key.verifying_key();

        let did = Did::from_ed25519(&verifying_key).unwrap();
        assert_eq!(did.key_type, "Ed25519");
        assert_eq!(did.public_key.len(), 32);
    }

    #[test]
    fn test_did_from_public_key_fn() {
        use ed25519_dalek::SigningKey;
        let signing_key = SigningKey::from_bytes(&[0x99u8; 32]);
        let verifying_key = signing_key.verifying_key();

        let did_str = did_from_public_key(&verifying_key).unwrap();
        assert!(did_str.starts_with("did:key:"));

        // Round-trip extraction
        let extracted_key = public_key_from_did(&did_str).unwrap();
        assert_eq!(extracted_key, verifying_key.as_bytes().to_vec());
    }

    #[test]
    fn test_public_key_from_did() {
        use ed25519_dalek::SigningKey;
        let signing_key = SigningKey::from_bytes(&[0x77u8; 32]);
        let verifying_key = signing_key.verifying_key();

        let did_str = did_from_public_key(&verifying_key).unwrap();
        let pk = public_key_from_did(&did_str).unwrap();
        assert_eq!(pk, verifying_key.as_bytes().to_vec());
    }

    #[test]
    fn test_did_parsing() {
        // Test valid DID (known example with multicodec prefix)
        let did = Did::parse("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK").unwrap();
        assert_eq!(did.method, "key");

        // Test invalid format
        assert!(Did::parse("not-a-did").is_err());
        assert!(Did::parse("did:unsupported:identifier").is_err());
    }

    #[test]
    fn test_did_document() {
        let public_key = vec![0u8; 32];
        let did = Did::new(&public_key, "Ed25519").unwrap();

        let document = did.to_document().unwrap();
        assert_eq!(document.id, did.to_string());
        assert!(!document.verification_method.is_empty());
        assert!(!document.authentication.is_empty());
        assert!(document.context.contains(&"https://www.w3.org/ns/did/v1".to_string()));
    }

    #[test]
    fn test_generate_from_identity() {
        let mnemonic_str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let mnemonic = bip39::Mnemonic::parse(mnemonic_str).unwrap();
        let mut identity = Identity::from_mnemonic(&mnemonic).unwrap();

        let did = generate_did_from_identity(&mut identity).unwrap();
        assert_eq!(did.method, "key");
        assert_eq!(did.key_type, "Ed25519");
        assert_eq!(did.public_key.len(), 32);
    }

    #[test]
    fn test_deterministic_did() {
        // Same mnemonic should produce same DID
        let mnemonic_str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let mnemonic = bip39::Mnemonic::parse(mnemonic_str).unwrap();

        let mut identity1 = Identity::from_mnemonic(&mnemonic).unwrap();
        let mut identity2 = Identity::from_mnemonic(&mnemonic).unwrap();

        let did1 = generate_did_from_identity(&mut identity1).unwrap();
        let did2 = generate_did_from_identity(&mut identity2).unwrap();

        assert_eq!(did1.to_string(), did2.to_string());
    }
}