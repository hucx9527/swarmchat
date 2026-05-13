/**
 * ChatService — manages chat rooms, message encryption, sending, and receiving.
 */

import { ChatMessage, ChatRoom, MessagePayload, MessageType, MessageStatus } from '../types';
import { ScpCoreBridge } from './ScpCoreBridge';
import { RelayService } from './RelayService';
import { IdentityService } from './IdentityService';
import { StorageService } from './StorageService';
import { v4 as uuidv4 } from 'uuid';

export const ChatService = {
  /**
   * Create or get a direct chat room with a peer.
   */
  async getOrCreateDirectRoom(peerDid: string, peerName?: string): Promise<ChatRoom> {
    const rooms = this.loadRooms();
    const roomId = this.generateRoomId('direct', [peerDid]);

    if (rooms[roomId]) {
      return rooms[roomId];
    }

    // Initialize X3DH session for encryption
    const sessionId = uuidv4();

    const room: ChatRoom = {
      id: roomId,
      type: 'direct',
      name: peerName || peerDid.substring(0, 20) + '...',
      participants: [peerDid],
      unreadCount: 0,
      encryptionScheme: 'double-ratchet',
      sessionId,
      createdAt: Date.now(),
    };

    rooms[roomId] = room;
    this.saveRooms(rooms);
    return room;
  },

  /**
   * Send a text message to a peer.
   */
  async sendText(roomId: string, text: string, replyToId?: string): Promise<ChatMessage> {
    const active = IdentityService.getActiveIdentity();
    if (!active) throw new Error('No active identity');

    const room = this.loadRooms()[roomId];
    if (!room) throw new Error('Room not found');

    const message: ChatMessage = {
      id: uuidv4(),
      roomId,
      senderDid: active.did,
      type: MessageType.TEXT,
      payload: { type: MessageType.TEXT, body: text },
      timestamp: Date.now(),
      ttl: 604800,
      status: MessageStatus.SENDING,
      isOutgoing: true,
      replyToId,
    };

    // Save locally immediately
    this.saveMessage(roomId, message);

    // Encrypt with Double Ratchet
    const envelope = JSON.stringify({
      id: message.id,
      protocol: 'scp/1.0',
      type: MessageType.TEXT,
      from: active.did,
      to: room.participants,
      timestamp: message.timestamp,
      ttl: message.ttl,
      encryption: {
        scheme: 'double-ratchet',
        session_id: room.sessionId,
        nonce: uuidv4().substring(0, 12),
      },
    });

    const ciphertext = await ScpCoreBridge.doubleRatchetEncrypt(
      room.sessionId,
      JSON.stringify(message.payload),
    );

    // Store on relay for offline delivery
    try {
      await RelayService.storeMessage(
        room.participants[0],
        active.did,
        btoa(JSON.stringify({ envelope, payload: ciphertext })),
        message.id,
        message.ttl,
      );
      message.status = MessageStatus.SENT;
    } catch (error) {
      message.status = MessageStatus.FAILED;
    }

    this.updateMessage(roomId, message);

    // Update room's last message
    room.lastMessage = message;
    this.saveRooms(this.loadRooms());

    return message;
  },

  /**
   * Send a file/image message.
   */
  async sendFile(
    roomId: string,
    type: MessageType.IMAGE | MessageType.FILE | MessageType.VIDEO | MessageType.AUDIO,
    fileInfo: { cid: string; fileName: string; mime: string; size: number; width?: number; height?: number },
  ): Promise<ChatMessage> {
    const active = IdentityService.getActiveIdentity();
    if (!active) throw new Error('No active identity');

    const message: ChatMessage = {
      id: uuidv4(),
      roomId,
      senderDid: active.did,
      type,
      payload: {
        type,
        cid: fileInfo.cid,
        fileName: fileInfo.fileName,
        mime: fileInfo.mime,
        fileSize: fileInfo.size,
        width: fileInfo.width,
        height: fileInfo.height,
      },
      timestamp: Date.now(),
      ttl: 604800,
      status: MessageStatus.SENDING,
      isOutgoing: true,
    };

    this.saveMessage(roomId, message);

    // Upload to relay (in production, also upload file via Bitswap/IPFS)
    try {
      await RelayService.storeMessage(
        this.loadRooms()[roomId]?.participants[0] || '',
        active.did,
        btoa(JSON.stringify(message.payload)),
        message.id,
        message.ttl,
      );
      message.status = MessageStatus.SENT;
    } catch {
      message.status = MessageStatus.FAILED;
    }

    this.updateMessage(roomId, message);
    return message;
  },

  /**
   * Sync messages for the active identity from the relay.
   */
  async syncMessages(since: number = 0): Promise<void> {
    const active = IdentityService.getActiveIdentity();
    if (!active) return;

    try {
      const response = await RelayService.syncMessages(active.did, since, 50);
      for (const msg of response.messages) {
        // Decode and decrypt
        const decoded = JSON.parse(atob(msg.envelope));
        const payloadStr = await ScpCoreBridge.doubleRatchetDecrypt(
          '', // sessionId lookup needed
          decoded.payload,
        );

        const payload = JSON.parse(payloadStr) as MessagePayload;
        const roomId = this.generateRoomId('direct', [msg.from_did]);

        // Only save if we don't already have it
        const existing = this.loadMessages(roomId);
        if (!existing.some(m => m.id === msg.id)) {
          const message: ChatMessage = {
            id: msg.id,
            roomId,
            senderDid: msg.from_did,
            type: payload.type || MessageType.TEXT,
            payload,
            timestamp: msg.timestamp * 1000,
            ttl: msg.ttl,
            status: MessageStatus.DELIVERED,
            isOutgoing: false,
          };
          this.saveMessage(roomId, message);
        }
      }
    } catch (error) {
      console.warn('Sync failed:', error);
    }
  },

  // ---- Storage Helpers ----

  loadRooms(): Record<string, ChatRoom> {
    const json = StorageService.loadChatRooms();
    return json ? JSON.parse(json) : {};
  },

  saveRooms(rooms: Record<string, ChatRoom>): void {
    StorageService.saveChatRooms(JSON.stringify(rooms));
  },

  loadMessages(roomId: string): ChatMessage[] {
    const json = StorageService.loadMessages(roomId);
    return json ? JSON.parse(json) : [];
  },

  saveMessage(roomId: string, message: ChatMessage): void {
    const messages = this.loadMessages(roomId);
    messages.push(message);
    StorageService.saveMessages(roomId, JSON.stringify(messages));
  },

  updateMessage(roomId: string, updated: ChatMessage): void {
    const messages = this.loadMessages(roomId);
    const idx = messages.findIndex(m => m.id === updated.id);
    if (idx >= 0) {
      messages[idx] = updated;
    }
    StorageService.saveMessages(roomId, JSON.stringify(messages));
  },

  generateRoomId(type: string, participantDids: string[]): string {
    const sorted = [...participantDids].sort().join(':');
    return `${type}:${sorted}`;
  },

  /**
   * Delete a message (local delete, not from relay).
   */
  deleteMessage(roomId: string, messageId: string): void {
    const messages = this.loadMessages(roomId);
    const filtered = messages.filter(m => m.id !== messageId);
    StorageService.saveMessages(roomId, JSON.stringify(filtered));
  },
};

export default ChatService;
