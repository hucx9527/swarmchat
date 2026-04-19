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
            let (header, encrypted) = state.encrypt(group_id, message).unwrap();
            encrypted_messages.push((header, encrypted));
            println!("Encrypted message {} at iteration {}", i + 1, header.iteration);
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