/**
 * ChatInput — bottom bar with text input, send button,
 * and attachment picker for the chat screen.
 */

import React, { useState, useRef, useCallback, memo } from 'react';
import {
  View,
  TextInput,
  TouchableOpacity,
  Text,
  StyleSheet,
  KeyboardAvoidingView,
  Platform,
  Alert,
} from 'react-native';
import { THEME } from '../utils/constants';
import { MAX_FILE_SIZE } from '../utils/constants';

interface Props {
  onSendText: (text: string) => void;
  onSendFile: (fileInfo: {
    uri: string;
    fileName: string;
    mime: string;
    size: number;
  }) => void;
  onTypingChange?: (isTyping: boolean) => void;
  disabled?: boolean;
}

function ChatInput({ onSendText, onSendFile, onTypingChange, disabled }: Props) {
  const [text, setText] = useState('');
  const [isTyping, setIsTyping] = useState(false);
  const inputRef = useRef<TextInput>(null);
  const typingTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  const handleSend = useCallback(() => {
    const trimmed = text.trim();
    if (!trimmed || disabled) return;

    onSendText(trimmed);
    setText('');
    setIsTyping(false);
    onTypingChange?.(false);
    if (typingTimer.current) {
      clearTimeout(typingTimer.current);
      typingTimer.current = null;
    }
  }, [text, disabled, onSendText, onTypingChange]);

  const handleChangeText = useCallback(
    (value: string) => {
      setText(value);

      // Debounced typing indicator
      if (!isTyping && value.length > 0) {
        setIsTyping(true);
        onTypingChange?.(true);
      }

      if (typingTimer.current) {
        clearTimeout(typingTimer.current);
      }

      if (value.length > 0) {
        typingTimer.current = setTimeout(() => {
          setIsTyping(false);
          onTypingChange?.(false);
        }, 2000);
      } else {
        setIsTyping(false);
        onTypingChange?.(false);
      }
    },
    [isTyping, onTypingChange],
  );

  const handleAttachment = useCallback(() => {
    // In a full implementation this would launch react-native-image-picker
    // or react-native-document-picker.
    // Here we simulate the picker flow with a placeholder action.
    Alert.alert(
      'Attach',
      'Choose attachment type',
      [
        {
          text: 'Image',
          onPress: () => {
            // Simulate image pick
            const mockFile = {
              uri: 'file:///mock/photo.jpg',
              fileName: 'photo.jpg',
              mime: 'image/jpeg',
              size: 102400,
            };
            onSendFile(mockFile);
          },
        },
        {
          text: 'File',
          onPress: () => {
            const mockFile = {
              uri: 'file:///mock/document.pdf',
              fileName: 'document.pdf',
              mime: 'application/pdf',
              size: 512000,
            };
            onSendFile(mockFile);
          },
        },
        { text: 'Cancel', style: 'cancel' },
      ],
      { cancelable: true },
    );
  }, [onSendFile]);

  const canSend = text.trim().length > 0 && !disabled;

  return (
    <KeyboardAvoidingView
      behavior={Platform.OS === 'ios' ? 'padding' : undefined}
      keyboardVerticalOffset={Platform.OS === 'ios' ? 90 : 0}>
      <View style={styles.container}>
        {/* Attachment button */}
        <TouchableOpacity
          style={styles.attachButton}
          onPress={handleAttachment}
          disabled={disabled}
          activeOpacity={0.6}>
          <Text style={styles.attachIcon}>+</Text>
        </TouchableOpacity>

        {/* Text input */}
        <View style={styles.inputWrapper}>
          <TextInput
            ref={inputRef}
            style={styles.input}
            value={text}
            onChangeText={handleChangeText}
            placeholder="Message..."
            placeholderTextColor={THEME.textMuted}
            multiline
            maxLength={4096}
            editable={!disabled}
            returnKeyType="default"
            blurOnSubmit={false}
          />
        </View>

        {/* Send button */}
        <TouchableOpacity
          style={[styles.sendButton, canSend && styles.sendButtonActive]}
          onPress={handleSend}
          disabled={!canSend}
          activeOpacity={0.6}>
          <Text
            style={[styles.sendIcon, canSend && styles.sendIconActive]}>
            ➤
          </Text>
        </TouchableOpacity>
      </View>
    </KeyboardAvoidingView>
  );
}

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    alignItems: 'flex-end',
    backgroundColor: THEME.surface,
    borderTopWidth: 1,
    borderTopColor: THEME.border,
    paddingHorizontal: 8,
    paddingVertical: 8,
    paddingBottom: Platform.OS === 'ios' ? 24 : 8,
  },
  attachButton: {
    width: 36,
    height: 36,
    borderRadius: 18,
    backgroundColor: THEME.border,
    justifyContent: 'center',
    alignItems: 'center',
    marginRight: 6,
    marginBottom: 2,
  },
  attachIcon: {
    fontSize: 22,
    fontWeight: '300',
    color: THEME.text,
    lineHeight: 24,
  },
  inputWrapper: {
    flex: 1,
    backgroundColor: THEME.background,
    borderRadius: 20,
    borderWidth: 1,
    borderColor: THEME.border,
    paddingHorizontal: 12,
    maxHeight: 120,
  },
  input: {
    fontSize: 15,
    color: THEME.text,
    paddingVertical: 8,
    minHeight: 36,
    maxHeight: 100,
  },
  sendButton: {
    width: 40,
    height: 40,
    borderRadius: 20,
    justifyContent: 'center',
    alignItems: 'center',
    marginLeft: 6,
    marginBottom: 0,
  },
  sendButtonActive: {
    backgroundColor: THEME.primary,
  },
  sendIcon: {
    fontSize: 18,
    color: THEME.textMuted,
  },
  sendIconActive: {
    color: THEME.white,
  },
});

export default memo(ChatInput);
