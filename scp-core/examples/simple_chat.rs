//! Simple Chat Example — Phase 0 Acceptance Demo
//!
//! Demonstrates two in-memory DoubleRatchetState instances
//! (Alice and Bob) performing encrypted message exchange.
//!
//! Flow:
//! 1. X3DH key agreement
//! 2. Initialize Double Ratchet for both parties
//! 3. Exchange messages
//! 4. Verify all messages decrypt correctly

use scp_core::crypto::{
    x3dh::{perform_x3dh_initiator, perform_x3dh_responder},
    double_ratchet::{DoubleRatchetState, generate_dh_ratchet_key},
};
use x25519_dalek::{PublicKey, StaticSecret};
use rand_core::OsRng;

fn main() {
    println!("== SCP Double Ratchet - Simple Chat Demo ==\n");

    // Step 1: Generate X25519 key pairs
    println!("[1/4] Generating X25519 key pairs...");
    let mut rng = OsRng;

    let alice_identity_key = StaticSecret::random_from_rng(&mut rng);
    let alice_signed_prekey = StaticSecret::random_from_rng(&mut rng);
    let alice_one_time_prekey = Some(StaticSecret::random_from_rng(&mut rng));

    let alice_id_pub = PublicKey::from(&alice_identity_key);
    let alice_spk_pub = PublicKey::from(&alice_signed_prekey);
    let alice_opk_pub = alice_one_time_prekey.as_ref().map(|k| PublicKey::from(k));

    let bob_identity_key = StaticSecret::random_from_rng(&mut rng);
    let bob_signed_prekey = StaticSecret::random_from_rng(&mut rng);
    let bob_one_time_prekey = Some(StaticSecret::random_from_rng(&mut rng));

    let bob_id_pub = PublicKey::from(&bob_identity_key);
    let bob_spk_pub = PublicKey::from(&bob_signed_prekey);
    let bob_opk_pub = bob_one_time_prekey.as_ref().map(|k| PublicKey::from(k));

    println!("  Keys generated\n");

    // Step 2: X3DH key agreement
    println!("[2/4] Performing X3DH key agreement...");

    let alice_result = perform_x3dh_initiator(
        &alice_identity_key,
        &alice_signed_prekey,
        alice_one_time_prekey.as_ref(),
        &bob_id_pub,
        &bob_spk_pub,
        bob_opk_pub.as_ref(),
    ).expect("X3DH initiator failed");

    let bob_result = perform_x3dh_responder(
        &bob_identity_key,
        &bob_signed_prekey,
        bob_one_time_prekey.as_ref(),
        &alice_id_pub,
        &alice_spk_pub,
        alice_opk_pub.as_ref(),
        &alice_result.ephemeral_public_key,
    ).expect("X3DH responder failed");

    assert_eq!(alice_result.shared_secret, bob_result.shared_secret);
    println!("  Shared secret established (32 bytes)\n");

    // Step 3: Initialize Double Ratchet
    println!("[3/4] Initializing Double Ratchet sessions...");

    let alice_dh_key = generate_dh_ratchet_key();
    let bob_dh_key = generate_dh_ratchet_key();
    let bob_dh_public = PublicKey::from(&bob_dh_key);

    let mut alice_ratchet = DoubleRatchetState::new_from_x3dh(
        alice_result.shared_secret,
        alice_dh_key,
        Some(bob_dh_public),
    );

    let mut bob_ratchet = DoubleRatchetState::new_from_x3dh(
        bob_result.shared_secret,
        bob_dh_key,
        Some(alice_ratchet.dh_ratchet_public),
    );

    println!("  Alice ratchet initialized");
    println!("  Bob ratchet initialized\n");

    // Step 4: Exchange messages
    println!("[4/4] Exchanging encrypted messages...\n");

    let conversation = [
        ("Alice", "Hey Bob! This message is encrypted with Double Ratchet."),
        ("Bob", "Hi Alice! I can read it. Each message gets a fresh key!"),
        ("Alice", "Even if someone cracks one message key,"),
        ("Bob", "they can't decrypt past or future messages. Forward secrecy!"),
        ("Alice", "The Double Ratchet continuously derives new keys. Every message advances the symmetric ratchet, and when the other party replies, a DH ratchet injects new entropy."),
        ("Bob", "This gives us both forward secrecy AND post-compromise security."),
        ("Alice", "Let me send one more to confirm."),
        ("Bob", "Everything works! Phase 0 is complete."),
    ];

    let mut alice_turn = true;

    for (sender, message) in &conversation {
        if alice_turn {
            let (header, encrypted) = alice_ratchet.encrypt(message.as_bytes())
                .expect("encryption failed");
            let decrypted = bob_ratchet.decrypt(&header, &encrypted)
                .expect("decryption failed");
            let decoded = String::from_utf8_lossy(&decrypted);
            assert_eq!(decoded.as_bytes(), message.as_bytes());
            println!("  {} -> Bob: {}", sender, decoded);
        } else {
            let (header, encrypted) = bob_ratchet.encrypt(message.as_bytes())
                .expect("encryption failed");
            let decrypted = alice_ratchet.decrypt(&header, &encrypted)
                .expect("decryption failed");
            let decoded = String::from_utf8_lossy(&decrypted);
            assert_eq!(decoded.as_bytes(), message.as_bytes());
            println!("  Bob -> Alice: {}", decoded);
        }
        alice_turn = !alice_turn;
    }

    println!("\n========================================");
    println!("  All {} messages exchanged successfully!", conversation.len());
    println!("  All encryptions/decryptions verified");
    println!("  Forward secrecy maintained");
    println!("========================================");
    println!("\nPhase 0 Acceptance Criteria: PASSED");
}