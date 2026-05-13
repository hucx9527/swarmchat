// DID (Decentralized Identifier) utilities

const ED25519_MULTICODEC_PREFIX = [0xed, 0x01];

/**
 * Generate a did:key from an Ed25519 public key.
 */
export function generateDid(publicKey: Uint8Array): string {
  const multicodec = new Uint8Array([...ED25519_MULTICODEC_PREFIX, ...publicKey]);
  const identifier = base58Encode(multicodec);
  return `did:key:${identifier}`;
}

/**
 * Extract the raw Ed25519 public key (32 bytes) from a did:key string.
 */
export function publicKeyFromDid(did: string): Uint8Array {
  const parts = did.split(':');
  if (parts.length < 3 || parts[0] !== 'did' || parts[1] !== 'key') {
    throw new Error(`Invalid DID format: ${did}`);
  }

  const identifier = parts.slice(2).join(':');
  const multicodec = base58Decode(identifier);

  if (multicodec.length < 2 + 32) {
    throw new Error(`DID key too short: ${multicodec.length} bytes`);
  }

  // Verify Ed25519 prefix
  if (multicodec[0] !== 0xed || multicodec[1] !== 0x01) {
    throw new Error(`Unsupported key type: 0x${multicodec[0].toString(16)}${multicodec[1].toString(16)}`);
  }

  return multicodec.slice(2, 34);
}

/**
 * Parse a DID string and return its components.
 */
export function parseDid(did: string): { method: string; identifier: string; publicKey: Uint8Array } {
  const publicKey = publicKeyFromDid(did);
  const parts = did.split(':');
  return {
    method: parts[1],
    identifier: parts.slice(2).join(':'),
    publicKey,
  };
}

/**
 * Validate that a string is a valid did:key.
 */
export function isValidDid(did: string): boolean {
  try {
    publicKeyFromDid(did);
    return true;
  } catch {
    return false;
  }
}

// ---- Base58 (simplified for did:key) ----

const BASE58_ALPHABET = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';

function base58Encode(data: Uint8Array): string {
  // Count leading zeros
  let zeros = 0;
  while (zeros < data.length && data[zeros] === 0) {
    zeros++;
  }

  // Convert to base58
  const digits = [0];
  for (let i = 0; i < data.length; i++) {
    let carry = data[i];
    for (let j = 0; j < digits.length; j++) {
      carry += digits[j] * 256;
      digits[j] = carry % 58;
      carry = Math.floor(carry / 58);
    }
    while (carry > 0) {
      digits.push(carry % 58);
      carry = Math.floor(carry / 58);
    }
  }

  // Prepend '1' for each leading zero
  let result = '1'.repeat(zeros);
  for (let i = digits.length - 1; i >= 0; i--) {
    result += BASE58_ALPHABET[digits[i]];
  }

  return result;
}

function base58Decode(encoded: string): Uint8Array {
  // Count leading '1's
  let zeros = 0;
  while (zeros < encoded.length && encoded[zeros] === '1') {
    zeros++;
  }

  // Convert from base58
  const bytes = [0];
  for (let i = zeros; i < encoded.length; i++) {
    const digit = BASE58_ALPHABET.indexOf(encoded[i]);
    if (digit === -1) {
      throw new Error(`Invalid base58 character: ${encoded[i]}`);
    }
    let carry = digit;
    for (let j = 0; j < bytes.length; j++) {
      carry += bytes[j] * 58;
      bytes[j] = carry % 256;
      carry = Math.floor(carry / 256);
    }
    while (carry > 0) {
      bytes.push(carry % 256);
      carry = Math.floor(carry / 256);
    }
  }

  // Prepend zero bytes
  const result = new Uint8Array(zeros + bytes.length);
  result.fill(0, 0, zeros);
  for (let i = 0; i < bytes.length; i++) {
    result[zeros + bytes.length - 1 - i] = bytes[i];
  }

  return result;
}

export default { generateDid, publicKeyFromDid, parseDid, isValidDid };
