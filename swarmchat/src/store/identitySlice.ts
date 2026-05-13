import { createSlice, createAsyncThunk, PayloadAction } from '@reduxjs/toolkit';
import { Identity, IdentityState } from '../types';
import { IdentityService } from '../services/IdentityService';

const initialState: IdentityState = {
  store: null,
  activeIdentity: null,
  isCreating: false,
  isRecovering: false,
  error: null,
};

export const createIdentity = createAsyncThunk(
  'identity/create',
  async (params: { label: string; nickname?: string; description?: string }) => {
    return await IdentityService.create(params.label, params.nickname, params.description);
  },
);

export const recoverIdentity = createAsyncThunk(
  'identity/recover',
  async (params: { mnemonic: string; label: string }) => {
    return await IdentityService.recover(params.mnemonic, params.label);
  },
);

export const loadIdentity = createAsyncThunk(
  'identity/load',
  async () => {
    const store = IdentityService.loadStore();
    const active = IdentityService.getActiveIdentity();
    return { store, active };
  },
);

const identitySlice = createSlice({
  name: 'identity',
  initialState,
  reducers: {
    setDefault(state, action: PayloadAction<string>) {
      IdentityService.setDefault(action.payload);
      const store = IdentityService.loadStore();
      state.store = store;
      state.activeIdentity = store.identities[action.payload] || null;
    },
    deleteIdentity(state, action: PayloadAction<string>) {
      IdentityService.delete(action.payload);
      const store = IdentityService.loadStore();
      state.store = store;
      state.activeIdentity = store.defaultLabel ? store.identities[store.defaultLabel] : null;
    },
    clearError(state) {
      state.error = null;
    },
  },
  extraReducers: (builder) => {
    builder
      .addCase(createIdentity.pending, (state) => {
        state.isCreating = true;
        state.error = null;
      })
      .addCase(createIdentity.fulfilled, (state, action) => {
        state.isCreating = false;
        const store = IdentityService.loadStore();
        state.store = store;
        state.activeIdentity = action.payload;
      })
      .addCase(createIdentity.rejected, (state, action) => {
        state.isCreating = false;
        state.error = action.error.message || 'Failed to create identity';
      })
      .addCase(recoverIdentity.pending, (state) => {
        state.isRecovering = true;
        state.error = null;
      })
      .addCase(recoverIdentity.fulfilled, (state, action) => {
        state.isRecovering = false;
        const store = IdentityService.loadStore();
        state.store = store;
        state.activeIdentity = action.payload;
      })
      .addCase(recoverIdentity.rejected, (state, action) => {
        state.isRecovering = false;
        state.error = action.error.message || 'Failed to recover identity';
      })
      .addCase(loadIdentity.fulfilled, (state, action) => {
        state.store = action.payload.store;
        state.activeIdentity = action.payload.active;
      });
  },
});

export const { setDefault, deleteIdentity, clearError } = identitySlice.actions;
export default identitySlice.reducer;
