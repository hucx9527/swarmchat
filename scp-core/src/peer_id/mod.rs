//! PeerId module for SwarmChat
//! 
//! This module implements PeerId generation and management for P2P networking.
//! PeerId is a unique identifier for network peers, derived from public keys.

use crate::did::Did;
use sha2::{Sha256, Digest};
use base58::{FromBase58, ToBase58};
use serde::{Deserialize, Serialize};
use std::fmt;

/// PeerId-related errors
#[derive(Debug, thiserror::Error)]
pub enum PeerIdError {
    #[error("Invalid PeerId format: {0}")]
    InvalidFormat(String),
    
    #[error("Invalid multihash: {0}")]
    InvalidMultihash(String),
    
    #[error("Hash generation failed: {0}")]
    HashGeneration(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// PeerId representation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerId {
    /// Multihash prefix (simplified - in full implementation would include codec)
    pub prefix: Vec<u8>,
    
    /// Hash of the public key
    pub hash: Vec<u8>,
    
    /// Full multihash bytes (prefix + hash)
    pub multihash: Vec<u8>,
    
    /// Base58-encoded string representation
    pub encoded: String,
    
    /// Source DID (optional)
    pub source_did: Option<String>,
}

impl PeerId {
    /// Create a new PeerId from a public key
    pub fn from_public_key(public_key: &[u8]) -> Result<Self, PeerIdError> {
        // Generate SHA-256 hash of the public key
        let mut hasher = Sha256::new();
        hasher.update(public_key);
        let hash = hasher.finalize().to_vec();
        
        // Simplified multihash format: [0x12, 0x20] + hash
        // 0x12 = sha2-256, 0x20 = 32 bytes (length)
        let prefix = vec![0x12, 0x20];
        let mut multihash = prefix.clone();
        multihash.extend_from_slice(&hash);
        
        // Base58 encode
        let encoded = multihash.to_base58();
        
        Ok(PeerId {
            prefix,
            hash,
            multihash,
            encoded,
            source_did: None,
        })
    }
    
    /// Create a PeerId from a DID
    pub fn from_did(did: &Did) -> Result<Self, PeerIdError> {
        let peer_id = Self::from_public_key(&did.public_key)?;
        
        Ok(PeerId {
            source_did: Some(did.to_string()),
            ..peer_id
        })
    }
    
    /// Parse a PeerId string into a PeerId struct
    pub fn parse(peer_id_str: &str) -> Result<Self, PeerIdError> {
        // Decode from base58
        let multihash = peer_id_str.from_base58()
            .map_err(|e| PeerIdError::InvalidFormat(format!("Base58 decode error: {:?}", e)))?;
        
        if multihash.len() < 2 {
            return Err(PeerIdError::InvalidFormat(
                "Multihash too short".to_string()
            ));
        }
        
        // Extract prefix (first 2 bytes)
        let prefix = multihash[..2].to_vec();
        
        // Verify prefix indicates SHA-256 (0x12) with 32 bytes (0x20)
        if prefix[0] != 0x12 || prefix[1] != 0x20 {
            return Err(PeerIdError::InvalidMultihash(format!(
                "Unsupported multihash codec/length: {:02x}{:02x}",
                prefix[0], prefix[1]
            )));
        }
        
        // Extract hash (remaining bytes)
        let hash = multihash[2..].to_vec();
        
        if hash.len() != 32 {
            return Err(PeerIdError::InvalidMultihash(format!(
                "Invalid hash length: expected 32, got {}",
                hash.len()
            )));
        }
        
        Ok(PeerId {
            prefix,
            hash,
            multihash: multihash.clone(),
            encoded: peer_id_str.to_string(),
            source_did: None,
        })
    }
    
    /// Convert PeerId to string representation
    pub fn to_string(&self) -> String {
        self.encoded.clone()
    }
    
    /// Get the hash as hex string
    pub fn hash_hex(&self) -> String {
        hex::encode(&self.hash)
    }
    
    /// Verify that this PeerId matches a public key
    pub fn verify_public_key(&self, public_key: &[u8]) -> Result<bool, PeerIdError> {
        let mut hasher = Sha256::new();
        hasher.update(public_key);
        let computed_hash = hasher.finalize().to_vec();
        
        Ok(computed_hash == self.hash)
    }
    
    /// Get the multihash bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        self.multihash.clone()
    }
}

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.encoded)
    }
}

/// Generate a PeerId from an identity (via DID)
pub fn generate_peerid_from_did(did: &Did) -> Result<PeerId, PeerIdError> {
    PeerId::from_did(did)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::did::Did;
    
    #[test]
    fn test_peerid_creation() {
        // Create a test public key
        let public_key = vec![
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
            0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
            0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
            0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
        ];
        
        let peer_id = PeerId::from_public_key(&public_key).unwrap();
        assert_eq!(peer_id.prefix, vec![0x12, 0x20]);
        assert_eq!(peer_id.hash.len(), 32);
        assert_eq!(peer_id.multihash.len(), 34); // 2 + 32
        
        // Test string representation
        let peer_id_str = peer_id.to_string();
        assert!(!peer_id_str.is_empty());
        
        // Test parsing
        let parsed = PeerId::parse(&peer_id_str).unwrap();
        assert_eq!(parsed.hash, peer_id.hash);
        assert_eq!(parsed.multihash, peer_id.multihash);
        
        // Test verification
        assert!(peer_id.verify_public_key(&public_key).unwrap());
    }
    
    #[test]
    fn test_peerid_from_did() {
        let public_key = vec![0u8; 32];
        let did = Did::new(&public_key, "Ed25519").unwrap();
        
        let peer_id = PeerId::from_did(&did).unwrap();
        assert_eq!(peer_id.prefix, vec![0x12, 0x20]);
        assert_eq!(peer_id.hash.len(), 32);
        assert!(peer_id.source_did.is_some());
        assert_eq!(peer_id.source_did.unwrap(), did.to_string());
    }
    
    #[test]
    fn test_peerid_parsing_errors() {
        // Test invalid base58
        assert!(PeerId::parse("invalid!@#").is_err());
        
        // Test too short
        assert!(PeerId::parse("1").is_err());
        
        // Test invalid prefix
        let invalid_multihash = vec![0x00, 0x01]; // Invalid codec
        let invalid_encoded = invalid_multihash.to_base58();
        assert!(PeerId::parse(&invalid_encoded).is_err());
    }
    
    #[test]
    fn test_hash_hex() {
        let public_key = vec![0u8; 32];
        let peer_id = PeerId::from_public_key(&public_key).unwrap();
        
        let hash_hex = peer_id.hash_hex();
        assert_eq!(hash_hex.len(), 64); // 32 bytes * 2 hex chars per byte
        
        // Verify it's valid hex
        hex::decode(&hash_hex).unwrap();
    }
    
    #[test]
    fn test_verify_public_key() {
        let public_key1 = vec![1u8; 32];
        let public_key2 = vec![2u8; 32];
        
        let peer_id = PeerId::from_public_key(&public_key1).unwrap();
        
        // Should verify with correct key
        assert!(peer_id.verify_public_key(&public_key1).unwrap());
        
        // Should not verify with different key
        assert!(!peer_id.verify_public_key(&public_key2).unwrap());
    }
}