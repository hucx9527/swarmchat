import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { ChatMessage, ChatRoom, ChatsState } from '../types';

const initialState: ChatsState = {
  rooms: {},
  messages: {},
  activeRoomId: null,
  typingUsers: {},
  loading: false,
};

const chatSlice = createSlice({
  name: 'chats',
  initialState,
  reducers: {
    setRooms(state, action: PayloadAction<Record<string, ChatRoom>>) {
      state.rooms = action.payload;
    },
    addRoom(state, action: PayloadAction<ChatRoom>) {
      state.rooms[action.payload.id] = action.payload;
    },
    updateRoom(state, action: PayloadAction<{ roomId: string; updates: Partial<ChatRoom> }>) {
      if (state.rooms[action.payload.roomId]) {
        Object.assign(state.rooms[action.payload.roomId], action.payload.updates);
      }
    },
    setActiveRoom(state, action: PayloadAction<string | null>) {
      state.activeRoomId = action.payload;
    },
    addMessage(state, action: PayloadAction<{ roomId: string; message: ChatMessage }>) {
      if (!state.messages[action.payload.roomId]) {
        state.messages[action.payload.roomId] = [];
      }
      const msgs = state.messages[action.payload.roomId];
      if (!msgs.find(m => m.id === action.payload.message.id)) {
        msgs.push(action.payload.message);
        msgs.sort((a, b) => a.timestamp - b.timestamp);
      }
      // Update room's last message
      if (state.rooms[action.payload.roomId]) {
        state.rooms[action.payload.roomId].lastMessage = action.payload.message;
      }
    },
    updateMessageStatus(
      state,
      action: PayloadAction<{ roomId: string; messageId: string; status: ChatMessage['status'] }>,
    ) {
      const msgs = state.messages[action.payload.roomId];
      if (msgs) {
        const msg = msgs.find(m => m.id === action.payload.messageId);
        if (msg) msg.status = action.payload.status;
      }
    },
    setTyping(state, action: PayloadAction<{ roomId: string; users: string[] }>) {
      state.typingUsers[action.payload.roomId] = action.payload.users;
    },
    incrementUnread(state, action: PayloadAction<string>) {
      if (state.rooms[action.payload]) {
        state.rooms[action.payload].unreadCount++;
      }
    },
    resetUnread(state, action: PayloadAction<string>) {
      if (state.rooms[action.payload]) {
        state.rooms[action.payload].unreadCount = 0;
      }
    },
    setLoading(state, action: PayloadAction<boolean>) {
      state.loading = action.payload;
    },
  },
});

export const {
  setRooms, addRoom, updateRoom, setActiveRoom,
  addMessage, updateMessageStatus, setTyping,
  incrementUnread, resetUnread, setLoading,
} = chatSlice.actions;
export default chatSlice.reducer;
