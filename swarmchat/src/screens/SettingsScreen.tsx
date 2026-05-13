/**
 * SettingsScreen — application settings for relay URL, discovery,
 * file auto-download, theme, and biometric lock.
 */

import React, { useCallback, useState } from 'react';
import {
  View,
  Text,
  ScrollView,
  TextInput,
  TouchableOpacity,
  Switch,
  StyleSheet,
  Alert,
} from 'react-native';
import { useSelector, useDispatch } from 'react-redux';
import type { RootState, AppDispatch } from '../store';
import { updateSettings, setRelayUrl as setRelayUrlAction } from '../store/settingsSlice';
import { THEME, DEFAULT_RELAY_URL, PROTOCOL_VERSION, APP_NAME } from '../utils/constants';

export default function SettingsScreen() {
  const dispatch = useDispatch<AppDispatch>();
  const settings = useSelector((state: RootState) => state.settings);
  const [relayUrlInput, setRelayUrlInput] = useState(settings.relayUrl);

  const handleRelaySave = useCallback(() => {
    const url = relayUrlInput.trim();
    if (!url) {
      Alert.alert('Required', 'Please enter a relay URL.');
      return;
    }
    dispatch(setRelayUrlAction(url));
    Alert.alert('Saved', 'Relay URL updated.');
  }, [relayUrlInput, dispatch]);

  const handleToggle = useCallback(
    (key: string, value: boolean) => {
      dispatch(updateSettings({ [key]: value }));
    },
    [dispatch],
  );

  return (
    <ScrollView contentContainerStyle={styles.container}>
      {/* Relay Section */}
      <Text style={styles.sectionTitle}>Relay Node</Text>
      <View style={styles.card}>
        <Text style={styles.fieldLabel}>Relay URL</Text>
        <View style={styles.inputRow}>
          <TextInput
            style={styles.input}
            value={relayUrlInput}
            onChangeText={setRelayUrlInput}
            placeholder={DEFAULT_RELAY_URL}
            placeholderTextColor={THEME.textMuted}
            autoCapitalize="none"
            autoCorrect={false}
            keyboardType="url"
          />
          <TouchableOpacity style={styles.saveButton} onPress={handleRelaySave}>
            <Text style={styles.saveButtonText}>Save</Text>
          </TouchableOpacity>
        </View>
      </View>

      {/* Discovery Section */}
      <Text style={styles.sectionTitle}>Network & Discovery</Text>
      <View style={styles.card}>
        <View style={styles.toggleRow}>
          <View style={styles.toggleInfo}>
            <Text style={styles.toggleLabel}>Local Discovery (mDNS)</Text>
            <Text style={styles.toggleDesc}>
              Automatically find peers on the same network
            </Text>
          </View>
          <Switch
            value={settings.enableMdns}
            onValueChange={v => handleToggle('enableMdns', v)}
            trackColor={{ false: THEME.border, true: THEME.primary + '88' }}
            thumbColor={settings.enableMdns ? THEME.primary : THEME.textMuted}
          />
        </View>

        <View style={styles.divider} />

        <View style={styles.toggleRow}>
          <View style={styles.toggleInfo}>
            <Text style={styles.toggleLabel}>Relay Client</Text>
            <Text style={styles.toggleDesc}>
              Use relay nodes for NAT traversal and offline messages
            </Text>
          </View>
          <Switch
            value={settings.enableRelay}
            onValueChange={v => handleToggle('enableRelay', v)}
            trackColor={{ false: THEME.border, true: THEME.primary + '88' }}
            thumbColor={settings.enableRelay ? THEME.primary : THEME.textMuted}
          />
        </View>
      </View>

      {/* Files Section */}
      <Text style={styles.sectionTitle}>Files & Storage</Text>
      <View style={styles.card}>
        <View style={styles.toggleRow}>
          <View style={styles.toggleInfo}>
            <Text style={styles.toggleLabel}>Auto-download Files</Text>
            <Text style={styles.toggleDesc}>
              Automatically download files when connected to Wi-Fi
            </Text>
          </View>
          <Switch
            value={settings.autoDownloadFiles}
            onValueChange={v => handleToggle('autoDownloadFiles', v)}
            trackColor={{ false: THEME.border, true: THEME.primary + '88' }}
            thumbColor={settings.autoDownloadFiles ? THEME.primary : THEME.textMuted}
          />
        </View>

        <View style={styles.divider} />

        <View style={styles.toggleRow}>
          <View style={styles.toggleInfo}>
            <Text style={styles.toggleLabel}>Biometric Lock</Text>
            <Text style={styles.toggleDesc}>
              Require fingerprint/face to open the app
            </Text>
          </View>
          <Switch
            value={settings.biometricLock}
            onValueChange={v => handleToggle('biometricLock', v)}
            trackColor={{ false: THEME.border, true: THEME.primary + '88' }}
            thumbColor={settings.biometricLock ? THEME.primary : THEME.textMuted}
          />
        </View>
      </View>

      {/* Theme Section */}
      <Text style={styles.sectionTitle}>Appearance</Text>
      <View style={styles.card}>
        <View style={styles.themeSelector}>
          {(['system', 'light', 'dark'] as const).map(theme => (
            <TouchableOpacity
              key={theme}
              style={[
                styles.themeOption,
                settings.theme === theme && styles.themeOptionActive,
              ]}
              onPress={() => handleToggle('theme', theme)}>
              <Text
                style={[
                  styles.themeOptionText,
                  settings.theme === theme && styles.themeOptionTextActive,
                ]}>
                {theme.charAt(0).toUpperCase() + theme.slice(1)}
              </Text>
            </TouchableOpacity>
          ))}
        </View>
      </View>

      {/* About Section */}
      <Text style={styles.sectionTitle}>About</Text>
      <View style={styles.card}>
        <View style={styles.aboutRow}>
          <Text style={styles.aboutLabel}>App</Text>
          <Text style={styles.aboutValue}>{APP_NAME}</Text>
        </View>
        <View style={styles.divider} />
        <View style={styles.aboutRow}>
          <Text style={styles.aboutLabel}>Protocol</Text>
          <Text style={styles.aboutValue}>{PROTOCOL_VERSION}</Text>
        </View>
        <View style={styles.divider} />
        <View style={styles.aboutRow}>
          <Text style={styles.aboutLabel}>Version</Text>
          <Text style={styles.aboutValue}>0.1.0-alpha</Text>
        </View>
        <View style={styles.divider} />
        <View style={styles.aboutRow}>
          <Text style={styles.aboutLabel}>Build</Text>
          <Text style={styles.aboutValue}>React Native · TypeScript</Text>
        </View>
      </View>

      <View style={{ height: 40 }} />
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {
    backgroundColor: THEME.background,
    padding: 16,
    paddingTop: 24,
  },
  sectionTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: THEME.textMuted,
    textTransform: 'uppercase',
    letterSpacing: 0.5,
    marginBottom: 8,
    marginTop: 16,
    marginLeft: 4,
  },
  card: {
    backgroundColor: THEME.surface,
    borderRadius: 12,
    borderWidth: 1,
    borderColor: THEME.border,
    padding: 14,
    marginBottom: 4,
  },
  fieldLabel: {
    fontSize: 13,
    fontWeight: '500',
    color: THEME.text,
    marginBottom: 8,
  },
  inputRow: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  input: {
    flex: 1,
    backgroundColor: THEME.background,
    borderRadius: 8,
    borderWidth: 1,
    borderColor: THEME.border,
    paddingHorizontal: 12,
    paddingVertical: 10,
    fontSize: 14,
    color: THEME.text,
    fontFamily: 'monospace',
  },
  saveButton: {
    backgroundColor: THEME.primary,
    borderRadius: 8,
    paddingVertical: 10,
    paddingHorizontal: 18,
    marginLeft: 10,
  },
  saveButtonText: {
    color: THEME.white,
    fontSize: 14,
    fontWeight: '600',
  },
  toggleRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingVertical: 4,
  },
  toggleInfo: {
    flex: 1,
    marginRight: 16,
  },
  toggleLabel: {
    fontSize: 14,
    fontWeight: '500',
    color: THEME.text,
    marginBottom: 2,
  },
  toggleDesc: {
    fontSize: 11,
    color: THEME.textMuted,
  },
  divider: {
    height: 1,
    backgroundColor: THEME.border,
    marginVertical: 10,
  },
  themeSelector: {
    flexDirection: 'row',
    backgroundColor: THEME.background,
    borderRadius: 8,
    padding: 3,
  },
  themeOption: {
    flex: 1,
    paddingVertical: 8,
    alignItems: 'center',
    borderRadius: 6,
  },
  themeOptionActive: {
    backgroundColor: THEME.primary,
  },
  themeOptionText: {
    fontSize: 13,
    fontWeight: '500',
    color: THEME.textMuted,
  },
  themeOptionTextActive: {
    color: THEME.white,
  },
  aboutRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingVertical: 2,
  },
  aboutLabel: {
    fontSize: 14,
    color: THEME.textMuted,
  },
  aboutValue: {
    fontSize: 14,
    color: THEME.text,
    fontFamily: 'monospace',
  },
});
