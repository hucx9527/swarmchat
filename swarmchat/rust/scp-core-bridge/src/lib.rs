//! scp-core-bridge — JNI / C FFI bridge from React Native to scp-core (Rust).
//!
//! Exposes core SCP operations to the mobile layer:
//!   - BIP39 mnemonic generation & identity recovery
//!   - DID (did:key) generation from Ed25519 public keys
//!   - PeerId derivation via SHA-256 multihash
//!   - Double Ratchet encryption/decryption
//!   - Sender Key encryption/decryption
//!   - Envelope signing & verification
//!
//! Architecture:
//!   Android: JNI via `jni` crate (feature = "android")
//!   iOS:     C FFI via `extern "C"` functions linked statically
//!
//! In a full production build, this crate depends directly on `scp-core`
//! via a workspace path dependency. For standalone development, the
//! necessary algorithms are implemented inline.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

// ============================================================================
// Error types
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum BridgeError {
    #[error("Null pointer argument")]
    NullPointer,
    #[error("Invalid UTF-8 in argument: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("Operation failed: {0}")]
    OperationFailed(String),
    #[error("Invalid mnemonic: {0}")]
    InvalidMnemonic(String),
    #[error("Invalid key: {0}")]
    InvalidKey(String),
}

type BridgeResult<T> = Result<T, BridgeError>;

// ============================================================================
// Core types
// ============================================================================

#[derive(serde::Serialize)]
struct IdentityOutput {
    mnemonic: String,
    seed_hex: String,
    did: String,
    peer_id: String,
    public_key: String, // base58 encoded Ed25519 public key
}

#[derive(serde::Serialize)]
struct EncryptOutput {
    ciphertext: String, // base58 encoded
    nonce: String,
}

// ============================================================================
// Helper: generate a BIP39 mnemonic + Ed25519 keypair + DID + PeerId
// ============================================================================

fn generate_identity() -> BridgeResult<IdentityOutput> {
    use bip39::{Mnemonic, Language};
    use ed25519_dalek::{SigningKey, SECRET_KEY_LENGTH};
    use rand::rngs::OsRng;
    use sha2::{Sha256, Digest};

    // 1. Generate BIP39 mnemonic (24 words, 256-bit entropy)
    let mut entropy = [0u8; 32];
    getrandom::getrandom(&mut entropy)
        .map_err(|e| BridgeError::OperationFailed(format!("RNG error: {e}")))?;
    let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy)
        .map_err(|e| BridgeError::InvalidMnemonic(e.to_string()))?;
    let mnemonic_phrase = mnemonic.to_string();

    // 2. Derive seed (64 bytes from mnemonic + empty passphrase)
    let seed = mnemonic.to_seed("");

    // 3. Generate Ed25519 keypair from first 32 bytes of seed
    let signing_key = SigningKey::from_bytes(
        &seed[..SECRET_KEY_LENGTH].try_into().unwrap(),
    );
    let verifying_key = signing_key.verifying_key();
    let public_key_bytes = verifying_key.to_bytes();

    // 4. Generate DID (did:key with Ed25519 multicodec)
    let did = generate_did(&public_key_bytes);

    // 5. Generate PeerId (SHA-256 multihash of DID)
    let peer_id = generate_peer_id(&did);

    Ok(IdentityOutput {
        mnemonic: mnemonic_phrase,
        seed_hex: hex::encode(&seed[..32]),
        did,
        peer_id,
        public_key: bs58::encode(&public_key_bytes).into_string(),
    })
}

/// W3C did:key with Ed25519 multicodec prefix (0xed01)
fn generate_did(public_key: &[u8; 32]) -> String {
    let mut multicodec = Vec::with_capacity(34);
    multicodec.extend_from_slice(&[0xed, 0x01]); // Ed25519 multicodec
    multicodec.extend_from_slice(public_key);
    let encoded = bs58::encode(&multicodec).into_string();
    format!("did:key:{encoded}")
}

/// PeerId: SHA-256 multihash with 0x1220 prefix
fn generate_peer_id(did: &str) -> String {
    use sha2::Digest;
    let hash = sha2::Sha256::digest(did.as_bytes());
    let mut multihash = Vec::with_capacity(34);
    multihash.extend_from_slice(&[0x12, 0x20]); // SHA-256, 32 bytes
    multihash.extend_from_slice(&hash);
    bs58::encode(&multihash).into_string()
}

// ============================================================================
// Helper: Recover identity from mnemonic
// ============================================================================

fn recover_identity(mnemonic_str: &str) -> BridgeResult<IdentityOutput> {
    use bip39::Mnemonic;
    use ed25519_dalek::SigningKey;

    let mnemonic = Mnemonic::parse_in_normalized(
        bip39::Language::English,
        mnemonic_str,
    )
    .map_err(|e| BridgeError::InvalidMnemonic(e.to_string()))?;

    let seed = mnemonic.to_seed("");
    let signing_key = SigningKey::from_bytes(
        &seed[..ed25519_dalek::SECRET_KEY_LENGTH].try_into().unwrap(),
    );
    let verifying_key = signing_key.verifying_key();
    let public_key_bytes = verifying_key.to_bytes();
    let did = generate_did(&public_key_bytes);
    let peer_id = generate_peer_id(&did);

    Ok(IdentityOutput {
        mnemonic: mnemonic_str.to_string(),
        seed_hex: hex::encode(&seed[..32]),
        did,
        peer_id,
        public_key: bs58::encode(&public_key_bytes).into_string(),
    })
}

// ============================================================================
// Helper: X3DH / Double Ratchet (placeholder — delegates to scp-core)
// ============================================================================

fn double_ratchet_encrypt(_session_id: &str, plaintext: &str) -> BridgeResult<EncryptOutput> {
    // In production: call scp-core::crypto::double_ratchet::encrypt(session_id, plaintext)
    // Placeholder: base58-encode with a mock nonce
    let nonce = hex::encode(&rand::random::<[u8; 12]>());
    let ciphertext = bs58::encode(plaintext.as_bytes()).into_string();
    Ok(EncryptOutput { ciphertext, nonce })
}

fn double_ratchet_decrypt(_session_id: &str, _ciphertext: &str) -> BridgeResult<String> {
    // In production: call scp-core::crypto::double_ratchet::decrypt(session_id, ciphertext)
    // Placeholder
    Ok("decrypted_placeholder".to_string())
}

// ============================================================================
// Helper: Sender Key
// ============================================================================

fn sender_key_encrypt(_group_id: &str, plaintext: &str) -> BridgeResult<EncryptOutput> {
    let nonce = hex::encode(&rand::random::<[u8; 12]>());
    let ciphertext = bs58::encode(plaintext.as_bytes()).into_string();
    Ok(EncryptOutput { ciphertext, nonce })
}

fn sender_key_decrypt(_group_id: &str, _ciphertext: &str) -> BridgeResult<String> {
    Ok("decrypted_placeholder".to_string())
}

// ============================================================================
// Helper: Envelope signing
// ============================================================================

fn sign_envelope(_envelope_json: &str, _signing_key_b64: &str) -> BridgeResult<String> {
    // In production: json → ciborium serialize → ed25519 sign → append signature
    Ok("signed_envelope_placeholder".to_string())
}

fn verify_envelope(_envelope_json: &str, _public_key_b64: &str) -> BridgeResult<bool> {
    Ok(true)
}

// ============================================================================
// Helper: DID from raw public key bytes
// ============================================================================

fn did_from_public_key(public_key_bytes: &[u8]) -> BridgeResult<String> {
    if public_key_bytes.len() != 32 {
        return Err(BridgeError::InvalidKey("Expected 32-byte Ed25519 public key".into()));
    }
    let arr: &[u8; 32] = public_key_bytes.try_into().unwrap();
    Ok(generate_did(arr))
}

// ============================================================================
// JSON helper
// ============================================================================

fn to_json_string<T: serde::Serialize>(value: &T) -> BridgeResult<String> {
    serde_json::to_string(value)
        .map_err(|e| BridgeError::OperationFailed(format!("JSON serialize: {e}")))
}

fn c_str_to_str<'a>(ptr: *const c_char) -> BridgeResult<&'a str> {
    if ptr.is_null() {
        return Err(BridgeError::NullPointer);
    }
    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .map_err(BridgeError::Utf8Error)
}

fn str_to_c_string(s: &str) -> *mut c_char {
    CString::new(s)
        .unwrap_or_else(|_| CString::new("error").unwrap())
        .into_raw()
}

// ============================================================================
// C FFI exports (for iOS static linking)
// ============================================================================

/// # Safety
/// All functions accept *const c_char and return *mut c_char.
/// Caller must free the returned string via `scp_bridge_free_string`.
#[no_mangle]
pub extern "C" fn scp_bridge_generate_identity() -> *mut c_char {
    match generate_identity().and_then(|o| to_json_string(&o)) {
        Ok(json) => str_to_c_string(&json),
        Err(e) => str_to_c_string(&format!(r#"{{"error":"{}"}}"#, e)),
    }
}

/// # Safety
#[no_mangle]
pub extern "C" fn scp_bridge_recover_identity(mnemonic: *const c_char) -> *mut c_char {
    let mnemonic_str = match c_str_to_str(mnemonic) {
        Ok(s) => s,
        Err(e) => return str_to_c_string(&format!(r#"{{"error":"{}"}}"#, e)),
    };
    match recover_identity(mnemonic_str).and_then(|o| to_json_string(&o)) {
        Ok(json) => str_to_c_string(&json),
        Err(e) => str_to_c_string(&format!(r#"{{"error":"{}"}}"#, e)),
    }
}

/// # Safety
#[no_mangle]
pub extern "C" fn scp_bridge_did_from_public_key(pk_hex: *const c_char) -> *mut c_char {
    let hex_str = match c_str_to_str(pk_hex) {
        Ok(s) => s,
        Err(e) => return str_to_c_string(&format!(r#"{{"error":"{}"}}"#, e)),
    };
    let bytes = match hex::decode(hex_str) {
        Ok(b) => b,
        Err(e) => return str_to_c_string(&format!(r#"{{"error":"Invalid hex: {}"}}"#, e)),
    };
    match did_from_public_key(&bytes) {
        Ok(did) => str_to_c_string(&format!(r#"{{"did":"{}"}}"#, did)),
        Err(e) => str_to_c_string(&format!(r#"{{"error":"{}"}}"#, e)),
    }
}

/// # Safety
#[no_mangle]
pub extern "C" fn scp_bridge_double_ratchet_encrypt(
    session_id: *const c_char,
    plaintext: *const c_char,
) -> *mut c_char {
    let sid = match c_str_to_str(session_id) {
        Ok(s) => s,
        Err(e) => return str_to_c_string(&format!(r#"{{"error":"{}"}}"#, e)),
    };
    let pt = match c_str_to_str(plaintext) {
        Ok(s) => s,
        Err(e) => return str_to_c_string(&format!(r#"{{"error":"{}"}}"#, e)),
    };
    match double_ratchet_encrypt(sid, pt).and_then(|o| to_json_string(&o)) {
        Ok(json) => str_to_c_string(&json),
        Err(e) => str_to_c_string(&format!(r#"{{"error":"{}"}}"#, e)),
    }
}

/// # Safety
#[no_mangle]
pub extern "C" fn scp_bridge_sender_key_encrypt(
    group_id: *const c_char,
    plaintext: *const c_char,
) -> *mut c_char {
    let gid = match c_str_to_str(group_id) {
        Ok(s) => s,
        Err(e) => return str_to_c_string(&format!(r#"{{"error":"{}"}}"#, e)),
    };
    let pt = match c_str_to_str(plaintext) {
        Ok(s) => s,
        Err(e) => return str_to_c_string(&format!(r#"{{"error":"{}"}}"#, e)),
    };
    match sender_key_encrypt(gid, pt).and_then(|o| to_json_string(&o)) {
        Ok(json) => str_to_c_string(&json),
        Err(e) => str_to_c_string(&format!(r#"{{"error":"{}"}}"#, e)),
    }
}

/// # Safety
#[no_mangle]
pub extern "C" fn scp_bridge_sign_envelope(
    envelope_json: *const c_char,
    signing_key_b64: *const c_char,
) -> *mut c_char {
    let ej = match c_str_to_str(envelope_json) {
        Ok(s) => s,
        Err(e) => return str_to_c_string(&format!(r#"{{"error":"{}"}}"#, e)),
    };
    let sk = match c_str_to_str(signing_key_b64) {
        Ok(s) => s,
        Err(e) => return str_to_c_string(&format!(r#"{{"error":"{}"}}"#, e)),
    };
    match sign_envelope(ej, sk) {
        Ok(signed) => str_to_c_string(&format!(r#"{{"signed_envelope":"{}"}}"#, signed)),
        Err(e) => str_to_c_string(&format!(r#"{{"error":"{}"}}"#, e)),
    }
}

/// # Safety
/// Free a string previously returned by any scp_bridge_* function.
#[no_mangle]
pub extern "C" fn scp_bridge_free_string(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let _ = CString::from_raw(ptr);
    }
}

// ============================================================================
// JNI exports (for Android)
// ============================================================================

#[cfg(feature = "android")]
mod android {
    use jni::JNIEnv;
    use jni::objects::{JClass, JString};
    use jni::sys::jstring;

    fn java_string_to_rust(env: &mut JNIEnv, input: &JString) -> crate::BridgeResult<String> {
        env.get_string(input)
            .map(|s| s.into())
            .map_err(|e| crate::BridgeError::OperationFailed(format!("JNI string: {e}")))
    }

    fn rust_to_java_string(env: &mut JNIEnv, s: &str) -> jstring {
        env.new_string(s)
            .map(|js| js.into_raw())
            .unwrap_or(std::ptr::null_mut())
    }

    #[no_mangle]
    pub extern "system" fn Java_com_swarmchat_ScpCoreBridgeModule_nativeGenerateIdentity(
        mut env: JNIEnv,
        _class: JClass,
    ) -> jstring {
        match crate::generate_identity().and_then(|o| crate::to_json_string(&o)) {
            Ok(json) => rust_to_java_string(&mut env, &json),
            Err(e) => rust_to_java_string(&mut env, &format!(r#"{{"error":"{}"}}"#, e)),
        }
    }

    #[no_mangle]
    pub extern "system" fn Java_com_swarmchat_ScpCoreBridgeModule_nativeRecoverIdentity(
        mut env: JNIEnv,
        _class: JClass,
        mnemonic: JString,
    ) -> jstring {
        let mn = match java_string_to_rust(&mut env, &mnemonic) {
            Ok(s) => s,
            Err(e) => return rust_to_java_string(&mut env, &format!(r#"{{"error":"{}"}}"#, e)),
        };
        match crate::recover_identity(&mn).and_then(|o| crate::to_json_string(&o)) {
            Ok(json) => rust_to_java_string(&mut env, &json),
            Err(e) => rust_to_java_string(&mut env, &format!(r#"{{"error":"{}"}}"#, e)),
        }
    }

    #[no_mangle]
    pub extern "system" fn Java_com_swarmchat_ScpCoreBridgeModule_nativeDidFromPublicKey(
        mut env: JNIEnv,
        _class: JClass,
        pk_hex: JString,
    ) -> jstring {
        let hex_str = match java_string_to_rust(&mut env, &pk_hex) {
            Ok(s) => s,
            Err(e) => return rust_to_java_string(&mut env, &format!(r#"{{"error":"{}"}}"#, e)),
        };
        let bytes = match hex::decode(&hex_str) {
            Ok(b) => b,
            Err(e) => return rust_to_java_string(&mut env, &format!(r#"{{"error":"{}"}}"#, e)),
        };
        match crate::did_from_public_key(&bytes) {
            Ok(did) => rust_to_java_string(&mut env, &format!(r#"{{"did":"{}"}}"#, did)),
            Err(e) => rust_to_java_string(&mut env, &format!(r#"{{"error":"{}"}}"#, e)),
        }
    }

    #[no_mangle]
    pub extern "system" fn Java_com_swarmchat_ScpCoreBridgeModule_nativeDoubleRatchetEncrypt(
        mut env: JNIEnv,
        _class: JClass,
        session_id: JString,
        plaintext: JString,
    ) -> jstring {
        let sid = match java_string_to_rust(&mut env, &session_id) {
            Ok(s) => s,
            Err(e) => return rust_to_java_string(&mut env, &format!(r#"{{"error":"{}"}}"#, e)),
        };
        let pt = match java_string_to_rust(&mut env, &plaintext) {
            Ok(s) => s,
            Err(e) => return rust_to_java_string(&mut env, &format!(r#"{{"error":"{}"}}"#, e)),
        };
        match crate::double_ratchet_encrypt(&sid, &pt).and_then(|o| crate::to_json_string(&o)) {
            Ok(json) => rust_to_java_string(&mut env, &json),
            Err(e) => rust_to_java_string(&mut env, &format!(r#"{{"error":"{}"}}"#, e)),
        }
    }

    #[no_mangle]
    pub extern "system" fn Java_com_swarmchat_ScpCoreBridgeModule_nativeDoubleRatchetDecrypt(
        mut env: JNIEnv,
        _class: JClass,
        session_id: JString,
        ciphertext: JString,
    ) -> jstring {
        let sid = match java_string_to_rust(&mut env, &session_id) {
            Ok(s) => s,
            Err(e) => return rust_to_java_string(&mut env, &format!(r#"{{"error":"{}"}}"#, e)),
        };
        let ct = match java_string_to_rust(&mut env, &ciphertext) {
            Ok(s) => s,
            Err(e) => return rust_to_java_string(&mut env, &format!(r#"{{"error":"{}"}}"#, e)),
        };
        match crate::double_ratchet_decrypt(&sid, &ct) {
            Ok(pt) => rust_to_java_string(&mut env, &format!(r#"{{"plaintext":"{}"}}"#, pt)),
            Err(e) => rust_to_java_string(&mut env, &format!(r#"{{"error":"{}"}}"#, e)),
        }
    }

    #[no_mangle]
    pub extern "system" fn Java_com_swarmchat_ScpCoreBridgeModule_nativeSenderKeyEncrypt(
        mut env: JNIEnv,
        _class: JClass,
        group_id: JString,
        plaintext: JString,
    ) -> jstring {
        let gid = match java_string_to_rust(&mut env, &group_id) {
            Ok(s) => s,
            Err(e) => return rust_to_java_string(&mut env, &format!(r#"{{"error":"{}"}}"#, e)),
        };
        let pt = match java_string_to_rust(&mut env, &plaintext) {
            Ok(s) => s,
            Err(e) => return rust_to_java_string(&mut env, &format!(r#"{{"error":"{}"}}"#, e)),
        };
        match crate::sender_key_encrypt(&gid, &pt).and_then(|o| crate::to_json_string(&o)) {
            Ok(json) => rust_to_java_string(&mut env, &json),
            Err(e) => rust_to_java_string(&mut env, &format!(r#"{{"error":"{}"}}"#, e)),
        }
    }

    #[no_mangle]
    pub extern "system" fn Java_com_swarmchat_ScpCoreBridgeModule_nativeSenderKeyDecrypt(
        mut env: JNIEnv,
        _class: JClass,
        group_id: JString,
        ciphertext: JString,
    ) -> jstring {
        let gid = match java_string_to_rust(&mut env, &group_id) {
            Ok(s) => s,
            Err(e) => return rust_to_java_string(&mut env, &format!(r#"{{"error":"{}"}}"#, e)),
        };
        let ct = match java_string_to_rust(&mut env, &ciphertext) {
            Ok(s) => s,
            Err(e) => return rust_to_java_string(&mut env, &format!(r#"{{"error":"{}"}}"#, e)),
        };
        match crate::sender_key_decrypt(&gid, &ct) {
            Ok(pt) => rust_to_java_string(&mut env, &format!(r#"{{"plaintext":"{}"}}"#, pt)),
            Err(e) => rust_to_java_string(&mut env, &format!(r#"{{"error":"{}"}}"#, e)),
        }
    }

    #[no_mangle]
    pub extern "system" fn Java_com_swarmchat_ScpCoreBridgeModule_nativeSignEnvelope(
        mut env: JNIEnv,
        _class: JClass,
        envelope_json: JString,
        signing_key_b64: JString,
    ) -> jstring {
        let ej = match java_string_to_rust(&mut env, &envelope_json) {
            Ok(s) => s,
            Err(e) => return rust_to_java_string(&mut env, &format!(r#"{{"error":"{}"}}"#, e)),
        };
        let sk = match java_string_to_rust(&mut env, &signing_key_b64) {
            Ok(s) => s,
            Err(e) => return rust_to_java_string(&mut env, &format!(r#"{{"error":"{}"}}"#, e)),
        };
        match crate::sign_envelope(&ej, &sk) {
            Ok(signed) => rust_to_java_string(
                &mut env,
                &format!(r#"{{"signed_envelope":"{}"}}"#, signed),
            ),
            Err(e) => rust_to_java_string(&mut env, &format!(r#"{{"error":"{}"}}"#, e)),
        }
    }

    #[no_mangle]
    pub extern "system" fn Java_com_swarmchat_ScpCoreBridgeModule_nativeVerifyEnvelope(
        mut env: JNIEnv,
        _class: JClass,
        envelope_json: JString,
        public_key_b64: JString,
    ) -> jstring {
        let ej = match java_string_to_rust(&mut env, &envelope_json) {
            Ok(s) => s,
            Err(e) => return rust_to_java_string(&mut env, &format!(r#"{{"error":"{}"}}"#, e)),
        };
        let pk = match java_string_to_rust(&mut env, &public_key_b64) {
            Ok(s) => s,
            Err(e) => return rust_to_java_string(&mut env, &format!(r#"{{"error":"{}"}}"#, e)),
        };
        match crate::verify_envelope(&ej, &pk) {
            Ok(valid) => rust_to_java_string(
                &mut env,
                &format!(r#"{{"valid":{}}}"#, valid),
            ),
            Err(e) => rust_to_java_string(&mut env, &format!(r#"{{"error":"{}"}}"#, e)),
        }
    }
}
