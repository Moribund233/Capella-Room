/**
 * WebSocket 类型定义
 * 对应后端 WebSocket 协议
 */

// ========== 基础类型 ==========

/** 用户信息 */
export interface UserInfo {
  id: string
  username: string
  status: 'online' | 'away' | 'busy' | 'offline'
}

/** 被引用消息信息 */
export interface ReplyToInfo {
  id: string
  sender_id: string
  sender_name: string
  content: string
  created_at: string
}

/** 消息类型 */
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
  | 'SystemMessage'
  | 'RoomUpdated'
  // 用户状态
  | 'UpdateStatus'
  | 'UserStatusChanged'
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
  // 日志系统
  | 'LogEntry'
  | 'SubscribeLogs'
  | 'UnsubscribeLogs'
  | 'LogSubscriptionConfirmed'

/** 通知类型 */
export type NotificationType = 'new' | 'important' | 'warning' | 'info'

/** 通知 */
export interface Notification {
  id: string
  type: NotificationType
  title: string
  content: string
  data?: unknown
  created_at: string
  is_read: boolean
}

/** 待办信息 */
export interface PendingActionInfo {
  notification_id: string
  action_type: string
  title: string
  description: string
  deadline?: string
  data?: unknown
  created_at: string
}

/** 错过的消息 */
export interface MissedMessage {
  message_id: string
  room_id: string
  sender_id: string
  sender_name: string
  content: string
  reply_to?: string
  reply_to_message?: ReplyToInfo
  created_at: string
}

// ========== WebSocket 消息 ==========

/** WebSocket 消息基础接口 */
export interface WebSocketMessageBase {
  type: MessageType
}

// ========== 连接管理 ==========

/** 认证消息 */
export interface AuthMessage extends WebSocketMessageBase {
  type: 'Auth'
  payload: { token: string }
}

/** 认证结果 */
export interface AuthResultMessage extends WebSocketMessageBase {
  type: 'AuthResult'
  payload: { success: boolean; message: string }
}

/** 心跳 Ping */
export interface PingMessage extends WebSocketMessageBase {
  type: 'Ping'
}

/** 心跳 Pong */
export interface PongMessage extends WebSocketMessageBase {
  type: 'Pong'
}

/** 错误消息 */
export interface ErrorMessage extends WebSocketMessageBase {
  type: 'Error'
  payload: { code: string; message: string }
}

// ========== 房间管理 ==========

/** 加入房间 */
export interface JoinRoomMessage extends WebSocketMessageBase {
  type: 'JoinRoom'
  payload: { room_id: string }
}

/** 离开房间 */
export interface LeaveRoomMessage extends WebSocketMessageBase {
  type: 'LeaveRoom'
  payload: { room_id: string }
}

/** 房间加入结果 */
export interface RoomJoinedMessage extends WebSocketMessageBase {
  type: 'RoomJoined'
  payload: { room_id: string; user_id: string; username: string }
}

/** 房间离开结果 */
export interface RoomLeftMessage extends WebSocketMessageBase {
  type: 'RoomLeft'
  payload: { room_id: string; user_id: string; username: string }
}

/** 用户加入房间通知 */
export interface UserJoinedMessage extends WebSocketMessageBase {
  type: 'UserJoined'
  payload: { room_id: string; user_id: string; username: string }
}

/** 用户离开房间通知 */
export interface UserLeftMessage extends WebSocketMessageBase {
  type: 'UserLeft'
  payload: { room_id: string; user_id: string; username: string }
}

/** 在线用户列表 */
export interface OnlineUsersMessage extends WebSocketMessageBase {
  type: 'OnlineUsers'
  payload: { room_id: string; users: UserInfo[] }
}

// ========== 消息通信 ==========

/** 发送聊天消息 */
export interface ChatMessage extends WebSocketMessageBase {
  type: 'ChatMessage'
  payload: { room_id: string; content: string; reply_to?: string }
}

/** 新消息通知 */
export interface NewMessage extends WebSocketMessageBase {
  type: 'NewMessage'
  payload: {
    message_id: string
    room_id: string
    sender_id: string
    sender_name: string
    content: string
    reply_to?: string
    reply_to_message?: ReplyToInfo
    created_at: string
  }
}

/** 正在输入 */
export interface TypingMessage extends WebSocketMessageBase {
  type: 'Typing'
  payload: { room_id: string }
}

/** 停止输入 */
export interface StopTypingMessage extends WebSocketMessageBase {
  type: 'StopTyping'
  payload: { room_id: string }
}

/** 消息已读 */
export interface MessageReadMessage extends WebSocketMessageBase {
  type: 'MessageRead'
  payload: { message_id: string }
}

/** 消息已读回执 */
export interface MessageReadReceiptMessage extends WebSocketMessageBase {
  type: 'MessageReadReceipt'
  payload: { message_id: string; user_id: string }
}

/** 编辑消息 */
export interface EditMessage extends WebSocketMessageBase {
  type: 'EditMessage'
  payload: { message_id: string; new_content: string }
}

/** 消息已编辑 */
export interface MessageEditedMessage extends WebSocketMessageBase {
  type: 'MessageEdited'
  payload: { message_id: string; new_content: string; edited_at: string }
}

/** 删除消息 */
export interface DeleteMessage extends WebSocketMessageBase {
  type: 'DeleteMessage'
  payload: { message_id: string }
}

/** 消息已删除 */
export interface MessageDeletedMessage extends WebSocketMessageBase {
  type: 'MessageDeleted'
  payload: { message_id: string }
}

// ========== 系统消息 ==========

/** 系统广播 */
export interface SystemMessage extends WebSocketMessageBase {
  type: 'SystemMessage'
  payload: { content: string }
}

/** 房间更新 */
export interface RoomUpdatedMessage extends WebSocketMessageBase {
  type: 'RoomUpdated'
  payload: { room_id: string; name?: string; description?: string }
}

// ========== 用户状态 ==========

/** 更新用户状态 */
export interface UpdateStatusMessage extends WebSocketMessageBase {
  type: 'UpdateStatus'
  payload: { status: 'online' | 'away' | 'busy' | 'offline' }
}

/** 用户状态变更 */
export interface UserStatusChangedMessage extends WebSocketMessageBase {
  type: 'UserStatusChanged'
  payload: { user_id: string; username: string; status: string }
}

/** 获取全局在线用户 */
export interface GetOnlineUsersMessage extends WebSocketMessageBase {
  type: 'GetOnlineUsers'
}

/** 全局在线用户列表 */
export interface GlobalOnlineUsersMessage extends WebSocketMessageBase {
  type: 'GlobalOnlineUsers'
  payload: { users: UserInfo[]; total: number }
}

// ========== 断线重连 ==========

/** 重连请求 */
export interface ReconnectMessage extends WebSocketMessageBase {
  type: 'Reconnect'
  payload: { token: string; last_disconnect_at?: string }
}

/** 重连结果 */
export interface ReconnectResultMessage extends WebSocketMessageBase {
  type: 'ReconnectResult'
  payload: { success: boolean; message: string; rooms_to_rejoin?: string[] }
}

/** 获取错过的消息 */
export interface GetMissedMessagesMessage extends WebSocketMessageBase {
  type: 'GetMissedMessages'
  payload: { room_id: string; last_message_id?: string }
}

/** 错过的消息列表 */
export interface MissedMessagesMessage extends WebSocketMessageBase {
  type: 'MissedMessages'
  payload: { room_id: string; messages: MissedMessage[]; has_more: boolean }
}

/** 会话恢复完成 */
export interface SessionRestoredMessage extends WebSocketMessageBase {
  type: 'SessionRestored'
  payload: { restored_at: string; rooms_restored: number; total_unread: number }
}

// ========== 通知系统 ==========

/** 私信 */
export interface PrivateMessage extends WebSocketMessageBase {
  type: 'PrivateMessage'
  payload: {
    message_id: string
    sender_id: string
    sender_name: string
    content: string
    created_at: string
  }
}

/** @提及通知 */
export interface MentionedMessage extends WebSocketMessageBase {
  type: 'Mentioned'
  payload: {
    message_id: string
    room_id: string
    mentioned_by: string
    mentioned_by_name: string
    content_preview: string
    created_at: string
  }
}

/** 房间邀请 */
export interface RoomInvitationMessage extends WebSocketMessageBase {
  type: 'RoomInvitation'
  payload: {
    invitation_id: string
    room_id: string
    room_name: string
    invited_by: string
    invited_by_name: string
    created_at: string
  }
}

/** 系统通知 */
export interface SystemNotificationMessage extends WebSocketMessageBase {
  type: 'SystemNotification'
  payload: {
    notification_type: NotificationType
    title: string
    content: string
    data?: unknown
    created_at: string
  }
}

/** 文件上传完成 */
export interface FileUploadCompleteMessage extends WebSocketMessageBase {
  type: 'FileUploadComplete'
  payload: {
    file_id: string
    file_name: string
    file_url: string
    file_size: number
    uploaded_at: string
  }
}

// ========== 通知管理 ==========

/** 获取离线通知 */
export interface GetOfflineNotificationsMessage extends WebSocketMessageBase {
  type: 'GetOfflineNotifications'
  payload: { last_notification_id?: string; limit?: number }
}

/** 标记通知已读 */
export interface MarkNotificationReadMessage extends WebSocketMessageBase {
  type: 'MarkNotificationRead'
  payload: { notification_id: string }
}

/** 标记所有通知已读 */
export interface MarkAllNotificationsReadMessage extends WebSocketMessageBase {
  type: 'MarkAllNotificationsRead'
}

/** 离线通知列表 */
export interface OfflineNotificationsMessage extends WebSocketMessageBase {
  type: 'OfflineNotifications'
  payload: { notifications: Notification[]; has_more: boolean }
}

/** 通知已读确认 */
export interface NotificationReadConfirmMessage extends WebSocketMessageBase {
  type: 'NotificationReadConfirm'
  payload: { notification_id: string }
}

// ========== 待办通知 ==========

/** 待办通知 */
export interface PendingActionMessage extends WebSocketMessageBase {
  type: 'PendingAction'
  payload: {
    notification_id: string
    action_type: string
    title: string
    description: string
    deadline?: string
    data?: unknown
    created_at: string
  }
}

/** 响应待办 */
export interface RespondPendingActionMessage extends WebSocketMessageBase {
  type: 'RespondPendingAction'
  payload: { notification_id: string; action: string; comment?: string }
}

/** 待办响应确认 */
export interface PendingActionResponseMessage extends WebSocketMessageBase {
  type: 'PendingActionResponse'
  payload: { notification_id: string; success: boolean; message: string; new_status: string }
}

/** 获取待办列表 */
export interface GetPendingActionsMessage extends WebSocketMessageBase {
  type: 'GetPendingActions'
  payload: { action_type?: string }
}

/** 待办列表 */
export interface PendingActionsListMessage extends WebSocketMessageBase {
  type: 'PendingActionsList'
  payload: { actions: PendingActionInfo[]; total: number }
}

// ========== 日志系统 ==========

/** 日志条目 */
export interface LogEntryMessage extends WebSocketMessageBase {
  type: 'LogEntry'
  payload: {
    level: string
    target: string
    message: string
    timestamp: string
    fields?: Record<string, unknown>
  }
}

/** 订阅日志 */
export interface SubscribeLogsMessage extends WebSocketMessageBase {
  type: 'SubscribeLogs'
  payload: { level: string; module: string }
}

/** 取消订阅日志 */
export interface UnsubscribeLogsMessage extends WebSocketMessageBase {
  type: 'UnsubscribeLogs'
}

/** 日志订阅确认 */
export interface LogSubscriptionConfirmedMessage extends WebSocketMessageBase {
  type: 'LogSubscriptionConfirmed'
  payload: { success: boolean }
}

// ========== 联合类型 ==========

/** 所有可能的 WebSocket 消息 */
export type WebSocketMessage =
  | AuthMessage
  | AuthResultMessage
  | PingMessage
  | PongMessage
  | ErrorMessage
  | JoinRoomMessage
  | LeaveRoomMessage
  | RoomJoinedMessage
  | RoomLeftMessage
  | UserJoinedMessage
  | UserLeftMessage
  | OnlineUsersMessage
  | ChatMessage
  | NewMessage
  | TypingMessage
  | StopTypingMessage
  | MessageReadMessage
  | MessageReadReceiptMessage
  | EditMessage
  | MessageEditedMessage
  | DeleteMessage
  | MessageDeletedMessage
  | SystemMessage
  | RoomUpdatedMessage
  | UpdateStatusMessage
  | UserStatusChangedMessage
  | GetOnlineUsersMessage
  | GlobalOnlineUsersMessage
  | ReconnectMessage
  | ReconnectResultMessage
  | GetMissedMessagesMessage
  | MissedMessagesMessage
  | SessionRestoredMessage
  | PrivateMessage
  | MentionedMessage
  | RoomInvitationMessage
  | SystemNotificationMessage
  | FileUploadCompleteMessage
  | GetOfflineNotificationsMessage
  | MarkNotificationReadMessage
  | MarkAllNotificationsReadMessage
  | OfflineNotificationsMessage
  | NotificationReadConfirmMessage
  | PendingActionMessage
  | RespondPendingActionMessage
  | PendingActionResponseMessage
  | GetPendingActionsMessage
  | PendingActionsListMessage
  | LogEntryMessage
  | SubscribeLogsMessage
  | UnsubscribeLogsMessage
  | LogSubscriptionConfirmedMessage

// ========== 连接状态 ==========

/** WebSocket 连接状态 */
export type ConnectionStatus = 'connecting' | 'connected' | 'disconnected' | 'reconnecting' | 'error'

/** WebSocket 配置 */
export interface WebSocketConfig {
  /** 重连最大次数 */
  maxReconnectAttempts?: number
  /** 重连间隔（毫秒） */
  reconnectInterval?: number
  /** 心跳间隔（毫秒） */
  heartbeatInterval?: number
  /** 连接超时（毫秒） */
  connectTimeout?: number
}

/** WebSocket 事件处理器 */
export interface WebSocketEventHandlers {
  onConnect?: () => void
  onDisconnect?: () => void
  onError?: (error: Error) => void
  onMessage?: (message: WebSocketMessage) => void
}
