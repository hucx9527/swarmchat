/**
 * LoadingOverlay — a centered loading spinner with optional message.
 * Used during identity creation, message sync, and other async operations.
 */

import React, { memo } from 'react';
import {
  View,
  Text,
  ActivityIndicator,
  StyleSheet,
  Modal,
} from 'react-native';
import { THEME } from '../utils/constants';

interface Props {
  visible: boolean;
  message?: string;
  transparent?: boolean;
}

function LoadingOverlay({ visible, message, transparent = false }: Props) {
  if (!visible) return null;

  return (
    <Modal
      visible={visible}
      transparent
      animationType="fade"
      statusBarTranslucent>
      <View
        style={[
          styles.overlay,
          transparent ? styles.transparentBg : styles.solidBg,
        ]}>
        <View style={styles.card}>
          <ActivityIndicator size="large" color={THEME.primary} />
          {message && <Text style={styles.message}>{message}</Text>}
        </View>
      </View>
    </Modal>
  );
}

const styles = StyleSheet.create({
  overlay: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
  },
  solidBg: {
    backgroundColor: 'rgba(0, 0, 0, 0.6)',
  },
  transparentBg: {
    backgroundColor: 'transparent',
  },
  card: {
    backgroundColor: THEME.surface,
    borderRadius: 16,
    paddingHorizontal: 32,
    paddingVertical: 28,
    alignItems: 'center',
    borderWidth: 1,
    borderColor: THEME.border,
    minWidth: 160,
  },
  message: {
    marginTop: 16,
    fontSize: 14,
    color: THEME.text,
    textAlign: 'center',
  },
});

export default memo(LoadingOverlay);
