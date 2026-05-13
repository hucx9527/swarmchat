/**
 * RecoverIdentityScreen — recovers an existing identity from a 24-word BIP39 mnemonic.
 *
 * The user pastes or types their mnemonic phrase, provides a label, and
 * the identity is derived, verified, and persisted.
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
import { THEME, MNEMONIC_WORD_COUNT } from '../utils/constants';

export default function RecoverIdentityScreen() {
  const navigation = useNavigation();
  const { recover, isRecovering, error, dismissError } = useIdentity();

  const [mnemonic, setMnemonic] = useState('');
  const [label, setLabel] = useState('');

  const wordCount = mnemonic.trim()
    ? mnemonic.trim().split(/\s+/).length
    : 0;
  const isValidWordCount = wordCount === MNEMONIC_WORD_COUNT;

  const handleRecover = useCallback(async () => {
    const trimmedMnemonic = mnemonic.trim().toLowerCase();
    const trimmedLabel = label.trim();

    if (!trimmedMnemonic) {
      Alert.alert('Required', 'Please enter your recovery phrase.');
      return;
    }
    if (!isValidWordCount) {
      Alert.alert(
        'Invalid Phrase',
        `A recovery phrase must contain exactly ${MNEMONIC_WORD_COUNT} words. You entered ${wordCount}.`,
      );
      return;
    }
    if (!trimmedLabel) {
      Alert.alert('Required', 'Please enter a label for this identity.');
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
      await recover(trimmedMnemonic, trimmedLabel);
      Alert.alert('Recovered!', 'Your identity has been restored.', [
        { text: 'OK', onPress: () => navigation.goBack() },
      ]);
    } catch (err: any) {
      Alert.alert(
        'Recovery Failed',
        err.message || 'Could not recover identity. Check your mnemonic phrase.',
      );
    }
  }, [mnemonic, label, isValidWordCount, wordCount, recover, navigation]);

  return (
    <KeyboardAvoidingView
      style={{ flex: 1, backgroundColor: THEME.background }}
      behavior={Platform.OS === 'ios' ? 'padding' : undefined}>
      <ScrollView
        contentContainerStyle={styles.container}
        keyboardShouldPersistTaps="handled">
        <LoadingOverlay visible={isRecovering} message="Recovering identity..." />

        <Text style={styles.heading}>Recover Your Identity</Text>
        <Text style={styles.subtext}>
          Enter your 24-word BIP39 recovery phrase to restore your decentralized
          identity (DID, PeerId, and keys).
        </Text>

        {/* Mnemonic */}
        <View style={styles.labelRow}>
          <Text style={styles.inputLabel}>Recovery Phrase</Text>
          <Text
            style={[
              styles.wordCount,
              isValidWordCount ? styles.wordCountValid : styles.wordCountInvalid,
            ]}>
            {wordCount}/{MNEMONIC_WORD_COUNT} words
          </Text>
        </View>
        <TextInput
          style={[styles.input, styles.textArea]}
          value={mnemonic}
          onChangeText={setMnemonic}
          placeholder={`Enter your ${MNEMONIC_WORD_COUNT}-word recovery phrase...`}
          placeholderTextColor={THEME.textMuted}
          multiline
          numberOfLines={5}
          autoCapitalize="none"
          autoCorrect={false}
          textContentType="none"
          editable={!isRecovering}
        />

        {/* Label */}
        <Text style={styles.inputLabel}>Label *</Text>
        <TextInput
          style={styles.input}
          value={label}
          onChangeText={setLabel}
          placeholder="e.g. personal, recovered"
          placeholderTextColor={THEME.textMuted}
          autoCapitalize="none"
          autoCorrect={false}
          maxLength={32}
          editable={!isRecovering}
        />

        {/* Recover Button */}
        <TouchableOpacity
          style={[
            styles.recoverButton,
            (!isValidWordCount || !label.trim()) && styles.disabledButton,
          ]}
          onPress={handleRecover}
          disabled={!isValidWordCount || !label.trim() || isRecovering}
          activeOpacity={0.8}>
          <Text style={styles.recoverButtonText}>
            {isRecovering ? 'Recovering...' : 'Recover Identity'}
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

        {/* Tip */}
        <View style={styles.tipBox}>
          <Text style={styles.tipTitle}>💡 Tip</Text>
          <Text style={styles.tipText}>
            Your recovery phrase is case-insensitive. Words must be separated by
            spaces. The phrase was shown when you first created the identity.
          </Text>
        </View>
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
  labelRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginTop: 14,
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
  wordCount: {
    fontSize: 12,
    fontWeight: '600',
  },
  wordCountValid: {
    color: THEME.success,
  },
  wordCountInvalid: {
    color: THEME.textMuted,
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
    minHeight: 130,
    textAlignVertical: 'top',
    fontFamily: 'monospace',
    fontSize: 14,
    lineHeight: 22,
  },
  recoverButton: {
    backgroundColor: THEME.secondary,
    borderRadius: 12,
    paddingVertical: 16,
    alignItems: 'center',
    marginTop: 32,
  },
  disabledButton: {
    opacity: 0.4,
  },
  recoverButtonText: {
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
  tipBox: {
    backgroundColor: THEME.accent + '10',
    borderRadius: 10,
    padding: 14,
    marginTop: 28,
    borderLeftWidth: 3,
    borderLeftColor: THEME.accent,
  },
  tipTitle: {
    fontSize: 13,
    fontWeight: '600',
    color: THEME.accent,
    marginBottom: 4,
  },
  tipText: {
    fontSize: 12,
    color: THEME.textMuted,
    lineHeight: 17,
  },
});
