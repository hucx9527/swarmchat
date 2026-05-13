import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { Contact, ContactsState } from '../types';

const initialState: ContactsState = {
  contacts: {},
  pendingRequests: [],
  loading: false,
};

const contactsSlice = createSlice({
  name: 'contacts',
  initialState,
  reducers: {
    addContact(state, action: PayloadAction<Contact>) {
      state.contacts[action.payload.did] = action.payload;
    },
    removeContact(state, action: PayloadAction<string>) {
      delete state.contacts[action.payload];
    },
    updateContactStatus(state, action: PayloadAction<{ did: string; status: Contact['status'] }>) {
      if (state.contacts[action.payload.did]) {
        state.contacts[action.payload.did].status = action.payload.status;
      }
    },
    setContacts(state, action: PayloadAction<Record<string, Contact>>) {
      state.contacts = action.payload;
    },
    addPendingRequest(state, action: PayloadAction<string>) {
      if (!state.pendingRequests.includes(action.payload)) {
        state.pendingRequests.push(action.payload);
      }
    },
    removePendingRequest(state, action: PayloadAction<string>) {
      state.pendingRequests = state.pendingRequests.filter(d => d !== action.payload);
    },
    setLoading(state, action: PayloadAction<boolean>) {
      state.loading = action.payload;
    },
  },
});

export const {
  addContact, removeContact, updateContactStatus,
  setContacts, addPendingRequest, removePendingRequest, setLoading,
} = contactsSlice.actions;
export default contactsSlice.reducer;
