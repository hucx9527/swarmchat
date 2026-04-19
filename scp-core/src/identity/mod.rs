//! Simplified Identity module for SwarmChat
//! 
//! Basic BIP39 mnemonic generation and seed derivation

use bip39::Mnemonic;
use rand::Rng;
use std::path::Path;
use std::fs;

/// Simplified Identity structure
#[derive(Debug, Clone)]
pub struct Identity {
    /// BIP39 mnemonic phrase (24 words)
    pub mnemonic: Mnemonic,
    
    /// BIP39 seed derived from mnemonic
    pub seed: Vec<u8>,
}

/// Error types for identity operations
#[derive(Debug, thiserror::Error)]
pub enum IdentityError {
    #[error("Mnemonic generation failed: {0}")]
    MnemonicGeneration(String),
    
    #[error("Seed derivation failed: {0}")]
    SeedDerivation(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl Identity {
    /// Create a new identity with random mnemonic
    pub fn new() -> Result<Self, IdentityError> {
        // Generate 256 bits of entropy (32 bytes) for 24-word mnemonic
        let mut rng = rand::thread_rng();
        let mut entropy = [0u8; 32];
        rng.fill(&mut entropy);
        
        // Create mnemonic from entropy
        let mnemonic = Mnemonic::from_entropy(&entropy)
            .map_err(|e| IdentityError::MnemonicGeneration(e.to_string()))?;
        
        Self::from_mnemonic(&mnemonic)
    }
    
    /// Create identity from existing mnemonic
    pub fn from_mnemonic(mnemonic: &Mnemonic) -> Result<Self, IdentityError> {
        // Derive seed from mnemonic (no passphrase)
        // In bip39 v1.2.0, we need to use to_seed() method
        let seed = mnemonic.to_seed("");
        
        Ok(Identity {
            mnemonic: mnemonic.clone(),
            seed: seed.to_vec(),
        })
    }
    
    /// Get mnemonic phrase as string
    pub fn mnemonic_phrase(&self) -> String {
        self.mnemonic.to_string()
    }
    
    /// Get seed as hex string
    pub fn seed_hex(&self) -> String {
        hex::encode(&self.seed)
    }
    
    /// Save identity to file
    pub fn save_to_file(&self, path: &Path) -> Result<(), IdentityError> {
        let data = serde_json::json!({
            "mnemonic": self.mnemonic_phrase(),
            "seed_hex": self.seed_hex(),
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .to_string(),
        });
        
        fs::write(path, data.to_string())
            .map_err(|e| IdentityError::Io(e))?;
        
        Ok(())
    }
    
    /// Load identity from file
    pub fn load_from_file(path: &Path) -> Result<Self, IdentityError> {
        let data = fs::read_to_string(path)
            .map_err(|e| IdentityError::Io(e))?;
        
        let json: serde_json::Value = serde_json::from_str(&data)
            .map_err(|e| IdentityError::Serialization(e.to_string()))?;
        
        let mnemonic_phrase = json["mnemonic"].as_str()
            .ok_or_else(|| IdentityError::Serialization("Missing mnemonic field".to_string()))?;
        
        let mnemonic = Mnemonic::parse(mnemonic_phrase)
            .map_err(|e| IdentityError::MnemonicGeneration(e.to_string()))?;
        
        Self::from_mnemonic(&mnemonic)
    }
}

/// Generate a new random mnemonic (24 words)
pub fn generate_mnemonic() -> Result<Mnemonic, IdentityError> {
    // Generate 256 bits of entropy (32 bytes) for 24-word mnemonic
    let mut rng = rand::thread_rng();
    let mut entropy = [0u8; 32];
    rng.fill(&mut entropy);
    
    Mnemonic::from_entropy(&entropy)
        .map_err(|e| IdentityError::MnemonicGeneration(e.to_string()))
}

/// Derive seed from mnemonic
pub fn derive_seed(mnemonic: &Mnemonic, passphrase: &str) -> Vec<u8> {
    mnemonic.to_seed(passphrase).to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generate_identity() {
        let identity = Identity::new();
        assert!(identity.is_ok());
        
        let identity = identity.unwrap();
        
        // Check mnemonic has 24 words
        let phrase = identity.mnemonic_phrase();
        let words: Vec<&str> = phrase.split_whitespace().collect();
        assert_eq!(words.len(), 24);
        
        // Check seed is 64 bytes
        assert_eq!(identity.seed.len(), 64);
    }
    
    #[test]
    fn test_from_mnemonic() {
        // Generate a test mnemonic
        let mnemonic = generate_mnemonic().unwrap();
        let phrase = mnemonic.to_string();
        
        // Parse mnemonic
        let parsed_mnemonic = Mnemonic::parse(&phrase).unwrap();
        
        // Create identity from mnemonic
        let identity = Identity::from_mnemonic(&parsed_mnemonic);
        assert!(identity.is_ok());
        
        let identity = identity.unwrap();
        
        // Verify mnemonic matches
        assert_eq!(identity.mnemonic_phrase(), phrase);
    }
    
    #[test]
    fn test_key_derivation_consistency() {
        let mnemonic = generate_mnemonic().unwrap();
        
        // Create two identities from same mnemonic
        let identity1 = Identity::from_mnemonic(&mnemonic).unwrap();
        let identity2 = Identity::from_mnemonic(&mnemonic).unwrap();
        
        // They should have same seed
        assert_eq!(identity1.seed_hex(), identity2.seed_hex());
    }
}