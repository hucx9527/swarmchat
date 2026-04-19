//! Integration tests for SCP crypto modules
//!
//! Tests the complete cryptographic workflow:
//! 1. X3DH for initial key agreement
//! 2. Double Ratchet for 1:1 messaging
//! 3. Sender Key for group messaging

use scp_core::crypto::{
    x3dh::{perform_x3dh_initiator, perform_x3dh_responder},
    double_ratchet::{DoubleRatchetState, generate_dh_ratchet_key},
    sender_key::{SenderKeyState, SenderKeyMessageHeader, SenderKeyDistributionMessage},
};
use x25519_dalek::{PublicKey, StaticSecret};
use rand_core::OsRng;

/// Test scenario 1: Complete 1:1 messaging workflow
#[test]
fn test_complete_one_to_one_messaging() {
    println!("=== Test 1: Complete 1:1 Messaging Workflow ===\n");
    
    // Step 1: X3DH key agreement
    println!("Step 1: X3DH Key Agreement");
    let (alice_shared_secret, bob_shared_secret, alice_dh_key, bob_dh_key, bob_dh_public) = 
        setup_x3dh_session();
    
    assert_eq!(alice_shared_secret, bob_shared_secret);
    println!("✓ Shared secret established: {} bytes", alice_shared_secret.len());
    
    // Step 2: Initialize Double Ratchet
    println!("\nStep 2: Initialize Double Ratchet");
    let (mut alice_dr, mut bob_dr) = setup_double_ratchet(
        alice_shared_secret,
        bob_shared_secret,
        alice_dh_key,
        bob_dh_key,
        bob_dh_public,
    );
    
    println!("✓ Double Ratchet initialized for both parties");
    
    // Step 3: Exchange messages
    println!("\nStep 3: Message Exchange");
    
    let conversation = vec![
        ("Alice", "Hi Bob! How are you?"),
        ("Bob", "Hi Alice! I'm good, thanks!"),
        ("Alice", "Want to join our group chat?"),
        ("Bob", "Sure! What's the group about?"),
        ("Alice", "It's about Rust cryptography projects"),
        ("Bob", "Sounds interesting! Add me please"),
    ];
    
    let mut success_count = 0;
    let mut alice_turn = true;
    
    for (sender, message) in conversation {
        if alice_turn {
            // Alice sends message
            let (header, encrypted) = alice_dr.encrypt(message.as_bytes()).unwrap();
            let decrypted = bob_dr.decrypt(&header, &encrypted).unwrap();
            assert_eq!(decrypted, message.as_bytes());
            println!("  {} → Bob: {}", sender, message);
        } else {
            // Bob sends message
            let (header, encrypted) = bob_dr.encrypt(message.as_bytes()).unwrap();
            let decrypted = alice_dr.decrypt(&header, &encrypted).unwrap();
            assert_eq!(decrypted, message.as_bytes());
            println!("  {} → Alice: {}", sender, message);
        }
        
        success_count += 1;
        alice_turn = !alice_turn;
    }
    
    println!("✓ {} messages exchanged successfully", success_count);
    
    // Step 4: Test many messages (stress test)
    println!("\nStep 4: Stress Test (50 messages)");
    let mut stress_success = 0;
    
    for i in 0..50 {
        let message = format!("Message {}", i);
        
        if i % 2 == 0 {
            let (header, encrypted) = alice_dr.encrypt(message.as_bytes()).unwrap();
            let decrypted = bob_dr.decrypt(&header, &encrypted).unwrap();
            assert_eq!(decrypted, message.as_bytes());
        } else {
            let (header, encrypted) = bob_dr.encrypt(message.as_bytes()).unwrap();
            let decrypted = alice_dr.decrypt(&header, &encrypted).unwrap();
            assert_eq!(decrypted, message.as_bytes());
        }
        
        stress_success += 1;
    }
    
    println!("✓ {} stress test messages passed", stress_success);
    
    println!("\n✅ Test 1 PASSED: Complete 1:1 messaging workflow");
}

/// Test scenario 2: Group messaging with key distribution
#[test]
fn test_group_messaging_with_key_distribution() {
    println!("\n=== Test 2: Group Messaging with Key Distribution ===\n");
    
    // Create a group with 3 members
    let group_id = b"rust-crypto-group";
    let group_name = "Rust Crypto Enthusiasts";
    
    println!("Creating group: {}", group_name);
    println!("Group ID: {}", String::from_utf8_lossy(group_id));
    
    // Alice is the group admin
    println!("\nStep 1: Alice creates group and Sender Key");
    let alice_signing_key = [0x01u8; 32];
    let mut alice_sender_key = SenderKeyState::new(1, alice_signing_key);
    
    // Alice sends welcome message
    let welcome_msg = format!("Welcome to {}!", group_name);
    let (welcome_header, welcome_encrypted) = 
        alice_sender_key.encrypt(group_id, welcome_msg.as_bytes()).unwrap();
    println!("  Alice sent welcome message at iteration {}", welcome_header.iteration);
    
    // Create distribution message for new members
    let distribution_msg = alice_sender_key.create_distribution_message(1).unwrap();
    println!("  Created Sender Key distribution message");
    
    // Bob and Charlie join the group
    println!("\nStep 2: Bob and Charlie join the group");
    
    let mut bob_sender_key = SenderKeyState::new(1, alice_signing_key);
    let mut charlie_sender_key = SenderKeyState::new(1, alice_signing_key);
    
    bob_sender_key.process_distribution_message(&distribution_msg).unwrap();
    charlie_sender_key.process_distribution_message(&distribution_msg).unwrap();
    
    println!("  Bob joined and received Sender Key");
    println!("  Charlie joined and received Sender Key");
    
    // All members should be able to decrypt the welcome message
    println!("\nStep 3: Members decrypt welcome message");
    
    let bob_decrypted = bob_sender_key.decrypt(&welcome_header, &welcome_encrypted).unwrap();
    assert_eq!(bob_decrypted, welcome_msg.as_bytes());
    println!("  Bob decrypted: {}", String::from_utf8_lossy(&bob_decrypted));
    
    let charlie_decrypted = charlie_sender_key.decrypt(&welcome_header, &welcome_encrypted).unwrap();
    assert_eq!(charlie_decrypted, welcome_msg.as_bytes());
    println!("  Charlie decrypted: {}", String::from_utf8_lossy(&charlie_decrypted));
    
    // Group conversation
    println!("\nStep 4: Group conversation");
    
    let group_messages = vec![
        ("Alice", "Let's discuss our next project"),
        ("Bob", "I think we should build a secure chat app"),
        ("Charlie", "Great idea! I can help with the UI"),
        ("Alice", "Perfect! I'll handle the crypto backend"),
        ("Bob", "When should we start?"),
        ("Charlie", "How about next Monday?"),
    ];
    
    let mut encrypted_messages = Vec::new();
    
    for (sender, message) in group_messages {
        let (header, encrypted) = alice_sender_key.encrypt(group_id, message.as_bytes()).unwrap();
        encrypted_messages.push((sender, header, encrypted, message));
        println!("  {} → Group: {}", sender, message);
    }
    
    // All members decrypt all messages
    println!("\nStep 5: All members decrypt conversation");
    
    let mut total_decryptions = 0;
    let members = [
        ("Bob", &mut bob_sender_key),
        ("Charlie", &mut charlie_sender_key),
    ];
    
    for (sender, header, encrypted, expected) in &encrypted_messages {
        for (member_name, member_state) in &mut members {
            let decrypted = member_state.decrypt(header, encrypted).unwrap();
            assert_eq!(decrypted, expected.as_bytes());
            total_decryptions += 1;
        }
    }
    
    println!("✓ {} successful decryptions by group members", total_decryptions);
    
    // Test chain rotation
    println!("\nStep 6: Test Sender Key chain rotation");
    
    let new_chain_id = alice_sender_key.rotate_chain();
    println!("  Alice rotated to new chain ID: {}", new_chain_id);
    
    // Create new distribution message
    let new_distribution = alice_sender_key.create_distribution_message(new_chain_id).unwrap();
    
    // Distribute to members
    bob_sender_key.process_distribution_message(&new_distribution).unwrap();
    charlie_sender_key.process_distribution_message(&new_distribution).unwrap();
    
    println!("  New chain key distributed to all members");
    
    // Send message on new chain
    let new_msg = "We're now using a new encryption chain!";
    let (new_header, new_encrypted) = alice_sender_key.encrypt(group_id, new_msg.as_bytes()).unwrap();
    
    // All members should decrypt successfully
    let bob_new_decrypted = bob_sender_key.decrypt(&new_header, &new_encrypted).unwrap();
    let charlie_new_decrypted = charlie_sender_key.decrypt(&new_header, &new_encrypted).unwrap();
    
    assert_eq!(bob_new_decrypted, new_msg.as_bytes());
    assert_eq!(charlie_new_decrypted, new_msg.as_bytes());
    
    println!("  All members decrypted message from new chain");
    println!("  Message: {}", new_msg);
    
    println!("\n✅ Test 2 PASSED: Group messaging with key distribution");
}