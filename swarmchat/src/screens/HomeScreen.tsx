/**
 * HomeScreen — main chat list showing all conversation rooms
 * (direct, group, agent) sorted by most recent activity.
 *
 * From here the user can:
 *  - Tap a room to open the chat
 *  - Pull-to-refresh to sync messages from relay
 *  - Tap the "+" header button to add a friend
 *  - Long-press a room for options
 */

import React, { useCallback, useEffect } from 'react';
import {
  View,
  Text,
  FlatList,
  TouchableOpacity,
  StyleSheet,
  RefreshControl,
  Alert,
} from 'react-native';
import type { NativeStackNavigationProp } from '@react-navigation/native-stack';
import type { CompositeNavigationProp } from '@react-navigation/native';
import type { BottomTabNavigationProp } from '@react-navigation/bottom-tabs';
import { useNavigation } from '@react-navigation/native';
import { useChat } from '../hooks/useChat';
import type { ChatRoom } from '../types';
import { THEME } from '../utils/constants';

type HomeNav = CompositeNavigationProp<
  NativeStackNavigationProp<any>,
  BottomTabNavigationProp<any>
>;

function getRoomPreview(room: ChatRoom): string {
  if (!room.lastMessage) {
    return room.type === 'direct' ? 'Tap to start chatting' : 'No messages yet';
  }
  const m = room.lastMessage;
  const prefix = m.isOutgoing ? 'You: ' : '';
  return `${prefix}${m.payload.body ?? m.payload.fileName ?? m.type}`;
}

function getRoomTime(room: ChatRoom): string {
  const ts = room.lastMessage?.timestamp ?? room.createdAt;
  const d = new Date(ts);
  const now = new Date();
  if (d.toDateString() === now.toDateString()) {
    return `${d.getHours().toString().padStart(2, '0')}:${d.getMinutes().toString().padStart(2, '0')}`;
  }
  const yesterday = new Date(now);
  yesterday.setDate(yesterday.getDate() - 1);
  if (d.toDateString() === yesterday.toDateString()) {
    return 'Yesterday';
  }
  return `${d.getMonth() + 1}/${d.getDate()}`;
}

export default function HomeScreen() {
  const navigation = useNavigation<HomeNav>();
  const {
    roomList,
    activeRoomId,
    selectRoom,
    syncMessages,
    loading,
  } = useChat();

  // Sync on mount
  useEffect(() => {
    syncMessages();
  }, [syncMessages]);

  const handleRoomPress = useCallback(
    (room: ChatRoom) => {
      selectRoom(room.id);
      if (room.type === 'direct') {
        navigation.navigate('Home', {
          screen: 'Chat',
          params: {
            roomId: room.id,
            peerDid: room.participants[0],
            peerName: room.name,
          },
        } as any);
      } else if (room.type === 'group' || room.type === 'agent') {
        navigation.navigate('Home', {
          screen: 'GroupChat',
          params: {
            groupId: room.id,
            groupName: room.name,
          },
        } as any);
      }
    },
    [navigation, selectRoom],
  );

  const handleAddFriend = useCallback(() => {
    (navigation as any).navigate('AddFriend');
  }, [navigation]);

  const handleLongPress = useCallback((room: ChatRoom) => {
    Alert.alert(room.name, `Type: ${room.type}\nParticipants: ${room.participants.length}\nEncryption: ${room.encryptionScheme}`, [
      { text: 'OK', style: 'default' },
    ]);
  }, []);

  const renderRoom = useCallback(
    ({ item }: { item: ChatRoom }) => {
      const isActive = item.id === activeRoomId;
      const initials = item.name
        .split(/\s+/)
        .map(w => w[0] ?? '')
        .join('')
        .toUpperCase()
        .substring(0, 2);

      return (
        <TouchableOpacity
          style={[styles.roomItem, isActive && styles.roomItemActive]}
          onPress={() => handleRoomPress(item)}
          onLongPress={() => handleLongPress(item)}
          activeOpacity={0.6}>
          {/* Avatar */}
          <View
            style={[
              styles.avatar,
              { backgroundColor: item.type === 'group' ? THEME.secondary : THEME.primary },
            ]}>
            <Text style={styles.avatarText}>{initials}</Text>
          </View>

          {/* Info */}
          <View style={styles.roomInfo}>
            <View style={styles.topRow}>
              <Text style={styles.roomName} numberOfLines={1}>
                {item.name}
                {item.type === 'agent' && ' 🤖'}
                {item.type === 'group' && ' 👥'}
              </Text>
              <Text style={styles.roomTime}>{getRoomTime(item)}</Text>
            </View>
            <View style={styles.bottomRow}>
              <Text style={styles.roomPreview} numberOfLines={1}>
                {getRoomPreview(item)}
              </Text>
              {item.unreadCount > 0 && (
                <View style={styles.unreadBadge}>
                  <Text style={styles.unreadCount}>
                    {item.unreadCount > 99 ? '99+' : item.unreadCount}
                  </Text>
                </View>
              )}
            </View>
          </View>
        </TouchableOpacity>
      );
    },
    [activeRoomId, handleRoomPress, handleLongPress],
  );

  return (
    <View style={styles.container}>
      {/* Header */}
      <View style={styles.header}>
        <Text style={styles.headerTitle}>SwarmChat</Text>
        <TouchableOpacity style={styles.addButton} onPress={handleAddFriend}>
          <Text style={styles.addButtonText}>+</Text>
        </TouchableOpacity>
      </View>

      {/* Room List */}
      {roomList.length === 0 ? (
        <View style={styles.emptyContainer}>
          <Text style={styles.emptyIcon}>💬</Text>
          <Text style={styles.emptyTitle}>No conversations yet</Text>
          <Text style={styles.emptySubtext}>
            Tap the + button to add a friend and start chatting
          </Text>
        </View>
      ) : (
        <FlatList
          data={roomList}
          keyExtractor={item => item.id}
          renderItem={renderRoom}
          contentContainerStyle={styles.listContent}
          refreshControl={
            <RefreshControl
              refreshing={loading}
              onRefresh={syncMessages}
              tintColor={THEME.primary}
              colors={[THEME.primary]}
            />
          }
          ItemSeparatorComponent={() => <View style={styles.separator} />}
        />
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: THEME.background,
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingHorizontal: 16,
    paddingTop: 56,
    paddingBottom: 12,
    backgroundColor: THEME.surface,
    borderBottomWidth: 1,
    borderBottomColor: THEME.border,
  },
  headerTitle: {
    fontSize: 28,
    fontWeight: '700',
    color: THEME.text,
  },
  addButton: {
    width: 36,
    height: 36,
    borderRadius: 18,
    backgroundColor: THEME.primary,
    justifyContent: 'center',
    alignItems: 'center',
  },
  addButtonText: {
    fontSize: 22,
    color: THEME.white,
    lineHeight: 24,
    fontWeight: '300',
  },
  listContent: {
    flexGrow: 1,
  },
  roomItem: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingVertical: 12,
    paddingHorizontal: 16,
  },
  roomItemActive: {
    backgroundColor: THEME.primary + '10',
  },
  avatar: {
    width: 50,
    height: 50,
    borderRadius: 25,
    justifyContent: 'center',
    alignItems: 'center',
    marginRight: 12,
  },
  avatarText: {
    color: THEME.white,
    fontSize: 17,
    fontWeight: '600',
  },
  roomInfo: {
    flex: 1,
    justifyContent: 'center',
  },
  topRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 3,
  },
  roomName: {
    fontSize: 16,
    fontWeight: '500',
    color: THEME.text,
    flex: 1,
    marginRight: 8,
  },
  roomTime: {
    fontSize: 11,
    color: THEME.textMuted,
  },
  bottomRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  roomPreview: {
    fontSize: 13,
    color: THEME.textMuted,
    flex: 1,
    marginRight: 8,
  },
  unreadBadge: {
    backgroundColor: THEME.primary,
    borderRadius: 10,
    minWidth: 20,
    height: 20,
    justifyContent: 'center',
    alignItems: 'center',
    paddingHorizontal: 6,
  },
  unreadCount: {
    color: THEME.white,
    fontSize: 11,
    fontWeight: '600',
  },
  separator: {
    height: StyleSheet.hairlineWidth,
    backgroundColor: THEME.border,
    marginLeft: 78,
  },
  emptyContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    paddingHorizontal: 40,
  },
  emptyIcon: {
    fontSize: 52,
    marginBottom: 16,
  },
  emptyTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: THEME.text,
    marginBottom: 8,
  },
  emptySubtext: {
    fontSize: 14,
    color: THEME.textMuted,
    textAlign: 'center',
    lineHeight: 20,
  },
});
