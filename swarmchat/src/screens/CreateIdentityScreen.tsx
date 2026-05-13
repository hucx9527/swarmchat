/**
 * CreateIdentityScreen — lets the user create a new decentralized identity.
 *
 * Inputs: label (required), nickname (optional), description (optional)
 * Generates a BIP39 mnemonic, derives Ed25519 keys, creates a DID, and persists.
 * Shows recovery phrase after creation for the user to write down.
 */

import React, { useState, useCallback } from 'react';
import {
  View,
  Text,
  TextInput,
  TouchableOpacity,
  StyleSheet,
  ScrollView,
  Alert,
  KeyboardAvoidingView,
  Platform,
} from 'react-native';
import { useNavigation } from '@react-navigation/native';
import LoadingOverlay from '../components/LoadingOverlay';
import { useIdentity } from '../hooks/useIdentity';
import { THEME } from '../utils/constants';

export default function CreateIdentityScreen() {
  const navigation = useNavigation();
  const { create, isCreating, error, dismissError } = useIdentity();

  const [label, setLabel] = useState('');
  const [nickname, setNickname] = useState('');
  const [description, setDescription] = useState('');
  const [createdIdentity, setCreatedIdentity] = useState<{
    did: string;
    peerId: string;
    mnemonic: string;
  } | null>(null);

  const handleCreate = useCallback(async () => {
    const trimmedLabel = label.trim();
    if (!trimmedLabel) {
      Alert.alert('Required', 'Please enter a label for your identity.');
      return;
    }
    if (!/^[a-zA-Z0-9\-_]+$/.test(trimmedLabel)) {
      Alert.alert(
        'Invalid Label',
        'Label can only contain letters, numbers, hyphens, and underscores.',
      );
      return;
    }

    try {
      const identity = await create(
        trimmedLabel,
        nickname.trim() || undefined,
        description.trim() || undefined,
      );
      setCreatedIdentity({
        did: identity.did,
        peerId: identity.peerId,
        mnemonic: identity.mnemonic,
      });
    } catch (err: any) {
      Alert.alert('Error', err.message || 'Failed to create identity.');
    }
  }, [label, nickname, description, create]);

  const handleDone = useCallback(() => {
    // Navigate back to welcome, which will auto-redirect to Main
    navigation.goBack();
  }, [navigation]);

  // ---- Show recovery phrase after creation ----
  if (createdIdentity) {
    return (
      <ScrollView contentContainerStyle={styles.container}>
        <View style={styles.successCard}>
          <Text style={styles.successIcon}>🎉</Text>
          <Text style={styles.successTitle}>Identity Created!</Text>
          <Text style={styles.successLabel}>DID</Text>
          <Text style={styles.successValue} selectable>
            {createdIdentity.did}
          </Text>
          <Text style={styles.successLabel}>PeerId</Text>
          <Text style={styles.successValue} selectable>
            {createdIdentity.peerId}
          </Text>

          <View style={styles.mnemonicBox}>
            <Text style={styles.mnemonicWarning}>
              ⚠️ Save your recovery phrase securely. It is the only way to
              recover this identity.
            </Text>
            <Text style={styles.mnemonicPhrase} selectable>
              {createdIdentity.mnemonic}
            </Text>
          </View>

          <TouchableOpacity style={styles.doneButton} onPress={handleDone}>
            <Text style={styles.doneButtonText}>
              I've Saved My Recovery Phrase
            </Text>
          </TouchableOpacity>
        </View>
      </ScrollView>
    );
  }

  // ---- Creation form ----
  return (
    <KeyboardAvoidingView
      style={{ flex: 1, backgroundColor: THEME.background }}
      behavior={Platform.OS === 'ios' ? 'padding' : undefined}>
      <ScrollView
        contentContainerStyle={styles.container}
        keyboardShouldPersistTaps="handled">
        <LoadingOverlay visible={isCreating} message="Generating identity..." />

        <Text style={styles.heading}>Create a New Identity</Text>
        <Text style={styles.subtext}>
          Your identity includes a BIP39 mnemonic, Ed25519 key pair, and a W3C
          DID. All data stays on your device.
        </Text>

        {/* Label */}
        <Text style={styles.inputLabel}>Label *</Text>
        <TextInput
          style={styles.input}
          value={label}
          onChangeText={setLabel}
          placeholder="e.g. personal, work, mobile"
          placeholderTextColor={THEME.textMuted}
          autoCapitalize="none"
          autoCorrect={false}
          maxLength={32}
          editable={!isCreating}
        />

        {/* Nickname */}
        <Text style={styles.inputLabel}>Nickname (optional)</Text>
        <TextInput
          style={styles.input}
          value={nickname}
          onChangeText={setNickname}
          placeholder="e.g. Alice"
          placeholderTextColor={THEME.textMuted}
          maxLength={64}
          editable={!isCreating}
        />

        {/* Description */}
        <Text style={styles.inputLabel}>Description (optional)</Text>
        <TextInput
          style={[styles.input, styles.textArea]}
          value={description}
          onChangeText={setDescription}
          placeholder="What is this identity for?"
          placeholderTextColor={THEME.textMuted}
          multiline
          numberOfLines={3}
          maxLength={200}
          editable={!isCreating}
        />

        {/* Create Button */}
        <TouchableOpacity
          style={[styles.createButton, !label.trim() && styles.disabledButton]}
          onPress={handleCreate}
          disabled={!label.trim() || isCreating}
          activeOpacity={0.8}>
          <Text style={styles.createButtonText}>
            {isCreating ? 'Generating...' : 'Create Identity'}
          </Text>
        </TouchableOpacity>

        {error && (
          <View style={styles.errorBox}>
            <Text style={styles.errorText}>{error}</Text>
            <TouchableOpacity onPress={dismissError}>
              <Text style={styles.dismissText}>Dismiss</Text>
            </TouchableOpacity>
          </View>
        )}
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
    fontSize: 24,
    fontWeight: '700',
    color: THEME.text,
    marginBottom: 8,
  },
  subtext: {
    fontSize: 13,
    color: THEME.textMuted,
    lineHeight: 19,
    marginBottom: 28,
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
    minHeight: 80,
    textAlignVertical: 'top',
  },
  createButton: {
    backgroundColor: THEME.primary,
    borderRadius: 12,
    paddingVertical: 16,
    alignItems: 'center',
    marginTop: 32,
  },
  disabledButton: {
    opacity: 0.4,
  },
  createButtonText: {
    color: THEME.white,
    fontSize: 17,
    fontWeight: '600',
  },
  errorBox: {
    backgroundColor: THEME.danger + '22',
    borderRadius: 8,
    padding: 12,
    marginTop: 16,
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  errorText: {
    color: THEME.danger,
    fontSize: 13,
    flex: 1,
  },
  dismissText: {
    color: THEME.accent,
    fontSize: 13,
    fontWeight: '600',
    marginLeft: 12,
  },

  // Success state
  successCard: {
    alignItems: 'center',
    paddingTop: 20,
  },
  successIcon: {
    fontSize: 48,
    marginBottom: 12,
  },
  successTitle: {
    fontSize: 22,
    fontWeight: '700',
    color: THEME.text,
    marginBottom: 20,
  },
  successLabel: {
    fontSize: 12,
    fontWeight: '600',
    color: THEME.textMuted,
    textTransform: 'uppercase',
    letterSpacing: 0.5,
    marginTop: 12,
    alignSelf: 'flex-start',
  },
  successValue: {
    fontSize: 12,
    color: THEME.accent,
    fontFamily: 'monospace',
    marginTop: 4,
    backgroundColor: THEME.surface,
    borderRadius: 6,
    padding: 8,
    width: '100%',
  },
  mnemonicBox: {
    backgroundColor: THEME.warning + '15',
    borderRadius: 12,
    borderWidth: 1,
    borderColor: THEME.warning + '33',
    padding: 16,
    marginTop: 20,
    width: '100%',
  },
  mnemonicWarning: {
    fontSize: 12,
    color: THEME.warning,
    marginBottom: 10,
    lineHeight: 17,
  },
  mnemonicPhrase: {
    fontSize: 15,
    color: THEME.text,
    fontFamily: 'monospace',
    lineHeight: 24,
  },
  doneButton: {
    backgroundColor: THEME.success,
    borderRadius: 12,
    paddingVertical: 16,
    alignItems: 'center',
    marginTop: 28,
    width: '100%',
  },
  doneButtonText: {
    color: THEME.white,
    fontSize: 16,
    fontWeight: '600',
  },
});
