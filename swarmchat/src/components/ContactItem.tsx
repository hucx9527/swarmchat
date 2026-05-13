/**
 * ContactItem — renders a contact row with avatar, name, DID preview,
 * online status indicator, and optional action buttons.
 */

import React, { memo } from 'react';
import { View, Text, TouchableOpacity, StyleSheet } from 'react-native';
import type { Contact } from '../types';
import { ContactStatus } from '../types';
import { THEME } from '../utils/constants';

interface Props {
  contact: Contact;
  onPress: () => void;
  onLongPress?: () => void;
  showStatus?: boolean;
}

function ContactItem({ contact, onPress, onLongPress, showStatus = true }: Props) {
  const statusColor = (() => {
    switch (contact.status) {
      case ContactStatus.ONLINE:
        return THEME.success;
      case ContactStatus.AWAY:
        return THEME.warning;
      default:
        return THEME.textMuted;
    }
  })();

  const initials = contact.displayName
    .split(/\s+/)
    .map(w => w[0] ?? '')
    .join('')
    .toUpperCase()
    .substring(0, 2);

  return (
    <TouchableOpacity
      style={styles.container}
      onPress={onPress}
      onLongPress={onLongPress}
      activeOpacity={0.6}>
      {/* Avatar */}
      <View style={styles.avatarContainer}>
        <View style={[styles.avatar, { backgroundColor: THEME.primary }]}>
          <Text style={styles.avatarText}>{initials}</Text>
        </View>
        {showStatus && (
          <View style={[styles.statusDot, { backgroundColor: statusColor }]} />
        )}
      </View>

      {/* Info */}
      <View style={styles.info}>
        <Text style={styles.name} numberOfLines={1}>
          {contact.displayName}
          {contact.isAgent && (
            <Text style={styles.agentBadge}> 🤖</Text>
          )}
        </Text>
        <Text style={styles.did} numberOfLines={1}>
          {contact.did}
        </Text>
      </View>

      {/* Arrow indicator */}
      <Text style={styles.arrow}>›</Text>
    </TouchableOpacity>
  );
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
  avatarContainer: {
    position: 'relative',
    marginRight: 12,
  },
  avatar: {
    width: 48,
    height: 48,
    borderRadius: 24,
    justifyContent: 'center',
    alignItems: 'center',
  },
  avatarText: {
    color: THEME.white,
    fontSize: 16,
    fontWeight: '600',
  },
  statusDot: {
    position: 'absolute',
    bottom: 0,
    right: 0,
    width: 12,
    height: 12,
    borderRadius: 6,
    borderWidth: 2,
    borderColor: THEME.background,
  },
  info: {
    flex: 1,
    justifyContent: 'center',
  },
  name: {
    fontSize: 16,
    fontWeight: '500',
    color: THEME.text,
    marginBottom: 2,
  },
  agentBadge: {
    fontSize: 12,
  },
  did: {
    fontSize: 12,
    color: THEME.textMuted,
  },
  arrow: {
    fontSize: 22,
    color: THEME.textMuted,
    paddingLeft: 8,
  },
});

export default memo(ContactItem);
