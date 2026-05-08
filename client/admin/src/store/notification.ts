import { ref, computed, watch } from 'vue'
import { defineStore } from 'pinia'
import {
  getNotifications,
  getUnreadCount,
  markNotificationAsRead,
  markAllNotificationsAsRead,
  deleteNotification,
  subscribeToNotifications,
  type NotificationItem,
} from '@/api/notification'
import { useWebSocketStore } from './websocket'

/**
 * 通知状态管理 Store（HTTP + WebSocket 混合模式）
 *
 * 架构说明：
 * - HTTP API: 用于获取通知列表、标记已读/未读（主要方式）
 * - WebSocket: 仅用于实时接收新通知推送
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

  /** 未读通知数量（从 API 获取的准确值） */
  const unreadCountFromApi = ref(0)

  /** 加载状态 */
  const loading = ref(false)

  /** 错误信息 */
  const error = ref<string | null>(null)

  /** 是否已初始化 WebSocket 监听 */
  let isListenersInitialized = false

  /** 取消 WebSocket 订阅函数 */
  let unsubscribeFromWs: (() => void) | null = null

  // ========== 计算属性 ==========

  /**
   * 未读通知数量
   * 优先使用 API 返回的准确值，如果没有则计算本地列表
   */
  const unreadCount = computed(() => {
    // 如果 API 返回了未读数量，优先使用
    if (unreadCountFromApi.value > 0) {
      return unreadCountFromApi.value
    }
    // 否则从本地列表计算
    return notifications.value.filter((n: NotificationItem) => !n.is_read).length
  })

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
      // 增加未读计数
      unreadCountFromApi.value++
    }
  }

  // ========== 公共方法 ==========

  /**
   * 获取通知列表
   *
   * 使用 HTTP API 获取通知列表，支持分页和筛选
   *
   * @param limit 数量限制
   * @param offset 偏移量
   * @param unread_only 是否只获取未读通知，默认 true
   */
  async function fetchNotifications(
    limit: number = 50,
    offset: number = 0,
    unread_only: boolean = true,
  ): Promise<void> {
    loading.value = true
    error.value = null
    try {
      const response = await getNotifications({ limit, offset, unread_only })
      notifications.value = response.notifications
      unreadCountFromApi.value = response.unread_count
    } catch (err) {
      error.value = err instanceof Error ? err.message : '获取通知失败'
      console.error('[NotificationStore] 获取通知失败:', err)
    } finally {
      loading.value = false
    }
  }

  /**
   * 获取未读通知数量
   *
   * 使用 HTTP API 获取准确的未读数量
   */
  async function fetchUnreadCount(): Promise<void> {
    try {
      const count = await getUnreadCount()
      unreadCountFromApi.value = count
    } catch (err) {
      console.error('[NotificationStore] 获取未读数量失败:', err)
    }
  }

  /**
   * 刷新通知数据
   *
   * 同时获取通知列表和未读数量
   */
  async function refreshNotifications(limit: number = 50): Promise<void> {
    loading.value = true
    error.value = null
    try {
      // 并行获取通知列表和未读数量
      const [listResponse, count] = await Promise.all([
        getNotifications({ limit, offset: 0, unread_only: true }),
        getUnreadCount(),
      ])
      notifications.value = listResponse.notifications
      unreadCountFromApi.value = count
    } catch (err) {
      error.value = err instanceof Error ? err.message : '刷新通知失败'
      console.error('[NotificationStore] 刷新通知失败:', err)
    } finally {
      loading.value = false
    }
  }

  /**
   * 标记通知为已读
   *
   * 使用 HTTP API 标记已读
   *
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
      // 减少未读计数
      if (unreadCountFromApi.value > 0) {
        unreadCountFromApi.value--
      }
    } catch (err) {
      console.error('[NotificationStore] 标记已读失败:', err)
      throw err
    }
  }

  /**
   * 标记所有通知为已读
   *
   * 使用 HTTP API 标记所有已读
   */
  async function markAllAsRead(): Promise<void> {
    try {
      const markedCount = await markAllNotificationsAsRead()
      notifications.value.forEach((n: NotificationItem) => {
        n.is_read = true
        n.read_at = new Date().toISOString()
      })
      // 更新未读计数
      unreadCountFromApi.value = Math.max(0, unreadCountFromApi.value - markedCount)
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
      const notification = notifications.value.find((n: NotificationItem) => n.id === notificationId)
      notifications.value = notifications.value.filter((n: NotificationItem) => n.id !== notificationId)
      // 如果被删除的是未读通知，减少未读计数
      if (notification && !notification.is_read && unreadCountFromApi.value > 0) {
        unreadCountFromApi.value--
      }
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
    unreadCountFromApi.value = 0
  }

  /**
   * 初始化 WebSocket 通知监听
   *
   * 仅订阅实时推送的新通知，不涉及获取历史通知
   *
   * Admin 系统关心的通知类型：
   * - SystemNotification: 系统广播通知
   * - FileUploadComplete: 文件上传完成通知
   * - PendingAction: 待办事项通知（需要管理员处理）
   */
  function initWebSocketListeners(): void {
    // 避免重复初始化
    if (isListenersInitialized) {
      return
    }

    // 订阅 WebSocket 通知推送
    unsubscribeFromWs = subscribeToNotifications((notification) => {
      console.log('[NotificationStore] 收到 WebSocket 通知:', notification)
      addNotification(notification)
    })

    // 监听连接状态变化，连接成功后自动刷新通知
    watch(
      () => useWebSocketStore().isConnected,
      (isConnected) => {
        if (isConnected) {
          console.log('[NotificationStore] WebSocket 已连接，自动刷新通知')
          refreshNotifications(50)
        }
      },
      { immediate: true },
    )

    isListenersInitialized = true
    console.log('[NotificationStore] WebSocket 监听器已初始化')
  }

  /**
   * 清理 WebSocket 监听
   */
  function cleanupWebSocketListeners(): void {
    if (unsubscribeFromWs) {
      unsubscribeFromWs()
      unsubscribeFromWs = null
    }
    isListenersInitialized = false
  }

  return {
    // 状态
    notifications,
    unreadCountFromApi,
    loading,
    error,
    // 计算属性
    unreadCount,
    unreadNotifications,
    hasUnread,
    // 方法
    fetchNotifications,
    fetchUnreadCount,
    refreshNotifications,
    markAsRead,
    markAllAsRead,
    removeNotification,
    addNotification,
    clearAllNotifications,
    initWebSocketListeners,
    cleanupWebSocketListeners,
  }
})
