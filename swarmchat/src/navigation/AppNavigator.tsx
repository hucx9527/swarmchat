import React from 'react';
import { NavigationContainer } from '@react-navigation/native';
import { createNativeStackNavigator } from '@react-navigation/native-stack';
import { createBottomTabNavigator } from '@react-navigation/bottom-tabs';
import { useSelector } from 'react-redux';
import { RootState } from '../store';
import type { RootStackParamList, MainTabParamList, HomeStackParamList } from '../types';

import WelcomeScreen from '../screens/WelcomeScreen';
import CreateIdentityScreen from '../screens/CreateIdentityScreen';
import RecoverIdentityScreen from '../screens/RecoverIdentityScreen';
import HomeScreen from '../screens/HomeScreen';
import ChatScreen from '../screens/ChatScreen';
import GroupChatScreen from '../screens/GroupChatScreen';
import AddFriendScreen from '../screens/AddFriendScreen';
import QRScanScreen from '../screens/QRScanScreen';
import ContactsScreen from '../screens/ContactsScreen';
import GroupsScreen from '../screens/GroupsScreen';
import ProfileScreen from '../screens/ProfileScreen';
import SettingsScreen from '../screens/SettingsScreen';
import ContactDetailScreen from '../screens/ContactDetailScreen';
import { THEME } from '../utils/constants';

const RootStack = createNativeStackNavigator<RootStackParamList>();
const MainTab = createBottomTabNavigator<MainTabParamList>();
const HomeStack = createNativeStackNavigator<HomeStackParamList>();

function HomeStackNavigator() {
  return (
    <HomeStack.Navigator
      screenOptions={{
        headerStyle: { backgroundColor: THEME.surface },
        headerTintColor: THEME.text,
        headerTitleStyle: { fontWeight: '600' },
        contentStyle: { backgroundColor: THEME.background },
      }}>
      <HomeStack.Screen
        name="ChatList"
        component={HomeScreen}
        options={{ title: 'SwarmChat' }}
      />
      <HomeStack.Screen
        name="Chat"
        component={ChatScreen}
        options={({ route }) => ({ title: route.params?.peerName || 'Chat' })}
      />
      <HomeStack.Screen
        name="GroupChat"
        component={GroupChatScreen}
        options={({ route }) => ({ title: route.params?.groupName || 'Group' })}
      />
      <HomeStack.Screen
        name="ContactDetail"
        component={ContactDetailScreen}
        options={{ title: 'Contact' }}
      />
    </HomeStack.Navigator>
  );
}

function MainTabNavigator() {
  return (
    <MainTab.Navigator
      screenOptions={{
        headerShown: false,
        tabBarStyle: {
          backgroundColor: THEME.surface,
          borderTopColor: THEME.border,
        },
        tabBarActiveTintColor: THEME.primary,
        tabBarInactiveTintColor: THEME.textMuted,
      }}>
      <MainTab.Screen
        name="Home"
        component={HomeStackNavigator}
        options={{ tabBarLabel: 'Chats' }}
      />
      <MainTab.Screen
        name="Contacts"
        component={ContactsScreen}
        options={{ tabBarLabel: 'Contacts' }}
      />
      <MainTab.Screen
        name="Groups"
        component={GroupsScreen}
        options={{ tabBarLabel: 'Groups' }}
      />
      <MainTab.Screen
        name="Profile"
        component={ProfileScreen}
        options={{ tabBarLabel: 'Profile' }}
      />
    </MainTab.Navigator>
  );
}

export default function AppNavigator() {
  const hasIdentity = useSelector((state: RootState) =>
    !!state.identity.activeIdentity,
  );

  return (
    <NavigationContainer>
      <RootStack.Navigator
        screenOptions={{
          headerShown: false,
          contentStyle: { backgroundColor: THEME.background },
        }}>
        {!hasIdentity ? (
          <>
            <RootStack.Screen name="Welcome" component={WelcomeScreen} />
            <RootStack.Screen
              name="CreateIdentity"
              component={CreateIdentityScreen}
              options={{ headerShown: true, title: 'Create Identity', headerStyle: { backgroundColor: THEME.surface }, headerTintColor: THEME.text }}
            />
            <RootStack.Screen
              name="RecoverIdentity"
              component={RecoverIdentityScreen}
              options={{ headerShown: true, title: 'Recover Identity', headerStyle: { backgroundColor: THEME.surface }, headerTintColor: THEME.text }}
            />
          </>
        ) : (
          <>
            <RootStack.Screen name="Main" component={MainTabNavigator} />
            <RootStack.Screen
              name="AddFriend"
              component={AddFriendScreen}
              options={{ headerShown: true, title: 'Add Friend', headerStyle: { backgroundColor: THEME.surface }, headerTintColor: THEME.text, presentation: 'modal' }}
            />
            <RootStack.Screen
              name="QRScan"
              component={QRScanScreen}
              options={{ headerShown: true, title: 'Scan QR Code', headerStyle: { backgroundColor: THEME.surface }, headerTintColor: THEME.text, presentation: 'modal' }}
            />
            <RootStack.Screen
              name="Settings"
              component={SettingsScreen}
              options={{ headerShown: true, title: 'Settings', headerStyle: { backgroundColor: THEME.surface }, headerTintColor: THEME.text }}
            />
          </>
        )}
      </RootStack.Navigator>
    </NavigationContainer>
  );
}
