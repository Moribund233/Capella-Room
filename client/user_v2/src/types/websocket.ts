import type { UserStatus } from './user'
import type { ReplyToMessage } from './message'

// ========== 连接状态 ==========

export type ConnectionState = 'connecting' | 'connected' | 'disconnected' | 'reconnecting'

// ========== 消息信封格式（匹配后端 serde(tag = "type", content = "payload")） ==========

/** 通用 WebSocket 消息信封 */
export interface WSMessageEnvelope<T = unknown> {
  type: string
  payload?: T
}

/** @deprecated 使用 WSMessageEnvelope */
export type WebSocketMessage = WSMessageEnvelope

// ========== 发送消息类型 ==========

/** 客户端认证 */
export interface AuthPayload {
  token: string
}

/** 发送聊天消息 */
export interface ChatMessagePayload {
  room_id: string
  content: string
  reply_to?: string | null
  message_type?: string
}

/** 加入房间 */
export interface JoinRoomPayload {
  room_id: string
}

/** 离开房间 */
export interface LeaveRoomPayload {
  room_id: string
}

/** 更新状态 */
export interface UpdateStatusPayload {
  status: UserStatus
}

/** 消息已读 */
export interface MessageReadPayload {
  message_id: string
}

/** 添加反应 */
export interface AddReactionPayload {
  message_id: string
  emoji: string
}

/** 移除反应 */
export interface RemoveReactionPayload {
  message_id: string
  emoji: string
}

/** 反应已添加（广播） */
export interface ReactionAddedPayload {
  message_id: string
  room_id: string
  user_id: string
  emoji: string
}

/** 反应已移除（广播） */
export interface ReactionRemovedPayload {
  message_id: string
  room_id: string
  user_id: string
  emoji: string
}

/** 编辑消息 */
export interface EditMessagePayload {
  message_id: string
  new_content: string
}

/** 删除消息 */
export interface DeleteMessagePayload {
  message_id: string
}

/** 置顶消息 */
export interface PinMessagePayload {
  message_id: string
  room_id: string
}

/** 取消置顶 */
export interface UnpinMessagePayload {
  message_id: string
  room_id: string
}

/** 消息已置顶（广播） */
export interface MessagePinnedPayload {
  message_id: string
  room_id: string
  pinned_by: string
  pinned_by_name: string
  content_preview: string
  pinned_at: string
}

/** 消息已取消置顶（广播） */
export interface MessageUnpinnedPayload {
  message_id: string
  room_id: string
  unpinned_by: string
  unpinned_at: string
}

/** 获取离线消息 */
export interface GetMissedMessagesPayload {
  room_id: string
  last_message_id?: string | null
}

/** 获取在线用户 */
// GetOnlineUsers 无 payload

/** 正在输入 */
export interface TypingPayload {
  room_id: string
}

/** 停止输入 */
export interface StopTypingPayload {
  room_id: string
}

/** 重连 */
export interface ReconnectPayload {
  token: string
  last_disconnect_at?: string | null
}

// ========== 接收消息类型 ==========

export interface AuthResultPayload {
  success: boolean
  message: string
}

export interface NewMessagePayload {
  message_id: string
  room_id: string
  sender_id: string
  sender_name: string
  content: string
  message_type: string
  reply_to: string | null
  reply_to_message: ReplyToMessage | null
  created_at: string
}

export interface RoomJoinedPayload {
  room_id: string
  user_id: string
  username: string
}

export interface RoomLeftPayload {
  room_id: string
  user_id: string
  username: string
}

export interface UserJoinedPayload {
  room_id: string
  user_id: string
  username: string
}

export interface UserLeftPayload {
  room_id: string
  user_id: string
  username: string
}

export interface OnlineUsersPayload {
  room_id: string
  users: Array<{
    id: string
    username: string
    avatar_url: string | null
    status: UserStatus
  }>
}

export interface UserStatusChangedPayload {
  user_id: string
  username: string
  status: UserStatus
}

export interface UserTypingPayload {
  room_id: string
  user_id: string
  username: string
}

export interface UserStopTypingPayload {
  room_id: string
  user_id: string
  username: string
}

export interface MessageReadReceiptPayload {
  message_id: string
  user_id: string
}

export interface MessageEditedPayload {
  message_id: string
  new_content: string
  edited_at: string
}

export interface MessageDeletedPayload {
  message_id: string
}

export interface ErrorPayload {
  code: string
  message: string
}

export interface ReconnectResultPayload {
  success: boolean
  message: string
  rooms_to_rejoin?: string[] | null
}

export interface MissedMessagesPayload {
  room_id: string
  messages: Array<{
    message_id: string
    room_id: string
    sender_id: string
    sender_name: string
    content: string
    reply_to: string | null
    reply_to_message: ReplyToMessage | null
    created_at: string
  }>
  has_more: boolean
}

export interface SessionRestoredPayload {
  restored_at: string
  rooms_restored: number
  total_unread: number
}

export interface GlobalOnlineUsersPayload {
  users: Array<{
    id: string
    username: string
    avatar_url: string | null
    status: UserStatus
  }>
  total: number
}

/** 系统消息（广播） */
export interface SystemMessagePayload {
  content: string
}

/** 消息预览（用于房间列表中的最后消息） */
export interface MessagePreview {
  id: string
  content: string
  sender_name: string
  created_at: string
}

/** 房间消息摘要（用于房间列表实时更新） */
export interface RoomMessageSummaryPayload {
  room_id: string
  last_message: MessagePreview
  unread_count: number
}

// ========== 消息类型枚举 ==========

export enum WSMessageType {
  // 客户端发送
  AUTH = 'Auth',
  CHAT_MESSAGE = 'ChatMessage',
  TYPING = 'Typing',
  STOP_TYPING = 'StopTyping',
  JOIN_ROOM = 'JoinRoom',
  LEAVE_ROOM = 'LeaveRoom',
  UPDATE_STATUS = 'UpdateStatus',
  MESSAGE_READ = 'MessageRead',
  EDIT_MESSAGE = 'EditMessage',
  DELETE_MESSAGE = 'DeleteMessage',
  GET_MISSED_MESSAGES = 'GetMissedMessages',
  GET_ONLINE_USERS = 'GetOnlineUsers',
  ADD_REACTION = 'AddReaction',
  REMOVE_REACTION = 'RemoveReaction',
  PIN_MESSAGE = 'PinMessage',
  UNPIN_MESSAGE = 'UnpinMessage',
  RECONNECT = 'Reconnect',
  PING = 'Ping',
  PONG = 'Pong',

  // 服务端推送
  AUTH_RESULT = 'AuthResult',
  NEW_MESSAGE = 'NewMessage',
  ROOM_JOINED = 'RoomJoined',
  ROOM_LEFT = 'RoomLeft',
  USER_JOINED = 'UserJoined',
  USER_LEFT = 'UserLeft',
  ONLINE_USERS = 'OnlineUsers',
  USER_STATUS_CHANGED = 'UserStatusChanged',
  USER_TYPING = 'UserTyping',
  USER_STOP_TYPING = 'UserStopTyping',
  MESSAGE_READ_RECEIPT = 'MessageReadReceipt',
  MESSAGE_EDITED = 'MessageEdited',
  MESSAGE_DELETED = 'MessageDeleted',
  MISSED_MESSAGES = 'MissedMessages',
  RECONNECT_RESULT = 'ReconnectResult',
  SESSION_RESTORED = 'SessionRestored',
  REACTION_ADDED = 'ReactionAdded',
  REACTION_REMOVED = 'ReactionRemoved',
  GLOBAL_ONLINE_USERS = 'GlobalOnlineUsers',
  MESSAGE_PINNED = 'MessagePinned',
  MESSAGE_UNPINNED = 'MessageUnpinned',
  ERROR = 'Error',
  SYSTEM_MESSAGE = 'SystemMessage',
  MENTIONED = 'Mentioned',
  ROOM_MESSAGE_SUMMARY = 'RoomMessageSummary',
}

// ========== 消息处理器类型 ==========

export type MessageHandler<T = unknown> = (payload: T) => void

export type ConnectionStateHandler = (state: ConnectionState) => void

// ========== 辅助函数 ==========

/** 构建要发送的 WebSocket 消息 JSON 字符串 */
export function buildMessage(type: string, payload?: unknown): string {
  if (payload === undefined) {
    return JSON.stringify({ type })
  }
  return JSON.stringify({ type, payload })
}

/** 解析接收到的 WebSocket 消息 */
export function parseMessage(data: string): WSMessageEnvelope | null {
  try {
    return JSON.parse(data) as WSMessageEnvelope
  } catch {
    return null
  }
}
