/**
 * useIdentity — hook for identity lifecycle operations.
 *
 * Wraps Redux identitySlice actions and IdentityService for
 * create, recover, load, switch, and delete identity flows.
 */

import { useCallback } from 'react';
import { useSelector, useDispatch } from 'react-redux';
import type { RootState, AppDispatch } from '../store';
import {
  createIdentityAsync,
  recoverIdentityAsync,
  loadIdentityAsync,
  setDefaultIdentity,
  deleteIdentityAction,
  clearError,
} from '../store/identitySlice';
import IdentityService from '../services/IdentityService';

export function useIdentity() {
  const dispatch = useDispatch<AppDispatch>();
  const {
    store,
    activeIdentity,
    isCreating,
    isRecovering,
    error,
  } = useSelector((state: RootState) => state.identity);

  const allIdentities = store ? Object.values(store.identities) : [];
  const hasIdentity = !!activeIdentity;

  /** Create a new identity (BIP39 mnemonic + DID + PeerId) */
  const create = useCallback(
    async (label: string, nickname?: string, description?: string) => {
      return dispatch(createIdentityAsync({ label, nickname, description })).unwrap();
    },
    [dispatch],
  );

  /** Recover an existing identity from 24-word mnemonic */
  const recover = useCallback(
    async (mnemonic: string, label: string) => {
      return dispatch(recoverIdentityAsync({ mnemonic, label })).unwrap();
    },
    [dispatch],
  );

  /** Load persisted identity store into Redux */
  const load = useCallback(() => {
    dispatch(loadIdentityAsync());
  }, [dispatch]);

  /** Set the active / default identity by label */
  const setDefault = useCallback(
    (label: string) => {
      IdentityService.setDefault(label);
      dispatch(setDefaultIdentity(label));
    },
    [dispatch],
  );

  /** Delete an identity and remove its data */
  const deleteIdentity = useCallback(
    (label: string) => {
      IdentityService.delete(label);
      dispatch(deleteIdentityAction(label));
    },
    [dispatch],
  );

  /** Dismiss any error */
  const dismissError = useCallback(() => {
    dispatch(clearError());
  }, [dispatch]);

  /** Get a specific identity by label */
  const getIdentity = useCallback(
    (label: string) => {
      return store?.identities[label] ?? null;
    },
    [store],
  );

  return {
    // state
    store,
    activeIdentity,
    allIdentities,
    hasIdentity,
    isCreating,
    isRecovering,
    error,

    // actions
    create,
    recover,
    load,
    setDefault,
    deleteIdentity,
    dismissError,
    getIdentity,
  };
}

export default useIdentity;
