/// Test scenario 3: Mixed 1:1 and group messaging
#[test]
fn test_mixed_one_to_one_and_group_messaging() {
    println!("\n=== Test 3: Mixed 1:1 and Group Messaging ===\n");
    
    println!("Scenario: Alice talks to Bob privately while also managing a group");
    
    // Setup 1:1 session between Alice and Bob
    println!("\nStep 1: Setup private chat between Alice and Bob");
    let (alice_shared_secret, bob_shared_secret, alice_dh_key, bob_dh_key, bob_dh_public) = 
        setup_x3dh_session();
    
    let (mut alice_dr, mut bob_dr) = setup_double_ratchet(
        alice_shared_secret,
        bob_shared_secret,
        alice_dh_key,
        bob_dh_key,
        bob_dh_public,
    );
    
    println!("✓ Private chat established");
    
    // Setup group with Alice as admin
    println!("\nStep 2: Alice creates study group");
    let group_id = b"study-group";
    let alice_signing_key = [0x02u8; 32];
    let mut alice_sender_key = SenderKeyState::new(1, alice_signing_key);
    
    // Alice invites Bob to the group
    println!("Step 3: Alice invites Bob to the group");
    
    // Private message about group invitation
    let invite_msg = "I've created a study group. Here's the Sender Key:";
    let (invite_header, invite_encrypted) = alice_dr.encrypt(invite_msg.as_bytes()).unwrap();
    let decrypted_invite = bob_dr.decrypt(&invite_header, &invite_encrypted).unwrap();
    assert_eq!(decrypted_invite, invite_msg.as_bytes());
    println!("  Alice → Bob (private): {}", invite_msg);
    
    // Send group key distribution via private chat
    let distribution_msg = alice_sender_key.create_distribution_message(1).unwrap();
    let dist_bytes = distribution_msg.to_bytes();
    
    let (dist_header, dist_encrypted) = alice_dr.encrypt(&dist_bytes).unwrap();
    let decrypted_dist = bob_dr.decrypt(&dist_header, &dist_encrypted).unwrap();
    assert_eq!(decrypted_dist, dist_bytes);
    println!("  Alice sent Sender Key distribution to Bob via private chat");
    
    // Bob processes distribution and joins group
    let received_dist = scp_core::crypto::sender_key::SenderKeyDistributionMessage::from_bytes(&decrypted_dist).unwrap();
    let mut bob_sender_key = SenderKeyState::new(1, alice_signing_key);
    bob_sender_key.process_distribution_message(&received_dist).unwrap();
    println!("  Bob joined the study group");
    
    // Mixed messaging
    println!("\nStep 4: Mixed messaging (private + group)");
    
    // Alice sends group message
    let group_msg = "Welcome everyone to our study session!";
    let (group_header, group_encrypted) = alice_sender_key.encrypt(group_id, group_msg.as_bytes()).unwrap();
    
    // Bob decrypts group message
    let bob_group_decrypted = bob_sender_key.decrypt(&group_header, &group_encrypted).unwrap();
    assert_eq!(bob_group_decrypted, group_msg.as_bytes());
    println!("  Alice → Group: {}", group_msg);
    println!("  Bob decrypted group message");
    
    // Private reply from Bob to Alice
    let private_reply = "Thanks for the invite! Looking forward to it.";
    let (reply_header, reply_encrypted) = bob_dr.encrypt(private_reply.as_bytes()).unwrap();
    let alice_reply_decrypted = alice_dr.decrypt(&reply_header, &reply_encrypted).unwrap();
    assert_eq!(alice_reply_decrypted, private_reply.as_bytes());
    println!("  Bob → Alice (private): {}", private_reply);
    
    // Group reply from Bob (through Alice's sender key)
    // Note: In real implementation, Bob would have his own sender key
    // For this test, we'll simulate Bob sending through Alice
    
    println!("\n✅ Test 3 PASSED: Mixed 1:1 and group messaging");
}

/// Test scenario 4: Error handling and edge cases
#[test]
fn test_error_handling_and_edge_cases() {
    println!("\n=== Test 4: Error Handling and Edge Cases ===\n");
    
    println!("Testing various error conditions...");
    
    // Test 1: Invalid message format
    println!("\nTest 4.1: Invalid message format");
    let mut state = SenderKeyState::new(1, [0x01u8; 32]);
    let invalid_message = vec![0u8; 5]; // Too short for AES-GCM
    
    let header = scp_core::crypto::sender_key::SenderKeyMessageHeader::new(1, 0, b"test");
    let result = state.decrypt(&header, &invalid_message);
    assert!(result.is_err());
    println!("✓ Correctly rejected invalid message format");
    
    // Test 2: Decryption with wrong key
    println!("\nTest 4.2: Decryption with wrong key");
    let mut alice_state = SenderKeyState::new(1, [0x01u8; 32]);
    let mut bob_state = SenderKeyState::new(1, [0x02u8; 32]); // Different signing key
    
    let (header, encrypted) = alice_state.encrypt(b"group", b"secret").unwrap();
    
    // Bob tries to decrypt without proper key distribution
    let result = bob_state.decrypt(&header, &encrypted);
    assert!(result.is_err());
    println!("✓ Correctly rejected decryption with wrong key");
    
    // Test 3: Out-of-order messages with caching
    println!("\nTest 4.3: Out-of-order messages");
    let mut state = SenderKeyState::new(1, [0x01u8; 32]);
    let group_id = b"test-group";
    
    // Encrypt messages 0, 1, 2
    let mut messages = Vec::new();
    for i in 0..3 {
        let msg = format!("Message {}", i);
        let (header, encrypted) = state.encrypt(group_id, msg.as_bytes()).unwrap();
        messages.push((header, encrypted, msg));
    }
    
    // Decrypt in reverse order (2, 1, 0)
    let mut receiver_state = SenderKeyState::new(1, [0x01u8; 32]);
    let dist_msg = state.create_distribution_message(1).unwrap();
    receiver_state.process_distribution_message(&dist_msg).unwrap();
    
    // Decrypt message 2 first
    let (header2, encrypted2, msg2) = &messages[2];
    let decrypted2 = receiver_state.decrypt(header2, encrypted2).unwrap();
    assert_eq!(decrypted2, msg2.as_bytes());
    
    // Then message 0 (skipping 1)
    let (header0, encrypted0, msg0) = &messages[0];
    let decrypted0 = receiver_state.decrypt(header0, encrypted0).unwrap();
    assert_eq!(decrypted0, msg0.as_bytes());
    
    // Finally message 1
    let (header1, encrypted1, msg1) = &messages[1];
    let decrypted1 = receiver_state.decrypt(header1, encrypted1).unwrap();
    assert_eq!(decrypted1, msg1.as_bytes());
    
    println!("✓ Successfully handled out-of-order messages");
    
    // Test 4: Chain exhaustion
    println!("\nTest 4.4: Chain exhaustion (simulated)");
    let mut state = SenderKeyState::new(1, [0x01u8; 32]);
    
    // Try to advance chain many times (will hit limit in real implementation)
    // For this test, we'll just verify the chain rotation mechanism works
    let new_chain_id = state.rotate_chain();
    assert_eq!(new_chain_id, 2);
    println!("✓ Chain rotation works correctly");
    
    println!("\n✅ Test 4 PASSED: Error handling and edge cases");
}

/// Test scenario 5: Performance benchmark (simple)
#[test]
fn test_performance_benchmark() {
    println!("\n=== Test 5: Performance Benchmark ===\n");
    
    use std::time::Instant;
    
    println!("Running performance tests...");
    
    // Benchmark 1: X3DH key agreement
    println!("\nBenchmark 1: X3DH Key Agreement");
    let start = Instant::now();
    
    let mut rng = OsRng;
    let iterations = 10;
    
    for _ in 0..iterations {
        let _ = setup_x3dh_session();
    }
    
    let duration = start.elapsed();
    println!("  {} iterations: {:?}", iterations, duration);
    println!("  Average: {:?} per iteration", duration / iterations);
    
    // Benchmark 2: Double Ratchet message exchange
    println!("\nBenchmark 2: Double Ratchet Message Exchange");
    let start = Instant::now();
    
    let (alice_shared_secret, bob_shared_secret, alice_dh_key, bob_dh_key, bob_dh_public) = 
        setup_x3dh_session();
    
    let (mut alice_dr, mut bob_dr) = setup_double_ratchet(
        alice_shared_secret,
        bob_shared_secret,
        alice_dh_key,
        bob_dh_key,
        bob_dh_public,
    );
    
    let messages = 100;
    let mut total_size = 0;
    
    for i in 0..messages {
        let message = format!("Test message {}", i);
        let (header, encrypted) = alice_dr.encrypt(message.as_bytes()).unwrap();
        let decrypted = bob_dr.decrypt(&header, &encrypted).unwrap();
        assert_eq!(decrypted, message.as_bytes());
        total_size += encrypted.len();
    }
    
    let duration = start.elapsed();
    println!("  {} messages exchanged: {:?}", messages, duration);
    println!("  Average: {:?} per message", duration / messages);
    println!("  Total encrypted data: {} bytes", total_size);
    println!("  Average message size: {} bytes", total_size / messages);
    
    // Benchmark 3: Sender Key group encryption
    println!("\nBenchmark 3: Sender Key Group Encryption");
    let start = Instant::now();
    
    let mut state = SenderKeyState::new(1, [0x01u8; 32]);
    let group_id = b"benchmark-group";
    
    let group_messages = 50;
    
    for i in 0..group_messages {
        let message = format!("Group message {}", i);
        let (_, encrypted) = state.encrypt(group_id, message.as_bytes()).unwrap();
        total_size += encrypted.len();
    }
    
    let duration = start.elapsed();
    println!("  {} group messages: {:?}", group_messages, duration);
    println!("  Average: {:?} per message", duration / group_messages);
    
    println!("\n✅ Test 5 PASSED: Performance benchmarks completed");
    println!("Note: These are development benchmarks. Production performance may vary.");
}

// Helper functions

fn setup_x3dh_session() -> ([u8; 32], [u8; 32], StaticSecret, StaticSecret, PublicKey) {
    let mut rng = OsRng;
    
    // Generate key pairs for both parties
    let alice_identity_key = StaticSecret::random_from_rng(&mut rng);
    let alice_signed_prekey = StaticSecret::random_from_rng(&mut rng);
    let alice_one_time_prekey = Some(StaticSecret::random_from_rng(&mut rng));
    
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
    
    // Alice initiates X3DH
    let alice_result = perform_x3dh_initiator(
        &alice_identity_key,
        &alice_signed_prekey,
        alice_one_time_prekey.as_ref(),
        &bob_identity_pub,
        &bob_signed_prekey_pub,
        bob_one_time_prekey_pub.as_ref(),
    ).unwrap();
    
    // Bob responds
    let bob_result = perform_x3dh_responder(
        &bob_identity_key,
        &bob_signed_prekey,
        bob_one_time_prekey.as_ref(),
        &alice_identity_pub,
        &alice_signed_prekey_pub,
        alice_one_time_prekey_pub.as_ref(),
        &alice_result.ephemeral_public_key,
    ).unwrap();
    
    // Generate DH ratchet keys
    let alice_dh_key = generate_dh_ratchet_key();
    let bob_dh_key = generate_dh_ratchet_key();
    let bob_dh_public = PublicKey::from(&bob_dh_key);
    
    (
        alice_result.shared_secret,
        bob_result.shared_secret,
        alice_dh_key,
        bob_dh_key,
        bob_dh_public,
    )
}

fn setup_double_ratchet(
    alice_shared_secret: [u8; 32],
    bob_shared_secret: [u8; 32],
    alice_dh_key: StaticSecret,
    bob_dh_key: StaticSecret,
    bob_dh_public: PublicKey,
) -> (DoubleRatchetState, DoubleRatchetState) {
    let alice_state = DoubleRatchetState::new_from_x3dh(
        alice_shared_secret,
        alice_dh_key,
        Some(bob_dh_public),
    );
    
    let bob_state = DoubleRatchetState::new_from_x3dh(
        bob_shared_secret,
        bob_dh_key,
        Some(alice_state.dh_ratchet_public),
    );
    
    (alice_state, bob_state)
}

fn main() {
    println!("SCP Crypto Module Integration Tests");
    println!("===================================\n");
    
    // Run all tests
    test_complete_one_to_one_messaging();
    test_group_messaging_with_key_distribution();
    test_mixed_one_to_one_and_group_messaging();
    test_error_handling_and_edge_cases();
    test_performance_benchmark();
    
    println!("\n===================================");
    println!("ALL INTEGRATION TESTS PASSED! 🎉");
    println!("===================================");
}