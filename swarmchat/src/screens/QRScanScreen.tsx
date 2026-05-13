/**
 * QRScanScreen — camera-based QR code scanner for importing a contact's DID.
 *
 * In a production build, this uses react-native-camera or expo-barcode-scanner.
 * Here we provide the full UI with a simulated scan flow for development.
 */

import React, { useState, useCallback, useEffect } from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  StyleSheet,
  TextInput,
  Alert,
} from 'react-native';
import { useNavigation } from '@react-navigation/native';
import { extractDidFromScan, isScpQrCode } from '../utils/qr';
import { THEME } from '../utils/constants';

export default function QRScanScreen() {
  const navigation = useNavigation();
  const [manualQr, setManualQr] = useState('');
  const [scanResult, setScanResult] = useState<string | null>(null);

  // Simulate QR scan for development (tap to "detect" a mock QR code)
  const handleSimulateScan = useCallback(() => {
    // In production: camera detects QR → callback fires with data
    const mockQr = 'scp://eyJ2IjoxLCJkaWQiOiJkaWQ6a2V5OkNLOHNreGE3Q1hKVzQ5Zm5DaTUxcTZBeGs3SE5HMmpONk14UUFLNWczRHhwIiwibmFtZSI6IkFsaWNlIn0';
    const did = extractDidFromScan(mockQr);

    if (did) {
      setScanResult(did);
      Alert.alert(
        'QR Code Scanned!',
        `Found DID:\n\n${did.substring(0, 40)}...\n\nAdd this contact?`,
        [
          { text: 'Cancel', style: 'cancel' },
          {
            text: 'Add',
            onPress: () => {
              // Navigate back to AddFriend with DID pre-filled
              navigation.goBack();
              // In production, pass the scanned DID back via route params or callback
            },
          },
        ],
      );
    } else {
      Alert.alert('Invalid QR', 'This QR code does not contain a valid SwarmChat DID.');
    }
  }, [navigation]);

  const handleManualSubmit = useCallback(() => {
    const trimmed = manualQr.trim();
    if (!trimmed) return;

    const did = extractDidFromScan(trimmed);
    if (did) {
      setScanResult(did);
      Alert.alert(
        'DID Parsed',
        `Parsed DID from input:\n\n${did}`,
        [
          { text: 'Cancel', style: 'cancel' },
          {
            text: 'Add Friend',
            onPress: () => navigation.goBack(),
          },
        ],
      );
    } else {
      Alert.alert('Invalid', 'Could not parse a valid DID from the input.');
    }
  }, [manualQr, navigation]);

  if (scanResult) {
    return (
      <View style={styles.container}>
        <View style={styles.resultCard}>
          <Text style={styles.resultIcon}>✅</Text>
          <Text style={styles.resultTitle}>DID Detected</Text>
          <Text style={styles.resultValue} selectable>
            {scanResult}
          </Text>
          <TouchableOpacity
            style={styles.resultButton}
            onPress={() => navigation.goBack()}>
            <Text style={styles.resultButtonText}>Return to Add Friend</Text>
          </TouchableOpacity>
        </View>
      </View>
    );
  }

  return (
    <View style={styles.container}>
      {/* Simulated Camera View */}
      <View style={styles.cameraSimulator}>
        <View style={styles.cameraFrame}>
          <View style={styles.cornerTL} />
          <View style={styles.cornerTR} />
          <View style={styles.cornerBL} />
          <View style={styles.cornerBR} />
        </View>
        <Text style={styles.cameraHint}>
          Point your camera at a{'\n'}SwarmChat QR code
        </Text>

        <TouchableOpacity style={styles.scanButton} onPress={handleSimulateScan}>
          <Text style={styles.scanButtonText}>📷 Tap to Simulate Scan</Text>
        </TouchableOpacity>
      </View>

      {/* Manual Input Fallback */}
      <View style={styles.manualSection}>
        <Text style={styles.manualLabel}>Or paste a QR string manually:</Text>
        <TextInput
          style={styles.manualInput}
          value={manualQr}
          onChangeText={setManualQr}
          placeholder="scp://..."
          placeholderTextColor={THEME.textMuted}
          autoCapitalize="none"
          autoCorrect={false}
        />
        <TouchableOpacity
          style={[styles.manualButton, !manualQr.trim() && styles.disabledButton]}
          onPress={handleManualSubmit}
          disabled={!manualQr.trim()}>
          <Text style={styles.manualButtonText}>Parse</Text>
        </TouchableOpacity>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: THEME.background,
  },
  cameraSimulator: {
    flex: 1,
    backgroundColor: '#000',
    justifyContent: 'center',
    alignItems: 'center',
  },
  cameraFrame: {
    width: 220,
    height: 220,
    position: 'relative',
  },
  cornerTL: {
    position: 'absolute', top: 0, left: 0,
    width: 30, height: 30,
    borderTopWidth: 3, borderLeftWidth: 3,
    borderColor: THEME.primary,
  },
  cornerTR: {
    position: 'absolute', top: 0, right: 0,
    width: 30, height: 30,
    borderTopWidth: 3, borderRightWidth: 3,
    borderColor: THEME.primary,
  },
  cornerBL: {
    position: 'absolute', bottom: 0, left: 0,
    width: 30, height: 30,
    borderBottomWidth: 3, borderLeftWidth: 3,
    borderColor: THEME.primary,
  },
  cornerBR: {
    position: 'absolute', bottom: 0, right: 0,
    width: 30, height: 30,
    borderBottomWidth: 3, borderRightWidth: 3,
    borderColor: THEME.primary,
  },
  cameraHint: {
    position: 'absolute',
    bottom: 120,
    color: THEME.textMuted,
    fontSize: 14,
    textAlign: 'center',
  },
  scanButton: {
    position: 'absolute',
    bottom: 40,
    backgroundColor: THEME.primary,
    borderRadius: 24,
    paddingHorizontal: 28,
    paddingVertical: 14,
  },
  scanButtonText: {
    color: THEME.white,
    fontSize: 15,
    fontWeight: '600',
  },
  manualSection: {
    padding: 20,
    backgroundColor: THEME.surface,
    borderTopWidth: 1,
    borderTopColor: THEME.border,
  },
  manualLabel: {
    fontSize: 13,
    color: THEME.textMuted,
    marginBottom: 8,
  },
  manualInput: {
    backgroundColor: THEME.background,
    borderRadius: 10,
    borderWidth: 1,
    borderColor: THEME.border,
    paddingHorizontal: 14,
    paddingVertical: 10,
    fontSize: 14,
    color: THEME.text,
    fontFamily: 'monospace',
    marginBottom: 10,
  },
  manualButton: {
    backgroundColor: THEME.secondary,
    borderRadius: 10,
    paddingVertical: 12,
    alignItems: 'center',
  },
  disabledButton: {
    opacity: 0.4,
  },
  manualButtonText: {
    color: THEME.white,
    fontSize: 15,
    fontWeight: '600',
  },

  // Result state
  resultCard: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 32,
  },
  resultIcon: {
    fontSize: 48,
    marginBottom: 12,
  },
  resultTitle: {
    fontSize: 22,
    fontWeight: '700',
    color: THEME.text,
    marginBottom: 16,
  },
  resultValue: {
    fontSize: 12,
    color: THEME.accent,
    fontFamily: 'monospace',
    backgroundColor: THEME.surface,
    borderRadius: 8,
    padding: 12,
    marginBottom: 24,
    width: '100%',
    textAlign: 'center',
  },
  resultButton: {
    backgroundColor: THEME.primary,
    borderRadius: 12,
    paddingHorizontal: 28,
    paddingVertical: 14,
  },
  resultButtonText: {
    color: THEME.white,
    fontSize: 16,
    fontWeight: '600',
  },
});
