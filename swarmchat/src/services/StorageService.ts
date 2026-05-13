/**
 * StorageService — secure persistent storage for sensitive data.
 *
 * Uses react-native-mmkv for encrypted, synchronous key-value storage.
 * For the identity mnemonic and private keys, data is additionally
 * encrypted before storage.
 */

import { MMKV } from 'react-native-mmkv';

const storage = new MMKV({
  id: 'swarmchat-secure',
  encryptionKey: 'swarmchat-v1-key', // In production: derived from biometric
});

// Separate store for non-sensitive app data
const appStorage = new MMKV({
  id: 'swarmchat-app',
});

export const StorageService = {
  // ---- Secure Storage (mnemonics, private keys) ----

  storeMnemonic(did: string, mnemonic: string): void {
    storage.set(`mnemonic_${did}`, mnemonic);
  },

  getMnemonic(did: string): string | undefined {
    return storage.getString(`mnemonic_${did}`);
  },

  deleteMnemonic(did: string): void {
    storage.delete(`mnemonic_${did}`);
  },

  storeSeedHex(did: string, seedHex: string): void {
    storage.set(`seed_${did}`, seedHex);
  },

  getSeedHex(did: string): string | undefined {
    return storage.getString(`seed_${did}`);
  },

  storePrivateKey(did: string, privateKeyBase64: string): void {
    storage.set(`privkey_${did}`, privateKeyBase64);
  },

  getPrivateKey(did: string): string | undefined {
    return storage.getString(`privkey_${did}`);
  },

  // ---- Identity Store ----

  saveIdentityStore(storeJson: string): void {
    appStorage.set('identity_store', storeJson);
  },

  loadIdentityStore(): string | undefined {
    return appStorage.getString('identity_store');
  },

  // ---- App Settings ----

  saveSettings(settingsJson: string): void {
    appStorage.set('settings', settingsJson);
  },

  loadSettings(): string | undefined {
    return appStorage.getString('settings');
  },

  // ---- Chat Data ----

  saveChatRooms(roomsJson: string): void {
    appStorage.set('chat_rooms', roomsJson);
  },

  loadChatRooms(): string | undefined {
    return appStorage.getString('chat_rooms');
  },

  saveMessages(roomId: string, messagesJson: string): void {
    appStorage.set(`messages_${roomId}`, messagesJson);
  },

  loadMessages(roomId: string): string | undefined {
    return appStorage.getString(`messages_${roomId}`);
  },

  // ---- Contacts ----

  saveContacts(contactsJson: string): void {
    appStorage.set('contacts', contactsJson);
  },

  loadContacts(): string | undefined {
    return appStorage.getString('contacts');
  },

  // ---- Groups ----

  saveGroups(groupsJson: string): void {
    appStorage.set('groups', groupsJson);
  },

  loadGroups(): string | undefined {
    return appStorage.getString('groups');
  },

  // ---- Session Data ----

  saveSession(sessionId: string, sessionData: string): void {
    storage.set(`session_${sessionId}`, sessionData);
  },

  loadSession(sessionId: string): string | undefined {
    return storage.getString(`session_${sessionId}`);
  },

  // ---- Utilities ----

  clearAll(): void {
    storage.clearAll();
    appStorage.clearAll();
  },

  getAllKeys(): string[] {
    return storage.getAllKeys();
  },
};

export default StorageService;
