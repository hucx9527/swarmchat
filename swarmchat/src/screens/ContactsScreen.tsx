/**
 * ContactsScreen — contact list with search/filter capabilities.
 *
 * Shows all contacts sorted by name, with online status indicators.
 * Tapping a contact navigates to ContactDetailScreen.
 * Pull-to-refresh is available. Header includes a "+" add button.
 */

import React, { useState, useCallback } from 'react';
import {
  View,
  Text,
  FlatList,
  TextInput,
  TouchableOpacity,
  StyleSheet,
} from 'react-native';
import { useNavigation } from '@react-navigation/native';
import ContactItem from '../components/ContactItem';
import { useContacts } from '../hooks/useContacts';
import { THEME } from '../utils/constants';

export default function ContactsScreen() {
  const navigation = useNavigation<any>();
  const { contactList, onlineContacts, searchContacts } = useContacts();
  const [searchQuery, setSearchQuery] = useState('');

  const filteredContacts = searchQuery.trim()
    ? searchContacts(searchQuery)
    : contactList;

  const handleContactPress = useCallback(
    (did: string) => {
      navigation.navigate('Home', {
        screen: 'ContactDetail',
        params: { did },
      });
    },
    [navigation],
  );

  const handleAddFriend = useCallback(() => {
    navigation.navigate('AddFriend');
  }, [navigation]);

  const renderContact = useCallback(
    ({ item }: any) => (
      <ContactItem
        contact={item}
        onPress={() => handleContactPress(item.did)}
      />
    ),
    [handleContactPress],
  );

  return (
    <View style={styles.container}>
      {/* Header */}
      <View style={styles.header}>
        <View>
          <Text style={styles.headerTitle}>Contacts</Text>
          <Text style={styles.headerSub}>
            {onlineContacts.length} online · {contactList.length} total
          </Text>
        </View>
        <TouchableOpacity style={styles.addButton} onPress={handleAddFriend}>
          <Text style={styles.addButtonText}>+</Text>
        </TouchableOpacity>
      </View>

      {/* Search */}
      <View style={styles.searchContainer}>
        <TextInput
          style={styles.searchInput}
          value={searchQuery}
          onChangeText={setSearchQuery}
          placeholder="Search contacts..."
          placeholderTextColor={THEME.textMuted}
          autoCapitalize="none"
          autoCorrect={false}
        />
        {searchQuery.length > 0 && (
          <TouchableOpacity
            style={styles.clearButton}
            onPress={() => setSearchQuery('')}>
            <Text style={styles.clearText}>✕</Text>
          </TouchableOpacity>
        )}
      </View>

      {/* Contact List */}
      {filteredContacts.length === 0 ? (
        <View style={styles.emptyContainer}>
          <Text style={styles.emptyIcon}>👤</Text>
          <Text style={styles.emptyTitle}>
            {searchQuery ? 'No matching contacts' : 'No contacts yet'}
          </Text>
          <Text style={styles.emptySubtext}>
            {searchQuery
              ? 'Try a different search term'
              : 'Tap + to add your first contact'}
          </Text>
        </View>
      ) : (
        <FlatList
          data={filteredContacts}
          keyExtractor={item => item.did}
          renderItem={renderContact}
          contentContainerStyle={styles.listContent}
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
  headerSub: {
    fontSize: 12,
    color: THEME.textMuted,
    marginTop: 2,
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
  searchContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    marginHorizontal: 16,
    marginVertical: 10,
    backgroundColor: THEME.surface,
    borderRadius: 10,
    borderWidth: 1,
    borderColor: THEME.border,
  },
  searchInput: {
    flex: 1,
    paddingHorizontal: 14,
    paddingVertical: 10,
    fontSize: 15,
    color: THEME.text,
  },
  clearButton: {
    paddingHorizontal: 14,
  },
  clearText: {
    fontSize: 16,
    color: THEME.textMuted,
  },
  listContent: {
    flexGrow: 1,
  },
  emptyContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    paddingHorizontal: 40,
  },
  emptyIcon: {
    fontSize: 48,
    marginBottom: 12,
  },
  emptyTitle: {
    fontSize: 17,
    fontWeight: '600',
    color: THEME.text,
    marginBottom: 6,
  },
  emptySubtext: {
    fontSize: 14,
    color: THEME.textMuted,
    textAlign: 'center',
    lineHeight: 20,
  },
});
