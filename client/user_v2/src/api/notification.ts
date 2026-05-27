import httpClient from '@/services/http'
import type { ApiResponse } from '@/types/api'

/**
 * 通知项（来自 HTTP API）
 */
export interface NotificationResponse {
  id: string
  notification_type: string
  title: string | null
  content: string
  data: {
    message_id?: string
    room_id?: string
    sender_id?: string
    sender_name?: string
    mentioned_by?: string
    mentioned_by_name?: string
    [key: string]: unknown
  } | null
  is_read: boolean
  read_at: string | null
  created_at: string
}

/**
 * 通知列表响应
 */
export interface NotificationListResponse {
  notifications: NotificationResponse[]
  unread_count: number
  has_more: boolean
}

/**
 * 获取通知列表查询参数
 */
export interface ListNotificationsParams {
  unread_only?: boolean
  limit?: number
  offset?: number
}

/**
 * 通知 API
 * 提供 HTTP 接口用于获取通知列表和标记已读
 */
export const notificationApi = {
  /**
   * 获取当前用户的通知列表
   * @param params 查询参数
   * @returns 通知列表响应
   */
  getNotifications(params: ListNotificationsParams = {}): Promise<ApiResponse<NotificationListResponse>> {
    const { unread_only = true, limit = 20, offset = 0 } = params
    return httpClient.get('/notifications', {
      params: { unread_only, limit, offset },
    })
  },

  /**
   * 获取未读通知数量
   * @returns 未读数量
   */
  getUnreadCount(): Promise<ApiResponse<{ count: number }>> {
    return httpClient.get('/notifications/unread-count')
  },

  /**
   * 标记通知为已读
   * @param notificationId 通知ID
   * @returns 操作结果
   */
  markAsRead(notificationId: string): Promise<ApiResponse<void>> {
    return httpClient.post(`/notifications/${notificationId}/read`)
  },

  /**
   * 标记所有通知为已读
   * @returns 操作结果
   */
  markAllAsRead(): Promise<ApiResponse<void>> {
    return httpClient.post('/notifications/read-all')
  },
}
