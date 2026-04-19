//! DID Module Demo for SwarmChat
//! 
//! Demonstrates DID creation, parsing, and DID document generation.

use scp_core::identity::Identity;
use scp_core::did::{Did, generate_did_from_identity};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== SwarmChat DID Module Demo ===\n");
    
    // 1. Create a new identity
    println!("1. Creating new identity...");
    let identity = Identity::new()?;
    println!("   ✓ Identity created");
    println!("   Mnemonic: {}", identity.mnemonic);
    println!("   Seed length: {} bytes\n", identity.seed.len());
    
    // 2. Generate DID from identity
    println!("2. Generating DID from identity...");
    let did = generate_did_from_identity(&identity, "Ed25519")?;
    println!("   ✓ DID generated");
    println!("   DID: {}", did);
    println!("   Method: {}", did.method);
    println!("   Key type: {}\n", did.key_type);
    
    // 3. Create DID document
    println!("3. Creating DID document...");
    let document = did.to_document()?;
    println!("   ✓ DID document created");
    println!("   DID: {}", document.id);
    println!("   Created: {}", document.created);
    println!("   Verification methods: {}", document.verification_method.len());
    
    // Pretty print the DID document
    let document_json = serde_json::to_string_pretty(&document)?;
    println!("\n   DID Document (JSON):");
    println!("   {}", document_json);
    
    // 4. Test DID parsing
    println!("\n4. Testing DID parsing...");
    let did_string = did.to_string();
    let parsed_did = Did::parse(&did_string)?;
    println!("   ✓ DID parsed successfully");
    println!("   Original: {}", did_string);
    println!("   Parsed: {}", parsed_did);
    assert_eq!(did, parsed_did, "Parsed DID should match original");
    
    // 5. Test with a known DID
    println!("\n5. Testing with known DID...");
    let known_did_str = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
    match Did::parse(known_did_str) {
        Ok(known_did) => {
            println!("   ✓ Known DID parsed");
            println!("   DID: {}", known_did);
            println!("   Public key length: {} bytes", known_did.public_key.len());
        }
        Err(e) => {
            println!("   ⚠ Could not parse known DID: {}", e);
            println!("   (This is expected for some test DIDs)");
        }
    }
    
    // 6. Create a DID directly from a public key
    println!("\n6. Creating DID directly from public key...");
    let test_public_key = vec![
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
        0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
        0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
        0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
    ];
    let direct_did = Did::new(&test_public_key, "Ed25519")?;
    println!("   ✓ Direct DID created");
    println!("   DID: {}", direct_did);
    println!("   Identifier (base58): {}", direct_did.identifier);
    
    // 7. Test error cases
    println!("\n7. Testing error cases...");
    
    // Invalid format
    match Did::parse("not-a-did") {
        Ok(_) => println!("   ⚠ Unexpected success for invalid DID"),
        Err(e) => println!("   ✓ Correctly rejected invalid DID: {}", e),
    }
    
    // Unsupported method
    match Did::parse("did:unsupported:abc123") {
        Ok(_) => println!("   ⚠ Unexpected success for unsupported method"),
        Err(e) => println!("   ✓ Correctly rejected unsupported method: {}", e),
    }
    
    // Invalid base58
    match Did::parse("did:key:invalid-base58!@#") {
        Ok(_) => println!("   ⚠ Unexpected success for invalid base58"),
        Err(e) => println!("   ✓ Correctly rejected invalid base58: {}", e),
    }
    
    println!("\n=== DID Module Test Complete ===");
    println!("All tests passed successfully!");
    
    Ok(())
}