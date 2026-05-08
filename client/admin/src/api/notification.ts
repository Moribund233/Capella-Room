/**
 * 通知管理 API（HTTP + WebSocket 混合模式）
 *
 * 架构说明：
 * - HTTP API: 用于获取通知列表、标记已读/未读（主要方式）
 * - WebSocket: 仅用于实时接收新通知推送
 *
 * 后端采用双写模式：所有通知先写入数据库，再推送 WebSocket（如果用户在线）
 */

import { http } from './request'
import { useWebSocketStore } from '@/store/websocket'
import type {
  NotificationItem,
} from '@/types'

/**
 * 通知列表响应数据
 */
interface NotificationsResponse {
  notifications: NotificationItem[]
  unread_count: number
  has_more: boolean
}

/**
 * 标记所有已读响应数据
 */
interface MarkAllReadResponse {
  marked_count: number
}

/**
 * 获取通知列表
 *
 * 使用 HTTP API 获取通知列表，支持分页和筛选
 *
 * @param params 查询参数
 * @param params.unread_only 是否只返回未读通知，默认 true
 * @param params.limit 返回数量限制，默认 50
 * @param params.offset 分页偏移量，默认 0
 * @returns Promise<NotificationsResponse>
 */
export async function getNotifications(
  params: { unread_only?: boolean; limit?: number; offset?: number } = {},
): Promise<NotificationsResponse> {
  const { unread_only = true, limit = 50, offset = 0 } = params

  const response = await http.get<NotificationsResponse>(
    `/notifications?unread_only=${unread_only}&limit=${limit}&offset=${offset}`,
  )

  if (!response.success || !response.data) {
    throw new Error(response.error || response.message || '获取通知列表失败')
  }

  return response.data
}

/**
 * 获取未读通知数量
 *
 * @returns Promise<number> 未读通知数量
 */
export async function getUnreadCount(): Promise<number> {
  const response = await http.get<number>('/notifications/unread-count')

  if (!response.success || response.data === undefined || response.data === null) {
    throw new Error(response.error || response.message || '获取未读数量失败')
  }

  return response.data
}

/**
 * 标记通知为已读
 *
 * @param notificationId 通知ID
 * @returns Promise<void>
 */
export async function markNotificationAsRead(notificationId: string): Promise<void> {
  const response = await http.post<null>(`/notifications/${notificationId}/read`)

  if (!response.success) {
    throw new Error(response.error || response.message || '标记已读失败')
  }
}

/**
 * 标记所有通知为已读
 *
 * @returns Promise<number> 被标记为已读的通知数量
 */
export async function markAllNotificationsAsRead(): Promise<number> {
  const response = await http.post<MarkAllReadResponse>('/notifications/read-all')

  if (!response.success || !response.data) {
    throw new Error(response.error || response.message || '标记全部已读失败')
  }

  return response.data.marked_count
}

/**
 * 删除通知
 *
 * 注意：当前后端 API 可能不支持删除通知，此函数保留用于未来扩展
 *
 * @param _notificationId 通知ID
 */
export async function deleteNotification(_notificationId: string): Promise<void> {
  // 当前后端 API 暂不支持删除通知
  console.warn('[NotificationAPI] 删除通知功能暂不可用')
  // 如果需要实现，可以使用：
  // await http.delete(`/notifications/${notificationId}`)
}

// ==================== WebSocket 相关（仅用于接收实时推送）====================

/**
 * WebSocket 通知类型到 NotificationDbType 的映射
 *
 * 注意：HTTP API 返回的类型与 WebSocket 推送的类型可能不同
 * HTTP API 使用: mention, private_message, room_invitation, system, file_upload, pending_action
 * WebSocket 使用: Mentioned, PrivateMessage, RoomInvitation, SystemNotification, FileUploadComplete, PendingAction
 */
const WS_TYPE_TO_DB_TYPE: Record<string, NotificationItem['notification_type']> = {
  'PrivateMessage': 'private_message',
  'Mentioned': 'mention',
  'RoomInvitation': 'room_invitation',
  'SystemNotification': 'system',
  'FileUploadComplete': 'file_upload',
  'PendingAction': 'pending_action',
}

/**
 * 从 WebSocket 推送数据创建通知项
 *
 * @param type WebSocket 消息类型
 * @param payload WebSocket 消息载荷
 * @returns NotificationItem | null
 */
export function createNotificationFromWsPayload(
  type: string,
  payload: Record<string, unknown>,
): NotificationItem | null {
  const dbType = WS_TYPE_TO_DB_TYPE[type]
  if (!dbType) {
    return null
  }

  const baseNotification: NotificationItem = {
    id: (payload.id as string) || crypto.randomUUID(),
    notification_type: dbType,
    title: null,
    content: '',
    data: payload,
    is_read: false,
    read_at: null,
    created_at: (payload.created_at as string) || new Date().toISOString(),
  }

  switch (type) {
    case 'PrivateMessage':
      return {
        ...baseNotification,
        id: (payload.message_id as string) || baseNotification.id,
        title: '新私信',
        content: `${payload.sender_name}: ${payload.content}`,
      }

    case 'Mentioned':
      return {
        ...baseNotification,
        id: (payload.message_id as string) || baseNotification.id,
        title: '有人提到了你',
        content: `${payload.mentioned_by_name} 在消息中提到了你`,
      }

    case 'RoomInvitation':
      return {
        ...baseNotification,
        id: (payload.invitation_id as string) || baseNotification.id,
        title: '房间邀请',
        content: `${payload.invited_by_name} 邀请你加入房间 "${payload.room_name}"`,
      }

    case 'SystemNotification':
      return {
        ...baseNotification,
        title: (payload.title as string) || '系统通知',
        content: (payload.content as string) || '',
      }

    case 'FileUploadComplete':
      return {
        ...baseNotification,
        id: (payload.file_id as string) || baseNotification.id,
        title: '文件上传完成',
        content: `文件 "${payload.file_name}" 上传完成`,
      }

    case 'PendingAction':
      return {
        ...baseNotification,
        id: (payload.action_id as string) || baseNotification.id,
        title: (payload.title as string) || '待办事项',
        content: (payload.description as string) || '',
      }

    default:
      return null
  }
}

/**
 * 订阅 WebSocket 通知推送
 *
 * 仅用于实时接收新通知，不涉及获取历史通知或标记已读
 *
 * @param handler 通知处理器
 * @returns 取消订阅函数
 */
export function subscribeToNotifications(
  handler: (notification: NotificationItem) => void,
): () => void {
  const wsStore = useWebSocketStore()

  // 需要监听的通知类型
  const notificationTypes = [
    'PrivateMessage',
    'Mentioned',
    'RoomInvitation',
    'SystemNotification',
    'FileUploadComplete',
    'PendingAction',
  ]

  // 为每种通知类型创建处理器
  const handlers: Array<{ type: string; fn: (payload: unknown) => void }> = []

  notificationTypes.forEach((type) => {
    const wsHandler = (payload: unknown) => {
      const notification = createNotificationFromWsPayload(type, payload as Record<string, unknown>)
      if (notification) {
        handler(notification)
      }
    }
    wsStore.on(type, wsHandler)
    handlers.push({ type, fn: wsHandler })
  })

  // 返回取消订阅函数
  return () => {
    handlers.forEach(({ type, fn }) => {
      wsStore.off(type, fn)
    })
  }
}

// 导出类型
export type { NotificationItem }
