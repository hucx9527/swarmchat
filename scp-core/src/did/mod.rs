//! DID (Decentralized Identifier) module for SwarmChat
//! 
//! This module implements W3C Decentralized Identifiers (DIDs) with the `did:key` method.
//! It provides DID generation, parsing, and DID document creation.

use crate::identity::Identity;
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
    
    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// DID (Decentralized Identifier) representation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Did {
    /// DID method (e.g., "key")
    pub method: String,
    
    /// DID identifier (base58-encoded public key)
    pub identifier: String,
    
    /// Public key bytes (raw)
    pub public_key: Vec<u8>,
    
    /// Key type (e.g., "Ed25519", "X25519")
    pub key_type: String,
}

/// DID Document as per W3C specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidDocument {
    /// The DID itself
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
    /// Create a new DID from a public key
    pub fn new(public_key: &[u8], key_type: &str) -> Result<Self, DidError> {
        // For now, we'll use a simplified approach without multicodec
        // In a full implementation, we would add multicodec prefixes
        let identifier = public_key.to_base58();
        
        Ok(Did {
            method: "key".to_string(),
            identifier: identifier.clone(),
            public_key: public_key.to_vec(),
            key_type: key_type.to_string(),
        })
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
        
        // For did:key, the identifier is the base58-encoded public key
        let identifier = parts[2..].join(":");
        let public_key = identifier.from_base58()
            .map_err(|e| DidError::InvalidMultibase(format!("Base58 decode error: {:?}", e)))?;
        
        // For now, assume Ed25519 keys (32 bytes)
        let key_type = if public_key.len() == 32 {
            "Ed25519".to_string()
        } else {
            "Unknown".to_string()
        };
        
        Ok(Did {
            method,
            identifier,
            public_key,
            key_type,
        })
    }
    
    /// Convert DID to string representation
    pub fn to_string(&self) -> String {
        format!("did:{}:{}", self.method, self.identifier)
    }
    
    /// Generate a DID document for this DID
    pub fn to_document(&self) -> Result<DidDocument, DidError> {
        let did_string = self.to_string();
        let timestamp = chrono::Utc::now().to_rfc3339();
        
        // Create verification method
        let verification_method = VerificationMethod {
            id: format!("{}#{}", did_string, "key-1"),
            vm_type: match self.key_type.as_str() {
                "Ed25519" => "Ed25519VerificationKey2020".to_string(),
                "X25519" => "X25519KeyAgreementKey2020".to_string(),
                _ => "JsonWebKey2020".to_string(),
            },
            controller: did_string.clone(),
            public_key_multibase: format!("z{}", self.identifier), // 'z' indicates base58-btc
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

/// Generate a DID from an identity
pub fn generate_did_from_identity(identity: &Identity, key_type: &str) -> Result<Did, DidError> {
    // Use the seed from the identity as the basis for the public key
    // In a real implementation, we would derive the actual public key from the seed
    let seed = &identity.seed;
    
    // Use first 32 bytes of seed as "public key" for demonstration
    let public_key = if seed.len() >= 32 {
        seed[..32].to_vec()
    } else {
        // Pad with zeros if seed is too short (shouldn't happen with BIP39)
        let mut key = seed.clone();
        key.resize(32, 0);
        key
    };
    
    Did::new(&public_key, key_type)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bip39::Mnemonic;
    use crate::identity::Identity;
    
    #[test]
    fn test_did_creation() {
        // Create a test public key
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
        
        // Test parsing
        let parsed = Did::parse(&did_string).unwrap();
        assert_eq!(parsed.method, "key");
        assert_eq!(parsed.public_key, public_key);
    }
    
    #[test]
    fn test_did_parsing() {
        // Test valid DID
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
    }
    
    #[test]
    fn test_generate_from_identity() {
        // Create a test identity
        let mnemonic_str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let mnemonic = bip39::Mnemonic::parse(mnemonic_str).unwrap();
        let identity = Identity::from_mnemonic(&mnemonic).unwrap();
        
        let did = generate_did_from_identity(&identity, "Ed25519").unwrap();
        assert_eq!(did.method, "key");
        assert_eq!(did.public_key.len(), 32);
    }
}