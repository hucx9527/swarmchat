/**
 * ChatScreen — 1-on-1 direct messaging screen.
 *
 * Shows the message history for a direct chat room with auto-scroll,
 * a ChatInput bar, and inline media/file rendering.
 * Supports Double Ratchet encryption per session.
 */

import React, { useCallback, useEffect, useRef, useState } from 'react';
import {
  View,
  FlatList,
  StyleSheet,
  KeyboardAvoidingView,
  Platform,
  Alert,
} from 'react-native';
import type { NativeStackScreenProps } from '@react-navigation/native-stack';
import type { HomeStackParamList } from '../types';
import { MessageType } from '../types';
import MessageBubble from '../components/MessageBubble';
import ChatInput from '../components/ChatInput';
import { useChat } from '../hooks/useChat';
import { useContacts } from '../hooks/useContacts';
import { THEME } from '../utils/constants';

type Props = NativeStackScreenProps<HomeStackParamList, 'Chat'>;

export default function ChatScreen({ route }: Props) {
  const { roomId, peerDid, peerName } = route.params;
  const {
    activeRoom,
    activeMessages,
    openDirectRoom,
    sendText,
    sendFile,
    syncMessages,
    notifyTyping,
  } = useChat();
  const flatListRef = useRef<FlatList>(null);

  // Ensure the room is loaded
  useEffect(() => {
    if (peerDid) {
      openDirectRoom(peerDid, peerName);
    }
  }, [peerDid, peerName, openDirectRoom]);

  // Auto-scroll to bottom when new messages arrive
  useEffect(() => {
    if (activeMessages.length > 0) {
      setTimeout(() => {
        flatListRef.current?.scrollToEnd({ animated: true });
      }, 150);
    }
  }, [activeMessages.length]);

  const handleSendText = useCallback(
    async (text: string) => {
      try {
        await sendText(text);
      } catch (err: any) {
        Alert.alert('Send Failed', err.message || 'Could not send message.');
      }
    },
    [sendText],
  );

  const handleSendFile = useCallback(
    async (fileInfo: { uri: string; fileName: string; mime: string; size: number }) => {
      try {
        // Determine message type based on MIME
        let msgType: MessageType = MessageType.FILE;
        if (fileInfo.mime.startsWith('image/')) {
          msgType = MessageType.IMAGE;
        } else if (fileInfo.mime.startsWith('video/')) {
          msgType = MessageType.VIDEO;
        } else if (fileInfo.mime.startsWith('audio/')) {
          msgType = MessageType.AUDIO;
        }

        await sendFile(msgType, {
          cid: `placeholder-${Date.now()}`,
          fileName: fileInfo.fileName,
          mime: fileInfo.mime,
          size: fileInfo.size,
        });
      } catch (err: any) {
        Alert.alert('Send Failed', err.message || 'Could not send file.');
      }
    },
    [sendFile],
  );

  const handleTypingChange = useCallback(
    (isTyping: boolean) => {
      notifyTyping(isTyping);
    },
    [notifyTyping],
  );

  const renderMessage = useCallback(
    ({ item }: any) => {
      const showSender = !item.isOutgoing;
      return (
        <MessageBubble
          message={item}
          showSender={showSender}
          onLongPress={() => {
            Alert.alert('Message', 'Options: Copy / Delete / Forward', [
              { text: 'Copy', onPress: () => {} },
              { text: 'Cancel', style: 'cancel' },
            ]);
          }}
        />
      );
    },
    [],
  );

  return (
    <KeyboardAvoidingView
      style={styles.container}
      behavior={Platform.OS === 'ios' ? 'padding' : undefined}
      keyboardVerticalOffset={Platform.OS === 'ios' ? 90 : 0}>
      <FlatList
        ref={flatListRef}
        data={activeMessages}
        keyExtractor={item => item.id}
        renderItem={renderMessage}
        contentContainerStyle={styles.messageList}
        inverted={false}
        onEndReachedThreshold={0.3}
        ListHeaderComponent={<View style={{ height: 8 }} />}
        ListFooterComponent={<View style={{ height: 8 }} />}
      />

      <ChatInput
        onSendText={handleSendText}
        onSendFile={handleSendFile}
        onTypingChange={handleTypingChange}
      />
    </KeyboardAvoidingView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: THEME.background,
  },
  messageList: {
    flexGrow: 1,
    paddingBottom: 4,
  },
});
