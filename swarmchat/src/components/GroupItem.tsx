/**
 * GroupItem — renders a group chat row with group icon, name,
 * member count, last message preview, and unread badge.
 */

import React, { memo } from 'react';
import { View, Text, TouchableOpacity, StyleSheet } from 'react-native';
import type { Group } from '../types';
import { THEME } from '../utils/constants';

interface Props {
  group: Group;
  onPress: () => void;
  onLongPress?: () => void;
}

function GroupItem({ group, onPress, onLongPress }: Props) {
  const initials = group.name
    .split(/\s+/)
    .map(w => w[0] ?? '')
    .join('')
    .toUpperCase()
    .substring(0, 2);

  const lastMessagePreview = group.lastMessage
    ? (() => {
        const m = group.lastMessage;
        const sender =
          m.senderDid === group.createdBy
            ? ''
            : group.members.find(mb => mb.did === m.senderDid)?.displayName ?? m.senderDid.substring(0, 8);
        const prefix = sender ? `${sender}: ` : '';
        const body = m.payload.body ?? m.payload.fileName ?? m.type;
        return `${prefix}${body}`;
      })()
    : `Created · ${formatDate(group.createdAt)}`;

  return (
    <TouchableOpacity
      style={styles.container}
      onPress={onPress}
      onLongPress={onLongPress}
      activeOpacity={0.6}>
      {/* Avatar */}
      <View style={[styles.avatar, { backgroundColor: THEME.secondary }]}>
        <Text style={styles.avatarText}>{initials}</Text>
      </View>

      {/* Info */}
      <View style={styles.info}>
        <View style={styles.topRow}>
          <Text style={styles.name} numberOfLines={1}>
            {group.name}
          </Text>
          {group.lastMessage && (
            <Text style={styles.time}>
              {formatTime(group.lastMessage.timestamp)}
            </Text>
          )}
        </View>
        <View style={styles.bottomRow}>
          <Text style={styles.lastMessage} numberOfLines={1}>
            {lastMessagePreview}
          </Text>
          {group.unreadCount > 0 && (
            <View style={styles.unreadBadge}>
              <Text style={styles.unreadCount}>
                {group.unreadCount > 99 ? '99+' : group.unreadCount}
              </Text>
            </View>
          )}
        </View>
        <Text style={styles.memberCount}>
          {group.members.length} member{group.members.length !== 1 ? 's' : ''}
          {group.joinPolicy === 'invite' ? ' · invite-only' : ' · open'}
        </Text>
      </View>

      <Text style={styles.arrow}>›</Text>
    </TouchableOpacity>
  );
}

function formatTime(timestamp: number): string {
  const d = new Date(timestamp);
  const now = new Date();
  const isToday = d.toDateString() === now.toDateString();
  if (isToday) {
    return `${d.getHours().toString().padStart(2, '0')}:${d.getMinutes().toString().padStart(2, '0')}`;
  }
  const yesterday = new Date(now);
  yesterday.setDate(yesterday.getDate() - 1);
  if (d.toDateString() === yesterday.toDateString()) {
    return 'Yesterday';
  }
  return `${d.getMonth() + 1}/${d.getDate()}`;
}

function formatDate(timestamp: number): string {
  const d = new Date(timestamp);
  return `${d.getMonth() + 1}/${d.getDate()}/${d.getFullYear().toString().substr(-2)}`;
}

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingVertical: 12,
    paddingHorizontal: 16,
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: THEME.border,
  },
  avatar: {
    width: 48,
    height: 48,
    borderRadius: 24,
    justifyContent: 'center',
    alignItems: 'center',
    marginRight: 12,
  },
  avatarText: {
    color: THEME.white,
    fontSize: 16,
    fontWeight: '600',
  },
  info: {
    flex: 1,
    justifyContent: 'center',
  },
  topRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 2,
  },
  name: {
    fontSize: 16,
    fontWeight: '500',
    color: THEME.text,
    flex: 1,
    marginRight: 8,
  },
  time: {
    fontSize: 12,
    color: THEME.textMuted,
  },
  bottomRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 2,
  },
  lastMessage: {
    fontSize: 13,
    color: THEME.textMuted,
    flex: 1,
    marginRight: 8,
  },
  unreadBadge: {
    backgroundColor: THEME.primary,
    borderRadius: 10,
    minWidth: 20,
    height: 20,
    justifyContent: 'center',
    alignItems: 'center',
    paddingHorizontal: 6,
  },
  unreadCount: {
    color: THEME.white,
    fontSize: 11,
    fontWeight: '600',
  },
  memberCount: {
    fontSize: 11,
    color: THEME.accent,
    marginTop: 1,
  },
  arrow: {
    fontSize: 22,
    color: THEME.textMuted,
    paddingLeft: 8,
  },
});

export default memo(GroupItem);
