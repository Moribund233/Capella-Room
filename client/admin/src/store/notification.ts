import { ref, computed, watch } from 'vue'
import { defineStore } from 'pinia'
import {
  getNotifications,
  getUnreadNotifications,
  markNotificationAsRead,
  markAllNotificationsAsRead,
  deleteNotification,
  type NotificationItem,
} from '@/api/notification'
import { useWebSocketStore } from './websocket'

/**
 * 通知状态管理 Store（WebSocket 版本）
 *
 * 功能：
 * - 管理通知列表和未读计数
 * - 提供获取、标记已读、删除通知的方法
 * - 支持 WebSocket 实时通知更新
 *
 * Admin 系统仅订阅系统级通知：
 * - SystemNotification: 系统广播通知
 * - FileUploadComplete: 文件上传完成通知
 * - PendingAction: 待办事项通知（需要管理员处理）
 */
export const useNotificationStore = defineStore('notification', () => {
  // ========== 状态 ==========

  /** 通知列表 */
  const notifications = ref<NotificationItem[]>([])

  /** 加载状态 */
  const loading = ref(false)

  /** 错误信息 */
  const error = ref<string | null>(null)

  /** 是否已初始化 WebSocket 监听 */
  let isListenersInitialized = false

  // ========== 计算属性 ==========

  /** 未读通知数量 */
  const unreadCount = computed(() => notifications.value.filter((n: NotificationItem) => !n.is_read).length)

  /** 未读通知列表 */
  const unreadNotifications = computed(() => notifications.value.filter((n: NotificationItem) => !n.is_read))

  /** 是否有未读通知 */
  const hasUnread = computed(() => unreadCount.value > 0)

  // ========== 私有方法 ==========

  /**
   * 添加通知（用于 WebSocket 实时推送）
   * @param notification 通知项
   */
  function addNotification(notification: NotificationItem): void {
    // 避免重复添加
    const exists = notifications.value.some((n: NotificationItem) => n.id === notification.id)
    if (!exists) {
      notifications.value.unshift(notification)
    }
  }

  // ========== 公共方法 ==========

  /**
   * 获取通知列表
   * @param limit 数量限制
   * @param offset 偏移量
   */
  async function fetchNotifications(limit: number = 50, offset: number = 0): Promise<void> {
    loading.value = true
    error.value = null
    try {
      const response = await getNotifications({ limit, offset })
      notifications.value = response.notifications
    } catch (err) {
      error.value = err instanceof Error ? err.message : '获取通知失败'
      console.error('[NotificationStore] 获取通知失败:', err)
    } finally {
      loading.value = false
    }
  }

  /**
   * 获取未读通知
   * @param limit 数量限制
   */
  async function fetchUnreadNotifications(limit: number = 50): Promise<void> {
    loading.value = true
    error.value = null
    try {
      const items = await getUnreadNotifications(limit)
      // 合并未读通知到列表中，避免重复
      const existingIds = new Set(notifications.value.map((n: NotificationItem) => n.id))
      const newItems = items.filter(item => !existingIds.has(item.id))
      notifications.value = [...newItems, ...notifications.value]
    } catch (err) {
      error.value = err instanceof Error ? err.message : '获取未读通知失败'
      console.error('[NotificationStore] 获取未读通知失败:', err)
    } finally {
      loading.value = false
    }
  }

  /**
   * 标记通知为已读
   * @param notificationId 通知ID
   */
  async function markAsRead(notificationId: string): Promise<void> {
    try {
      await markNotificationAsRead(notificationId)
      const notification = notifications.value.find((n: NotificationItem) => n.id === notificationId)
      if (notification) {
        notification.is_read = true
        notification.read_at = new Date().toISOString()
      }
    } catch (err) {
      console.error('[NotificationStore] 标记已读失败:', err)
      throw err
    }
  }

  /**
   * 标记所有通知为已读
   */
  async function markAllAsRead(): Promise<void> {
    try {
      await markAllNotificationsAsRead()
      notifications.value.forEach((n: NotificationItem) => {
        n.is_read = true
        n.read_at = new Date().toISOString()
      })
    } catch (err) {
      console.error('[NotificationStore] 标记全部已读失败:', err)
      throw err
    }
  }

  /**
   * 删除通知
   * @param notificationId 通知ID
   */
  async function removeNotification(notificationId: string): Promise<void> {
    try {
      await deleteNotification(notificationId)
      notifications.value = notifications.value.filter((n: NotificationItem) => n.id !== notificationId)
    } catch (err) {
      console.error('[NotificationStore] 删除通知失败:', err)
      throw err
    }
  }

  /**
   * 清空所有通知
   */
  function clearAllNotifications(): void {
    notifications.value = []
  }

  /**
   * 初始化 WebSocket 通知监听
   * 仅订阅 Admin 系统关心的通知类型：
   * - SystemNotification: 系统广播通知
   * - FileUploadComplete: 文件上传完成通知
   * - PendingAction: 待办事项通知（需要管理员处理）
   */
  function initWebSocketListeners(): void {
    // 避免重复初始化
    if (isListenersInitialized) {
      return
    }

    const wsStore = useWebSocketStore()

    // 监听系统通知
    wsStore.on('SystemNotification', (payload: unknown) => {
      const data = payload as Record<string, unknown>
      addNotification({
        id: crypto.randomUUID(),
        notification_type: 'system_notification',
        title: (data.title as string) || '系统通知',
        content: data.content as string,
        data: data as Record<string, unknown>,
        is_read: false,
        read_at: null,
        created_at: data.created_at as string,
      })
    })

    // 监听文件上传完成通知
    wsStore.on('FileUploadComplete', (payload: unknown) => {
      const data = payload as Record<string, unknown>
      addNotification({
        id: data.file_id as string,
        notification_type: 'file_upload_complete',
        title: '文件上传完成',
        content: `文件 "${data.file_name}" 上传完成`,
        data: data as Record<string, unknown>,
        is_read: false,
        read_at: null,
        created_at: data.uploaded_at as string,
      })
    })

    // 监听待办通知（需要管理员处理）
    wsStore.on('PendingAction', (payload: unknown) => {
      const data = payload as Record<string, unknown>
      addNotification({
        id: data.action_id as string,
        notification_type: 'pending_action',
        title: '待办事项',
        content: data.description as string,
        data: data as Record<string, unknown>,
        is_read: false,
        read_at: null,
        created_at: data.created_at as string,
      })
    })

    // 监听通知已读确认
    wsStore.on('NotificationReadConfirm', (payload: unknown) => {
      const data = payload as { notification_id: string }
      const notification = notifications.value.find((n: NotificationItem) => n.id === data.notification_id)
      if (notification) {
        notification.is_read = true
        notification.read_at = new Date().toISOString()
      }
    })

    // 监听连接状态变化，连接成功后自动获取通知
    watch(
      () => wsStore.isConnected,
      (isConnected) => {
        if (isConnected) {
          console.log('[NotificationStore] WebSocket 已连接，自动获取通知')
          fetchNotifications(50, 0)
        }
      },
      { immediate: true }
    )

    isListenersInitialized = true
    console.log('[NotificationStore] WebSocket 监听器已初始化')
  }

  return {
    // 状态
    notifications,
    loading,
    error,
    // 计算属性
    unreadCount,
    unreadNotifications,
    hasUnread,
    // 方法
    fetchNotifications,
    fetchUnreadNotifications,
    markAsRead,
    markAllAsRead,
    removeNotification,
    addNotification,
    clearAllNotifications,
    initWebSocketListeners,
  }
})
