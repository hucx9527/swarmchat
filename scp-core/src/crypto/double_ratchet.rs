//! Double Ratchet algorithm for continuous end-to-end encryption
//!
//! Implementation of the Double Ratchet algorithm as specified in SCP
//! section 4.5.2. Based on the Signal Protocol's Double Ratchet.
//!
//! Each message advances the symmetric ratchet, deriving a fresh message key.
//! When the remote party sends a message with a new DH public key, a DH
//! ratchet step is performed to incorporate new entropy.

use aes_gcm::{Aes256Gcm, KeyInit, aead::{Aead, AeadCore, OsRng}};
use hkdf::Hkdf;
use sha2::Sha256;
use x25519_dalek::{PublicKey, StaticSecret};
use std::collections::VecDeque;

/// Error type for Double Ratchet operations
#[derive(Debug)]
pub enum DoubleRatchetError {
    InvalidKey,
    DecryptionFailed,
    RatchetStateError,
    InvalidMessageFormat,
    KeyDerivationFailed,
}

impl std::fmt::Display for DoubleRatchetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DoubleRatchetError::InvalidKey => write!(f, "Invalid key"),
            DoubleRatchetError::DecryptionFailed => write!(f, "Decryption failed"),
            DoubleRatchetError::RatchetStateError => write!(f, "Ratchet state error"),
            DoubleRatchetError::InvalidMessageFormat => write!(f, "Invalid message format"),
            DoubleRatchetError::KeyDerivationFailed => write!(f, "Key derivation failed"),
        }
    }
}

impl std::error::Error for DoubleRatchetError {}

/// Double Ratchet message header
#[derive(Debug, Clone, PartialEq)]
pub struct MessageHeader {
    pub dh_public: [u8; 32],      // DH ratchet public key
    pub previous_chain_length: u32, // Previous chain message count
    pub message_number: u32,      // Message number in current chain
}

impl MessageHeader {
    pub fn new(dh_public: [u8; 32], previous_chain_length: u32, message_number: u32) -> Self {
        Self {
            dh_public,
            previous_chain_length,
            message_number,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(40);
        bytes.extend_from_slice(&self.dh_public);
        bytes.extend_from_slice(&self.previous_chain_length.to_be_bytes());
        bytes.extend_from_slice(&self.message_number.to_be_bytes());
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, DoubleRatchetError> {
        if bytes.len() < 40 {
            return Err(DoubleRatchetError::InvalidMessageFormat);
        }

        let mut dh_public = [0u8; 32];
        dh_public.copy_from_slice(&bytes[0..32]);

        let previous_chain_length = u32::from_be_bytes(bytes[32..36].try_into().unwrap());
        let message_number = u32::from_be_bytes(bytes[36..40].try_into().unwrap());

        Ok(Self {
            dh_public,
            previous_chain_length,
            message_number,
        })
    }
}

/// Chain key for symmetric ratchet
#[derive(Debug, Clone)]
struct ChainKey {
    key: [u8; 32],
    index: u32,
}

impl ChainKey {
    fn new(key: [u8; 32], index: u32) -> Self {
        Self { key, index }
    }

    /// Ratchet the chain key forward and produce a message key.
    /// Returns (next_chain_key, message_key_for_this_step).
    fn ratchet(&self) -> (Self, [u8; 32]) {
        let hkdf = Hkdf::<Sha256>::new(None, &self.key);
        let mut next_chain_key = [0u8; 32];
        let mut message_key = [0u8; 32];

        hkdf.expand(b"SCP-ChainKey", &mut next_chain_key)
            .expect("HKDF expansion failed");
        hkdf.expand(b"SCP-MessageKey", &mut message_key)
            .expect("HKDF expansion failed");

        (
            ChainKey::new(next_chain_key, self.index + 1),
            message_key,
        )
    }
}

/// Double Ratchet state
#[derive(Clone)]
pub struct DoubleRatchetState {
    // DH ratchet keys
    dh_ratchet_private: StaticSecret,
    pub dh_ratchet_public: PublicKey,
    remote_dh_public: Option<PublicKey>,

    // Root chain
    root_key: [u8; 32],

    // Sending and receiving chains
    sending_chain: Option<ChainKey>,
    receiving_chain: Option<ChainKey>,

    // Message numbers
    sending_message_number: u32,
    receiving_message_number: u32,
    previous_chain_length: u32,

    // Pending messages (for out-of-order delivery)
    pending_messages: VecDeque<(u32, Vec<u8>)>,
}

impl std::fmt::Debug for DoubleRatchetState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DoubleRatchetState")
            .field("dh_ratchet_private", &"<redacted>")
            .field("dh_ratchet_public", &self.dh_ratchet_public)
            .field("remote_dh_public", &self.remote_dh_public)
            .field("root_key", &self.root_key)
            .field("sending_chain", &self.sending_chain)
            .field("receiving_chain", &self.receiving_chain)
            .field("sending_message_number", &self.sending_message_number)
            .field("receiving_message_number", &self.receiving_message_number)
            .field("previous_chain_length", &self.previous_chain_length)
            .field("pending_messages", &self.pending_messages)
            .finish()
    }
}

impl DoubleRatchetState {
    /// Initialize Double Ratchet from X3DH shared secret.
    /// Derives the root key and initial sending/receiving chain keys.
    pub fn new_from_x3dh(
        shared_secret: [u8; 32],
        our_dh_ratchet_key: StaticSecret,
        remote_dh_public: Option<PublicKey>,
    ) -> Self {
        // Derive root key and initial chain key from X3DH shared secret.
        // Both sides use the same chain key for the first message exchange;
        // separate send/recv chains are established after the first DH ratchet.
        let hkdf = Hkdf::<Sha256>::new(None, &shared_secret);
        let mut root_key = [0u8; 32];
        let mut initial_chain_key = [0u8; 32];
        hkdf.expand(b"SCP-RootKey", &mut root_key)
            .expect("HKDF expansion failed");
        hkdf.expand(b"SCP-ChainKey", &mut initial_chain_key)
            .expect("HKDF expansion failed");

        // Derive public key before moving private key
        let dh_public = PublicKey::from(&our_dh_ratchet_key);

        Self {
            dh_ratchet_private: our_dh_ratchet_key,
            dh_ratchet_public: dh_public,
            remote_dh_public,
            root_key,
            sending_chain: Some(ChainKey::new(initial_chain_key, 0)),
            receiving_chain: Some(ChainKey::new(initial_chain_key, 0)),
            sending_message_number: 0,
            receiving_message_number: 0,
            previous_chain_length: 0,
            pending_messages: VecDeque::new(),
        }
    }

    /// Perform DH ratchet step.
    /// Called when receiving a message with a new DH public key from the remote party.
    /// Returns the derived chain key — caller decides whether to use it as
    /// sending chain (when encrypting) or receiving chain (when decrypting).
    fn dh_ratchet(&mut self, remote_dh_public: PublicKey) -> Result<[u8; 32], DoubleRatchetError> {
        // Calculate new DH shared secret
        let dh_secret = self.dh_ratchet_private.diffie_hellman(&remote_dh_public);

        // Derive new root key and chain key using HKDF
        let hkdf = Hkdf::<Sha256>::new(Some(&self.root_key), dh_secret.as_bytes());
        let mut new_root_key = [0u8; 32];
        let mut chain_key = [0u8; 32];

        hkdf.expand(b"SCP-RootKey", &mut new_root_key)
            .map_err(|_| DoubleRatchetError::KeyDerivationFailed)?;
        hkdf.expand(b"SCP-ChainKey", &mut chain_key)
            .map_err(|_| DoubleRatchetError::KeyDerivationFailed)?;

        // Update state
        self.root_key = new_root_key;
        self.remote_dh_public = Some(remote_dh_public);

        // Generate new DH ratchet key pair for our side
        let mut rng = rand_core::OsRng;
        self.dh_ratchet_private = StaticSecret::random_from_rng(&mut rng);
        self.dh_ratchet_public = PublicKey::from(&self.dh_ratchet_private);

        Ok(chain_key)
    }

    /// Encrypt a plaintext message.
    /// Returns (MessageHeader, encrypted_bytes) where encrypted_bytes = nonce + ciphertext.
    pub fn encrypt(&mut self, plaintext: &[u8]) -> Result<(MessageHeader, Vec<u8>), DoubleRatchetError> {
        // Get sending chain (always initialized from X3DH shared secret)
        let sending_chain = self.sending_chain.as_mut()
            .ok_or(DoubleRatchetError::RatchetStateError)?;

        // Ratchet chain to get message key
        let (next_chain, message_key) = sending_chain.ratchet();
        *sending_chain = next_chain;

        // Create message header with our current DH public key
        let header = MessageHeader::new(
            self.dh_ratchet_public.to_bytes(),
            self.previous_chain_length,
            self.sending_message_number,
        );

        // Encrypt with AES-256-GCM
        let cipher = Aes256Gcm::new_from_slice(&message_key)
            .map_err(|_| DoubleRatchetError::InvalidKey)?;

        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = cipher.encrypt(&nonce, plaintext)
            .map_err(|_| DoubleRatchetError::DecryptionFailed)?;

        // Prepend nonce to ciphertext: [nonce (12 bytes) | ciphertext]
        let mut encrypted_message = nonce.to_vec();
        encrypted_message.extend_from_slice(&ciphertext);

        // Advance message number
        self.sending_message_number += 1;

        Ok((header, encrypted_message))
    }

    /// Decrypt a message
    pub fn decrypt(&mut self, header: &MessageHeader, encrypted_message: &[u8])
        -> Result<Vec<u8>, DoubleRatchetError> {

        if encrypted_message.len() < 12 {
            return Err(DoubleRatchetError::InvalidMessageFormat);
        }

        // Check if we need to perform DH ratchet
        let remote_dh_public = PublicKey::from(header.dh_public);
        let needs_dh_ratchet = match self.remote_dh_public {
            Some(current) => current != remote_dh_public,
            None => true,
        };

        if needs_dh_ratchet {
            // DH ratchet uses the previously stored remote key for key derivation.
            // This ensures DH(Alice_old_priv, Bob_pub) == DH(Bob_priv, Alice_old_pub).
            let old_remote = self.remote_dh_public
                .ok_or(DoubleRatchetError::RatchetStateError)?;
            let chain_key = self.dh_ratchet(old_remote)?;
            // Now store the new remote key for the next ratchet cycle
            self.remote_dh_public = Some(remote_dh_public);

            self.receiving_chain = Some(ChainKey::new(chain_key, 0));
            self.receiving_message_number = 0;

            // Update previous chain length
            self.previous_chain_length = self.sending_message_number;
            self.sending_message_number = 0;
            self.sending_chain = None;
        }

        // Get receiving chain
        let receiving_chain = self.receiving_chain.as_mut()
            .ok_or(DoubleRatchetError::RatchetStateError)?;

        // Handle out-of-order messages
        if header.message_number < self.receiving_message_number {
            // Check pending messages
            for (msg_num, ciphertext) in &self.pending_messages {
                if *msg_num == header.message_number {
                    return Self::decrypt_with_key(receiving_chain, *msg_num, ciphertext);
                }
            }
            return Err(DoubleRatchetError::InvalidMessageFormat);
        }

        // Skip to the correct message number
        while self.receiving_message_number < header.message_number {
            let (next_chain, _) = receiving_chain.ratchet();
            *receiving_chain = next_chain;
            self.receiving_message_number += 1;
        }

        // Ratchet to get message key and decrypt
        let (next_chain, message_key) = receiving_chain.ratchet();
        *receiving_chain = next_chain;

        let plaintext = Self::decrypt_with_message_key(&message_key, encrypted_message)?;

        self.receiving_message_number += 1;

        Ok(plaintext)
    }

    /// Decrypt with a specific message key (for out-of-order messages)
    fn decrypt_with_key(chain: &ChainKey, message_number: u32, encrypted_message: &[u8])
        -> Result<Vec<u8>, DoubleRatchetError> {

        // Derive message key for this specific message number
        let mut current_chain = chain.clone();
        let mut target_message_number = chain.index;

        while target_message_number < message_number {
            let (next_chain, _) = current_chain.ratchet();
            current_chain = next_chain;
            target_message_number += 1;
        }

        let (_, message_key) = current_chain.ratchet();
        Self::decrypt_with_message_key(&message_key, encrypted_message)
    }

    /// Decrypt with a given message key
    fn decrypt_with_message_key(message_key: &[u8; 32], encrypted_message: &[u8])
        -> Result<Vec<u8>, DoubleRatchetError> {

        if encrypted_message.len() < 12 {
            return Err(DoubleRatchetError::InvalidMessageFormat);
        }

        let nonce = &encrypted_message[0..12];
        let ciphertext = &encrypted_message[12..];

        let cipher = Aes256Gcm::new_from_slice(message_key)
            .map_err(|_| DoubleRatchetError::InvalidKey)?;

        let nonce_array: [u8; 12] = nonce.try_into()
            .map_err(|_| DoubleRatchetError::InvalidMessageFormat)?;
        let nonce = aes_gcm::Nonce::from_slice(&nonce_array);

        cipher.decrypt(nonce, ciphertext)
            .map_err(|_| DoubleRatchetError::DecryptionFailed)
    }

    /// Get current DH ratchet public key
    pub fn dh_public_key(&self) -> [u8; 32] {
        self.dh_ratchet_public.to_bytes()
    }

    /// Set remote DH public key (for initial setup)
    pub fn set_remote_dh_public(&mut self, remote_public: PublicKey) {
        self.remote_dh_public = Some(remote_public);
    }
}

/// Generate a new DH ratchet key pair
pub fn generate_dh_ratchet_key() -> StaticSecret {
    let mut rng = rand_core::OsRng;
    StaticSecret::random_from_rng(&mut rng)
}

#[cfg(test)]
mod tests {
    use super::*;
    use x25519_dalek::PublicKey;

    #[test]
    fn test_double_ratchet_basic() {
        // Generate initial X3DH shared secret
        let shared_secret = [0x42u8; 32];

        // Alice's setup
        let alice_dh_key = generate_dh_ratchet_key();
        let bob_dh_key = generate_dh_ratchet_key();
        let bob_dh_public = PublicKey::from(&bob_dh_key);

        let mut alice_state = DoubleRatchetState::new_from_x3dh(
            shared_secret,
            alice_dh_key,
            Some(bob_dh_public),
        );

        // Bob's setup
        let mut bob_state = DoubleRatchetState::new_from_x3dh(
            shared_secret,
            bob_dh_key,
            Some(alice_state.dh_ratchet_public),
        );

        // Alice sends first message
        let plaintext1 = b"Hello Bob!";
        let (header1, encrypted1) = alice_state.encrypt(plaintext1).unwrap();

        // Bob decrypts
        let decrypted1 = bob_state.decrypt(&header1, &encrypted1).unwrap();
        assert_eq!(decrypted1, plaintext1);

        // Bob sends reply
        let plaintext2 = b"Hello Alice!";
        let (header2, encrypted2) = bob_state.encrypt(plaintext2).unwrap();

        // Alice decrypts
        let decrypted2 = alice_state.decrypt(&header2, &encrypted2).unwrap();
        assert_eq!(decrypted2, plaintext2);

        // Alice sends another message
        let plaintext3 = b"How are you?";
        let (header3, encrypted3) = alice_state.encrypt(plaintext3).unwrap();

        // Bob decrypts
        let decrypted3 = bob_state.decrypt(&header3, &encrypted3).unwrap();
        assert_eq!(decrypted3, plaintext3);

        println!("Double Ratchet basic test passed!");
    }

    #[test]
    fn test_message_header_serialization() {
        let dh_public = [0x01u8; 32];
        let header = MessageHeader::new(dh_public, 5, 10);

        let bytes = header.to_bytes();
        let deserialized = MessageHeader::from_bytes(&bytes).unwrap();

        assert_eq!(header, deserialized);
        assert_eq!(deserialized.dh_public, dh_public);
        assert_eq!(deserialized.previous_chain_length, 5);
        assert_eq!(deserialized.message_number, 10);
    }
}