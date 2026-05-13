import { configureStore } from '@reduxjs/toolkit';
import identityReducer from './identitySlice';
import contactsReducer from './contactsSlice';
import chatsReducer from './chatSlice';
import groupsReducer from './groupSlice';
import settingsReducer from './settingsSlice';
import networkReducer from './networkSlice';

export const store = configureStore({
  reducer: {
    identity: identityReducer,
    contacts: contactsReducer,
    chats: chatsReducer,
    groups: groupsReducer,
    settings: settingsReducer,
    network: networkReducer,
  },
  middleware: (getDefaultMiddleware) =>
    getDefaultMiddleware({
      serializableCheck: false,
    }),
});

export type RootState = ReturnType<typeof store.getState>;
export type AppDispatch = typeof store.dispatch;
