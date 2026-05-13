/**
 * QRCode — displays a QR code for the given value.
 *
 * Uses react-native-qrcode-svg for native rendering.
 * Falls back to a placeholder view if the library is not available.
 */

import React, { memo } from 'react';
import { View, Text, StyleSheet } from 'react-native';
import { THEME } from '../utils/constants';

// Attempt to import QR code generator; fall back gracefully
let QRCodeSvg: React.ComponentType<any> | null = null;
try {
  // eslint-disable-next-line @typescript-eslint/no-var-requires
  QRCodeSvg = require('react-native-qrcode-svg').default;
} catch {
  // Library not installed; will use placeholder
}

interface Props {
  value: string;
  size?: number;
  backgroundColor?: string;
  color?: string;
}

function QRCode({ value, size = 220, backgroundColor = THEME.white, color = THEME.background }: Props) {
  if (QRCodeSvg) {
    return (
      <View style={[styles.container, { width: size, height: size, backgroundColor }]}>
        <QRCodeSvg
          value={value}
          size={size - 16}
          backgroundColor={backgroundColor}
          color={color}
        />
      </View>
    );
  }

  // Placeholder fallback when react-native-qrcode-svg is not available
  return (
    <View style={[styles.placeholder, { width: size, height: size }]}>
      <View style={styles.qrPattern}>
        {Array.from({ length: 7 }).map((_, row) => (
          <View key={row} style={styles.qrRow}>
            {Array.from({ length: 7 }).map((_, col) => (
              <View
                key={col}
                style={[
                  styles.qrCell,
                  {
                    backgroundColor: (row + col) % 3 === 0 ? THEME.background : THEME.white,
                  },
                ]}
              />
            ))}
          </View>
        ))}
      </View>
      <Text style={styles.placeholderLabel}>QR Code</Text>
      <Text style={styles.placeholderValue} numberOfLines={2}>
        {value.substring(0, 40)}...
      </Text>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    borderRadius: 12,
    padding: 8,
    justifyContent: 'center',
    alignItems: 'center',
  },
  placeholder: {
    borderRadius: 12,
    backgroundColor: THEME.white,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 16,
  },
  qrPattern: {
    width: 140,
    height: 140,
    marginBottom: 12,
    borderWidth: 2,
    borderColor: THEME.background,
  },
  qrRow: {
    flexDirection: 'row',
    flex: 1,
  },
  qrCell: {
    flex: 1,
    margin: 1,
  },
  placeholderLabel: {
    fontSize: 14,
    fontWeight: '600',
    color: THEME.background,
    marginBottom: 4,
  },
  placeholderValue: {
    fontSize: 10,
    color: THEME.textMuted,
    textAlign: 'center',
  },
});

export default memo(QRCode);
