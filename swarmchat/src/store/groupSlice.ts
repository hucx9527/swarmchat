import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { Group, GroupsState } from '../types';

const initialState: GroupsState = { groups: {}, loading: false };

const groupSlice = createSlice({
  name: 'groups',
  initialState,
  reducers: {
    setGroups(state, action: PayloadAction<Record<string, Group>>) {
      state.groups = action.payload;
    },
    addGroup(state, action: PayloadAction<Group>) {
      state.groups[action.payload.id] = action.payload;
    },
    removeGroup(state, action: PayloadAction<string>) {
      delete state.groups[action.payload];
    },
    addMember(state, action: PayloadAction<{ groupId: string; member: Group['members'][0] }>) {
      const group = state.groups[action.payload.groupId];
      if (group && !group.members.find(m => m.did === action.payload.member.did)) {
        group.members.push(action.payload.member);
      }
    },
    removeMember(state, action: PayloadAction<{ groupId: string; did: string }>) {
      const group = state.groups[action.payload.groupId];
      if (group) {
        group.members = group.members.filter(m => m.did !== action.payload.did);
      }
    },
    setLoading(state, action: PayloadAction<boolean>) {
      state.loading = action.payload;
    },
  },
});

export const { setGroups, addGroup, removeGroup, addMember, removeMember, setLoading } = groupSlice.actions;
export default groupSlice.reducer;
