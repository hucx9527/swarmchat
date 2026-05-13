//! Identity module for SwarmChat - SCP Protocol
//!
//! Implements BIP39 mnemonic generation, seed derivation, and
//! deterministic key derivation for Ed25519 (signing) and X25519 (encryption) keys.
//! Based on SCP Specification section 4.3.

use serde::{Deserialize, Serialize};

use bip39::Mnemonic;
use hkdf::Hkdf;
use sha2::Sha256;
use ed25519_dalek::{SigningKey, VerifyingKey};
use x25519_dalek::{StaticSecret as X25519Secret, PublicKey as X25519Public};
use rand::Rng;
use std::path::Path;
use std::fs;

/// Full key material derived from an identity
#[derive(Clone)]
pub struct DerivedKeys {
    /// Ed25519 signing key pair
    pub signing_key: SigningKey,
    /// Ed25519 verifying key (public)
    pub verifying_key: VerifyingKey,
    /// X25519 encryption private key
    pub encryption_key: X25519Secret,
    /// X25519 encryption public key
    pub encryption_public: X25519Public,
    /// Raw seed bytes (64 bytes)
    pub seed: [u8; 64],
}

impl std::fmt::Debug for DerivedKeys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DerivedKeys")
            .field("signing_key", &self.signing_key)
            .field("verifying_key", &self.verifying_key)
            .field("encryption_key", &"<redacted>")
            .field("encryption_public", &self.encryption_public)
            .field("seed", &self.seed)
            .finish()
    }
}

/// Simplified Identity structure
#[derive(Debug, Clone)]
pub struct Identity {
    /// BIP39 mnemonic phrase (24 words)
    pub mnemonic: Mnemonic,

    /// BIP39 seed derived from mnemonic (64 bytes)
    pub seed: Vec<u8>,

    /// Cached derived keys (lazy, computed on first access)
    derived_keys: Option<Box<DerivedKeys>>,
}

impl Serialize for Identity {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Identity", 3)?;
        state.serialize_field("mnemonic", &self.mnemonic_phrase())?;
        state.serialize_field("seed_hex", &self.seed_hex())?;
        state.serialize_field("derived_keys", &"<omitted>")?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Identity {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct IdentityData {
            mnemonic: String,
        }

        let data = IdentityData::deserialize(deserializer)?;
        let mnemonic = Mnemonic::parse(&data.mnemonic)
            .map_err(serde::de::Error::custom)?;
        Identity::from_mnemonic(&mnemonic)
            .map_err(serde::de::Error::custom)
    }
}

/// Error types for identity operations
#[derive(Debug, thiserror::Error)]
pub enum IdentityError {
    #[error("Mnemonic generation failed: {0}")]
    MnemonicGeneration(String),

    #[error("Seed derivation failed: {0}")]
    SeedDerivation(String),

    #[error("Key derivation failed: {0}")]
    KeyDerivation(String),

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
        let seed = mnemonic.to_seed("");

        Ok(Identity {
            mnemonic: mnemonic.clone(),
            seed: seed.to_vec(),
            derived_keys: None,
        })
    }

    /// Derive all cryptographic keys from the BIP39 seed.
    /// Uses HKDF-SHA256 with domain-separated info strings.
    pub fn derive_keys(&mut self) -> Result<DerivedKeys, IdentityError> {
        if let Some(ref keys) = self.derived_keys {
            return Ok(*keys.clone());
        }

        let seed: [u8; 64] = self.seed.as_slice().try_into()
            .map_err(|_| IdentityError::KeyDerivation("Invalid seed length".to_string()))?;

        // Master HKDF: IKM = seed, salt = None
        let hkdf = Hkdf::<Sha256>::new(None, &seed);

        // Derive Ed25519 signing key (32 bytes)
        let mut signing_seed = [0u8; 32];
        hkdf.expand(b"SCP-Identity-Ed25519-SigningKey", &mut signing_seed)
            .map_err(|_| IdentityError::KeyDerivation("HKDF expand failed for Ed25519".to_string()))?;

        // Derive X25519 encryption key (32 bytes)
        let mut encryption_seed = [0u8; 32];
        hkdf.expand(b"SCP-Identity-X25519-EncryptionKey", &mut encryption_seed)
            .map_err(|_| IdentityError::KeyDerivation("HKDF expand failed for X25519".to_string()))?;

        // Build Ed25519 key pair
        let signing_key = SigningKey::from_bytes(&signing_seed);
        let verifying_key = signing_key.verifying_key();

        // Build X25519 key pair
        let encryption_key = X25519Secret::from(encryption_seed);
        let encryption_public = X25519Public::from(&encryption_key);

        let keys = DerivedKeys {
            signing_key,
            verifying_key,
            encryption_key,
            encryption_public,
            seed,
        };

        self.derived_keys = Some(Box::new(keys.clone()));
        Ok(keys)
    }

    /// Get the Ed25519 verifying key (public key) for this identity
    pub fn public_signing_key(&mut self) -> Result<VerifyingKey, IdentityError> {
        let keys = self.derive_keys()?;
        Ok(keys.verifying_key)
    }

    /// Get the X25519 public key for this identity
    pub fn public_encryption_key(&mut self) -> Result<X25519Public, IdentityError> {
        let keys = self.derive_keys()?;
        Ok(keys.encryption_public)
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

/// Generate a new random mnemonic (24 words, 256-bit entropy)
pub fn generate_mnemonic() -> Result<Mnemonic, IdentityError> {
    let mut rng = rand::thread_rng();
    let mut entropy = [0u8; 32];
    rng.fill(&mut entropy);

    Mnemonic::from_entropy(&entropy)
        .map_err(|e| IdentityError::MnemonicGeneration(e.to_string()))
}

/// Derive seed from mnemonic with optional passphrase
pub fn derive_seed(mnemonic: &Mnemonic, passphrase: &str) -> Vec<u8> {
    mnemonic.to_seed(passphrase).to_vec()
}

/// Derive Ed25519 and X25519 keys from a BIP39 seed.
/// This is the primary key derivation function for SCP.
pub fn derive_keys(seed: &[u8]) -> Result<DerivedKeys, IdentityError> {
    let seed_array: [u8; 64] = seed.try_into()
        .map_err(|_| IdentityError::KeyDerivation("Seed must be exactly 64 bytes".to_string()))?;

    // Master HKDF
    let hkdf = Hkdf::<Sha256>::new(None, &seed_array);

    // Ed25519 signing key
    let mut signing_seed = [0u8; 32];
    hkdf.expand(b"SCP-Identity-Ed25519-SigningKey", &mut signing_seed)
        .map_err(|_| IdentityError::KeyDerivation("HKDF expand failed for Ed25519".to_string()))?;

    // X25519 encryption key
    let mut encryption_seed = [0u8; 32];
    hkdf.expand(b"SCP-Identity-X25519-EncryptionKey", &mut encryption_seed)
        .map_err(|_| IdentityError::KeyDerivation("HKDF expand failed for X25519".to_string()))?;

    let signing_key = SigningKey::from_bytes(&signing_seed);
    let verifying_key = signing_key.verifying_key();
    let encryption_key = X25519Secret::from(encryption_seed);
    let encryption_public = X25519Public::from(&encryption_key);

    Ok(DerivedKeys {
        signing_key,
        verifying_key,
        encryption_key,
        encryption_public,
        seed: seed_array,
    })
}

/// Generate a new random identity with full key derivation
pub fn generate_identity_with_keys() -> Result<(Identity, DerivedKeys), IdentityError> {
    let mut identity = Identity::new()?;
    let keys = identity.derive_keys()?;
    Ok((identity, keys))
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
        let mnemonic = generate_mnemonic().unwrap();
        let phrase = mnemonic.to_string();

        let parsed_mnemonic = Mnemonic::parse(&phrase).unwrap();
        let identity = Identity::from_mnemonic(&parsed_mnemonic);
        assert!(identity.is_ok());

        let identity = identity.unwrap();
        assert_eq!(identity.mnemonic_phrase(), phrase);
    }

    #[test]
    fn test_key_derivation_consistency() {
        let mnemonic = generate_mnemonic().unwrap();

        let identity1 = Identity::from_mnemonic(&mnemonic).unwrap();
        let identity2 = Identity::from_mnemonic(&mnemonic).unwrap();

        // Same mnemonic → same seed
        assert_eq!(identity1.seed_hex(), identity2.seed_hex());
    }

    #[test]
    fn test_derive_keys_from_seed() {
        let seed = [0x42u8; 64];
        let keys = derive_keys(&seed).unwrap();

        // Verify keys are not all zeros
        assert_ne!(keys.signing_key.to_bytes(), [0u8; 32]);
        assert_ne!(keys.encryption_key.to_bytes(), [0u8; 32]);

        // Verify public keys can be derived
        let signing_pub = keys.verifying_key;
        assert_eq!(signing_pub.as_bytes().len(), 32);

        let enc_pub = keys.encryption_public;
        assert_eq!(enc_pub.as_bytes().len(), 32);
    }

    #[test]
    fn test_derive_keys_deterministic() {
        let seed = [0x42u8; 64];
        let keys1 = derive_keys(&seed).unwrap();
        let keys2 = derive_keys(&seed).unwrap();

        // Same seed must produce identical keys
        assert_eq!(keys1.signing_key.to_bytes(), keys2.signing_key.to_bytes());
        assert_eq!(keys1.encryption_key.to_bytes(), keys2.encryption_key.to_bytes());
        assert_eq!(keys1.verifying_key, keys2.verifying_key);
        assert_eq!(keys1.encryption_public, keys2.encryption_public);
    }

    #[test]
    fn test_different_seeds_different_keys() {
        let seed1 = [0x42u8; 64];
        let seed2 = [0x99u8; 64];
        let keys1 = derive_keys(&seed1).unwrap();
        let keys2 = derive_keys(&seed2).unwrap();

        assert_ne!(keys1.signing_key.to_bytes(), keys2.signing_key.to_bytes());
        assert_ne!(keys1.encryption_key.to_bytes(), keys2.encryption_key.to_bytes());
    }

    #[test]
    fn test_identity_derive_keys() {
        let mut identity = Identity::new().unwrap();
        let keys = identity.derive_keys().unwrap();

        // Keys should be derived successfully
        assert_eq!(identity.seed.len(), 64);
        assert_eq!(keys.seed.len(), 64);

        // Second call should return cached keys
        let keys2 = identity.derive_keys().unwrap();
        assert_eq!(keys.signing_key.to_bytes(), keys2.signing_key.to_bytes());
    }

    #[test]
    fn test_generate_identity_with_keys() {
        let (identity, keys) = generate_identity_with_keys().unwrap();

        assert_eq!(identity.seed.len(), 64);
        assert_eq!(keys.seed.len(), 64);
        assert_eq!(identity.seed.as_slice(), &keys.seed[..]);
    }
}