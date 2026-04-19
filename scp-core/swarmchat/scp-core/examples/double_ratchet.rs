use scp_core::crypto::double_ratchet::{DoubleRatchetState, generate_dh_ratchet_key};
use x25519_dalek::PublicKey;

fn main() {
    println!("=== Testing Double Ratchet Algorithm ===\n");
    
    // Generate initial X3DH shared secret
    let shared_secret = [0x42u8; 32];
    println!("Shared secret established via X3DH");
    
    // Alice's setup
    let alice_dh_key = generate_dh_ratchet_key();
    let bob_dh_key = generate_dh_ratchet_key();
    let bob_dh_public = PublicKey::from(&bob_dh_key);
    
    let mut alice_state = DoubleRatchetState::new_from_x3dh(
        shared_secret,
        alice_dh_key,
        Some(bob_dh_public),
    );
    println!("Alice initialized Double Ratchet");
    
    // Bob's setup
    let mut bob_state = DoubleRatchetState::new_from_x3dh(
        shared_secret,
        bob_dh_key,
        Some(alice_state.dh_ratchet_public),
    );
    println!("Bob initialized Double Ratchet");
    
    // Test 1: Alice sends first message
    println!("\n--- Test 1: Alice sends first message ---");
    let plaintext1 = b"Hello Bob! This is Alice.";
    println!("Plaintext: {}", String::from_utf8_lossy(plaintext1));
    
    let (header1, encrypted1) = alice_state.encrypt(plaintext1).unwrap();
    println!("Encrypted message length: {} bytes", encrypted1.len());
    
    let decrypted1 = bob_state.decrypt(&header1, &encrypted1).unwrap();
    println!("Decrypted: {}", String::from_utf8_lossy(&decrypted1));
    assert_eq!(decrypted1, plaintext1);
    println!("✓ Test 1 passed!");
    
    // Test 2: Bob replies
    println!("\n--- Test 2: Bob replies ---");
    let plaintext2 = b"Hi Alice! This is Bob.";
    println!("Plaintext: {}", String::from_utf8_lossy(plaintext2));
    
    let (header2, encrypted2) = bob_state.encrypt(plaintext2).unwrap();
    println!("Encrypted message length: {} bytes", encrypted2.len());
    
    let decrypted2 = alice_state.decrypt(&header2, &encrypted2).unwrap();
    println!("Decrypted: {}", String::from_utf8_lossy(&decrypted2));
    assert_eq!(decrypted2, plaintext2);
    println!("✓ Test 2 passed!");
    
    // Test 3: Multiple messages
    println!("\n--- Test 3: Multiple messages exchange ---");
    let messages = [
        b"Message 1 from Alice",
        b"Message 2 from Alice", 
        b"Message 3 from Alice",
    ];
    
    for (i, message) in messages.iter().enumerate() {
        println!("\nMessage {}:", i + 1);
        println!("  Plaintext: {}", String::from_utf8_lossy(message));
        
        let (header, encrypted) = alice_state.encrypt(message).unwrap();
        println!("  Encrypted: {} bytes", encrypted.len());
        
        let decrypted = bob_state.decrypt(&header, &encrypted).unwrap();
        println!("  Decrypted: {}", String::from_utf8_lossy(&decrypted));
        assert_eq!(decrypted, *message);
    }
    
    println!("\n=== All Double Ratchet tests passed! ===");
}