//! X3DH (Extended Triple Diffie-Hellman) key agreement protocol

use x25519_dalek::{PublicKey, StaticSecret};
use hkdf::Hkdf;
use sha2::Sha256;
use rand_core::OsRng;

/// Error type for X3DH operations
#[derive(Debug)]
pub enum X3DHError {
    InvalidPublicKey,
    KeyDerivationFailed,
    InvalidPrekeyBundle,
}

impl std::fmt::Display for X3DHError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            X3DHError::InvalidPublicKey => write!(f, "Invalid public key"),
            X3DHError::KeyDerivationFailed => write!(f, "Key derivation failed"),
            X3DHError::InvalidPrekeyBundle => write!(f, "Invalid prekey bundle"),
        }
    }
}

impl std::error::Error for X3DHError {}

/// Perform X3DH key agreement (initiator side)
pub fn perform_x3dh_initiator(
    our_identity_key: &StaticSecret,
    our_ephemeral_key: &StaticSecret,
    their_identity_key: &PublicKey,
    their_signed_prekey: &PublicKey,
    their_one_time_prekey: Option<&PublicKey>,
) -> Result<[u8; 32], X3DHError> {
    // DH1 = DH(I_A, SPK_B)
    let dh1 = our_identity_key.diffie_hellman(their_signed_prekey);
    
    // DH2 = DH(E_A, IK_B)
    let dh2 = our_ephemeral_key.diffie_hellman(their_identity_key);
    
    // DH3 = DH(E_A, SPK_B)
    let dh3 = our_ephemeral_key.diffie_hellman(their_signed_prekey);
    
    // Prepare the concatenated DH outputs
    let mut dh_outputs = Vec::new();
    dh_outputs.extend_from_slice(dh1.as_bytes());
    dh_outputs.extend_from_slice(dh2.as_bytes());
    dh_outputs.extend_from_slice(dh3.as_bytes());
    
    // DH4 = DH(E_A, OPK_B) if available
    if let Some(their_opk) = their_one_time_prekey {
        let dh4 = our_ephemeral_key.diffie_hellman(their_opk);
        dh_outputs.extend_from_slice(dh4.as_bytes());
    }
    
    // Use HKDF to derive the shared secret
    let hkdf = Hkdf::<Sha256>::new(None, &dh_outputs);
    let mut shared_secret = [0u8; 32];
    hkdf.expand(b"SCP-X3DH", &mut shared_secret)
        .map_err(|_| X3DHError::KeyDerivationFailed)?;
    
    Ok(shared_secret)
}

/// Generate an ephemeral key for X3DH
pub fn generate_ephemeral_key() -> StaticSecret {
    let mut rng = OsRng;
    StaticSecret::random_from_rng(&mut rng)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_x3dh_basic() {
        // Generate keys
        let mut rng = OsRng;
        
        // Alice's keys
        let alice_identity = StaticSecret::random_from_rng(&mut rng);
        let alice_ephemeral = generate_ephemeral_key();
        
        // Bob's keys
        let bob_identity = StaticSecret::random_from_rng(&mut rng);
        let bob_signed_prekey = StaticSecret::random_from_rng(&mut rng);
        let bob_one_time_prekey = StaticSecret::random_from_rng(&mut rng);
        
        // Get public keys
        let bob_identity_pub = PublicKey::from(&bob_identity);
        let bob_signed_prekey_pub = PublicKey::from(&bob_signed_prekey);
        let bob_one_time_prekey_pub = PublicKey::from(&bob_one_time_prekey);
        
        // Alice initiates X3DH with Bob
        let alice_shared = perform_x3dh_initiator(
            &alice_identity,
            &alice_ephemeral,
            &bob_identity_pub,
            &bob_signed_prekey_pub,
            Some(&bob_one_time_prekey_pub),
        ).unwrap();
        
        // Verify shared secret is 32 bytes
        assert_eq!(alice_shared.len(), 32);
        
        // Verify it's not all zeros
        assert_ne!(alice_shared, [0u8; 32]);
        
        println!("X3DH test passed! Shared secret generated successfully.");
    }
}