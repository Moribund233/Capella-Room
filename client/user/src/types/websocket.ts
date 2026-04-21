/**
 * WebSocket 类型定义
 * 对应后端 WebSocket 协议
 */

// 连接状态
export type ConnectionStatus = 'connecting' | 'connected' | 'disconnected' | 'reconnecting'

// 用户状态
export type UserStatus = 'online' | 'away' | 'busy' | 'offline'

// 用户信息
export interface WebSocketUserInfo {
  id: string
  username: string
  status: UserStatus
  avatar_url?: string
}

// 消息类型
export type MessageType =
  // 连接管理
  | 'Auth'
  | 'AuthResult'
  | 'Ping'
  | 'Pong'
  | 'Error'
  // 房间管理
  | 'JoinRoom'
  | 'LeaveRoom'
  | 'RoomJoined'
  | 'RoomLeft'
  | 'UserJoined'
  | 'UserLeft'
  | 'OnlineUsers'
  // 消息通信
  | 'Chat'
  | 'ChatMessage'
  | 'NewMessage'
  | 'Typing'
  | 'StopTyping'
  | 'MessageRead'
  | 'MessageReadReceipt'
  | 'EditMessage'
  | 'MessageEdited'
  | 'DeleteMessage'
  | 'MessageDeleted'
  // 系统消息
  | 'System'
  | 'SystemMessage'
  | 'RoomUpdated'
  // 用户状态
  | 'StatusUpdate'
  | 'UpdateStatus'
  | 'UserStatusChanged'
  | 'UserStatusUpdate'
  | 'GetOnlineUsers'
  | 'GlobalOnlineUsers'
  // 断线重连
  | 'Reconnect'
  | 'ReconnectResult'
  | 'GetMissedMessages'
  | 'MissedMessages'
  | 'SessionRestored'
  // 通知系统
  | 'PrivateMessage'
  | 'Mentioned'
  | 'RoomInvitation'
  | 'SystemNotification'
  | 'FileUploadComplete'
  | 'GetOfflineNotifications'
  | 'MarkNotificationRead'
  | 'MarkAllNotificationsRead'
  | 'OfflineNotifications'
  | 'NotificationReadConfirm'
  // 待办通知
  | 'PendingAction'
  | 'RespondPendingAction'
  | 'PendingActionResponse'
  | 'GetPendingActions'
  | 'PendingActionsList'

// WebSocket 消息
export interface WebSocketMessage {
  type: MessageType
  payload?: unknown
  timestamp?: string
}

// 认证消息
export interface AuthMessage {
  token: string
}

// 认证结果
export interface AuthResult {
  success: boolean
  message?: string
  user?: WebSocketUserInfo
}

// 聊天消息
export interface ChatMessageData {
  id: string
  content: string
  sender: WebSocketUserInfo
  room_id: string
  created_at: string
  type: 'text' | 'image' | 'file'
  reply_to?: {
    id: string
    sender_id: string
    sender_name: string
    content: string
  }
}

// 房间加入/离开
export interface JoinRoomData {
  room_id: string
}

export interface LeaveRoomData {
  room_id: string
}

export interface RoomJoinedData {
  room_id: string
  user: WebSocketUserInfo
}

export interface RoomLeftData {
  room_id: string
  user_id: string
}

// 在线用户
export interface OnlineUsersData {
  room_id: string
  users: WebSocketUserInfo[]
  count: number
}

// 用户状态变更
export interface UserStatusChangedData {
  user_id: string
  status: UserStatus
}

// 输入状态
export interface TypingData {
  room_id: string
  user_id: string
  username: string
}

// 系统消息
export interface SystemMessageData {
  room_id?: string
  content: string
  type: 'info' | 'warning' | 'error' | 'success'
}

// WebSocket 配置
export interface WebSocketConfig {
  maxReconnectAttempts?: number
  reconnectInterval?: number
  heartbeatInterval?: number
  connectTimeout?: number
}

// WebSocket 事件处理器
export interface WebSocketEventHandlers {
  onConnect?: () => void
  onDisconnect?: () => void
  onError?: (error: Error) => void
  onMessage?: (message: WebSocketMessage) => void
}

// 通知
export interface Notification {
  id: string
  type: 'new' | 'important' | 'warning' | 'info'
  title: string
  content: string
  data?: unknown
  created_at: string
  is_read: boolean
}
