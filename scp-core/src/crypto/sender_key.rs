//! Sender Key protocol for group messaging
//!
//! Implementation of the Sender Key distribution and ratchet as specified
//! in SCP section 4.5.2. Uses AES-256-GCM with a symmetric ratchet that
//! advances per message. Chains are rotated every 100 messages or on demand.
//!
//! Key concepts:
//! - Each sender has their own Sender Key chain per group
//! - The chain key is distributed to all group members via a signed Distribution Message
//! - Each message advances the ratchet; receivers can skip ahead for out-of-order messages
//! - Chain rotation provides forward secrecy within groups

use aes_gcm::{Aes256Gcm, KeyInit, aead::{Aead, AeadCore, OsRng}};
use hkdf::Hkdf;
use sha2::Sha256;
use std::collections::HashMap;

/// Sender Key message header
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SenderKeyMessageHeader {
    pub chain_id: u32,
    pub iteration: u32,
    pub group_id: Vec<u8>,
}

impl SenderKeyMessageHeader {
    pub fn new(chain_id: u32, iteration: u32, group_id: &[u8]) -> Self {
        Self {
            chain_id,
            iteration,
            group_id: group_id.to_vec(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(8 + self.group_id.len());
        bytes.extend_from_slice(&self.chain_id.to_be_bytes());
        bytes.extend_from_slice(&self.iteration.to_be_bytes());
        bytes.extend_from_slice(&(self.group_id.len() as u16).to_be_bytes());
        bytes.extend_from_slice(&self.group_id);
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SenderKeyError> {
        if bytes.len() < 10 {
            return Err(SenderKeyError::InvalidMessageHeader);
        }

        let chain_id = u32::from_be_bytes(bytes[0..4].try_into().unwrap());
        let iteration = u32::from_be_bytes(bytes[4..8].try_into().unwrap());
        let group_id_len = u16::from_be_bytes(bytes[8..10].try_into().unwrap()) as usize;

        if bytes.len() < 10 + group_id_len {
            return Err(SenderKeyError::InvalidMessageHeader);
        }

        let group_id = bytes[10..10 + group_id_len].to_vec();

        Ok(Self {
            chain_id,
            iteration,
            group_id,
        })
    }
}

/// Sender Key distribution message
/// Sent securely (e.g., via Double Ratchet) to share a chain key with group members.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SenderKeyDistributionMessage {
    pub chain_id: u32,
    pub iteration: u32,
    pub chain_key: [u8; 32],
    pub signing_key: [u8; 32],
    pub signature: Vec<u8>,
}

impl SenderKeyDistributionMessage {
    pub fn new(
        chain_id: u32,
        iteration: u32,
        chain_key: [u8; 32],
        signing_key: [u8; 32],
        signature: Vec<u8>,
    ) -> Self {
        Self {
            chain_id,
            iteration,
            chain_key,
            signing_key,
            signature,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(72 + self.signature.len());
        bytes.extend_from_slice(&self.chain_id.to_be_bytes());
        bytes.extend_from_slice(&self.iteration.to_be_bytes());
        bytes.extend_from_slice(&self.chain_key);
        bytes.extend_from_slice(&self.signing_key);
        bytes.extend_from_slice(&(self.signature.len() as u16).to_be_bytes());
        bytes.extend_from_slice(&self.signature);
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SenderKeyError> {
        if bytes.len() < 74 {
            return Err(SenderKeyError::InvalidDistributionMessage);
        }

        let chain_id = u32::from_be_bytes(bytes[0..4].try_into().unwrap());
        let iteration = u32::from_be_bytes(bytes[4..8].try_into().unwrap());

        let mut chain_key = [0u8; 32];
        chain_key.copy_from_slice(&bytes[8..40]);

        let mut signing_key = [0u8; 32];
        signing_key.copy_from_slice(&bytes[40..72]);

        let sig_len = u16::from_be_bytes(bytes[72..74].try_into().unwrap()) as usize;

        if bytes.len() < 74 + sig_len {
            return Err(SenderKeyError::InvalidDistributionMessage);
        }

        let signature = bytes[74..74 + sig_len].to_vec();

        Ok(Self {
            chain_id,
            iteration,
            chain_key,
            signing_key,
            signature,
        })
    }
}

/// Error type for Sender Key operations
#[derive(Debug)]
pub enum SenderKeyError {
    InvalidKey,
    EncryptionFailed,
    DecryptionFailed,
    InvalidMessageHeader,
    InvalidDistributionMessage,
    ChainNotFound,
}

impl std::fmt::Display for SenderKeyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SenderKeyError::InvalidKey => write!(f, "Invalid key"),
            SenderKeyError::EncryptionFailed => write!(f, "Encryption failed"),
            SenderKeyError::DecryptionFailed => write!(f, "Decryption failed"),
            SenderKeyError::InvalidMessageHeader => write!(f, "Invalid message header"),
            SenderKeyError::InvalidDistributionMessage => write!(f, "Invalid distribution message"),
            SenderKeyError::ChainNotFound => write!(f, "Chain key not found"),
        }
    }
}

impl std::error::Error for SenderKeyError {}

/// Internal representation of a single chain iteration state
#[derive(Debug, Clone)]
struct SenderChain {
    chain_id: u32,
    iteration: u32,
    chain_key: [u8; 32],
    /// Cached message keys for out-of-order delivery
    /// Maps iteration -> message_key
    message_key_cache: HashMap<u32, [u8; 32]>,
}

impl SenderChain {
    fn new(chain_id: u32, chain_key: [u8; 32]) -> Self {
        Self {
            chain_id,
            iteration: 0,
            chain_key,
            message_key_cache: HashMap::new(),
        }
    }

    /// Advance the chain and produce a message key.
    /// Also caches the key for out-of-order access.
    fn ratchet(&mut self) -> [u8; 32] {
        let hkdf = Hkdf::<Sha256>::new(None, &self.chain_key);
        let mut next_chain_key = [0u8; 32];
        let mut message_key = [0u8; 32];

        hkdf.expand(b"SCP-SenderChainKey", &mut next_chain_key)
            .expect("HKDF expansion failed");
        hkdf.expand(b"SCP-SenderMessageKey", &mut message_key)
            .expect("HKDF expansion failed");

        // Cache this message key before advancing
        self.message_key_cache.insert(self.iteration, message_key);

        self.chain_key = next_chain_key;
        self.iteration += 1;

        message_key
    }

    /// Skip ahead to a specific iteration, caching message keys along the way.
    /// Returns the message key for the target iteration.
    fn skip_to(&mut self, target_iteration: u32) -> Option<[u8; 32]> {
        // If we already have the key cached, return it
        if let Some(key) = self.message_key_cache.get(&target_iteration) {
            return Some(*key);
        }

        // Ratchet forward until we reach the target
        while self.iteration <= target_iteration {
            let key = self.ratchet();
            if self.iteration - 1 == target_iteration {
                return Some(key);
            }
        }

        // Should not reach here if target is valid
        None
    }
}

/// Maximum messages per chain before rotation
const MAX_MESSAGES_PER_CHAIN: u32 = 100;

/// Sender Key state for a sender within a group
#[derive(Debug, Clone)]
pub struct SenderKeyState {
    /// Current active chain (for sending)
    current_chain: Option<SenderChain>,
    /// All chains indexed by chain_id (for receiving)
    chains: HashMap<u32, SenderChain>,
    /// Next chain ID to assign
    next_chain_id: u32,
    /// Signing key material for distribution message authentication
    signing_key: [u8; 32],
}

impl SenderKeyState {
    /// Create a new Sender Key state with a given chain ID and signing key
    pub fn new(chain_id: u32, signing_key: [u8; 32]) -> Self {
        let mut initial_chain_key = [0u8; 32];
        // Generate a random initial chain key
        getrandom::getrandom(&mut initial_chain_key)
            .expect("Failed to generate random chain key");

        let chain = SenderChain::new(chain_id, initial_chain_key);
        let mut chains = HashMap::new();
        chains.insert(chain_id, chain.clone());

        Self {
            current_chain: Some(chain),
            chains,
            next_chain_id: chain_id + 1,
            signing_key,
        }
    }

    /// Encrypt a message for the group
    pub fn encrypt(
        &mut self,
        group_id: &[u8],
        plaintext: &[u8],
    ) -> Result<(SenderKeyMessageHeader, Vec<u8>), SenderKeyError> {
        let chain = self.current_chain.as_mut()
            .ok_or(SenderKeyError::ChainNotFound)?;

        // Check if we need to rotate (every 100 messages)
        if chain.iteration >= MAX_MESSAGES_PER_CHAIN {
            self.rotate_chain();
        }

        let chain = self.current_chain.as_mut()
            .ok_or(SenderKeyError::ChainNotFound)?;

        // Derive message key from chain
        let message_key = chain.ratchet();

        // Create header
        let header = SenderKeyMessageHeader::new(chain.chain_id, chain.iteration - 1, group_id);

        // Encrypt with AES-256-GCM
        let cipher = Aes256Gcm::new_from_slice(&message_key)
            .map_err(|_| SenderKeyError::InvalidKey)?;

        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = cipher.encrypt(&nonce, plaintext)
            .map_err(|_| SenderKeyError::EncryptionFailed)?;

        // Prepend nonce
        let mut encrypted_message = nonce.to_vec();
        encrypted_message.extend_from_slice(&ciphertext);

        Ok((header, encrypted_message))
    }

    /// Decrypt a message from the group.
    /// Supports out-of-order delivery via message key caching.
    pub fn decrypt(
        &mut self,
        header: &SenderKeyMessageHeader,
        encrypted_message: &[u8],
    ) -> Result<Vec<u8>, SenderKeyError> {
        if encrypted_message.len() < 12 {
            return Err(SenderKeyError::DecryptionFailed);
        }

        // Get or initialize the chain for this chain_id
        if !self.chains.contains_key(&header.chain_id) {
            return Err(SenderKeyError::ChainNotFound);
        }

        // Clone the chain and skip to the correct iteration
        let chain = self.chains.get(&header.chain_id).unwrap();
        let mut chain = chain.clone();

        // Use skip_to to get the message key (handles caching for out-of-order)
        let message_key = chain.skip_to(header.iteration)
            .ok_or(SenderKeyError::ChainNotFound)?;

        // Update the stored chain
        self.chains.insert(header.chain_id, chain);

        // Decrypt
        let nonce = &encrypted_message[0..12];
        let ciphertext = &encrypted_message[12..];

        let cipher = Aes256Gcm::new_from_slice(&message_key)
            .map_err(|_| SenderKeyError::InvalidKey)?;

        let nonce_array: [u8; 12] = nonce.try_into()
            .map_err(|_| SenderKeyError::DecryptionFailed)?;
        let nonce = aes_gcm::Nonce::from_slice(&nonce_array);

        cipher.decrypt(nonce, ciphertext)
            .map_err(|_| SenderKeyError::DecryptionFailed)
    }

    /// Create a distribution message to share the current chain key with new members
    pub fn create_distribution_message(
        &self,
        chain_id: u32,
    ) -> Result<SenderKeyDistributionMessage, SenderKeyError> {
        let chain = self.chains.get(&chain_id)
            .ok_or(SenderKeyError::ChainNotFound)?;

        // In production, the signature would be computed using Ed25519 over
        // (chain_id || iteration || chain_key || signing_key).
        // For now, we use a simple placeholder signature.
        let signature = vec![0u8; 64]; // Placeholder Ed25519 signature

        Ok(SenderKeyDistributionMessage {
            chain_id: chain.chain_id,
            iteration: chain.iteration,
            chain_key: chain.chain_key,
            signing_key: self.signing_key,
            signature,
        })
    }

    /// Process a distribution message received from a sender
    pub fn process_distribution_message(
        &mut self,
        message: &SenderKeyDistributionMessage,
    ) -> Result<(), SenderKeyError> {
        let chain = SenderChain::new(message.chain_id, message.chain_key);

        // If the distribution message has a non-zero iteration, catch up the chain
        let mut chain = chain;
        for _ in 0..message.iteration {
            let _ = chain.ratchet();
        }

        self.chains.insert(message.chain_id, chain);

        // Update next_chain_id if needed
        if message.chain_id >= self.next_chain_id {
            self.next_chain_id = message.chain_id + 1;
        }

        Ok(())
    }

    /// Rotate to a new chain (for forward secrecy).
    /// Returns the new chain ID.
    pub fn rotate_chain(&mut self) -> u32 {
        let new_id = self.next_chain_id;
        self.next_chain_id += 1;

        // Generate a new random chain key
        let mut new_chain_key = [0u8; 32];
        getrandom::getrandom(&mut new_chain_key)
            .expect("Failed to generate random chain key");

        let new_chain = SenderChain::new(new_id, new_chain_key);
        self.chains.insert(new_id, new_chain.clone());
        self.current_chain = Some(new_chain);

        new_id
    }

    /// Get the current chain ID
    pub fn current_chain_id(&self) -> u32 {
        self.current_chain.as_ref()
            .map(|c| c.chain_id)
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sender_key_basic() {
        println!("Testing basic Sender Key operations...");

        // Create Sender Key state for Alice
        let alice_signing_key = [0x01u8; 32];
        let mut alice_state = SenderKeyState::new(1, alice_signing_key);

        let group_id = b"test-group";

        // Alice encrypts a message
        let plaintext1 = b"Hello group!";
        let (header1, encrypted1) = alice_state.encrypt(group_id, plaintext1).unwrap();

        println!("Alice encrypted message 1, iteration: {}", header1.iteration);

        // Alice encrypts another message
        let plaintext2 = b"Second message";
        let (header2, encrypted2) = alice_state.encrypt(group_id, plaintext2).unwrap();

        println!("Alice encrypted message 2, iteration: {}", header2.iteration);

        // Create a receiver state (Bob) with the same chain
        let mut bob_state = SenderKeyState::new(1, alice_signing_key);

        // Bob processes distribution message (simulating key sharing)
        let dist_message = alice_state.create_distribution_message(1).unwrap();
        bob_state.process_distribution_message(&dist_message).unwrap();

        // Bob decrypts messages (out-of-order to test caching)
        println!("\nBob decrypting messages...");

        // Decrypt second message first
        let decrypted2 = bob_state.decrypt(&header2, &encrypted2).unwrap();
        assert_eq!(decrypted2, plaintext2);
        println!("✓ Decrypted message 2: {}", String::from_utf8_lossy(&decrypted2));

        // Decrypt first message (out-of-order)
        let decrypted1 = bob_state.decrypt(&header1, &encrypted1).unwrap();
        assert_eq!(decrypted1, plaintext1);
        println!("✓ Decrypted message 1: {}", String::from_utf8_lossy(&decrypted1));

        println!("\nBasic Sender Key test passed!");
    }

    #[test]
    fn test_message_header_serialization() {
        let group_id = b"test-group-123";
        let header = SenderKeyMessageHeader::new(42, 100, group_id);

        let bytes = header.to_bytes();
        let deserialized = SenderKeyMessageHeader::from_bytes(&bytes).unwrap();

        assert_eq!(header, deserialized);
        assert_eq!(deserialized.chain_id, 42);
        assert_eq!(deserialized.iteration, 100);
        assert_eq!(deserialized.group_id, group_id);

        println!("Message header serialization test passed!");
    }

    #[test]
    fn test_distribution_message_serialization() {
        let chain_id = 1;
        let iteration = 0;
        let chain_key = [0x42u8; 32];
        let signing_key = [0x01u8; 32];
        let signature = vec![0x99u8; 64];

        let message = SenderKeyDistributionMessage::new(
            chain_id,
            iteration,
            chain_key,
            signing_key,
            signature.clone(),
        );

        let bytes = message.to_bytes();
        let deserialized = SenderKeyDistributionMessage::from_bytes(&bytes).unwrap();

        assert_eq!(deserialized.chain_id, chain_id);
        assert_eq!(deserialized.iteration, iteration);
        assert_eq!(deserialized.chain_key, chain_key);
        assert_eq!(deserialized.signing_key, signing_key);
        assert_eq!(deserialized.signature, signature);

        println!("Distribution message serialization test passed!");
    }

    #[test]
    fn test_chain_rotation() {
        let signing_key = [0x01u8; 32];
        let mut state = SenderKeyState::new(1, signing_key);

        assert_eq!(state.current_chain_id(), 1);

        // Rotate chain
        let new_chain_id = state.rotate_chain();
        assert_eq!(new_chain_id, 2);
        assert_eq!(state.current_chain_id(), 2);

        println!("Chain rotation test passed!");
    }

    #[test]
    fn test_multiple_messages() {
        let signing_key = [0x01u8; 32];
        let mut state = SenderKeyState::new(1, signing_key);
        let group_id = b"test-group";

        // Send multiple messages
        let messages = [
            b"Message 1",
            b"Message 2",
            b"Message 3",
            b"Message 4",
            b"Message 5",
        ];

        let mut encrypted_messages = Vec::new();

        for (i, message) in messages.iter().enumerate() {
            let (header, encrypted) = state.encrypt(group_id, *message).unwrap();
            let iteration = header.iteration;
            encrypted_messages.push((header, encrypted));
            println!("Encrypted message {} at iteration {}", i + 1, iteration);
        }

        // Create receiver and process distribution
        let mut receiver_state = SenderKeyState::new(1, signing_key);
        let dist_message = state.create_distribution_message(1).unwrap();
        receiver_state.process_distribution_message(&dist_message).unwrap();

        // Decrypt all messages
        for (i, (header, encrypted)) in encrypted_messages.iter().enumerate() {
            let decrypted = receiver_state.decrypt(header, encrypted).unwrap();
            assert_eq!(decrypted, messages[i]);
            println!("✓ Decrypted message {}: {}", i + 1, String::from_utf8_lossy(&decrypted));
        }

        println!("Multiple messages test passed!");
    }
}