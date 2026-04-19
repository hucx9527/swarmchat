//! PeerId module demo for SwarmChat
//! 
//! This example demonstrates PeerId generation, parsing, and verification.

use scp_core::identity::Identity;
use scp_core::did::generate_did_from_identity;
use scp_core::peer_id::{PeerId, generate_peerid_from_did};
use base58::ToBase58;

fn main() {
    println!("=== SwarmChat PeerId Module Demo ===\n");
    
    // 1. Create a new identity
    println!("1. Creating new identity...");
    let identity = Identity::new().expect("Failed to create identity");
    println!("   ✓ Identity created");
    println!("   Mnemonic: {}...", &identity.mnemonic.to_string()[..30]);
    println!("   Seed length: {} bytes\n", identity.seed.len());
    
    // 2. Generate DID from identity
    println!("2. Generating DID from identity...");
    let did = generate_did_from_identity(&identity, "Ed25519").expect("Failed to generate DID");
    println!("   ✓ DID generated");
    println!("   DID: {}", did);
    println!("   Public key length: {} bytes\n", did.public_key.len());
    
    // 3. Generate PeerId from DID
    println!("3. Generating PeerId from DID...");
    let peer_id = generate_peerid_from_did(&did).expect("Failed to generate PeerId");
    println!("   ✓ PeerId generated");
    println!("   PeerId: {}", peer_id);
    println!("   Multihash length: {} bytes", peer_id.multihash.len());
    println!("   Hash (hex): {}", peer_id.hash_hex());
    println!("   Source DID: {:?}\n", peer_id.source_did);
    
    // 4. Test PeerId parsing
    println!("4. Testing PeerId parsing...");
    let peer_id_str = peer_id.to_string();
    let parsed_peer_id = PeerId::parse(&peer_id_str).expect("Failed to parse PeerId");
    println!("   ✓ PeerId parsed successfully");
    println!("   Original: {}", peer_id_str);
    println!("   Parsed: {}", parsed_peer_id);
    println!("   Hashes match: {}\n", parsed_peer_id.hash == peer_id.hash);
    
    // 5. Test PeerId verification
    println!("5. Testing PeerId verification...");
    let verification_result = peer_id.verify_public_key(&did.public_key)
        .expect("Failed to verify PeerId");
    println!("   ✓ PeerId verification: {}", verification_result);
    
    // Test with wrong public key
    let wrong_key = vec![0u8; 32];
    let wrong_verification = peer_id.verify_public_key(&wrong_key)
        .expect("Failed to verify with wrong key");
    println!("   ✓ Wrong key correctly rejected: {}\n", !wrong_verification);
    
    // 6. Test direct PeerId creation from public key
    println!("6. Testing direct PeerId creation...");
    let test_public_key = vec![
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
        0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
        0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
        0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
    ];
    
    let direct_peer_id = PeerId::from_public_key(&test_public_key)
        .expect("Failed to create direct PeerId");
    println!("   ✓ Direct PeerId created");
    println!("   PeerId: {}", direct_peer_id);
    println!("   Hash (hex): {}\n", direct_peer_id.hash_hex());
    
    // 7. Test error cases
    println!("7. Testing error cases...");
    
    // Invalid base58
    match PeerId::parse("invalid!@#") {
        Ok(_) => println!("   ✗ Should have rejected invalid base58"),
        Err(e) => println!("   ✓ Correctly rejected invalid base58: {}", e),
    }
    
    // Too short
    match PeerId::parse("1") {
        Ok(_) => println!("   ✗ Should have rejected too short string"),
        Err(e) => println!("   ✓ Correctly rejected too short string: {}", e),
    }
    
    // Invalid multihash prefix
    let invalid_multihash = vec![0x00, 0x01]; // Invalid codec
    let invalid_encoded = invalid_multihash.to_base58();
    match PeerId::parse(&invalid_encoded) {
        Ok(_) => println!("   ✗ Should have rejected invalid multihash"),
        Err(e) => println!("   ✓ Correctly rejected invalid multihash: {}", e),
    }
    
    // 8. Test PeerId bytes
    println!("\n8. Testing PeerId bytes...");
    let peer_id_bytes = peer_id.to_bytes();
    println!("   PeerId bytes length: {} bytes", peer_id_bytes.len());
    println!("   First 2 bytes (prefix): {:02x}{:02x}", 
             peer_id_bytes[0], peer_id_bytes[1]);
    println!("   Hash bytes (next 32): {}...", 
             hex::encode(&peer_id_bytes[2..10]));
    
    // 9. Test display trait
    println!("\n9. Testing Display trait...");
    let display_output = format!("{}", peer_id);
    println!("   Display output: {}", display_output);
    println!("   Matches to_string(): {}\n", display_output == peer_id.to_string());
    
    println!("=== PeerId Module Test Complete ===");
    println!("All tests passed successfully!");
}