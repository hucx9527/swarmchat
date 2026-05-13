/**
 * ProfileScreen — user profile showing the active identity, DID, PeerId,
 * QR code for sharing, and identity management options.
 */

import React, { useCallback, useState } from 'react';
import {
  View,
  Text,
  ScrollView,
  TouchableOpacity,
  StyleSheet,
  Alert,
  Modal,
  FlatList,
} from 'react-native';
import { useNavigation } from '@react-navigation/native';
import { useIdentity } from '../hooks/useIdentity';
import { useContacts } from '../hooks/useContacts';
import QRCode from '../components/QRCode';
import { encodeDidQrPayload } from '../utils/qr';
import { THEME } from '../utils/constants';

export default function ProfileScreen() {
  const navigation = useNavigation<any>();
  const {
    activeIdentity,
    allIdentities,
    setDefault,
    deleteIdentity,
  } = useIdentity();
  const [switchModalVisible, setSwitchModalVisible] = useState(false);

  const handleSwitchIdentity = useCallback(
    (label: string) => {
      setDefault(label);
      setSwitchModalVisible(false);
      Alert.alert('Switched', `Active identity is now "${label}"`);
    },
    [setDefault],
  );

  const handleDeleteIdentity = useCallback(
    (label: string) => {
      Alert.alert(
        'Delete Identity',
        `Are you sure you want to delete "${label}"? This cannot be undone unless you have the recovery phrase.`,
        [
          { text: 'Cancel', style: 'cancel' },
          {
            text: 'Delete',
            style: 'destructive',
            onPress: () => {
              deleteIdentity(label);
            },
          },
        ],
      );
    },
    [deleteIdentity],
  );

  const handleSettings = useCallback(() => {
    navigation.navigate('Settings');
  }, [navigation]);

  if (!activeIdentity) {
    return (
      <View style={styles.centered}>
        <Text style={styles.emptyText}>No identity loaded</Text>
      </View>
    );
  }

  const qrValue = encodeDidQrPayload(
    activeIdentity.did,
    activeIdentity.nickname ?? undefined,
    activeIdentity.publicKey,
  );

  return (
    <ScrollView contentContainerStyle={styles.container}>
      {/* Avatar */}
      <View style={styles.avatarSection}>
        <View style={styles.avatarCircle}>
          <Text style={styles.avatarText}>
            {(activeIdentity.nickname ?? activeIdentity.label)
              .substring(0, 2)
              .toUpperCase()}
          </Text>
        </View>
        <Text style={styles.nickname}>
          {activeIdentity.nickname ?? activeIdentity.label}
        </Text>
        {activeIdentity.description && (
          <Text style={styles.description}>{activeIdentity.description}</Text>
        )}
        <Text style={styles.label}>Label: {activeIdentity.label}</Text>
      </View>

      {/* QR Code */}
      <View style={styles.qrSection}>
        <Text style={styles.sectionTitle}>Your QR Code</Text>
        <Text style={styles.sectionSub}>
          Share this with others to let them add you
        </Text>
        <View style={styles.qrWrapper}>
          <QRCode value={qrValue} size={200} />
        </View>
      </View>

      {/* Identity Details */}
      <View style={styles.detailsSection}>
        <Text style={styles.sectionTitle}>Identity Details</Text>

        <View style={styles.detailRow}>
          <Text style={styles.detailLabel}>DID</Text>
          <Text style={styles.detailValue} selectable numberOfLines={2}>
            {activeIdentity.did}
          </Text>
        </View>

        <View style={styles.detailRow}>
          <Text style={styles.detailLabel}>PeerId</Text>
          <Text style={styles.detailValue} selectable numberOfLines={1}>
            {activeIdentity.peerId}
          </Text>
        </View>

        <View style={styles.detailRow}>
          <Text style={styles.detailLabel}>Public Key</Text>
          <Text style={styles.detailValue} selectable numberOfLines={1}>
            {activeIdentity.publicKey.substring(0, 32)}...
          </Text>
        </View>

        <View style={styles.detailRow}>
          <Text style={styles.detailLabel}>Created</Text>
          <Text style={styles.detailValue}>
            {new Date(activeIdentity.createdAt).toLocaleDateString()}
          </Text>
        </View>

        <View style={styles.detailRow}>
          <Text style={styles.detailLabel}>Default</Text>
          <Text style={styles.detailValue}>
            {activeIdentity.isDefault ? '✅ Yes' : 'No'}
          </Text>
        </View>
      </View>

      {/* Actions */}
      <View style={styles.actionsSection}>
        <TouchableOpacity
          style={styles.actionButton}
          onPress={() => setSwitchModalVisible(true)}>
          <Text style={styles.actionButtonText}>Switch Identity</Text>
        </TouchableOpacity>

        <TouchableOpacity style={styles.actionButton} onPress={handleSettings}>
          <Text style={styles.actionButtonText}>Settings</Text>
        </TouchableOpacity>

        {allIdentities.length > 1 && (
          <TouchableOpacity
            style={[styles.actionButton, styles.dangerButton]}
            onPress={() => handleDeleteIdentity(activeIdentity.label)}>
            <Text style={styles.dangerButtonText}>Delete This Identity</Text>
          </TouchableOpacity>
        )}
      </View>

      {/* Identity Switch Modal */}
      <Modal
        visible={switchModalVisible}
        transparent
        animationType="slide"
        onRequestClose={() => setSwitchModalVisible(false)}>
        <View style={styles.modalOverlay}>
          <View style={styles.modalContent}>
            <View style={styles.modalHeader}>
              <Text style={styles.modalTitle}>Switch Identity</Text>
              <TouchableOpacity onPress={() => setSwitchModalVisible(false)}>
                <Text style={styles.modalClose}>✕</Text>
              </TouchableOpacity>
            </View>
            {allIdentities.map(id => (
              <TouchableOpacity
                key={id.label}
                style={[
                  styles.identityOption,
                  id.label === activeIdentity.label && styles.identityOptionActive,
                ]}
                onPress={() => handleSwitchIdentity(id.label)}>
                <Text style={styles.identityLabel}>{id.label}</Text>
                {id.nickname && (
                  <Text style={styles.identityNickname}>{id.nickname}</Text>
                )}
                {id.isDefault && (
                  <Text style={styles.defaultBadge}>Default</Text>
                )}
              </TouchableOpacity>
            ))}
          </View>
        </View>
      </Modal>
    </ScrollView>
  );
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
  },
  emptyText: {
    color: THEME.textMuted,
    fontSize: 15,
  },
  avatarSection: {
    alignItems: 'center',
    paddingTop: 72,
    paddingBottom: 24,
  },
  avatarCircle: {
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
  nickname: {
    fontSize: 22,
    fontWeight: '700',
    color: THEME.text,
  },
  description: {
    fontSize: 13,
    color: THEME.textMuted,
    marginTop: 4,
  },
  label: {
    fontSize: 12,
    color: THEME.accent,
    marginTop: 4,
    fontFamily: 'monospace',
  },
  qrSection: {
    paddingHorizontal: 24,
    marginBottom: 28,
    alignItems: 'center',
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: THEME.text,
    marginBottom: 4,
  },
  sectionSub: {
    fontSize: 12,
    color: THEME.textMuted,
    marginBottom: 16,
  },
  qrWrapper: {
    backgroundColor: THEME.white,
    borderRadius: 16,
    padding: 16,
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
  actionButton: {
    backgroundColor: THEME.surface,
    borderRadius: 10,
    paddingVertical: 14,
    alignItems: 'center',
    marginBottom: 10,
    borderWidth: 1,
    borderColor: THEME.border,
  },
  actionButtonText: {
    color: THEME.text,
    fontSize: 15,
    fontWeight: '500',
  },
  dangerButton: {
    borderColor: THEME.danger + '44',
  },
  dangerButtonText: {
    color: THEME.danger,
    fontSize: 15,
    fontWeight: '500',
  },
  // Modal
  modalOverlay: {
    flex: 1,
    backgroundColor: 'rgba(0,0,0,0.5)',
    justifyContent: 'flex-end',
  },
  modalContent: {
    backgroundColor: THEME.surface,
    borderTopLeftRadius: 20,
    borderTopRightRadius: 20,
    maxHeight: '50%',
    paddingBottom: 30,
  },
  modalHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingHorizontal: 20,
    paddingTop: 18,
    paddingBottom: 12,
    borderBottomWidth: 1,
    borderBottomColor: THEME.border,
  },
  modalTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: THEME.text,
  },
  modalClose: {
    fontSize: 20,
    color: THEME.textMuted,
  },
  identityOption: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: 20,
    paddingVertical: 14,
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: THEME.border,
  },
  identityOptionActive: {
    backgroundColor: THEME.primary + '15',
  },
  identityLabel: {
    fontSize: 16,
    fontWeight: '500',
    color: THEME.text,
    flex: 1,
  },
  identityNickname: {
    fontSize: 14,
    color: THEME.textMuted,
    marginRight: 12,
  },
  defaultBadge: {
    fontSize: 11,
    color: THEME.success,
    fontWeight: '600',
  },
});
