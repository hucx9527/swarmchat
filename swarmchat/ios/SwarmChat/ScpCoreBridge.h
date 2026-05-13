/**
 * ScpCoreBridge.h
 * SwarmChat iOS
 *
 * Bridge header declaring the C FFI functions exported by scp-core-bridge
 * (Rust static library). These are linked into the iOS app binary.
 */

#import <React/RCTBridgeModule.h>

@interface ScpCoreBridge : NSObject <RCTBridgeModule>

@end

// C FFI function declarations (linked from librust_scp_core_bridge.a)

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Generate a new identity.
 * Returns a JSON string: {"mnemonic","seed_hex","did","peer_id","public_key"}
 * Caller must free with scp_bridge_free_string.
 */
char* scp_bridge_generate_identity(void);

/**
 * Recover an identity from a BIP39 mnemonic.
 */
char* scp_bridge_recover_identity(const char* mnemonic);

/**
 * Derive a did:key from a hex-encoded 32-byte Ed25519 public key.
 */
char* scp_bridge_did_from_public_key(const char* pk_hex);

/**
 * Double Ratchet encrypt.
 * session_id: unique session identifier
 * plaintext:  message to encrypt
 */
char* scp_bridge_double_ratchet_encrypt(const char* session_id, const char* plaintext);

/**
 * Sender Key encrypt for group messaging.
 */
char* scp_bridge_sender_key_encrypt(const char* group_id, const char* plaintext);

/**
 * Sign a message envelope.
 */
char* scp_bridge_sign_envelope(const char* envelope_json, const char* signing_key_b64);

/**
 * Free a string previously returned by any scp_bridge_* function.
 */
void scp_bridge_free_string(char* ptr);

#ifdef __cplusplus
}
#endif
