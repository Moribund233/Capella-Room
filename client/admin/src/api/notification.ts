/**
 * 通知管理 API（WebSocket 版本）
 * 通过 WebSocket 进行通知的获取、标记已读等操作
 */

import { useWebSocketStore } from '@/store/websocket'
import type {
  NotificationItem,
  GetOfflineNotificationsParams,
  OfflineNotificationsPayload,
  MarkNotificationReadParams,
  NotificationReadConfirmPayload,
  WsMessage,
} from '@/types'

/**
 * 获取 WebSocket Store 实例
 */
function getWsStore() {
  return useWebSocketStore()
}

/**
 * 检查 WebSocket 是否已连接
 */
function isWsConnected(): boolean {
  const wsStore = getWsStore()
  return wsStore.isConnected
}

/**
 * 获取通知列表（通过 WebSocket 获取离线通知）
 * @param params 查询参数
 * @returns Promise<NotificationItem[]>
 */
export async function getNotifications(
  params: { limit?: number; offset?: number } = {},
): Promise<{ notifications: NotificationItem[]; total: number }> {
  // 检查 WebSocket 连接状态
  if (!isWsConnected()) {
    console.warn('[NotificationAPI] WebSocket 未连接，跳过获取通知')
    return { notifications: [], total: 0 }
  }

  return new Promise((resolve, reject) => {
    const wsStore = getWsStore()
    const limit = params.limit || 50

    // 发送获取离线通知请求
    const message: WsMessage<GetOfflineNotificationsParams> = {
      type: 'GetOfflineNotifications',
      payload: {
        last_notification_id: null,
        limit,
      },
    }

    // 设置一次性监听器
    const handler = (payload: unknown) => {
      const data = payload as OfflineNotificationsPayload
      wsStore.off('OfflineNotifications', handler)
      resolve({
        notifications: data.notifications || [],
        total: data.notifications?.length || 0,
      })
    }

    // 设置超时
    const timeout = setTimeout(() => {
      wsStore.off('OfflineNotifications', handler)
      reject(new Error('获取通知超时'))
    }, 10000)

    // 监听响应
    wsStore.on('OfflineNotifications', (payload: unknown) => {
      clearTimeout(timeout)
      handler(payload)
    })

    // 发送请求
    wsStore.send(message)
  })
}

/**
 * 获取未读通知列表
 * @param limit 最大数量
 * @returns Promise<NotificationItem[]>
 */
export async function getUnreadNotifications(limit: number = 50): Promise<NotificationItem[]> {
  // 检查 WebSocket 连接状态
  if (!isWsConnected()) {
    console.warn('[NotificationAPI] WebSocket 未连接，跳过获取未读通知')
    return []
  }

  const result = await getNotifications({ limit, offset: 0 })
  return result.notifications.filter(n => !n.is_read)
}

/**
 * 标记通知为已读
 * @param notificationId 通知ID
 */
export async function markNotificationAsRead(notificationId: string): Promise<void> {
  // 检查 WebSocket 连接状态
  if (!isWsConnected()) {
    console.warn('[NotificationAPI] WebSocket 未连接，无法标记已读')
    throw new Error('WebSocket 未连接')
  }

  return new Promise((resolve, reject) => {
    const wsStore = getWsStore()
    const message: WsMessage<MarkNotificationReadParams> = {
      type: 'MarkNotificationRead',
      payload: {
        notification_id: notificationId,
      },
    }

    // 设置超时
    const timeout = setTimeout(() => {
      wsStore.off('NotificationReadConfirm', handler)
      reject(new Error('标记已读超时'))
    }, 10000)

    // 监听确认
    const handler = (payload: unknown) => {
      const data = payload as NotificationReadConfirmPayload
      if (data.notification_id === notificationId) {
        clearTimeout(timeout)
        wsStore.off('NotificationReadConfirm', handler)
        resolve()
      }
    }

    wsStore.on('NotificationReadConfirm', handler)
    wsStore.send(message)
  })
}

/**
 * 标记所有通知为已读
 */
export async function markAllNotificationsAsRead(): Promise<void> {
  // 检查 WebSocket 连接状态
  if (!isWsConnected()) {
    console.warn('[NotificationAPI] WebSocket 未连接，无法标记全部已读')
    throw new Error('WebSocket 未连接')
  }

  const wsStore = getWsStore()
  const message: WsMessage<never> = {
    type: 'MarkAllNotificationsRead',
  }
  wsStore.send(message)
}

/**
 * 删除通知（WebSocket 不支持删除，提供空实现）
 * @param _notificationId 通知ID
 */
export async function deleteNotification(_notificationId: string): Promise<void> {
  // WebSocket 协议暂不支持删除通知
  console.warn('[NotificationAPI] WebSocket 模式不支持删除通知')
}

// 导出类型
export type { NotificationItem }
