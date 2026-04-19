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
            // Perform DH ratchet
            self.dh_ratchet(remote_dh_public)?;
            
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
                    return self.decrypt_with_key(receiving_chain, *msg_num, ciphertext);
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
        
        let plaintext = self.decrypt_with_message_key(&message_key, encrypted_message)?;
        
        self.receiving_message_number += 1;
        
        Ok(plaintext)
    }
    
    /// Decrypt with a specific message key
    fn decrypt_with_key(&self, chain: &ChainKey, message_number: u32, encrypted_message: &[u8]) 
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
        self.decrypt_with_message_key(&message_key, encrypted_message)
    }
    
    /// Decrypt with a given message key
    fn decrypt_with_message_key(&self, message_key: &[u8; 32], encrypted_message: &[u8]) 
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
        let mut rng = rand_core::OsRng;
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