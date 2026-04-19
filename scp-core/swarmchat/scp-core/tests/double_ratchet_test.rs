use scp_core::crypto::double_ratchet::{DoubleRatchetState, generate_dh_ratchet_key};
use x25519_dalek::PublicKey;

#[test]
fn test_double_ratchet_integration() {
    println!("Testing Double Ratchet integration...");
    
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
    println!("Alice sent message 1");
    
    // Bob decrypts
    let decrypted1 = bob_state.decrypt(&header1, &encrypted1).unwrap();
    assert_eq!(decrypted1, plaintext1);
    println!("Bob decrypted message 1");
    
    // Bob sends reply
    let plaintext2 = b"Hello Alice!";
    let (header2, encrypted2) = bob_state.encrypt(plaintext2).unwrap();
    println!("Bob sent message 2");
    
    // Alice decrypts
    let decrypted2 = alice_state.decrypt(&header2, &encrypted2).unwrap();
    assert_eq!(decrypted2, plaintext2);
    println!("Alice decrypted message 2");
    
    // Alice sends another message
    let plaintext3 = b"How are you?";
    let (header3, encrypted3) = alice_state.encrypt(plaintext3).unwrap();
    println!("Alice sent message 3");
    
    // Bob decrypts
    let decrypted3 = bob_state.decrypt(&header3, &encrypted3).unwrap();
    assert_eq!(decrypted3, plaintext3);
    println!("Bob decrypted message 3");
    
    // Test multiple messages in sequence
    for i in 0..5 {
        let message = format!("Message {}", i);
        let (header, encrypted) = alice_state.encrypt(message.as_bytes()).unwrap();
        let decrypted = bob_state.decrypt(&header, &encrypted).unwrap();
        assert_eq!(decrypted, message.as_bytes());
        println!("Message {} passed", i);
    }
    
    println!("Double Ratchet integration test passed!");
}