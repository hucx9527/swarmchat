/**
 * GroupsScreen — list of all group chats the user participates in.
 *
 * Shows sorted groups with member count, last message preview, and
 * unread badges. Tapping a group opens GroupChatScreen.
 * The header "+" button creates a new group (placeholder dialog).
 */

import React, { useCallback, useState } from 'react';
import {
  View,
  Text,
  FlatList,
  TouchableOpacity,
  StyleSheet,
  Alert,
  TextInput,
  Modal,
} from 'react-native';
import { useNavigation } from '@react-navigation/native';
import { useSelector, useDispatch } from 'react-redux';
import type { RootState, AppDispatch } from '../store';
import { addGroup } from '../store/groupSlice';
import GroupItem from '../components/GroupItem';
import GroupService from '../services/GroupService';
import type { Group } from '../types';
import { THEME } from '../utils/constants';

export default function GroupsScreen() {
  const navigation = useNavigation<any>();
  const dispatch = useDispatch<AppDispatch>();
  const { groups, loading } = useSelector((state: RootState) => state.groups);
  const groupList = Object.values(groups).sort((a, b) => {
    const aTime = a.lastMessage?.timestamp ?? a.createdAt;
    const bTime = b.lastMessage?.timestamp ?? b.createdAt;
    return bTime - aTime;
  });

  const [createModalVisible, setCreateModalVisible] = useState(false);
  const [newGroupName, setNewGroupName] = useState('');
  const [newGroupDesc, setNewGroupDesc] = useState('');

  const handleGroupPress = useCallback(
    (group: Group) => {
      navigation.navigate('Home', {
        screen: 'GroupChat',
        params: { groupId: group.id, groupName: group.name },
      });
    },
    [navigation],
  );

  const handleCreateGroup = useCallback(async () => {
    const name = newGroupName.trim();
    if (!name) {
      Alert.alert('Required', 'Please enter a group name.');
      return;
    }

    try {
      const group = await GroupService.createGroup(
        name,
        newGroupDesc.trim() || undefined,
      );
      dispatch(addGroup(group));
      setCreateModalVisible(false);
      setNewGroupName('');
      setNewGroupDesc('');
    } catch (err: any) {
      Alert.alert('Error', err.message || 'Failed to create group.');
    }
  }, [newGroupName, newGroupDesc, dispatch]);

  const renderGroup = useCallback(
    ({ item }: { item: Group }) => (
      <GroupItem group={item} onPress={() => handleGroupPress(item)} />
    ),
    [handleGroupPress],
  );

  return (
    <View style={styles.container}>
      {/* Header */}
      <View style={styles.header}>
        <View>
          <Text style={styles.headerTitle}>Groups</Text>
          <Text style={styles.headerSub}>
            {groupList.length} group{groupList.length !== 1 ? 's' : ''}
          </Text>
        </View>
        <TouchableOpacity
          style={styles.addButton}
          onPress={() => setCreateModalVisible(true)}>
          <Text style={styles.addButtonText}>+</Text>
        </TouchableOpacity>
      </View>

      {/* Group List */}
      {groupList.length === 0 ? (
        <View style={styles.emptyContainer}>
          <Text style={styles.emptyIcon}>👥</Text>
          <Text style={styles.emptyTitle}>No groups yet</Text>
          <Text style={styles.emptySubtext}>
            Tap + to create a group chat
          </Text>
        </View>
      ) : (
        <FlatList
          data={groupList}
          keyExtractor={item => item.id}
          renderItem={renderGroup}
          contentContainerStyle={styles.listContent}
        />
      )}

      {/* Create Group Modal */}
      <Modal
        visible={createModalVisible}
        transparent
        animationType="slide"
        onRequestClose={() => setCreateModalVisible(false)}>
        <View style={styles.modalOverlay}>
          <View style={styles.modalContent}>
            <View style={styles.modalHeader}>
              <Text style={styles.modalTitle}>Create Group</Text>
              <TouchableOpacity onPress={() => setCreateModalVisible(false)}>
                <Text style={styles.modalClose}>✕</Text>
              </TouchableOpacity>
            </View>

            <View style={styles.modalBody}>
              <Text style={styles.inputLabel}>Group Name *</Text>
              <TextInput
                style={styles.input}
                value={newGroupName}
                onChangeText={setNewGroupName}
                placeholder="e.g. Team Chat"
                placeholderTextColor={THEME.textMuted}
                maxLength={64}
              />

              <Text style={styles.inputLabel}>Description (optional)</Text>
              <TextInput
                style={[styles.input, styles.textArea]}
                value={newGroupDesc}
                onChangeText={setNewGroupDesc}
                placeholder="What is this group about?"
                placeholderTextColor={THEME.textMuted}
                multiline
                numberOfLines={2}
                maxLength={200}
              />

              <TouchableOpacity
                style={[styles.createButton, !newGroupName.trim() && styles.disabledButton]}
                onPress={handleCreateGroup}
                disabled={!newGroupName.trim()}>
                <Text style={styles.createButtonText}>Create Group</Text>
              </TouchableOpacity>
            </View>
          </View>
        </View>
      </Modal>
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
    backgroundColor: THEME.secondary,
    justifyContent: 'center',
    alignItems: 'center',
  },
  addButtonText: {
    fontSize: 22,
    color: THEME.white,
    lineHeight: 24,
    fontWeight: '300',
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
  // Modal
  modalOverlay: {
    flex: 1,
    backgroundColor: 'rgba(0,0,0,0.5)',
    justifyContent: 'center',
    paddingHorizontal: 24,
  },
  modalContent: {
    backgroundColor: THEME.surface,
    borderRadius: 16,
    borderWidth: 1,
    borderColor: THEME.border,
  },
  modalHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingHorizontal: 20,
    paddingTop: 18,
    paddingBottom: 12,
    borderBottomWidth: 1,
    borderBottomColor: THEME.border,
  },
  modalTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: THEME.text,
  },
  modalClose: {
    fontSize: 20,
    color: THEME.textMuted,
  },
  modalBody: {
    padding: 20,
  },
  inputLabel: {
    fontSize: 13,
    fontWeight: '600',
    color: THEME.textMuted,
    marginBottom: 6,
    marginTop: 12,
    textTransform: 'uppercase',
    letterSpacing: 0.5,
  },
  input: {
    backgroundColor: THEME.background,
    borderRadius: 10,
    borderWidth: 1,
    borderColor: THEME.border,
    paddingHorizontal: 14,
    paddingVertical: 12,
    fontSize: 15,
    color: THEME.text,
  },
  textArea: {
    minHeight: 60,
    textAlignVertical: 'top',
  },
  createButton: {
    backgroundColor: THEME.primary,
    borderRadius: 12,
    paddingVertical: 16,
    alignItems: 'center',
    marginTop: 24,
  },
  disabledButton: {
    opacity: 0.4,
  },
  createButtonText: {
    color: THEME.white,
    fontSize: 16,
    fontWeight: '600',
  },
});
