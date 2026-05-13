/**
 * useChat — hook for chat messaging operations.
 *
 * Wraps Redux chatSlice actions and ChatService.
 * Provides reactive room list, message list, and send/sync actions.
 */

import { useCallback, useEffect } from 'react';
import { useSelector, useDispatch } from 'react-redux';
import type { RootState, AppDispatch } from '../store';
import {
  setRooms,
  addRoom,
  setActiveRoom,
  addMessage,
  updateMessageStatus,
  setTyping,
  incrementUnread,
  resetUnread,
} from '../store/chatSlice';
import ChatService from '../services/ChatService';
import type { ChatMessage, ChatRoom, MessageType } from '../types';
import { MessageStatus } from '../types';

export function useChat() {
  const dispatch = useDispatch<AppDispatch>();
  const {
    rooms,
    messages,
    activeRoomId,
    typingUsers,
    loading,
  } = useSelector((state: RootState) => state.chats);

  const roomList = Object.values(rooms).sort((a, b) => {
    const aTime = a.lastMessage?.timestamp ?? a.createdAt;
    const bTime = b.lastMessage?.timestamp ?? b.createdAt;
    return bTime - aTime;
  });

  const activeRoom = activeRoomId ? rooms[activeRoomId] : null;
  const activeMessages = activeRoomId ? (messages[activeRoomId] ?? []) : [];

  /** Load all rooms from storage into Redux */
  const loadRooms = useCallback(() => {
    const roomMap = ChatService.loadRooms();
    dispatch(setRooms(roomMap));
  }, [dispatch]);

  /** Open or create a direct chat room */
  const openDirectRoom = useCallback(
    async (peerDid: string, peerName?: string): Promise<ChatRoom> => {
      const room = await ChatService.getOrCreateDirectRoom(peerDid, peerName);
      dispatch(addRoom(room));
      dispatch(setActiveRoom(room.id));
      return room;
    },
    [dispatch],
  );

  /** Set the active room for viewing */
  const selectRoom = useCallback(
    (roomId: string) => {
      dispatch(setActiveRoom(roomId));
      dispatch(resetUnread(roomId));
    },
    [dispatch],
  );

  /** Send a text message in the active room */
  const sendText = useCallback(
    async (text: string, replyToId?: string): Promise<ChatMessage | null> => {
      if (!activeRoomId) throw new Error('No active room');
      const msg = await ChatService.sendText(activeRoomId, text, replyToId);
      dispatch(addMessage(msg));
      return msg;
    },
    [activeRoomId, dispatch],
  );

  /** Send a file/image/video/audio message */
  const sendFile = useCallback(
    async (
      type: MessageType,
      fileInfo: { cid: string; fileName: string; mime: string; size: number; width?: number; height?: number },
    ): Promise<ChatMessage | null> => {
      if (!activeRoomId) throw new Error('No active room');
      const msg = await ChatService.sendFile(activeRoomId, type as any, fileInfo);
      dispatch(addMessage(msg));
      return msg;
    },
    [activeRoomId, dispatch],
  );

  /** Mark a message as read / delivered */
  const markRead = useCallback(
    (messageId: string) => {
      dispatch(updateMessageStatus({ messageId, status: MessageStatus.READ }));
    },
    [dispatch],
  );

  /** Sync messages from relay */
  const syncMessages = useCallback(async () => {
    await ChatService.syncMessages();
    loadRooms();
  }, [loadRooms]);

  /** Notify typing indicator (debounced in UI) */
  const notifyTyping = useCallback(
    (isTyping: boolean) => {
      if (!activeRoomId) return;
      dispatch(setTyping({ roomId: activeRoomId, isTyping }));
    },
    [activeRoomId, dispatch],
  );

  /** Load messages for a specific room */
  const loadMessages = useCallback(
    (roomId: string): ChatMessage[] => {
      return ChatService.loadMessages(roomId);
    },
    [],
  );

  /** Auto-load rooms on mount */
  useEffect(() => {
    loadRooms();
  }, [loadRooms]);

  return {
    // state
    rooms,
    roomList,
    activeRoom,
    activeRoomId,
    activeMessages,
    typingUsers,
    loading,

    // actions
    loadRooms,
    openDirectRoom,
    selectRoom,
    sendText,
    sendFile,
    markRead,
    syncMessages,
    notifyTyping,
    loadMessages,
  };
}

export default useChat;
