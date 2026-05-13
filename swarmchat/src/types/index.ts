// ============================================================================
// SCP Protocol Types for React Native
// ============================================================================

// ---- Identity ----
export interface Identity {
  label: string;
  nickname?: string;
  description?: string;
  did: string;
  peerId: string;
  publicKey: string;
  mnemonic: string;
  seedHex: string;
  createdAt: number;
  isDefault: boolean;
}

export interface IdentityStore {
  identities: Record<string, Identity>;
  defaultLabel?: string;
  schemaVersion: number;
}

// ---- DID / Peer ----
export interface DidDocument {
  '@context': string[];
  id: string;
  verificationMethod: VerificationMethod[];
  authentication: string[];
  assertionMethod: string[];
  keyAgreement: string[];
  created: string;
  updated: string;
}

export interface VerificationMethod {
  id: string;
  type: string;
  controller: string;
  publicKeyMultibase: string;
}

// ---- Contact (Friend) ----
export interface Contact {
  did: string;
  peerId: string;
  displayName: string;
  avatarCid?: string;
  publicKey: string;
  status: ContactStatus;
  addedAt: number;
  lastSeen?: number;
  isAgent: boolean;
  agentCapabilities?: string[];
}

export enum ContactStatus {
  ONLINE = 'online',
  AWAY = 'away',
  OFFLINE = 'offline',
}

// ---- Messages ----
export enum MessageType {
  TEXT = 'scp.message.v1.text',
  IMAGE = 'scp.message.v1.image',
  FILE = 'scp.message.v1.file',
  VIDEO = 'scp.message.v1.video',
  AUDIO = 'scp.message.v1.audio',
  GROUP_CREATE = 'scp.group.v1.create',
  GROUP_INVITE = 'scp.group.v1.invite',
  GROUP_JOIN = 'scp.group.v1.join',
  GROUP_LEAVE = 'scp.group.v1.leave',
  AGENT_CARD = 'scp.agent.v1.card',
  AGENT_TASK = 'scp.agent.v1.task',
  AGENT_RESULT = 'scp.agent.v1.result',
  SYSTEM_TYPING = 'scp.system.v1.typing',
  SYSTEM_READ = 'scp.system.v1.read',
  SYSTEM_PRESENCE = 'scp.system.v1.presence',
  CONTROL_ACK = 'scp.control.v1.ack',
}

export interface MessagePayload {
  type: MessageType;
  body?: string;         // text content
  cid?: string;          // IPFS CID for files/images
  mime?: string;         // MIME type
  fileName?: string;     // Original file name
  fileSize?: number;     // File size in bytes
  duration?: number;     // Audio/video duration
  width?: number;
  height?: number;
  thumbnailCid?: string;
}

export interface ChatMessage {
  id: string;
  roomId: string;
  senderDid: string;
  type: MessageType;
  payload: MessagePayload;
  timestamp: number;
  ttl: number;
  status: MessageStatus;
  isOutgoing: boolean;
  replyToId?: string;
}

export enum MessageStatus {
  SENDING = 'sending',
  SENT = 'sent',
  DELIVERED = 'delivered',
  READ = 'read',
  FAILED = 'failed',
}

// ---- Groups ----
export interface Group {
  id: string;
  name: string;
  description?: string;
  avatarCid?: string;
  joinPolicy: 'invite' | 'open';
  whoCanSend: 'all' | 'admins';
  members: GroupMember[];
  createdAt: number;
  createdBy: string;
  unreadCount: number;
  lastMessage?: ChatMessage;
}

export interface GroupMember {
  did: string;
  displayName: string;
  role: 'admin' | 'member';
  joinedAt: number;
}

// ---- Chat Rooms ----
export interface ChatRoom {
  id: string;
  type: 'direct' | 'group' | 'agent';
  name: string;
  avatarCid?: string;
  participants: string[];
  lastMessage?: ChatMessage;
  unreadCount: number;
  encryptionScheme: 'double-ratchet' | 'sender-key';
  sessionId: string;
  createdAt: number;
}

// ---- File Transfer ----
export interface FileTransfer {
  id: string;
  cid: string;
  fileName: string;
  mime: string;
  size: number;
  peerDid: string;
  direction: 'upload' | 'download';
  progress: number; // 0-100
  status: 'pending' | 'transferring' | 'completed' | 'failed';
  localPath?: string;
}

// ---- Network ----
export interface NetworkStatus {
  connected: boolean;
  relayUrl: string;
  peerCount: number;
  natStatus: string;
  externalAddresses: string[];
}

// ---- Navigation ----
export type RootStackParamList = {
  Welcome: undefined;
  CreateIdentity: undefined;
  RecoverIdentity: undefined;
  Main: undefined;
};

export type MainTabParamList = {
  Home: undefined;
  Contacts: undefined;
  Groups: undefined;
  Profile: undefined;
};

export type HomeStackParamList = {
  ChatList: undefined;
  Chat: { roomId: string; peerDid?: string; peerName?: string };
  GroupChat: { groupId: string; groupName?: string };
  ContactDetail: { did: string };
};

// ---- App State (Redux) ----
export interface AppState {
  identity: IdentityState;
  contacts: ContactsState;
  chats: ChatsState;
  groups: GroupsState;
  settings: SettingsState;
  network: NetworkState;
}

export interface IdentityState {
  store: IdentityStore | null;
  activeIdentity: Identity | null;
  isCreating: boolean;
  isRecovering: boolean;
  error: string | null;
}

export interface ContactsState {
  contacts: Record<string, Contact>;
  pendingRequests: string[];
  loading: boolean;
}

export interface ChatsState {
  rooms: Record<string, ChatRoom>;
  messages: Record<string, ChatMessage[]>;
  activeRoomId: string | null;
  typingUsers: Record<string, string[]>;
  loading: boolean;
}

export interface GroupsState {
  groups: Record<string, Group>;
  loading: boolean;
}

export interface SettingsState {
  relayUrl: string;
  enableMdns: boolean;
  enableRelay: boolean;
  autoDownloadFiles: boolean;
  maxFileSize: number; // bytes
  theme: 'system' | 'light' | 'dark';
  biometricLock: boolean;
}

export interface NetworkState {
  status: NetworkStatus;
  fileTransfers: Record<string, FileTransfer>;
}
