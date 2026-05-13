/**
 * ContactDetailScreen — detailed view for a single contact.
 *
 * Displays the contact's DID, PeerId, status, and provides actions:
 * "Send Message" (opens chat), "Remove Contact", "Verify Identity".
 */

import React, { useCallback, useState } from 'react';
import {
  View,
  Text,
  ScrollView,
  TouchableOpacity,
  StyleSheet,
  Alert,
} from 'react-native';
import type { NativeStackScreenProps } from '@react-navigation/native-stack';
import type { HomeStackParamList } from '../types';
import { useContacts } from '../hooks/useContacts';
import { useChat } from '../hooks/useChat';
import { THEME } from '../utils/constants';

type Props = NativeStackScreenProps<HomeStackParamList, 'ContactDetail'>;

export default function ContactDetailScreen({ route, navigation }: Props) {
  const { did } = route.params;
  const { getContact, removeContact, updateStatus } = useContacts();
  const { openDirectRoom } = useChat();
  const contact = getContact(did);

  const handleSendMessage = useCallback(async () => {
    if (!contact) return;
    const room = await openDirectRoom(contact.did, contact.displayName);
    navigation.navigate('Chat', {
      roomId: room.id,
      peerDid: contact.did,
      peerName: contact.displayName,
    });
  }, [contact, openDirectRoom, navigation]);

  const handleRemove = useCallback(() => {
    if (!contact) return;
    Alert.alert(
      'Remove Contact',
      `Remove ${contact.displayName} from your contacts?`,
      [
        { text: 'Cancel', style: 'cancel' },
        {
          text: 'Remove',
          style: 'destructive',
          onPress: () => {
            removeContact(contact.did);
            navigation.goBack();
          },
        },
      ],
    );
  }, [contact, removeContact, navigation]);

  const handleVerify = useCallback(() => {
    Alert.alert(
      'Verify Identity',
      'In a production build, this would verify the contact\'s DID signature against their public key. Verification status: OK (placeholder)',
    );
  }, []);

  if (!contact) {
    return (
      <View style={styles.centered}>
        <Text style={styles.emptyText}>Contact not found</Text>
        <Text style={styles.didText}>{did}</Text>
      </View>
    );
  }

  const initials = contact.displayName
    .split(/\s+/)
    .map(w => w[0] ?? '')
    .join('')
    .toUpperCase()
    .substring(0, 2);

  return (
    <ScrollView contentContainerStyle={styles.container}>
      {/* Avatar & Name */}
      <View style={styles.header}>
        <View style={styles.avatar}>
          <Text style={styles.avatarText}>{initials}</Text>
        </View>
        <Text style={styles.displayName}>{contact.displayName}</Text>
        <View style={styles.statusRow}>
          <View
            style={[
              styles.statusDot,
              { backgroundColor: getStatusColor(contact.status) },
            ]}
          />
          <Text style={styles.statusText}>{contact.status}</Text>
          {contact.isAgent && (
            <View style={styles.agentBadge}>
              <Text style={styles.agentBadgeText}>Agent</Text>
            </View>
          )}
        </View>
      </View>

      {/* Details */}
      <View style={styles.detailsSection}>
        <View style={styles.detailRow}>
          <Text style={styles.detailLabel}>DID</Text>
          <Text style={styles.detailValue} selectable>
            {contact.did}
          </Text>
        </View>

        <View style={styles.detailRow}>
          <Text style={styles.detailLabel}>PeerId</Text>
          <Text style={styles.detailValue} selectable>
            {contact.peerId}
          </Text>
        </View>

        {contact.publicKey && (
          <View style={styles.detailRow}>
            <Text style={styles.detailLabel}>Public Key</Text>
            <Text style={styles.detailValue} selectable numberOfLines={1}>
              {contact.publicKey.substring(0, 32)}...
            </Text>
          </View>
        )}

        <View style={styles.detailRow}>
          <Text style={styles.detailLabel}>Added</Text>
          <Text style={styles.detailValue}>
            {new Date(contact.addedAt).toLocaleDateString()}
          </Text>
        </View>

        {contact.lastSeen && (
          <View style={styles.detailRow}>
            <Text style={styles.detailLabel}>Last Seen</Text>
            <Text style={styles.detailValue}>
              {new Date(contact.lastSeen).toLocaleString()}
            </Text>
          </View>
        )}

        {contact.isAgent && contact.agentCapabilities && (
          <View style={styles.detailRow}>
            <Text style={styles.detailLabel}>Capabilities</Text>
            <Text style={styles.detailValue}>
              {contact.agentCapabilities.join(', ')}
            </Text>
          </View>
        )}
      </View>

      {/* Actions */}
      <View style={styles.actionsSection}>
        <TouchableOpacity
          style={styles.primaryAction}
          onPress={handleSendMessage}>
          <Text style={styles.primaryActionText}>💬 Send Message</Text>
        </TouchableOpacity>

        <TouchableOpacity style={styles.action} onPress={handleVerify}>
          <Text style={styles.actionText}>🔍 Verify Identity</Text>
        </TouchableOpacity>

        <TouchableOpacity
          style={[styles.action, styles.dangerAction]}
          onPress={handleRemove}>
          <Text style={styles.dangerActionText}>🗑 Remove Contact</Text>
        </TouchableOpacity>
      </View>
    </ScrollView>
  );
}

function getStatusColor(status: string): string {
  switch (status) {
    case 'online':
      return THEME.success;
    case 'away':
      return THEME.warning;
    default:
      return THEME.textMuted;
  }
}

const styles = StyleSheet.create({
  container: {
    backgroundColor: THEME.background,
    paddingBottom: 40,
  },
  centered: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: THEME.background,
    padding: 24,
  },
  emptyText: {
    fontSize: 16,
    color: THEME.textMuted,
    marginBottom: 8,
  },
  didText: {
    fontSize: 11,
    color: THEME.border,
    fontFamily: 'monospace',
  },
  header: {
    alignItems: 'center',
    paddingTop: 32,
    paddingBottom: 24,
  },
  avatar: {
    width: 80,
    height: 80,
    borderRadius: 40,
    backgroundColor: THEME.primary,
    justifyContent: 'center',
    alignItems: 'center',
    marginBottom: 12,
  },
  avatarText: {
    color: THEME.white,
    fontSize: 28,
    fontWeight: '700',
  },
  displayName: {
    fontSize: 22,
    fontWeight: '700',
    color: THEME.text,
    marginBottom: 6,
  },
  statusRow: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  statusDot: {
    width: 10,
    height: 10,
    borderRadius: 5,
    marginRight: 6,
  },
  statusText: {
    fontSize: 13,
    color: THEME.textMuted,
    textTransform: 'capitalize',
    marginRight: 8,
  },
  agentBadge: {
    backgroundColor: THEME.primary + '33',
    borderRadius: 8,
    paddingHorizontal: 8,
    paddingVertical: 2,
  },
  agentBadgeText: {
    fontSize: 11,
    color: THEME.primary,
    fontWeight: '600',
  },
  detailsSection: {
    paddingHorizontal: 24,
    marginBottom: 28,
  },
  detailRow: {
    backgroundColor: THEME.surface,
    borderRadius: 8,
    padding: 12,
    marginBottom: 8,
    borderWidth: 1,
    borderColor: THEME.border,
  },
  detailLabel: {
    fontSize: 11,
    fontWeight: '600',
    color: THEME.textMuted,
    textTransform: 'uppercase',
    letterSpacing: 0.5,
    marginBottom: 4,
  },
  detailValue: {
    fontSize: 13,
    color: THEME.text,
    fontFamily: 'monospace',
  },
  actionsSection: {
    paddingHorizontal: 24,
  },
  primaryAction: {
    backgroundColor: THEME.primary,
    borderRadius: 12,
    paddingVertical: 16,
    alignItems: 'center',
    marginBottom: 10,
  },
  primaryActionText: {
    color: THEME.white,
    fontSize: 16,
    fontWeight: '600',
  },
  action: {
    backgroundColor: THEME.surface,
    borderRadius: 10,
    paddingVertical: 14,
    alignItems: 'center',
    marginBottom: 10,
    borderWidth: 1,
    borderColor: THEME.border,
  },
  actionText: {
    color: THEME.text,
    fontSize: 15,
    fontWeight: '500',
  },
  dangerAction: {
    borderColor: THEME.danger + '44',
  },
  dangerActionText: {
    color: THEME.danger,
    fontSize: 15,
    fontWeight: '500',
  },
});
