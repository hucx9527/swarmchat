/**
 * QR utility — DID QR code encoding/decoding.
 *
 * SCAN Protocol: QR code format for SwarmChat is a JSON payload
 * encoded as a base64url string prefixed with "scp://".
 *
 * Format: scp://<base64url({v:1, did, name?, pk?})>
 *
 * This module provides:
 *  - encodeDidQrPayload: build a QR-scannable string for a DID
 *  - decodeDidQrPayload: parse a scanned QR string back to {did, name, pk}
 *  - isScpQrCode: check if a scanned string is an SCP QR payload
 *  - buildAddFriendQrValue: convenience for sharing own DID
 */

export interface ScpQrPayload {
  v: number;
  did: string;
  name?: string;
  pk?: string; // public signing key (multibase or hex)
}

const SCP_QR_PREFIX = 'scp://';

/** Encode a DID + optional metadata into an SCP QR string */
export function encodeDidQrPayload(did: string, displayName?: string, publicKey?: string): string {
  const payload: ScpQrPayload = { v: 1, did };
  if (displayName) payload.name = displayName;
  if (publicKey) payload.pk = publicKey;
  const json = JSON.stringify(payload);
  const base64 = base64UrlEncode(json);
  return `${SCP_QR_PREFIX}${base64}`;
}

/** Decode an SCP QR string back to a structured payload */
export function decodeDidQrPayload(qrValue: string): ScpQrPayload | null {
  try {
    if (!isScpQrCode(qrValue)) return null;
    const base64 = qrValue.slice(SCP_QR_PREFIX.length);
    const json = base64UrlDecode(base64);
    const payload: ScpQrPayload = JSON.parse(json);
    if (!payload.did || typeof payload.v !== 'number') return null;
    return payload;
  } catch {
    return null;
  }
}

/** Check whether a scanned string is a valid SCP QR code */
export function isScpQrCode(value: string): boolean {
  return typeof value === 'string' && value.startsWith(SCP_QR_PREFIX);
}

/** Extract a DID from a raw scanned string (could be plain DID or SCP QR) */
export function extractDidFromScan(value: string): string | null {
  if (isScpQrCode(value)) {
    const decoded = decodeDidQrPayload(value);
    return decoded?.did ?? null;
  }
  // Plain DID format: did:key:...
  if (value.startsWith('did:key:')) {
    return value.trim();
  }
  return null;
}

// ---- base64url helpers (no external dependency) ----

const BASE64URL_CHARS = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_';

function base64UrlEncode(input: string): string {
  // Convert string to UTF-8 bytes manually via char codes
  const bytes: number[] = [];
  for (let i = 0; i < input.length; i++) {
    const code = input.charCodeAt(i);
    if (code < 0x80) {
      bytes.push(code);
    } else if (code < 0x800) {
      bytes.push(0xc0 | (code >> 6), 0x80 | (code & 0x3f));
    } else if (code < 0x10000) {
      bytes.push(0xe0 | (code >> 12), 0x80 | ((code >> 6) & 0x3f), 0x80 | (code & 0x3f));
    } else {
      bytes.push(
        0xf0 | (code >> 18),
        0x80 | ((code >> 12) & 0x3f),
        0x80 | ((code >> 6) & 0x3f),
        0x80 | (code & 0x3f),
      );
    }
  }

  let result = '';
  let buffer = 0;
  let bits = 0;
  for (const byte of bytes) {
    buffer = (buffer << 8) | byte;
    bits += 8;
    while (bits >= 6) {
      bits -= 6;
      result += BASE64URL_CHARS[(buffer >> bits) & 0x3f];
    }
  }
  if (bits > 0) {
    result += BASE64URL_CHARS[(buffer << (6 - bits)) & 0x3f];
  }
  // No padding in base64url
  return result;
}

function base64UrlDecode(input: string): string {
  const lookup: Record<string, number> = {};
  for (let i = 0; i < BASE64URL_CHARS.length; i++) {
    lookup[BASE64URL_CHARS[i]] = i;
  }

  const bytes: number[] = [];
  let buffer = 0;
  let bits = 0;
  for (const ch of input) {
    const val = lookup[ch];
    if (val === undefined) continue; // skip invalid chars
    buffer = (buffer << 6) | val;
    bits += 6;
    if (bits >= 8) {
      bits -= 8;
      bytes.push((buffer >> bits) & 0xff);
    }
  }

  // Decode UTF-8 bytes to string
  let result = '';
  let i = 0;
  while (i < bytes.length) {
    const b0 = bytes[i++];
    if (b0 < 0x80) {
      result += String.fromCharCode(b0);
    } else if ((b0 & 0xe0) === 0xc0 && i < bytes.length) {
      const b1 = bytes[i++];
      result += String.fromCharCode(((b0 & 0x1f) << 6) | (b1 & 0x3f));
    } else if ((b0 & 0xf0) === 0xe0 && i + 1 < bytes.length) {
      const b1 = bytes[i++];
      const b2 = bytes[i++];
      result += String.fromCharCode(((b0 & 0x0f) << 12) | ((b1 & 0x3f) << 6) | (b2 & 0x3f));
    } else if ((b0 & 0xf8) === 0xf0 && i + 2 < bytes.length) {
      const b1 = bytes[i++];
      const b2 = bytes[i++];
      const b3 = bytes[i++];
      const code = ((b0 & 0x07) << 18) | ((b1 & 0x3f) << 12) | ((b2 & 0x3f) << 6) | (b3 & 0x3f);
      result += String.fromCharCode(code);
    }
  }
  return result;
}

export default {
  encodeDidQrPayload,
  decodeDidQrPayload,
  isScpQrCode,
  extractDidFromScan,
};
