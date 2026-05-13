/**
 * WelcomeScreen — onboarding landing page.
 *
 * Shows the SwarmChat branding, a tagline, and two primary actions:
 * "Create Identity" and "Recover Identity".
 * Only shown when no identity exists in the Redux store.
 */

import React, { useCallback } from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  StyleSheet,
  StatusBar,
  ScrollView,
} from 'react-native';
import type { NativeStackNavigationProp } from '@react-navigation/native-stack';
import { useNavigation } from '@react-navigation/native';
import type { RootStackParamList } from '../types';
import { THEME, APP_NAME, PROTOCOL_VERSION } from '../utils/constants';

type Nav = NativeStackNavigationProp<RootStackParamList, 'Welcome'>;

export default function WelcomeScreen() {
  const navigation = useNavigation<Nav>();

  const handleCreate = useCallback(() => {
    navigation.navigate('CreateIdentity');
  }, [navigation]);

  const handleRecover = useCallback(() => {
    navigation.navigate('RecoverIdentity');
  }, [navigation]);

  return (
    <ScrollView
      contentContainerStyle={styles.container}
      bounces={false}>
      <StatusBar barStyle="light-content" backgroundColor={THEME.background} />

      {/* Logo / Branding */}
      <View style={styles.logoSection}>
        <View style={styles.logoCircle}>
          <Text style={styles.logoIcon}>🐝</Text>
        </View>
        <Text style={styles.appName}>{APP_NAME}</Text>
        <Text style={styles.tagline}>
          Decentralized communication{'\n'}powered by swarm intelligence
        </Text>
        <Text style={styles.version}>{PROTOCOL_VERSION}</Text>
      </View>

      {/* Info Cards */}
      <View style={styles.featuresSection}>
        <View style={styles.featureCard}>
          <Text style={styles.featureIcon}>🔐</Text>
          <Text style={styles.featureTitle}>End-to-End Encrypted</Text>
          <Text style={styles.featureDesc}>
            X3DH key agreement + Double Ratchet per session
          </Text>
        </View>

        <View style={styles.featureCard}>
          <Text style={styles.featureIcon}>🌐</Text>
          <Text style={styles.featureTitle}>Peer-to-Peer</Text>
          <Text style={styles.featureDesc}>
            libp2p with QUIC, Kademlia DHT, and GossipSub
          </Text>
        </View>

        <View style={styles.featureCard}>
          <Text style={styles.featureIcon}>🆔</Text>
          <Text style={styles.featureTitle}>Self-Sovereign Identity</Text>
          <Text style={styles.featureDesc}>
            W3C did:key with Ed25519 signatures
          </Text>
        </View>
      </View>

      {/* Action Buttons */}
      <View style={styles.actionSection}>
        <TouchableOpacity
          style={styles.primaryButton}
          onPress={handleCreate}
          activeOpacity={0.8}>
          <Text style={styles.primaryButtonText}>Create Identity</Text>
        </TouchableOpacity>

        <TouchableOpacity
          style={styles.secondaryButton}
          onPress={handleRecover}
          activeOpacity={0.8}>
          <Text style={styles.secondaryButtonText}>
            Recover Existing Identity
          </Text>
        </TouchableOpacity>
      </View>

      <Text style={styles.footer}>Swarm Communication Protocol · v0.1.0</Text>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {
    flexGrow: 1,
    backgroundColor: THEME.background,
    alignItems: 'center',
    paddingHorizontal: 24,
    paddingTop: 80,
    paddingBottom: 40,
  },
  logoSection: {
    alignItems: 'center',
    marginBottom: 40,
  },
  logoCircle: {
    width: 88,
    height: 88,
    borderRadius: 44,
    backgroundColor: THEME.surface,
    borderWidth: 2,
    borderColor: THEME.primary,
    justifyContent: 'center',
    alignItems: 'center',
    marginBottom: 16,
  },
  logoIcon: {
    fontSize: 40,
  },
  appName: {
    fontSize: 32,
    fontWeight: '700',
    color: THEME.text,
    letterSpacing: 1,
    marginBottom: 8,
  },
  tagline: {
    fontSize: 15,
    color: THEME.textMuted,
    textAlign: 'center',
    lineHeight: 22,
    marginBottom: 8,
  },
  version: {
    fontSize: 12,
    color: THEME.border,
    fontFamily: 'monospace',
  },
  featuresSection: {
    width: '100%',
    marginBottom: 36,
  },
  featureCard: {
    backgroundColor: THEME.surface,
    borderRadius: 12,
    padding: 16,
    marginBottom: 10,
    borderWidth: 1,
    borderColor: THEME.border,
    flexDirection: 'row',
    alignItems: 'center',
  },
  featureIcon: {
    fontSize: 22,
    marginRight: 12,
  },
  featureTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: THEME.text,
    marginBottom: 2,
  },
  featureDesc: {
    fontSize: 12,
    color: THEME.textMuted,
  },
  actionSection: {
    width: '100%',
    marginBottom: 32,
  },
  primaryButton: {
    backgroundColor: THEME.primary,
    borderRadius: 12,
    paddingVertical: 16,
    alignItems: 'center',
    marginBottom: 12,
  },
  primaryButtonText: {
    color: THEME.white,
    fontSize: 17,
    fontWeight: '600',
  },
  secondaryButton: {
    backgroundColor: THEME.surface,
    borderRadius: 12,
    paddingVertical: 16,
    alignItems: 'center',
    borderWidth: 1,
    borderColor: THEME.border,
  },
  secondaryButtonText: {
    color: THEME.accent,
    fontSize: 16,
    fontWeight: '500',
  },
  footer: {
    fontSize: 11,
    color: THEME.border,
  },
});
