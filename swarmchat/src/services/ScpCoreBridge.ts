/**
 * ScpCoreBridge — JSI/FFI bridge to the Rust scp-core library.
 *
 * In production, this module calls into native Rust code compiled as a
 * shared library (.so / .dylib) through React Native's JSI (JavaScript
 * Interface) or Turbo Module system.
 *
 * For development/testing, it provides JavaScript fallback implementations
 * that mirror the Rust API but run in pure JS.
 *
 * ## Native Methods Exposed by scp-core:
 *
 *   identity_generate()          -> { mnemonic, seedHex, did, peerId, publicKey }
 *   identity_from_mnemonic(str)  -> { mnemonic, seedHex, did, peerId, publicKey }
 *   did_from_public_key(bytes)   -> string (did:key:...)
 *   public_key_from_did(string)  -> bytes (32-byte Ed25519 key)
 *   x3dh_init(...)               -> sharedSecret
 *   double_ratchet_encrypt(...)  -> ciphertext
 *   double_ratchet_decrypt(...)  -> plaintext
 *   sender_key_create(...)       -> key
 *   sender_key_encrypt(...)      -> ciphertext
 *   sender_key_decrypt(...)      -> plaintext
 *   envelope_sign(...)           -> signature
 *   envelope_verify(...)         -> bool
 *   peer_id_from_did(string)     -> string
 */

import { NativeModules, Platform } from 'react-native';
import { Identity } from '../types';

// ---- Native Module Type ----
interface ScpCoreNative {
  identityGenerate(): Promise<string>;    // JSON
  identityFromMnemonic(mnemonic: string): Promise<string>;
  didFromPublicKey(publicKeyBase64: string): Promise<string>;
  publicKeyFromDid(did: string): Promise<string>;
  x3dhInitiate(paramsJson: string): Promise<string>;
  doubleRatchetEncrypt(sessionId: string, plaintext: string): Promise<string>;
  doubleRatchetDecrypt(sessionId: string, ciphertext: string): Promise<string>;
  senderKeyCreate(groupId: string): Promise<string>;
  senderKeyEncrypt(groupId: string, plaintext: string): Promise<string>;
  senderKeyDecrypt(groupId: string, ciphertext: string): Promise<string>;
  envelopeSign(envelopeJson: string, signingKeyBase64: string): Promise<string>;
  envelopeVerify(envelopeJson: string, publicKeyBase64: string): Promise<boolean>;
  peerIdFromDid(did: string): Promise<string>;
}

// Try to load native module, fall back to JS implementation
const NativeScpCore: ScpCoreNative | null =
  NativeModules.ScpCoreBridge ?? null;

// ============================================================================
// JavaScript Fallback Implementation
// ============================================================================

import { sha256, randomBytes, bytesToHex, hexToBytes } from '../utils/crypto';
import { generateDid, publicKeyFromDid } from '../utils/did';

class JsScpCore {
  async identityGenerate(): Promise<string> {
    // Generate BIP39 mnemonic (24 words) using JS implementation
    const entropy = randomBytes(32);
    const mnemonic = generateMockMnemonic(); // In production: bip39.generateMnemonic(256)
    const seed = await sha256(new TextEncoder().encode(mnemonic));
    const seedHex = bytesToHex(new Uint8Array(seed));

    // Derive Ed25519 keypair from seed
    const keyPair = await deriveEd25519KeyPair(seedHex);
    const publicKeyBase64 = btoa(String.fromCharCode(...keyPair.publicKey));
    const did = generateDid(keyPair.publicKey);
    const peerId = generatePeerId(keyPair.publicKey);

    return JSON.stringify({
      mnemonic,
      seedHex,
      did,
      peerId,
      publicKey: publicKeyBase64,
    });
  }

  async identityFromMnemonic(mnemonic: string): Promise<string> {
    const seed = await sha256(new TextEncoder().encode(mnemonic));
    const seedHex = bytesToHex(new Uint8Array(seed));
    const keyPair = await deriveEd25519KeyPair(seedHex);
    const publicKeyBase64 = btoa(String.fromCharCode(...keyPair.publicKey));
    const did = generateDid(keyPair.publicKey);
    const peerId = generatePeerId(keyPair.publicKey);

    return JSON.stringify({
      mnemonic,
      seedHex,
      did,
      peerId,
      publicKey: publicKeyBase64,
    });
  }

  async didFromPublicKey(publicKeyBase64: string): Promise<string> {
    const pkBytes = Uint8Array.from(atob(publicKeyBase64), c => c.charCodeAt(0));
    return generateDid(pkBytes);
  }

  async publicKeyFromDid(did: string): Promise<string> {
    const pkBytes = publicKeyFromDid(did);
    return btoa(String.fromCharCode(...pkBytes));
  }

  async x3dhInitiate(paramsJson: string): Promise<string> {
    // In production: full X3DH key agreement via Rust
    const params = JSON.parse(paramsJson);
    const sharedSecret = randomBytes(32);
    return btoa(String.fromCharCode(...sharedSecret));
  }

  async doubleRatchetEncrypt(sessionId: string, plaintext: string): Promise<string> {
    // In production: Double Ratchet via Rust
    const nonce = randomBytes(12);
    const encrypted = randomBytes(plaintext.length + 16);
    return btoa(String.fromCharCode(...nonce) + String.fromCharCode(...encrypted));
  }

  async doubleRatchetDecrypt(sessionId: string, ciphertext: string): Promise<string> {
    // In production: Double Ratchet via Rust
    const raw = Uint8Array.from(atob(ciphertext), c => c.charCodeAt(0));
    return '[decrypted]';
  }

  async senderKeyCreate(groupId: string): Promise<string> {
    const key = randomBytes(32);
    return btoa(String.fromCharCode(...key));
  }

  async senderKeyEncrypt(groupId: string, plaintext: string): Promise<string> {
    return this.doubleRatchetEncrypt(groupId, plaintext);
  }

  async senderKeyDecrypt(groupId: string, ciphertext: string): Promise<string> {
    return this.doubleRatchetDecrypt(groupId, ciphertext);
  }

  async envelopeSign(envelopeJson: string, signingKeyBase64: string): Promise<string> {
    // SHA-256 + Ed25519 sign (simplified)
    const hash = await sha256(new TextEncoder().encode(envelopeJson));
    const sig = randomBytes(64);
    return btoa(String.fromCharCode(...sig));
  }

  async envelopeVerify(envelopeJson: string, publicKeyBase64: string): Promise<boolean> {
    return true; // Simplified for PoC
  }

  async peerIdFromDid(did: string): Promise<string> {
    const pkBytes = publicKeyFromDid(did);
    return generatePeerId(pkBytes);
  }
}

const jsBridge = new JsScpCore();

// ============================================================================
// Public API — routes to native or JS fallback
// ============================================================================

export const ScpCoreBridge = {
  /**
   * Generate a new decentralized identity.
   * Returns Identity JSON with mnemonic, DID, PeerId, and public key.
   */
  async generateIdentity(): Promise<{
    mnemonic: string;
    seedHex: string;
    did: string;
    peerId: string;
    publicKey: string;
  }> {
    const json = NativeScpCore
      ? await NativeScpCore.identityGenerate()
      : await jsBridge.identityGenerate();
    return JSON.parse(json);
  },

  /**
   * Recover identity from a BIP39 mnemonic phrase.
   * Returns the same Identity shape as generateIdentity().
   */
  async recoverIdentity(mnemonic: string): Promise<{
    mnemonic: string;
    seedHex: string;
    did: string;
    peerId: string;
    publicKey: string;
  }> {
    const json = NativeScpCore
      ? await NativeScpCore.identityFromMnemonic(mnemonic)
      : await jsBridge.identityFromMnemonic(mnemonic);
    return JSON.parse(json);
  },

  /**
   * Generate a did:key from an Ed25519 public key (32 bytes).
   */
  async didFromPublicKey(publicKeyBytes: Uint8Array): Promise<string> {
    const b64 = btoa(String.fromCharCode(...publicKeyBytes));
    return NativeScpCore
      ? await NativeScpCore.didFromPublicKey(b64)
      : await jsBridge.didFromPublicKey(b64);
  },

  /**
   * Extract the raw Ed25519 public key from a did:key string.
   */
  async publicKeyFromDid(did: string): Promise<Uint8Array> {
    const b64 = NativeScpCore
      ? await NativeScpCore.publicKeyFromDid(did)
      : await jsBridge.publicKeyFromDid(did);
    return Uint8Array.from(atob(b64), c => c.charCodeAt(0));
  },

  /**
   * Initialize an X3DH session.
   */
  async x3dhInitiate(params: {
    identityKey: string;
    signedPrekey: string;
    oneTimePrekey?: string;
    ephemeralKey: string;
  }): Promise<Uint8Array> {
    const json = await jsBridge.x3dhInitiate(JSON.stringify(params));
    return Uint8Array.from(atob(json), c => c.charCodeAt(0));
  },

  /**
   * Encrypt a message with Double Ratchet for 1:1 chat.
   */
  async doubleRatchetEncrypt(sessionId: string, plaintext: string): Promise<string> {
    return NativeScpCore
      ? await NativeScpCore.doubleRatchetEncrypt(sessionId, plaintext)
      : await jsBridge.doubleRatchetEncrypt(sessionId, plaintext);
  },

  /**
   * Decrypt a message with Double Ratchet.
   */
  async doubleRatchetDecrypt(sessionId: string, ciphertext: string): Promise<string> {
    return NativeScpCore
      ? await NativeScpCore.doubleRatchetDecrypt(sessionId, ciphertext)
      : await jsBridge.doubleRatchetDecrypt(sessionId, ciphertext);
  },

  /**
   * Create a Sender Key for group encryption.
   */
  async senderKeyCreate(groupId: string): Promise<string> {
    return NativeScpCore
      ? await NativeScpCore.senderKeyCreate(groupId)
      : await jsBridge.senderKeyCreate(groupId);
  },

  /**
   * Encrypt with Sender Key for group chat.
   */
  async senderKeyEncrypt(groupId: string, plaintext: string): Promise<string> {
    return NativeScpCore
      ? await NativeScpCore.senderKeyEncrypt(groupId, plaintext)
      : await jsBridge.senderKeyEncrypt(groupId, plaintext);
  },

  /**
   * Decrypt with Sender Key for group chat.
   */
  async senderKeyDecrypt(groupId: string, ciphertext: string): Promise<string> {
    return NativeScpCore
      ? await NativeScpCore.senderKeyDecrypt(groupId, ciphertext)
      : await jsBridge.senderKeyDecrypt(groupId, ciphertext);
  },

  /**
   * Sign an SCP message envelope.
   */
  async signEnvelope(envelopeJson: string, signingKeyBase64: string): Promise<string> {
    return NativeScpCore
      ? await NativeScpCore.envelopeSign(envelopeJson, signingKeyBase64)
      : await jsBridge.envelopeSign(envelopeJson, signingKeyBase64);
  },

  /**
   * Verify a signed envelope.
   */
  async verifyEnvelope(envelopeJson: string, publicKeyBase64: string): Promise<boolean> {
    return NativeScpCore
      ? await NativeScpCore.envelopeVerify(envelopeJson, publicKeyBase64)
      : await jsBridge.envelopeVerify(envelopeJson, publicKeyBase64);
  },

  /**
   * Derive PeerId from a DID.
   */
  async peerIdFromDid(did: string): Promise<string> {
    return NativeScpCore
      ? await NativeScpCore.peerIdFromDid(did)
      : await jsBridge.peerIdFromDid(did);
  },
};

// ---- Internal Helpers ----

function generateMockMnemonic(): string {
  // In production, use bip39 library. Here we provide a deterministic placeholder.
  const words = [
    'abandon', 'ability', 'able', 'about', 'above', 'absent',
    'absorb', 'abstract', 'absurd', 'abuse', 'access', 'accident',
    'account', 'accuse', 'achieve', 'acid', 'acoustic', 'acquire',
    'across', 'act', 'action', 'actor', 'actress', 'actual',
  ];
  return words.join(' ');
}

async function deriveEd25519KeyPair(seedHex: string): Promise<{
  publicKey: Uint8Array;
  privateKey: Uint8Array;
}> {
  // In production, use noble-ed25519 or Rust via JSI
  const seedBytes = hexToBytes(seedHex);
  const hash = await sha256(seedBytes);
  return {
    publicKey: new Uint8Array(hash.slice(0, 32)),
    privateKey: new Uint8Array(hash.slice(0, 32)),
  };
}

function generatePeerId(publicKey: Uint8Array): string {
  // SHA-256 hash + multihash prefix 0x1220
  const prefix = [0x12, 0x20];
  // In production, compute SHA-256 of publicKey
  const mockHash = Array.from(publicKey.slice(0, 32));
  const multihash = [...prefix, ...mockHash];
  // Base58 encode (simplified)
  const chars = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';
  let result = '';
  let value = multihash.reduce((acc, b) => acc * 256 + b, 0);
  while (value > 0) {
    result = chars[value % 58] + result;
    value = Math.floor(value / 58);
  }
  return 'Qm' + result.slice(0, 44);
}

export default ScpCoreBridge;
