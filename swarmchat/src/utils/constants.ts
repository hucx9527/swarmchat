export const APP_NAME = 'SwarmChat';
export const PROTOCOL_VERSION = 'scp/1.0';
export const DEFAULT_RELAY_URL = 'http://localhost:8080';
export const DEFAULT_TTL = 604800; // 7 days in seconds
export const MAX_FILE_SIZE = 20 * 1024 * 1024; // 20 MB
export const MAX_GROUP_MEMBERS = 1000;
export const MESSAGE_PAGE_SIZE = 50;
export const MNEMONIC_WORD_COUNT = 24;
export const MNEMONIC_ENTROPY_BITS = 256;

export const THEME = {
  primary: '#6C5CE7',
  secondary: '#00CEC9',
  background: '#0D1117',
  surface: '#161B22',
  border: '#30363D',
  text: '#C9D1D9',
  textMuted: '#8B949E',
  accent: '#58A6FF',
  success: '#3FB950',
  warning: '#D29922',
  danger: '#F85149',
  white: '#FFFFFF',
  black: '#000000',
} as const;

export default { APP_NAME, PROTOCOL_VERSION, DEFAULT_RELAY_URL, DEFAULT_TTL, THEME };
