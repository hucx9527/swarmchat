//! X3DH (Extended Triple Diffie-Hellman) key agreement protocol
//!
//! Implementation of X3DH as specified in SCP section 4.5.1.
//! Supports both initiator and responder sides, producing a 32-byte
//! shared secret that feeds into the Double Ratchet algorithm.

use x25519_dalek::{PublicKey, StaticSecret};
use hkdf::Hkdf;
use sha2::Sha256;
use rand_core::OsRng;

/// Result of an X3DH key agreement exchange
#[derive(Debug, Clone)]
pub struct X3DHResult {
    /// The 32-byte shared secret (SK)
    pub shared_secret: [u8; 32],
    /// Ephemeral public key generated during this exchange
    pub ephemeral_public_key: PublicKey,
    /// Additional associated data (optional, for protocol binding)
    pub associated_data: Option<Vec<u8>>,
}

/// Error type for X3DH operations
#[derive(Debug)]
pub enum X3DHError {
    InvalidPublicKey,
    KeyDerivationFailed,
    InvalidPrekeyBundle,
    MissingRequiredPrekey,
}

impl std::fmt::Display for X3DHError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            X3DHError::InvalidPublicKey => write!(f, "Invalid public key"),
            X3DHError::KeyDerivationFailed => write!(f, "Key derivation failed"),
            X3DHError::InvalidPrekeyBundle => write!(f, "Invalid prekey bundle"),
            X3DHError::MissingRequiredPrekey => write!(f, "Missing required prekey"),
        }
    }
}

impl std::error::Error for X3DHError {}

/// Perform X3DH key agreement — initiator side.
///
/// This is the active side that initiates a session.
///
/// # Parameters (Alice — initiator)
/// - `our_identity_key`: Alice's long-term identity key (IK_A)
/// - `our_signed_prekey`: Alice's signed prekey (SPK_A)
/// - `our_one_time_prekey`: Alice's one-time prekey, if available (OPK_A)
/// - `their_identity_key`: Bob's identity public key (IK_B)
/// - `their_signed_prekey`: Bob's signed prekey public key (SPK_B)
/// - `their_one_time_prekey`: Bob's one-time prekey public key, if available (OPK_B)
///
/// # DH calculations
/// - DH1 = DH(IK_A, SPK_B)
/// - DH2 = DH(EK_A, IK_B)  — EK_A is freshly generated here
/// - DH3 = DH(EK_A, SPK_B)
/// - DH4 = DH(EK_A, OPK_B) — if OPK_B is available
pub fn perform_x3dh_initiator(
    our_identity_key: &StaticSecret,
    _our_signed_prekey: &StaticSecret,
    _our_one_time_prekey: Option<&StaticSecret>,
    their_identity_key: &PublicKey,
    their_signed_prekey: &PublicKey,
    their_one_time_prekey: Option<&PublicKey>,
) -> Result<X3DHResult, X3DHError> {
    // Generate fresh ephemeral key for this session
    let mut rng = OsRng;
    let ephemeral_key = StaticSecret::random_from_rng(&mut rng);
    let ephemeral_public = PublicKey::from(&ephemeral_key);

    // DH1 = DH(IK_A, SPK_B)
    let dh1 = our_identity_key.diffie_hellman(their_signed_prekey);

    // DH2 = DH(EK_A, IK_B)
    let dh2 = ephemeral_key.diffie_hellman(their_identity_key);

    // DH3 = DH(EK_A, SPK_B)
    let dh3 = ephemeral_key.diffie_hellman(their_signed_prekey);

    // Concatenate DH outputs
    let mut dh_outputs = Vec::new();
    dh_outputs.extend_from_slice(dh1.as_bytes());
    dh_outputs.extend_from_slice(dh2.as_bytes());
    dh_outputs.extend_from_slice(dh3.as_bytes());

    // DH4 = DH(EK_A, OPK_B) if available
    if let Some(their_opk) = their_one_time_prekey {
        let dh4 = ephemeral_key.diffie_hellman(their_opk);
        dh_outputs.extend_from_slice(dh4.as_bytes());
    }

    // HKDF to derive shared secret
    let hkdf = Hkdf::<Sha256>::new(None, &dh_outputs);
    let mut shared_secret = [0u8; 32];
    hkdf.expand(b"SCP-X3DH", &mut shared_secret)
        .map_err(|_| X3DHError::KeyDerivationFailed)?;

    Ok(X3DHResult {
        shared_secret,
        ephemeral_public_key: ephemeral_public,
        associated_data: None,
    })
}

/// Perform X3DH key agreement — responder side.
///
/// This is the passive side that responds to an initiation request.
///
/// # Parameters (Bob — responder)
/// - `our_identity_key`: Bob's long-term identity key (IK_B)
/// - `our_signed_prekey`: Bob's signed prekey (SPK_B)
/// - `our_one_time_prekey`: Bob's one-time prekey, if available (OPK_B)
/// - `their_identity_key`: Alice's identity public key (IK_A)
/// - `their_signed_prekey`: Alice's signed prekey public key (SPK_A)
/// - `their_one_time_prekey`: Alice's one-time prekey public key, if available (OPK_A)
/// - `their_ephemeral_key`: Alice's ephemeral public key from the initiation (EK_A)
///
/// # DH calculations
/// - DH1 = DH(SPK_B, IK_A)
/// - DH2 = DH(IK_B, EK_A)
/// - DH3 = DH(SPK_B, EK_A)
/// - DH4 = DH(OPK_B, EK_A) — if OPK_B was provided
pub fn perform_x3dh_responder(
    our_identity_key: &StaticSecret,
    our_signed_prekey: &StaticSecret,
    our_one_time_prekey: Option<&StaticSecret>,
    their_identity_key: &PublicKey,
    _their_signed_prekey: &PublicKey,
    _their_one_time_prekey: Option<&PublicKey>,
    their_ephemeral_key: &PublicKey,
) -> Result<X3DHResult, X3DHError> {
    // DH1 = DH(SPK_B, IK_A)
    let dh1 = our_signed_prekey.diffie_hellman(their_identity_key);

    // DH2 = DH(IK_B, EK_A)
    let dh2 = our_identity_key.diffie_hellman(their_ephemeral_key);

    // DH3 = DH(SPK_B, EK_A)
    let dh3 = our_signed_prekey.diffie_hellman(their_ephemeral_key);

    // Concatenate DH outputs
    let mut dh_outputs = Vec::new();
    dh_outputs.extend_from_slice(dh1.as_bytes());
    dh_outputs.extend_from_slice(dh2.as_bytes());
    dh_outputs.extend_from_slice(dh3.as_bytes());

    // DH4 = DH(OPK_B, EK_A) if prekey was provided
    if let Some(opk) = our_one_time_prekey {
        let dh4 = opk.diffie_hellman(their_ephemeral_key);
        dh_outputs.extend_from_slice(dh4.as_bytes());
    }

    // HKDF to derive shared secret
    let hkdf = Hkdf::<Sha256>::new(None, &dh_outputs);
    let mut shared_secret = [0u8; 32];
    hkdf.expand(b"SCP-X3DH", &mut shared_secret)
        .map_err(|_| X3DHError::KeyDerivationFailed)?;

    // Responder doesn't generate a new ephemeral key;
    // it returns its identity public for protocol consistency.
    let identity_public = PublicKey::from(our_identity_key);

    Ok(X3DHResult {
        shared_secret,
        ephemeral_public_key: identity_public,
        associated_data: None,
    })
}

/// Generate a random ephemeral key for X3DH
pub fn generate_ephemeral_key() -> StaticSecret {
    let mut rng = OsRng;
    StaticSecret::random_from_rng(&mut rng)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_x3dh_full_exchange() {
        let mut rng = OsRng;

        // ── Key generation ──

        // Alice's keys (initiator)
        let alice_identity = StaticSecret::random_from_rng(&mut rng);
        let alice_signed_prekey = StaticSecret::random_from_rng(&mut rng);
        let alice_one_time_prekey = Some(StaticSecret::random_from_rng(&mut rng));

        // Bob's keys (responder)
        let bob_identity = StaticSecret::random_from_rng(&mut rng);
        let bob_signed_prekey = StaticSecret::random_from_rng(&mut rng);
        let bob_one_time_prekey = Some(StaticSecret::random_from_rng(&mut rng));

        // Public keys
        let alice_identity_pub = PublicKey::from(&alice_identity);
        let alice_signed_prekey_pub = PublicKey::from(&alice_signed_prekey);
        let alice_one_time_prekey_pub = alice_one_time_prekey.as_ref().map(|k| PublicKey::from(k));

        let bob_identity_pub = PublicKey::from(&bob_identity);
        let bob_signed_prekey_pub = PublicKey::from(&bob_signed_prekey);
        let bob_one_time_prekey_pub = bob_one_time_prekey.as_ref().map(|k| PublicKey::from(k));

        // ── Alice initiates ──
        let alice_result = perform_x3dh_initiator(
            &alice_identity,
            &alice_signed_prekey,
            alice_one_time_prekey.as_ref(),
            &bob_identity_pub,
            &bob_signed_prekey_pub,
            bob_one_time_prekey_pub.as_ref(),
        ).unwrap();

        assert_eq!(alice_result.shared_secret.len(), 32);
        assert_ne!(alice_result.shared_secret, [0u8; 32]);

        // ── Bob responds ──
        let bob_result = perform_x3dh_responder(
            &bob_identity,
            &bob_signed_prekey,
            bob_one_time_prekey.as_ref(),
            &alice_identity_pub,
            &alice_signed_prekey_pub,
            alice_one_time_prekey_pub.as_ref(),
            &alice_result.ephemeral_public_key,
        ).unwrap();

        assert_eq!(bob_result.shared_secret.len(), 32);

        // ── Both sides should agree on the shared secret ──
        assert_eq!(alice_result.shared_secret, bob_result.shared_secret);

        println!("X3DH full exchange test passed!");
    }

    #[test]
    fn test_x3dh_with_one_time_prekey() {
        let mut rng = OsRng;

        let alice_identity = StaticSecret::random_from_rng(&mut rng);
        let alice_signed_prekey = StaticSecret::random_from_rng(&mut rng);

        let bob_identity = StaticSecret::random_from_rng(&mut rng);
        let bob_signed_prekey = StaticSecret::random_from_rng(&mut rng);
        let bob_one_time_prekey = Some(StaticSecret::random_from_rng(&mut rng));

        let alice_identity_pub = PublicKey::from(&alice_identity);
        let alice_signed_prekey_pub = PublicKey::from(&alice_signed_prekey);

        let bob_identity_pub = PublicKey::from(&bob_identity);
        let bob_signed_prekey_pub = PublicKey::from(&bob_signed_prekey);
        let bob_one_time_prekey_pub = bob_one_time_prekey.as_ref().map(|k| PublicKey::from(k));

        let alice_result = perform_x3dh_initiator(
            &alice_identity,
            &alice_signed_prekey,
            None,
            &bob_identity_pub,
            &bob_signed_prekey_pub,
            bob_one_time_prekey_pub.as_ref(),
        ).unwrap();

        let bob_result = perform_x3dh_responder(
            &bob_identity,
            &bob_signed_prekey,
            bob_one_time_prekey.as_ref(),
            &alice_identity_pub,
            &alice_signed_prekey_pub,
            None,
            &alice_result.ephemeral_public_key,
        ).unwrap();

        assert_eq!(alice_result.shared_secret, bob_result.shared_secret);
        println!("X3DH with one-time prekey test passed!");
    }

    #[test]
    fn test_x3dh_no_one_time_prekey() {
        let mut rng = OsRng;

        let alice_identity = StaticSecret::random_from_rng(&mut rng);
        let alice_signed_prekey = StaticSecret::random_from_rng(&mut rng);

        let bob_identity = StaticSecret::random_from_rng(&mut rng);
        let bob_signed_prekey = StaticSecret::random_from_rng(&mut rng);

        let alice_identity_pub = PublicKey::from(&alice_identity);
        let alice_signed_prekey_pub = PublicKey::from(&alice_signed_prekey);

        let bob_identity_pub = PublicKey::from(&bob_identity);
        let bob_signed_prekey_pub = PublicKey::from(&bob_signed_prekey);

        // No one-time prekeys
        let alice_result = perform_x3dh_initiator(
            &alice_identity,
            &alice_signed_prekey,
            None,
            &bob_identity_pub,
            &bob_signed_prekey_pub,
            None,
        ).unwrap();

        let bob_result = perform_x3dh_responder(
            &bob_identity,
            &bob_signed_prekey,
            None,
            &alice_identity_pub,
            &alice_signed_prekey_pub,
            None,
            &alice_result.ephemeral_public_key,
        ).unwrap();

        assert_eq!(alice_result.shared_secret, bob_result.shared_secret);
        println!("X3DH without one-time prekey test passed!");
    }

    #[test]
    fn test_shared_secret_uniqueness() {
        let mut rng = OsRng;

        // First exchange
        let ik_a1 = StaticSecret::random_from_rng(&mut rng);
        let spk_a1 = StaticSecret::random_from_rng(&mut rng);
        let ik_b1 = StaticSecret::random_from_rng(&mut rng);
        let spk_b1 = StaticSecret::random_from_rng(&mut rng);

        let result1 = perform_x3dh_initiator(
            &ik_a1, &spk_a1, None,
            &PublicKey::from(&ik_b1),
            &PublicKey::from(&spk_b1),
            None,
        ).unwrap();

        // Second exchange with different keys
        let ik_a2 = StaticSecret::random_from_rng(&mut rng);
        let spk_a2 = StaticSecret::random_from_rng(&mut rng);
        let ik_b2 = StaticSecret::random_from_rng(&mut rng);
        let spk_b2 = StaticSecret::random_from_rng(&mut rng);

        let result2 = perform_x3dh_initiator(
            &ik_a2, &spk_a2, None,
            &PublicKey::from(&ik_b2),
            &PublicKey::from(&spk_b2),
            None,
        ).unwrap();

        // Different keys should produce different secrets
        assert_ne!(result1.shared_secret, result2.shared_secret);
        println!("X3DH shared secret uniqueness test passed!");
    }
}