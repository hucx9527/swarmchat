import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { SettingsState } from '../types';

const initialState: SettingsState = {
  relayUrl: 'http://localhost:8080',
  enableMdns: true,
  enableRelay: true,
  autoDownloadFiles: false,
  maxFileSize: 20 * 1024 * 1024, // 20 MB
  theme: 'system',
  biometricLock: false,
};

const settingsSlice = createSlice({
  name: 'settings',
  initialState,
  reducers: {
    updateSettings(state, action: PayloadAction<Partial<SettingsState>>) {
      Object.assign(state, action.payload);
    },
    setRelayUrl(state, action: PayloadAction<string>) {
      state.relayUrl = action.payload;
    },
    setTheme(state, action: PayloadAction<SettingsState['theme']>) {
      state.theme = action.payload;
    },
  },
});

export const { updateSettings, setRelayUrl, setTheme } = settingsSlice.actions;
export default settingsSlice.reducer;
