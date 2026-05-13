import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { NetworkState, FileTransfer } from '../types';

const initialState: NetworkState = {
  status: {
    connected: false,
    relayUrl: 'http://localhost:8080',
    peerCount: 0,
    natStatus: 'unknown',
    externalAddresses: [],
  },
  fileTransfers: {},
};

const networkSlice = createSlice({
  name: 'network',
  initialState,
  reducers: {
    setConnected(state, action: PayloadAction<boolean>) {
      state.status.connected = action.payload;
    },
    updateNetworkStatus(state, action: PayloadAction<Partial<NetworkState['status']>>) {
      Object.assign(state.status, action.payload);
    },
    addFileTransfer(state, action: PayloadAction<FileTransfer>) {
      state.fileTransfers[action.payload.id] = action.payload;
    },
    updateFileTransfer(state, action: PayloadAction<{ id: string; updates: Partial<FileTransfer> }>) {
      if (state.fileTransfers[action.payload.id]) {
        Object.assign(state.fileTransfers[action.payload.id], action.payload.updates);
      }
    },
    removeFileTransfer(state, action: PayloadAction<string>) {
      delete state.fileTransfers[action.payload];
    },
  },
});

export const {
  setConnected, updateNetworkStatus,
  addFileTransfer, updateFileTransfer, removeFileTransfer,
} = networkSlice.actions;
export default networkSlice.reducer;
