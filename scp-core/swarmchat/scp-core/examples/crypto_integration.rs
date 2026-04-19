use scp_core::crypto::{
    x3dh::{perform_x3dh_initiator, perform_x3dh_responder, X3DHResult},
    double_ratchet::{DoubleRatchetState, generate_dh_ratchet_key},
    sender_key::SenderKeyState,
};
use x25519_dalek::{PublicKey, StaticSecret};

fn main() {
    println!("=== Testing Complete Crypto Module Suite ===\n");
    
    // Test 1: X3DH
    println!("--- Test 1: X3DH Key Agreement ---");
    test_x3dh();
    
    // Test 2: Double Ratchet
    println!("\n--- Test 2: Double Ratchet ---");
    test_double_ratchet();
    
    // Test 3: Sender Key
    println!("\n--- Test 3: Sender Key (Group Encryption) ---");
    test_sender_key();
    
    println!("\n=== All Crypto Module Tests Passed! ===");
}

fn test_x3dh() {
    // Generate key pairs
    let mut rng = rand_core::OsRng;
    
    // Alice's keys
    let alice_identity_key = StaticSecret::random_from_rng(&mut rng);
    let alice_signed_prekey = StaticSecret::random_from_rng(&mut rng);
    let alice_one_time_prekey = Some(StaticSecret::random_from_rng(&mut rng));
    
    // Bob's keys  
    let bob_identity_key = StaticSecret::random_from_rng(&mut rng);
    let bob_signed_prekey = StaticSecret::random_from_rng(&mut rng);
    let bob_one_time_prekey = Some(StaticSecret::random_from_rng(&mut rng));
    
    // Convert to public keys
    let alice_identity_pub = PublicKey::from(&alice_identity_key);
    let alice_signed_prekey_pub = PublicKey::from(&alice_signed_prekey);
    let alice_one_time_prekey_pub = alice_one_time_prekey.as_ref().map(|k| PublicKey::from(k));
    
    let bob_identity_pub = PublicKey::from(&bob_identity_key);
    let bob_signed_prekey_pub = PublicKey::from(&bob_signed_prekey);
    let bob_one_time_prekey_pub = bob_one_time_prekey.as_ref().map(|k| PublicKey::from(k));
    
    // Alice initiates X3DH as initiator
    let alice_result = perform_x3dh_initiator(
        &alice_identity_key,
        &alice_signed_prekey,
        alice_one_time_prekey.as_ref(),
        &bob_identity_pub,
        &bob_signed_prekey_pub,
        bob_one_time_prekey_pub.as_ref(),
    ).unwrap();
    
    // Bob responds as responder
    let bob_result = perform_x3dh_responder(
        &bob_identity_key,
        &bob_signed_prekey,
        bob_one_time_prekey.as_ref(),
        &alice_identity_pub,
        &alice_signed_prekey_pub,
        alice_one_time_prekey_pub.as_ref(),
        &alice_result.ephemeral_public_key,
    ).unwrap();
    
    // Both should have the same shared secret
    assert_eq!(alice_result.shared_secret, bob_result.shared_secret);
    println!("✓ X3DH shared secret established: {} bytes", alice_result.shared_secret.len());
}

fn test_double_ratchet() {
    // Use X3DH shared secret
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
    
    // Exchange messages
    let messages = [
        b"Hello from Alice",
        b"Hello from Bob",
        b"How are you?",
        b"I'm good, thanks!",
    ];
    
    let mut alice_turn = true;
    let mut success_count = 0;
    
    for message in messages.iter() {
        if alice_turn {
            let (header, encrypted) = alice_state.encrypt(message).unwrap();
            let decrypted = bob_state.decrypt(&header, &encrypted).unwrap();
            assert_eq!(decrypted, *message);
            success_count += 1;
        } else {
            let (header, encrypted) = bob_state.encrypt(message).unwrap();
            let decrypted = alice_state.decrypt(&header, &encrypted).unwrap();
            assert_eq!(decrypted, *message);
            success_count += 1;
        }
        alice_turn = !alice_turn;
    }
    
    println!("✓ Double Ratchet exchanged {} messages successfully", success_count);
}

fn test_sender_key() {
    // Create Sender Key state for group admin
    let signing_key = [0x01u8; 32];
    let mut admin_state = SenderKeyState::new(1, signing_key);
    let group_id = b"test-group-123";
    
    // Admin sends messages to group
    let admin_messages = [
        b"Welcome to the group!",
        b"Group rules: be respectful",
        b"Meeting at 3 PM tomorrow",
    ];
    
    let mut encrypted_messages = Vec::new();
    
    for (i, message) in admin_messages.iter().enumerate() {
        let (header, encrypted) = admin_state.encrypt(group_id, message).unwrap();
        encrypted_messages.push((header, encrypted));
        println!("  Admin sent message {} at iteration {}", i + 1, header.iteration);
    }
    
    // Create member states and distribute key
    let mut member_states = Vec::new();
    
    for member_id in 0..3 {
        let mut member_state = SenderKeyState::new(1, signing_key);
        let dist_message = admin_state.create_distribution_message(1).unwrap();
        member_state.process_distribution_message(&dist_message).unwrap();
        member_states.push(member_state);
        println!("  Member {} received Sender Key distribution", member_id + 1);
    }
    
    // All members should be able to decrypt messages
    let mut decryption_success = 0;
    
    for (i, (header, encrypted)) in encrypted_messages.iter().enumerate() {
        for (member_id, member_state) in member_states.iter_mut().enumerate() {
            let decrypted = member_state.decrypt(header, encrypted).unwrap();
            assert_eq!(decrypted, admin_messages[i]);
            decryption_success += 1;
        }
    }
    
    println!("✓ Sender Key: {} successful decryptions by group members", decryption_success);
    
    // Test chain rotation
    println!("\n  Testing chain rotation...");
    let new_chain_id = admin_state.rotate_chain();
    println!("  Admin rotated to new chain ID: {}", new_chain_id);
    
    // Send message on new chain
    let (new_header, new_encrypted) = admin_state.encrypt(group_id, b"New chain message").unwrap();
    println!("  Sent message on new chain, iteration: {}", new_header.iteration);
    
    // Distribute new chain key
    let new_dist_message = admin_state.create_distribution_message(new_chain_id).unwrap();
    for member_state in member_states.iter_mut() {
        member_state.process_distribution_message(&new_dist_message).unwrap();
        let decrypted = member_state.decrypt(&new_header, &new_encrypted).unwrap();
        assert_eq!(decrypted, b"New chain message");
    }
    
    println!("  ✓ Chain rotation and key distribution successful");
}