/**
 * AddFriendScreen — add a contact by entering their DID manually or
 * scanning a QR code.
 *
 * The user can:
 *  - Type/paste a DID (did:key:...) or SCP QR string (scp://...)
 *  - Provide a display name for the contact
 *  - Tap "Scan QR Code" to jump to the QRScanScreen
 */

import React, { useState, useCallback } from 'react';
import {
  View,
  Text,
  TextInput,
  TouchableOpacity,
  StyleSheet,
  Alert,
  ScrollView,
  KeyboardAvoidingView,
  Platform,
} from 'react-native';
import { useNavigation } from '@react-navigation/native';
import type { NativeStackNavigationProp } from '@react-navigation/native-stack';
import type { RootStackParamList } from '../types';
import { useContacts } from '../hooks/useContacts';
import { extractDidFromScan, isScpQrCode, decodeDidQrPayload } from '../utils/qr';
import { THEME } from '../utils/constants';

type Nav = NativeStackNavigationProp<RootStackParamList>;

export default function AddFriendScreen() {
  const navigation = useNavigation<Nav>();
  const { addContact } = useContacts();

  const [didInput, setDidInput] = useState('');
  const [displayName, setDisplayName] = useState('');

  const parsedDid = extractDidFromScan(didInput.trim());
  const qrPayload = isScpQrCode(didInput.trim())
    ? decodeDidQrPayload(didInput.trim())
    : null;

  const handleAdd = useCallback(async () => {
    const did = parsedDid;
    if (!did) {
      Alert.alert(
        'Invalid Input',
        'Please enter a valid DID (e.g. did:key:...) or scan a SwarmChat QR code.',
      );
      return;
    }

    const name = displayName.trim() || qrPayload?.name || did.substring(0, 24) + '...';
    const publicKey = qrPayload?.pk;

    try {
      addContact({
        did,
        peerId: did, // PeerId will be resolved via DHT in production
        displayName: name,
        publicKey: publicKey ?? '',
      });
      Alert.alert('Added!', `${name} has been added to your contacts.`, [
        { text: 'OK', onPress: () => navigation.goBack() },
      ]);
    } catch (err: any) {
      Alert.alert('Error', err.message || 'Failed to add contact.');
    }
  }, [parsedDid, displayName, qrPayload, addContact, navigation]);

  const handleScanQR = useCallback(() => {
    navigation.navigate('QRScan');
  }, [navigation]);

  return (
    <KeyboardAvoidingView
      style={{ flex: 1, backgroundColor: THEME.background }}
      behavior={Platform.OS === 'ios' ? 'padding' : undefined}>
      <ScrollView
        contentContainerStyle={styles.container}
        keyboardShouldPersistTaps="handled">
        <Text style={styles.heading}>Add Friend</Text>
        <Text style={styles.subtext}>
          Enter a DID or scan a QR code to add someone to your contacts.
        </Text>

        {/* DID Input */}
        <Text style={styles.inputLabel}>DID or QR String</Text>
        <TextInput
          style={[styles.input, styles.textArea]}
          value={didInput}
          onChangeText={setDidInput}
          placeholder="did:key:... or scp://..."
          placeholderTextColor={THEME.textMuted}
          autoCapitalize="none"
          autoCorrect={false}
          multiline
          numberOfLines={4}
        />

        {/* Pre-filled display name from QR */}
        {qrPayload?.name && !displayName && (
          <View style={styles.qrInfo}>
            <Text style={styles.qrLabel}>From QR: {qrPayload.name}</Text>
          </View>
        )}

        {parsedDid && (
          <View style={styles.parsedContainer}>
            <Text style={styles.parsedLabel}>Parsed DID</Text>
            <Text style={styles.parsedValue} selectable>
              {parsedDid}
            </Text>
          </View>
        )}

        {/* Display Name Input */}
        <Text style={styles.inputLabel}>Display Name (optional)</Text>
        <TextInput
          style={styles.input}
          value={displayName}
          onChangeText={setDisplayName}
          placeholder="How should they appear in your contacts?"
          placeholderTextColor={THEME.textMuted}
          autoCapitalize="words"
          maxLength={64}
        />

        {/* Add Button */}
        <TouchableOpacity
          style={[styles.addButton, !parsedDid && styles.disabledButton]}
          onPress={handleAdd}
          disabled={!parsedDid}
          activeOpacity={0.8}>
          <Text style={styles.addButtonText}>Add to Contacts</Text>
        </TouchableOpacity>

        {/* Divider */}
        <View style={styles.divider}>
          <View style={styles.dividerLine} />
          <Text style={styles.dividerText}>or</Text>
          <View style={styles.dividerLine} />
        </View>

        {/* Scan QR Button */}
        <TouchableOpacity
          style={styles.scanButton}
          onPress={handleScanQR}
          activeOpacity={0.8}>
          <Text style={styles.scanIcon}>📷</Text>
          <Text style={styles.scanButtonText}>Scan QR Code</Text>
        </TouchableOpacity>

        {/* Share own DID */}
        <TouchableOpacity
          style={styles.shareButton}
          onPress={() => {
            // In production, navigate to own QR share screen
            Alert.alert('Share Your DID', 'Go to Profile to share your QR code.');
          }}
          activeOpacity={0.8}>
          <Text style={styles.shareButtonText}>Share My QR Code</Text>
        </TouchableOpacity>
      </ScrollView>
    </KeyboardAvoidingView>
  );
}

const styles = StyleSheet.create({
  container: {
    flexGrow: 1,
    backgroundColor: THEME.background,
    padding: 24,
  },
  heading: {
    fontSize: 22,
    fontWeight: '700',
    color: THEME.text,
    marginBottom: 6,
  },
  subtext: {
    fontSize: 13,
    color: THEME.textMuted,
    lineHeight: 19,
    marginBottom: 24,
  },
  inputLabel: {
    fontSize: 13,
    fontWeight: '600',
    color: THEME.textMuted,
    marginBottom: 6,
    marginTop: 14,
    textTransform: 'uppercase',
    letterSpacing: 0.5,
  },
  input: {
    backgroundColor: THEME.surface,
    borderRadius: 10,
    borderWidth: 1,
    borderColor: THEME.border,
    paddingHorizontal: 14,
    paddingVertical: 12,
    fontSize: 15,
    color: THEME.text,
  },
  textArea: {
    minHeight: 100,
    textAlignVertical: 'top',
    fontFamily: 'monospace',
    fontSize: 13,
  },
  qrInfo: {
    backgroundColor: THEME.success + '15',
    borderRadius: 8,
    padding: 10,
    marginTop: 8,
  },
  qrLabel: {
    fontSize: 12,
    color: THEME.success,
  },
  parsedContainer: {
    backgroundColor: THEME.accent + '10',
    borderRadius: 8,
    padding: 10,
    marginTop: 8,
  },
  parsedLabel: {
    fontSize: 11,
    fontWeight: '600',
    color: THEME.accent,
    marginBottom: 4,
    textTransform: 'uppercase',
  },
  parsedValue: {
    fontSize: 12,
    color: THEME.text,
    fontFamily: 'monospace',
  },
  addButton: {
    backgroundColor: THEME.primary,
    borderRadius: 12,
    paddingVertical: 16,
    alignItems: 'center',
    marginTop: 28,
  },
  disabledButton: {
    opacity: 0.4,
  },
  addButtonText: {
    color: THEME.white,
    fontSize: 16,
    fontWeight: '600',
  },
  divider: {
    flexDirection: 'row',
    alignItems: 'center',
    marginVertical: 24,
  },
  dividerLine: {
    flex: 1,
    height: 1,
    backgroundColor: THEME.border,
  },
  dividerText: {
    marginHorizontal: 16,
    color: THEME.textMuted,
    fontSize: 13,
  },
  scanButton: {
    backgroundColor: THEME.surface,
    borderRadius: 12,
    paddingVertical: 16,
    alignItems: 'center',
    flexDirection: 'row',
    justifyContent: 'center',
    borderWidth: 1,
    borderColor: THEME.border,
  },
  scanIcon: {
    fontSize: 20,
    marginRight: 8,
  },
  scanButtonText: {
    color: THEME.text,
    fontSize: 15,
    fontWeight: '500',
  },
  shareButton: {
    marginTop: 12,
    alignItems: 'center',
    paddingVertical: 12,
  },
  shareButtonText: {
    color: THEME.accent,
    fontSize: 14,
  },
});
