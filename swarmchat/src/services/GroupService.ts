/**
 * GroupService — manages swarm groups, members, and group encryption.
 */

import { Group, GroupMember, ChatMessage, MessageType, MessageStatus } from '../types';
import { ScpCoreBridge } from './ScpCoreBridge';
import { RelayService } from './RelayService';
import { IdentityService } from './IdentityService';
import { StorageService } from './StorageService';
import { v4 as uuidv4 } from 'uuid';

export const GroupService = {
  /**
   * Create a new group.
   */
  async create(
    name: string,
    description?: string,
    joinPolicy: 'invite' | 'open' = 'invite',
  ): Promise<Group> {
    const active = IdentityService.getActiveIdentity();
    if (!active) throw new Error('No active identity');

    const groupId = `group:${uuidv4()}`;

    // Create Sender Key for group encryption
    const senderKey = await ScpCoreBridge.senderKeyCreate(groupId);

    const group: Group = {
      id: groupId,
      name,
      description,
      joinPolicy,
      whoCanSend: 'all',
      members: [{
        did: active.did,
        displayName: active.nickname || active.label,
        role: 'admin',
        joinedAt: Date.now(),
      }],
      createdAt: Date.now(),
      createdBy: active.did,
      unreadCount: 0,
    };

    const groups = this.loadGroups();
    groups[groupId] = group;
    this.saveGroups(groups);

    return group;
  },

  /**
   * Invite members to a group.
   */
  async invite(groupId: string, inviteeDids: string[]): Promise<void> {
    const active = IdentityService.getActiveIdentity();
    if (!active) throw new Error('No active identity');

    const groups = this.loadGroups();
    const group = groups[groupId];
    if (!group) throw new Error('Group not found');

    for (const did of inviteeDids) {
      // Send invite message via relay
      const inviteMsg = {
        type: MessageType.GROUP_INVITE,
        group_id: groupId,
        group_name: group.name,
        inviter: active.did,
        invitee: did,
      };

      await RelayService.storeMessage(
        did,
        active.did,
        btoa(JSON.stringify(inviteMsg)),
        uuidv4(),
        604800,
      );
    }
  },

  /**
   * Join a group (for open groups) or accept an invite.
   */
  async join(groupId: string): Promise<void> {
    const active = IdentityService.getActiveIdentity();
    if (!active) throw new Error('No active identity');

    const groups = this.loadGroups();
    const group = groups[groupId];

    if (group) {
      // Already know about this group
      if (!group.members.find(m => m.did === active.did)) {
        group.members.push({
          did: active.did,
          displayName: active.nickname || active.label,
          role: 'member',
          joinedAt: Date.now(),
        });
      }
    } else {
      // Create local group entry (fetched from DHT in production)
      groups[groupId] = {
        id: groupId,
        name: 'Unknown Group',
        joinPolicy: 'open',
        whoCanSend: 'all',
        members: [{
          did: active.did,
          displayName: active.nickname || active.label,
          role: 'member',
          joinedAt: Date.now(),
        }],
        createdAt: Date.now(),
        createdBy: '',
        unreadCount: 0,
      };
    }

    this.saveGroups(groups);
  },

  /**
   * Leave a group.
   */
  leave(groupId: string): void {
    const active = IdentityService.getActiveIdentity();
    if (!active) return;

    const groups = this.loadGroups();
    const group = groups[groupId];
    if (group) {
      group.members = group.members.filter(m => m.did !== active.did);
      if (group.members.length === 0) {
        delete groups[groupId];
      }
      this.saveGroups(groups);
    }
  },

  /**
   * Send a message to a group (encrypted with Sender Key).
   */
  async sendMessage(groupId: string, text: string): Promise<ChatMessage> {
    const active = IdentityService.getActiveIdentity();
    if (!active) throw new Error('No active identity');

    // Encrypt with Sender Key
    const ciphertext = await ScpCoreBridge.senderKeyEncrypt(
      groupId,
      JSON.stringify({ type: MessageType.TEXT, body: text }),
    );

    const message: ChatMessage = {
      id: uuidv4(),
      roomId: groupId,
      senderDid: active.did,
      type: MessageType.TEXT,
      payload: { type: MessageType.TEXT, body: text },
      timestamp: Date.now(),
      ttl: 604800,
      status: MessageStatus.SENDING,
      isOutgoing: true,
    };

    // Save locally
    const messages = StorageService.loadMessages(groupId)
      ? JSON.parse(StorageService.loadMessages(groupId)!)
      : [];
    messages.push(message);
    StorageService.saveMessages(groupId, JSON.stringify(messages));

    // Broadcast to all members via relay
    const group = this.loadGroups()[groupId];
    if (group) {
      for (const member of group.members) {
        if (member.did !== active.did) {
          try {
            await RelayService.storeMessage(
              member.did,
              active.did,
              btoa(JSON.stringify({ type: 'group', group_id: groupId, payload: ciphertext })),
              message.id,
              604800,
            );
          } catch {
            // Individual delivery failure; member will sync later
          }
        }
      }
    }

    message.status = MessageStatus.SENT;
    return message;
  },

  // ---- Storage ----

  loadGroups(): Record<string, Group> {
    const json = StorageService.loadGroups();
    return json ? JSON.parse(json) : {};
  },

  saveGroups(groups: Record<string, Group>): void {
    StorageService.saveGroups(JSON.stringify(groups));
  },

  getGroup(groupId: string): Group | undefined {
    return this.loadGroups()[groupId];
  },
};

export default GroupService;
