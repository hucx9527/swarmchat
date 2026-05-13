/**
 * MessageBubble — renders a single chat message with sender alignment,
 * timestamp, status indicator, and file/image preview where applicable.
 */

import React, { memo } from 'react';
import {
  View,
  Text,
  StyleSheet,
  TouchableOpacity,
  Image,
} from 'react-native';
import type { ChatMessage } from '../types';
import { MessageType, MessageStatus } from '../types';
import { THEME } from '../utils/constants';

interface Props {
  message: ChatMessage;
  onPress?: () => void;
  onLongPress?: () => void;
  showSender?: boolean;
}

function MessageBubble({ message, onPress, onLongPress, showSender }: Props) {
  const isOutgoing = message.isOutgoing;
  const isText = message.type === MessageType.TEXT;
  const isImage = message.type === MessageType.IMAGE;
  const isFile = message.type === MessageType.FILE;
  const isVideo = message.type === MessageType.VIDEO;
  const isSystem =
    message.type === MessageType.SYSTEM_TYPING ||
    message.type === MessageType.SYSTEM_READ ||
    message.type === MessageType.SYSTEM_PRESENCE;

  if (isSystem) {
    return (
      <View style={styles.systemContainer}>
        <Text style={styles.systemText}>
          {message.payload.body ?? message.type}
        </Text>
      </View>
    );
  }

  return (
    <TouchableOpacity
      activeOpacity={0.8}
      onPress={onPress}
      onLongPress={onLongPress}
      style={[styles.container, isOutgoing ? styles.outgoing : styles.incoming]}>
      {showSender && !isOutgoing && (
        <Text style={styles.senderName}>
          {message.senderDid.substring(0, 16)}...
        </Text>
      )}

      {/* Text message */}
      {isText && (
        <View
          style={[
            styles.bubble,
            isOutgoing ? styles.bubbleOutgoing : styles.bubbleIncoming,
          ]}>
          <Text
            style={[
              styles.messageText,
              isOutgoing ? styles.textOutgoing : styles.textIncoming,
            ]}>
            {message.payload.body}
          </Text>
        </View>
      )}

      {/* Image message */}
      {isImage && message.payload.cid && (
        <View style={styles.mediaContainer}>
          <View style={styles.mediaPlaceholder}>
            <Text style={styles.mediaIcon}>🖼</Text>
            <Text style={styles.mediaLabel}>
              {message.payload.fileName ?? 'Image'}
            </Text>
          </View>
        </View>
      )}

      {/* Video message */}
      {isVideo && message.payload.cid && (
        <View style={styles.mediaContainer}>
          <View style={styles.mediaPlaceholder}>
            <Text style={styles.mediaIcon}>🎬</Text>
            <Text style={styles.mediaLabel}>
              {message.payload.fileName ?? 'Video'}
            </Text>
          </View>
        </View>
      )}

      {/* File message */}
      {isFile && message.payload.cid && (
        <View
          style={[
            styles.bubble,
            isOutgoing ? styles.bubbleOutgoing : styles.bubbleIncoming,
          ]}>
          <View style={styles.fileRow}>
            <Text style={styles.fileIcon}>📎</Text>
            <View style={styles.fileInfo}>
              <Text
                style={[
                  styles.fileName,
                  isOutgoing ? styles.textOutgoing : styles.textIncoming,
                ]}
                numberOfLines={1}>
                {message.payload.fileName ?? 'File'}
              </Text>
              {message.payload.fileSize && (
                <Text style={styles.fileSize}>
                  {formatFileSize(message.payload.fileSize)}
                </Text>
              )}
            </View>
          </View>
        </View>
      )}

      {/* Reply indicator */}
      {message.replyToId && (
        <Text style={styles.replyIndicator}>↩ Reply</Text>
      )}

      {/* Timestamp & Status */}
      <View style={[styles.meta, isOutgoing ? styles.metaRight : styles.metaLeft]}>
        <Text style={styles.timestamp}>
          {formatTime(message.timestamp)}
        </Text>
        {isOutgoing && (
          <Text style={styles.status}>
            {statusIcon(message.status)}
          </Text>
        )}
      </View>
    </TouchableOpacity>
  );
}

function statusIcon(status: MessageStatus): string {
  switch (status) {
    case MessageStatus.SENDING:
      return '⏳';
    case MessageStatus.SENT:
      return '✓';
    case MessageStatus.DELIVERED:
      return '✓✓';
    case MessageStatus.READ:
      return '✓✓';
    case MessageStatus.FAILED:
      return '⚠';
    default:
      return '';
  }
}

function formatTime(timestamp: number): string {
  const d = new Date(timestamp);
  const h = d.getHours().toString().padStart(2, '0');
  const m = d.getMinutes().toString().padStart(2, '0');
  return `${h}:${m}`;
}

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

const styles = StyleSheet.create({
  container: {
    marginVertical: 2,
    marginHorizontal: 12,
    maxWidth: '80%',
  },
  outgoing: {
    alignSelf: 'flex-end',
  },
  incoming: {
    alignSelf: 'flex-start',
  },
  senderName: {
    fontSize: 11,
    color: THEME.accent,
    marginBottom: 2,
    marginLeft: 4,
  },
  bubble: {
    borderRadius: 16,
    paddingHorizontal: 14,
    paddingVertical: 10,
  },
  bubbleOutgoing: {
    backgroundColor: THEME.primary,
    borderBottomRightRadius: 4,
  },
  bubbleIncoming: {
    backgroundColor: THEME.surface,
    borderBottomLeftRadius: 4,
    borderWidth: 1,
    borderColor: THEME.border,
  },
  messageText: {
    fontSize: 15,
    lineHeight: 21,
  },
  textOutgoing: {
    color: THEME.white,
  },
  textIncoming: {
    color: THEME.text,
  },
  mediaContainer: {
    borderRadius: 12,
    overflow: 'hidden',
    borderWidth: 1,
    borderColor: THEME.border,
  },
  mediaPlaceholder: {
    width: 200,
    height: 150,
    backgroundColor: THEME.surface,
    justifyContent: 'center',
    alignItems: 'center',
  },
  mediaIcon: {
    fontSize: 36,
    marginBottom: 4,
  },
  mediaLabel: {
    fontSize: 12,
    color: THEME.textMuted,
  },
  fileRow: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  fileIcon: {
    fontSize: 28,
    marginRight: 10,
  },
  fileInfo: {
    flex: 1,
  },
  fileName: {
    fontSize: 14,
    fontWeight: '500',
  },
  fileSize: {
    fontSize: 11,
    color: THEME.textMuted,
    marginTop: 2,
  },
  systemContainer: {
    alignItems: 'center',
    marginVertical: 8,
  },
  systemText: {
    fontSize: 12,
    color: THEME.textMuted,
    fontStyle: 'italic',
  },
  replyIndicator: {
    fontSize: 11,
    color: THEME.textMuted,
    marginTop: 2,
    marginLeft: 4,
  },
  meta: {
    flexDirection: 'row',
    alignItems: 'center',
    marginTop: 2,
  },
  metaRight: {
    justifyContent: 'flex-end',
  },
  metaLeft: {
    justifyContent: 'flex-start',
  },
  timestamp: {
    fontSize: 11,
    color: THEME.textMuted,
    marginRight: 4,
  },
  status: {
    fontSize: 11,
  },
});

export default memo(MessageBubble);
