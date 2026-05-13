//! Integration tests for SCP crypto modules
//!
//! Tests the complete cryptographic workflow:
//! 1. X3DH for initial key agreement
//! 2. Double Ratchet for 1:1 messaging
//! 3. Sender Key for group messaging
//! 4. Mixed 1:1 and group messaging
//! 5. Error handling and edge cases
//! 6. Performance benchmarks

use scp_core::crypto::{
    x3dh::{perform_x3dh_initiator, perform_x3dh_responder},
    double_ratchet::{DoubleRatchetState, generate_dh_ratchet_key},
    sender_key::{SenderKeyState, SenderKeyMessageHeader, SenderKeyDistributionMessage},
};
use x25519_dalek::{PublicKey, StaticSecret};
use rand_core::OsRng;

// ═══════════════════════════════════════════════════════════
// Test Scenario 1: Complete 1:1 messaging workflow
// ═══════════════════════════════════════════════════════════

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
            let (header, encrypted) = alice_dr.encrypt(message.as_bytes()).unwrap();
            let decrypted = bob_dr.decrypt(&header, &encrypted).unwrap();
            assert_eq!(decrypted, message.as_bytes());
            println!("  {} → Bob: {}", sender, message);
        } else {
            let (header, encrypted) = bob_dr.encrypt(message.as_bytes()).unwrap();
            let decrypted = alice_dr.decrypt(&header, &encrypted).unwrap();
            assert_eq!(decrypted, message.as_bytes());
            println!("  {} → Alice: {}", sender, message);
        }

        success_count += 1;
        alice_turn = !alice_turn;
    }

    println!("✓ {} messages exchanged successfully", success_count);

    // Step 4: Stress test
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

// ═══════════════════════════════════════════════════════════
// Test Scenario 2: Group messaging with key distribution
// ═══════════════════════════════════════════════════════════

#[test]
fn test_group_messaging_with_key_distribution() {
    println!("\n=== Test 2: Group Messaging with Key Distribution ===\n");

    let group_id = b"rust-crypto-group";
    let group_name = "Rust Crypto Enthusiasts";

    println!("Creating group: {}", group_name);
    println!("Group ID: {}", String::from_utf8_lossy(group_id));

    // Alice creates group and Sender Key
    println!("\nStep 1: Alice creates group and Sender Key");
    let alice_signing_key = [0x01u8; 32];
    let mut alice_sender_key = SenderKeyState::new(1, alice_signing_key);

    // Welcome message
    let welcome_msg = format!("Welcome to {}!", group_name);
    let (welcome_header, welcome_encrypted) =
        alice_sender_key.encrypt(group_id, welcome_msg.as_bytes()).unwrap();
    println!("  Alice sent welcome message at iteration {}", welcome_header.iteration);

    // Distribution message
    let distribution_msg = alice_sender_key.create_distribution_message(1).unwrap();
    println!("  Created Sender Key distribution message");

    // Bob and Charlie join
    println!("\nStep 2: Bob and Charlie join the group");

    let mut bob_sender_key = SenderKeyState::new(1, alice_signing_key);
    let mut charlie_sender_key = SenderKeyState::new(1, alice_signing_key);

    bob_sender_key.process_distribution_message(&distribution_msg).unwrap();
    charlie_sender_key.process_distribution_message(&distribution_msg).unwrap();

    println!("  Bob joined and received Sender Key");
    println!("  Charlie joined and received Sender Key");

    // Decrypt welcome message
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

    // All members decrypt
    println!("\nStep 5: All members decrypt conversation");

    let mut total_decryptions = 0;
    let mut members = [
        ("Bob", &mut bob_sender_key),
        ("Charlie", &mut charlie_sender_key),
    ];

    for (_sender, header, encrypted, expected) in &encrypted_messages {
        for (_member_name, member_state) in &mut members {
            let decrypted = member_state.decrypt(header, encrypted).unwrap();
            assert_eq!(decrypted, expected.as_bytes());
            total_decryptions += 1;
        }
    }

    println!("✓ {} successful decryptions by group members", total_decryptions);

    // Chain rotation
    println!("\nStep 6: Test Sender Key chain rotation");

    let new_chain_id = alice_sender_key.rotate_chain();
    println!("  Alice rotated to new chain ID: {}", new_chain_id);

    let new_distribution = alice_sender_key.create_distribution_message(new_chain_id).unwrap();

    bob_sender_key.process_distribution_message(&new_distribution).unwrap();
    charlie_sender_key.process_distribution_message(&new_distribution).unwrap();

    println!("  New chain key distributed to all members");

    // Send message on new chain
    let new_msg = "We're now using a new encryption chain!";
    let (new_header, new_encrypted) = alice_sender_key.encrypt(group_id, new_msg.as_bytes()).unwrap();

    let bob_new_decrypted = bob_sender_key.decrypt(&new_header, &new_encrypted).unwrap();
    let charlie_new_decrypted = charlie_sender_key.decrypt(&new_header, &new_encrypted).unwrap();

    assert_eq!(bob_new_decrypted, new_msg.as_bytes());
    assert_eq!(charlie_new_decrypted, new_msg.as_bytes());

    println!("  All members decrypted message from new chain");
    println!("  Message: {}", new_msg);

    println!("\n✅ Test 2 PASSED: Group messaging with key distribution");
}

// ═══════════════════════════════════════════════════════════
// Test Scenario 3: Mixed 1:1 and group messaging
// ═══════════════════════════════════════════════════════════

#[test]
fn test_mixed_one_to_one_and_group_messaging() {
    println!("\n=== Test 3: Mixed 1:1 and Group Messaging ===\n");

    println!("Scenario: Alice talks to Bob privately while also managing a group");

    // Setup 1:1 session
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

    // Setup group
    println!("\nStep 2: Alice creates study group");
    let group_id = b"study-group";
    let alice_signing_key = [0x02u8; 32];
    let mut alice_sender_key = SenderKeyState::new(1, alice_signing_key);

    // Alice invites Bob via private chat
    println!("Step 3: Alice invites Bob to the group");

    let invite_msg = "I've created a study group. Here's the Sender Key:";
    let (invite_header, invite_encrypted) = alice_dr.encrypt(invite_msg.as_bytes()).unwrap();
    let decrypted_invite = bob_dr.decrypt(&invite_header, &invite_encrypted).unwrap();
    assert_eq!(decrypted_invite, invite_msg.as_bytes());
    println!("  Alice → Bob (private): {}", invite_msg);

    // Send Sender Key distribution via Double Ratchet
    let distribution_msg = alice_sender_key.create_distribution_message(1).unwrap();
    let dist_bytes = distribution_msg.to_bytes();

    let (dist_header, dist_encrypted) = alice_dr.encrypt(&dist_bytes).unwrap();
    let decrypted_dist = bob_dr.decrypt(&dist_header, &dist_encrypted).unwrap();
    assert_eq!(decrypted_dist, dist_bytes);
    println!("  Alice sent Sender Key distribution to Bob via private chat");

    // Bob processes and joins group
    let received_dist = SenderKeyDistributionMessage::from_bytes(&decrypted_dist).unwrap();
    let mut bob_sender_key = SenderKeyState::new(1, alice_signing_key);
    bob_sender_key.process_distribution_message(&received_dist).unwrap();
    println!("  Bob joined the study group");

    // Mixed messaging
    println!("\nStep 4: Mixed messaging (private + group)");

    // Alice sends group message
    let group_msg = "Welcome everyone to our study session!";
    let (group_header, group_encrypted) = alice_sender_key.encrypt(group_id, group_msg.as_bytes()).unwrap();

    let bob_group_decrypted = bob_sender_key.decrypt(&group_header, &group_encrypted).unwrap();
    assert_eq!(bob_group_decrypted, group_msg.as_bytes());
    println!("  Alice → Group: {}", group_msg);
    println!("  Bob decrypted group message");

    // Bob sends private reply
    let private_reply = "Thanks for the invite! Looking forward to it.";
    let (reply_header, reply_encrypted) = bob_dr.encrypt(private_reply.as_bytes()).unwrap();
    let alice_reply_decrypted = alice_dr.decrypt(&reply_header, &reply_encrypted).unwrap();
    assert_eq!(alice_reply_decrypted, private_reply.as_bytes());
    println!("  Bob → Alice (private): {}", private_reply);

    println!("\n✅ Test 3 PASSED: Mixed 1:1 and group messaging");
}

// ═══════════════════════════════════════════════════════════
// Test Scenario 4: Error handling and edge cases
// ═══════════════════════════════════════════════════════════

#[test]
fn test_error_handling_and_edge_cases() {
    println!("\n=== Test 4: Error Handling and Edge Cases ===\n");

    println!("Testing various error conditions...");

    // 4.1: Invalid message format (too short for AES-GCM)
    println!("\nTest 4.1: Invalid message format");
    let mut state = SenderKeyState::new(1, [0x01u8; 32]);
    let invalid_message = vec![0u8; 5];
    let header = SenderKeyMessageHeader::new(1, 0, b"test");
    let result = state.decrypt(&header, &invalid_message);
    assert!(result.is_err());
    println!("✓ Correctly rejected invalid message format");

    // 4.2: Decryption with wrong key
    println!("\nTest 4.2: Decryption with wrong key");
    let mut alice_state = SenderKeyState::new(1, [0x01u8; 32]);
    let mut bob_state = SenderKeyState::new(1, [0x02u8; 32]);

    let (header, encrypted) = alice_state.encrypt(b"group", b"secret").unwrap();
    let result = bob_state.decrypt(&header, &encrypted);
    assert!(result.is_err());
    println!("✓ Correctly rejected decryption with wrong key");

    // 4.3: Out-of-order messages
    println!("\nTest 4.3: Out-of-order messages");
    let mut state = SenderKeyState::new(1, [0x01u8; 32]);
    let group_id = b"test-group";

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

    // Message 2 first
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

    // 4.4: Chain rotation
    println!("\nTest 4.4: Chain rotation verification");
    let mut state = SenderKeyState::new(1, [0x01u8; 32]);
    let new_chain_id = state.rotate_chain();
    assert_eq!(new_chain_id, 2);
    println!("✓ Chain rotation works correctly");

    println!("\n✅ Test 4 PASSED: Error handling and edge cases");
}

// ═══════════════════════════════════════════════════════════
// Test Scenario 5: Performance benchmark
// ═══════════════════════════════════════════════════════════

#[test]
fn test_performance_benchmark() {
    println!("\n=== Test 5: Performance Benchmark ===\n");

    use std::time::Instant;

    println!("Running performance tests...");

    // Benchmark 1: X3DH key agreement
    println!("\nBenchmark 1: X3DH Key Agreement");
    let start = Instant::now();
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

    let messages: u32 = 100;
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
    println!("  Average message size: {} bytes", total_size / messages as usize);

    // Benchmark 3: Sender Key group encryption
    println!("\nBenchmark 3: Sender Key Group Encryption");
    let start = Instant::now();

    let mut state = SenderKeyState::new(1, [0x01u8; 32]);
    let group_id = b"benchmark-group";
    let group_messages = 50;

    for i in 0..group_messages {
        let message = format!("Group message {}", i);
        let (_, encrypted) = state.encrypt(group_id, message.as_bytes()).unwrap();
        let _ = encrypted.len(); // discard, benchmark measurement only
    }

    let duration = start.elapsed();
    println!("  {} group messages: {:?}", group_messages, duration);
    println!("  Average: {:?} per message", duration / group_messages);

    println!("\n✅ Test 5 PASSED: Performance benchmarks completed");
    println!("Note: These are development benchmarks. Production performance may vary.");
}

// ═══════════════════════════════════════════════════════════
// Helper Functions
// ═══════════════════════════════════════════════════════════

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

    test_complete_one_to_one_messaging();
    test_group_messaging_with_key_distribution();
    test_mixed_one_to_one_and_group_messaging();
    test_error_handling_and_edge_cases();
    test_performance_benchmark();

    println!("\n===================================");
    println!("ALL INTEGRATION TESTS PASSED!");
    println!("===================================");
}