use scp_core::identity::{Identity, generate_mnemonic};
use std::path::Path;

fn main() {
    println!("=== SwarmChat Identity Module Demo ===\n");
    
    // 1. Generate a new identity
    println!("1. Generating new identity...");
    let identity = Identity::new().expect("Failed to generate identity");
    
    println!("   Mnemonic (24 words):");
    println!("   {}", identity.mnemonic_phrase());
    println!("   Seed (hex): {}", identity.seed_hex());
    println!("   Seed length: {} bytes", identity.seed.len());
    
    // 2. Save identity to file
    println!("\n2. Saving identity to file...");
    let path = Path::new("test_identity.json");
    identity.save_to_file(path)
        .expect("Failed to save identity");
    println!("   Saved to: {:?}", path);
    
    // 3. Load identity from file
    println!("\n3. Loading identity from file...");
    let loaded_identity = Identity::load_from_file(path)
        .expect("Failed to load identity");
    
    // Verify loaded identity matches original
    assert_eq!(identity.mnemonic_phrase(), loaded_identity.mnemonic_phrase());
    assert_eq!(identity.seed_hex(), loaded_identity.seed_hex());
    println!("   ✓ Identity loaded successfully");
    
    // 4. Generate mnemonic separately
    println!("\n4. Generating standalone mnemonic...");
    let mnemonic = generate_mnemonic().expect("Failed to generate mnemonic");
    println!("   Generated mnemonic: {}", mnemonic.to_string());
    
    // 5. Create identity from existing mnemonic
    println!("\n5. Creating identity from existing mnemonic...");
    let identity2 = Identity::from_mnemonic(&mnemonic)
        .expect("Failed to create identity from mnemonic");
    println!("   ✓ Identity created from mnemonic");
    println!("   Seed hex: {}", identity2.seed_hex());
    
    // Clean up
    println!("\n6. Cleaning up...");
    std::fs::remove_file(path).expect("Failed to remove test file");
    println!("   ✓ Test file removed");
    
    println!("\n=== Identity Module Test Complete ===");
    println!("All tests passed successfully!");
}