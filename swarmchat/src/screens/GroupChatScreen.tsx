/**
 * GroupChatScreen — group messaging with Sender Key encryption.
 *
 * Displays messages for a group chat room, shows member presence,
 * and supports sending messages that get encrypted with the group's
 * Sender Key and broadcast to all members via the relay.
 */

import React, { useCallback, useEffect, useRef, useState } from 'react';
import {
  View,
  Text,
  FlatList,
  TouchableOpacity,
  StyleSheet,
  Alert,
  Modal,
  ScrollView,
} from 'react-native';
import type { NativeStackScreenProps } from '@react-navigation/native-stack';
import type { HomeStackParamList, GroupMember } from '../types';
import { MessageType } from '../types';
import MessageBubble from '../components/MessageBubble';
import ChatInput from '../components/ChatInput';
import { useChat } from '../hooks/useChat';
import { useSelector } from 'react-redux';
import type { RootState } from '../store';
import GroupService from '../services/GroupService';
import { THEME } from '../utils/constants';

type Props = NativeStackScreenProps<HomeStackParamList, 'GroupChat'>;

export default function GroupChatScreen({ route }: Props) {
  const { groupId } = route.params;
  const { sendText, sendFile, activeMessages, openDirectRoom } = useChat();
  const groups = useSelector((state: RootState) => state.groups.groups);
  const group = groups[groupId];
  const flatListRef = useRef<FlatList>(null);
  const [memberModalVisible, setMemberModalVisible] = useState(false);

  // Auto-scroll on new messages
  useEffect(() => {
    if (activeMessages.length > 0) {
      setTimeout(() => {
        flatListRef.current?.scrollToEnd({ animated: true });
      }, 150);
    }
  }, [activeMessages.length]);

  const handleSendText = useCallback(
    async (text: string) => {
      if (!group) return;
      try {
        await GroupService.sendMessage(groupId, text, MessageType.TEXT);
        // Refresh messages
      } catch (err: any) {
        Alert.alert('Send Failed', err.message || 'Could not send group message.');
      }
    },
    [group, groupId],
  );

  const handleSendFile = useCallback(
    async (fileInfo: { uri: string; fileName: string; mime: string; size: number }) => {
      if (!group) return;
      let msgType: MessageType = MessageType.FILE;
      if (fileInfo.mime.startsWith('image/')) msgType = MessageType.IMAGE;
      else if (fileInfo.mime.startsWith('video/')) msgType = MessageType.VIDEO;
      else if (fileInfo.mime.startsWith('audio/')) msgType = MessageType.AUDIO;

      try {
        await GroupService.sendMessage(groupId, '', msgType);
      } catch (err: any) {
        Alert.alert('Send Failed', err.message || 'Could not send file.');
      }
    },
    [group, groupId],
  );

  const renderMessage = useCallback(
    ({ item }: any) => (
      <MessageBubble
        message={item}
        showSender={true}
        onLongPress={() => {
          Alert.alert('Message', 'Options: Copy / Delete', [
            { text: 'Copy' },
            { text: 'Cancel', style: 'cancel' },
          ]);
        }}
      />
    ),
    [],
  );

  if (!group) {
    return (
      <View style={styles.centered}>
        <Text style={styles.emptyText}>Group not found</Text>
      </View>
    );
  }

  return (
    <View style={styles.container}>
      {/* Group Info Bar */}
      <TouchableOpacity
        style={styles.groupInfoBar}
        onPress={() => setMemberModalVisible(true)}>
        <Text style={styles.memberCount}>
          {group.members.length} member{group.members.length !== 1 ? 's' : ''} · {group.joinPolicy}
        </Text>
        <Text style={styles.infoArrow}>▼</Text>
      </TouchableOpacity>

      {/* Message List */}
      <FlatList
        ref={flatListRef}
        data={activeMessages}
        keyExtractor={item => item.id}
        renderItem={renderMessage}
        contentContainerStyle={styles.messageList}
        ListHeaderComponent={<View style={{ height: 8 }} />}
      />

      {/* Chat Input */}
      <ChatInput
        onSendText={handleSendText}
        onSendFile={handleSendFile}
      />

      {/* Member List Modal */}
      <Modal
        visible={memberModalVisible}
        transparent
        animationType="slide"
        onRequestClose={() => setMemberModalVisible(false)}>
        <View style={styles.modalOverlay}>
          <View style={styles.modalContent}>
            <View style={styles.modalHeader}>
              <Text style={styles.modalTitle}>Members</Text>
              <TouchableOpacity onPress={() => setMemberModalVisible(false)}>
                <Text style={styles.modalClose}>✕</Text>
              </TouchableOpacity>
            </View>
            <ScrollView>
              {group.members.map((member: GroupMember) => (
                <View key={member.did} style={styles.memberRow}>
                  <View style={styles.memberAvatar}>
                    <Text style={styles.memberAvatarText}>
                      {member.displayName.substring(0, 2).toUpperCase()}
                    </Text>
                  </View>
                  <View style={styles.memberInfo}>
                    <Text style={styles.memberName}>
                      {member.displayName}
                      {member.role === 'admin' && (
                        <Text style={styles.adminBadge}> Admin</Text>
                      )}
                    </Text>
                    <Text style={styles.memberDid} numberOfLines={1}>
                      {member.did}
                    </Text>
                  </View>
                </View>
              ))}
            </ScrollView>
          </View>
        </View>
      </Modal>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: THEME.background,
  },
  centered: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: THEME.background,
  },
  emptyText: {
    color: THEME.textMuted,
    fontSize: 15,
  },
  groupInfoBar: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingHorizontal: 16,
    paddingVertical: 8,
    backgroundColor: THEME.surface,
    borderBottomWidth: 1,
    borderBottomColor: THEME.border,
  },
  memberCount: {
    fontSize: 12,
    color: THEME.accent,
  },
  infoArrow: {
    fontSize: 10,
    color: THEME.textMuted,
  },
  messageList: {
    flexGrow: 1,
    paddingBottom: 4,
  },
  // Modal
  modalOverlay: {
    flex: 1,
    backgroundColor: 'rgba(0,0,0,0.5)',
    justifyContent: 'flex-end',
  },
  modalContent: {
    backgroundColor: THEME.surface,
    borderTopLeftRadius: 20,
    borderTopRightRadius: 20,
    maxHeight: '60%',
    paddingBottom: 30,
  },
  modalHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingHorizontal: 20,
    paddingTop: 18,
    paddingBottom: 12,
    borderBottomWidth: 1,
    borderBottomColor: THEME.border,
  },
  modalTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: THEME.text,
  },
  modalClose: {
    fontSize: 20,
    color: THEME.textMuted,
  },
  memberRow: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: 20,
    paddingVertical: 12,
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: THEME.border,
  },
  memberAvatar: {
    width: 40,
    height: 40,
    borderRadius: 20,
    backgroundColor: THEME.primary,
    justifyContent: 'center',
    alignItems: 'center',
    marginRight: 12,
  },
  memberAvatarText: {
    color: THEME.white,
    fontSize: 14,
    fontWeight: '600',
  },
  memberInfo: {
    flex: 1,
  },
  memberName: {
    fontSize: 15,
    fontWeight: '500',
    color: THEME.text,
  },
  adminBadge: {
    color: THEME.warning,
    fontSize: 12,
  },
  memberDid: {
    fontSize: 11,
    color: THEME.textMuted,
    marginTop: 2,
  },
});
