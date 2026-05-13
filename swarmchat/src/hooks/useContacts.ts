/**
 * useContacts — hook for contact/friend management.
 *
 * Wraps Redux contactsSlice with reactive contact list
 * and actions for adding, removing, and updating contacts.
 */

import { useCallback } from 'react';
import { useSelector, useDispatch } from 'react-redux';
import type { RootState, AppDispatch } from '../store';
import {
  addContactAction,
  removeContactAction,
  updateContactStatusAction,
  addPendingRequestAction,
  removePendingRequestAction,
} from '../store/contactsSlice';
import { ContactStatus } from '../types';
import type { Contact } from '../types';

export function useContacts() {
  const dispatch = useDispatch<AppDispatch>();
  const { contacts, pendingRequests, loading } = useSelector(
    (state: RootState) => state.contacts,
  );

  const contactList = Object.values(contacts).sort((a, b) =>
    a.displayName.localeCompare(b.displayName),
  );

  const onlineContacts = contactList.filter(
    c => c.status === ContactStatus.ONLINE || c.status === ContactStatus.AWAY,
  );

  /** Add a new contact by DID */
  const addContact = useCallback(
    (contact: Omit<Contact, 'status' | 'addedAt' | 'isAgent'> & { status?: ContactStatus; isAgent?: boolean }) => {
      const fullContact: Contact = {
        ...contact,
        status: contact.status ?? ContactStatus.OFFLINE,
        addedAt: Date.now(),
        isAgent: contact.isAgent ?? false,
      };
      dispatch(addContactAction(fullContact));
      return fullContact;
    },
    [dispatch],
  );

  /** Remove a contact */
  const removeContact = useCallback(
    (did: string) => {
      dispatch(removeContactAction(did));
    },
    [dispatch],
  );

  /** Update a contact's online status */
  const updateStatus = useCallback(
    (did: string, status: ContactStatus) => {
      dispatch(updateContactStatusAction({ did, status }));
    },
    [dispatch],
  );

  /** Add a pending contact request */
  const addPending = useCallback(
    (did: string) => {
      dispatch(addPendingRequestAction(did));
    },
    [dispatch],
  );

  /** Remove a pending contact request */
  const removePending = useCallback(
    (did: string) => {
      dispatch(removePendingRequestAction(did));
    },
    [dispatch],
  );

  /** Look up a contact by DID */
  const getContact = useCallback(
    (did: string): Contact | undefined => {
      return contacts[did];
    },
    [contacts],
  );

  /** Search contacts by name or DID */
  const searchContacts = useCallback(
    (query: string): Contact[] => {
      const q = query.toLowerCase();
      return contactList.filter(
        c =>
          c.displayName.toLowerCase().includes(q) ||
          c.did.toLowerCase().includes(q) ||
          c.peerId.toLowerCase().includes(q),
      );
    },
    [contactList],
  );

  return {
    // state
    contacts,
    contactList,
    onlineContacts,
    pendingRequests,
    loading,

    // actions
    addContact,
    removeContact,
    updateStatus,
    addPending,
    removePending,
    getContact,
    searchContacts,
  };
}

export default useContacts;
