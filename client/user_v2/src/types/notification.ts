/**
 * 通知系统类型定义
 * 对应后端 WebSocket 通知协议
 */

// ========== 通知类型 ==========

/** 通知类型枚举 */
export enum NotificationType {
  PRIVATE_MESSAGE = 'private_message',
  MENTION = 'mention',
  ROOM_INVITATION = 'room_invitation',
  SYSTEM = 'system',
  FILE_UPLOAD = 'file_upload',
  PENDING_ACTION = 'pending_action',
}

/** 通知显示类型 */
export type NotificationDisplayType = 'info' | 'success' | 'warning' | 'error'

// ========== 服务端推送的通知 Payload ==========

/** 私信通知 */
export interface PrivateMessageNotificationPayload {
  message_id: string
  sender_id: string
  sender_name: string
  content: string
  created_at: string
}

/** @提及通知 */
export interface MentionedNotificationPayload {
  message_id: string
  room_id: string
  mentioned_by: string
  mentioned_by_name: string
  content_preview: string
  created_at: string
}

/** 房间邀请通知 */
export interface RoomInvitationNotificationPayload {
  invitation_id: string
  room_id: string
  room_name: string
  invited_by: string
  invited_by_name: string
  created_at: string
}

/** 系统通知 */
export interface SystemNotificationPayload {
  notification_type: 'new' | 'important' | 'warning'
  title: string
  content: string
  data: unknown | null
  created_at: string
}

/** 文件上传完成通知 */
export interface FileUploadCompleteNotificationPayload {
  file_id: string
  file_name: string
  file_url: string
  file_size: number
  uploaded_at: string
}

/** 待办通知 */
export interface PendingActionNotificationPayload {
  notification_id: string
  action_type: string
  title: string
  description: string
  deadline: string | null
  data: unknown | null
  created_at: string
}

// ========== 客户端发送的消息 ==========

/** 获取离线通知 */
export interface GetOfflineNotificationsPayload {
  last_notification_id?: string | null
  limit?: number
}

/** 标记通知已读 */
export interface MarkNotificationReadPayload {
  notification_id: string
}

/** 响应待办通知 */
export interface RespondPendingActionPayload {
  notification_id: string
  action: 'approve' | 'reject' | 'snooze'
  comment?: string
}

/** 获取待办列表 */
export interface GetPendingActionsPayload {
  action_type?: string | null
}

// ========== 服务端响应 ==========

/** 离线通知列表 */
export interface OfflineNotificationItem {
  id: string
  notification_type: string
  title: string | null
  content: string
  data: {
    message_id?: string
    room_id?: string
    sender_id?: string
    sender_name?: string
    [key: string]: unknown
  } | null
  created_at: string
  is_read: boolean
}

export interface OfflineNotificationsPayload {
  notifications: OfflineNotificationItem[]
  has_more: boolean
}

/** 通知已读确认 */
export interface NotificationReadConfirmPayload {
  notification_id: string
}

/** 待办响应结果 */
export interface PendingActionResponsePayload {
  notification_id: string
  success: boolean
  message: string
  new_status: string
}

/** 待办列表 */
export interface PendingActionItem {
  notification_id: string
  action_type: string
  title: string
  description: string
  deadline: string | null
  data: unknown | null
  created_at: string
}

export interface PendingActionsListPayload {
  actions: PendingActionItem[]
}

// ========== 前端通知项（统一格式） ==========

/** 前端统一的通知项 */
export interface NotificationItem {
  id: string
  type: NotificationDisplayType
  title: string
  content: string
  isRead: boolean
  createdAt: string
  /** 关联数据 */
  data?: {
    messageId?: string
    roomId?: string
    senderId?: string
    senderName?: string
    fileId?: string
    fileUrl?: string
    invitationId?: string
    actionType?: string
    [key: string]: unknown
  }
}

// ========== WebSocket 消息类型扩展 ==========

export enum WSNotificationMessageType {
  // 服务端推送
  PRIVATE_MESSAGE = 'PrivateMessage',
  MENTIONED = 'Mentioned',
  ROOM_INVITATION = 'RoomInvitation',
  SYSTEM_NOTIFICATION = 'SystemNotification',
  FILE_UPLOAD_COMPLETE = 'FileUploadComplete',
  PENDING_ACTION = 'PendingAction',
  OFFLINE_NOTIFICATIONS = 'OfflineNotifications',
  NOTIFICATION_READ_CONFIRM = 'NotificationReadConfirm',
  PENDING_ACTION_RESPONSE = 'PendingActionResponse',
  PENDING_ACTIONS_LIST = 'PendingActionsList',

  // 客户端发送
  GET_OFFLINE_NOTIFICATIONS = 'GetOfflineNotifications',
  MARK_NOTIFICATION_READ = 'MarkNotificationRead',
  MARK_ALL_NOTIFICATIONS_READ = 'MarkAllNotificationsRead',
  RESPOND_PENDING_ACTION = 'RespondPendingAction',
  GET_PENDING_ACTIONS = 'GetPendingActions',
}
