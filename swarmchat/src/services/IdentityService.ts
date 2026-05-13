/**
 * IdentityService — manages decentralized identity lifecycle.
 */

import { Identity, IdentityStore } from '../types';
import { ScpCoreBridge } from './ScpCoreBridge';
import { StorageService } from './StorageService';

export const IdentityService = {
  /**
   * Create a new identity with a fresh BIP39 mnemonic.
   */
  async create(label: string, nickname?: string, description?: string): Promise<Identity> {
    const result = await ScpCoreBridge.generateIdentity();

    const identity: Identity = {
      label,
      nickname,
      description,
      did: result.did,
      peerId: result.peerId,
      publicKey: result.publicKey,
      mnemonic: result.mnemonic,
      seedHex: result.seedHex,
      createdAt: Date.now(),
      isDefault: false,
    };

    // Persist securely
    this.saveIdentity(identity);

    return identity;
  },

  /**
   * Recover an identity from a BIP39 mnemonic phrase.
   */
  async recover(mnemonic: string, label: string): Promise<Identity> {
    // Validate mnemonic (24 words)
    const words = mnemonic.trim().split(/\s+/);
    if (words.length !== 24) {
      throw new Error(`Invalid mnemonic: expected 24 words, got ${words.length}`);
    }

    const result = await ScpCoreBridge.recoverIdentity(mnemonic);

    const identity: Identity = {
      label,
      did: result.did,
      peerId: result.peerId,
      publicKey: result.publicKey,
      mnemonic: result.mnemonic,
      seedHex: result.seedHex,
      createdAt: Date.now(),
      isDefault: false,
    };

    this.saveIdentity(identity);
    return identity;
  },

  /**
   * Save an identity to the secure store.
   */
  saveIdentity(identity: Identity): void {
    // Save mnemonic and seed securely
    StorageService.storeMnemonic(identity.did, identity.mnemonic);
    StorageService.storeSeedHex(identity.did, identity.seedHex);

    // Update identity store
    const store = this.loadStore();
    store.identities[identity.label] = identity;

    // First identity becomes default
    if (!store.defaultLabel) {
      store.defaultLabel = identity.label;
      identity.isDefault = true;
    }

    StorageService.saveIdentityStore(JSON.stringify(store));
  },

  /**
   * Load the full identity store.
   */
  loadStore(): IdentityStore {
    const json = StorageService.loadIdentityStore();
    if (json) {
      try {
        return JSON.parse(json);
      } catch {
        // Corrupted data, return empty
      }
    }
    return { identities: {}, schemaVersion: 1 };
  },

  /**
   * Get the active/default identity.
   */
  getActiveIdentity(): Identity | null {
    const store = this.loadStore();
    if (store.defaultLabel && store.identities[store.defaultLabel]) {
      const identity = store.identities[store.defaultLabel];
      // Restore mnemonic from secure storage
      const mnemonic = StorageService.getMnemonic(identity.did);
      if (mnemonic) {
        identity.mnemonic = mnemonic;
      }
      return identity;
    }
    // Fall back to first available
    const labels = Object.keys(store.identities);
    if (labels.length > 0) {
      return store.identities[labels[0]];
    }
    return null;
  },

  /**
   * Get all identities.
   */
  getAllIdentities(): Identity[] {
    const store = this.loadStore();
    return Object.values(store.identities);
  },

  /**
   * Set the default identity.
   */
  setDefault(label: string): void {
    const store = this.loadStore();
    if (!store.identities[label]) {
      throw new Error(`Identity '${label}' not found`);
    }
    // Unset previous default
    Object.values(store.identities).forEach(id => { id.isDefault = false; });
    store.identities[label].isDefault = true;
    store.defaultLabel = label;
    StorageService.saveIdentityStore(JSON.stringify(store));
  },

  /**
   * Delete an identity.
   */
  delete(label: string): void {
    const store = this.loadStore();
    const identity = store.identities[label];
    if (!identity) return;

    StorageService.deleteMnemonic(identity.did);
    StorageService.deleteMnemonic(identity.did);
    delete store.identities[label];

    if (store.defaultLabel === label) {
      const remaining = Object.keys(store.identities);
      store.defaultLabel = remaining.length > 0 ? remaining[0] : undefined;
    }

    StorageService.saveIdentityStore(JSON.stringify(store));
  },

  /**
   * Check if any identity exists.
   */
  hasIdentity(): boolean {
    const store = this.loadStore();
    return Object.keys(store.identities).length > 0;
  },
};

export default IdentityService;
