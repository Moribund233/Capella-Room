import { defineStore } from 'pinia'
import { ref, computed, watch } from 'vue'
import { wsService } from '@/services/websocket'
import { useWebSocketStore } from './websocket'
import { notificationApi, type NotificationResponse } from '@/api/notification'
import {
  WSNotificationMessageType,
  type NotificationItem,
  type PrivateMessageNotificationPayload,
  type MentionedNotificationPayload,
  type RoomInvitationNotificationPayload,
  type SystemNotificationPayload,
  type FileUploadCompleteNotificationPayload,
  type PendingActionNotificationPayload,
  type PendingActionsListPayload,
} from '@/types/notification'

/**
 * 通知状态管理 Store
 * 负责管理通知的接收、存储、读取状态同步
 */
export const useNotificationStore = defineStore('notification', () => {
  // ========== State ==========
  /** 通知列表 */
  const notifications = ref<NotificationItem[]>([])
  /** 待办事项列表 */
  const pendingActions = ref<PendingActionNotificationPayload[]>([])
  /** 是否正在加载 */
  const loading = ref(false)
  /** 是否已初始化 */
  const initialized = ref(false)
  /** 是否还有更多离线通知 */
  const hasMoreOffline = ref(false)
  /** 从服务器获取的未读计数 */
  const serverUnreadCount = ref(0)
  /** 通知面板是否显示 */
  const isPanelOpen = ref(false)

  // ========== Getters ==========
  /** 本地未读通知数量（基于内存中的通知） */
  const localUnreadCount = computed(() => notifications.value.filter(n => !n.isRead).length)
  /** 未读待办数量 */
  const pendingCount = computed(() => pendingActions.value.length)
  /** 总未读数量（使用服务器计数，更准确） */
  const totalUnreadCount = computed(() => serverUnreadCount.value + pendingCount.value)
  /** 未读计数（用于徽标显示） */
  const unreadCount = computed(() => serverUnreadCount.value)
  /** 按日期分组的通知 */
  const groupedNotifications = computed(() => {
    const groups: Record<string, NotificationItem[]> = {}
    notifications.value.forEach(notification => {
      const date = new Date(notification.createdAt).toLocaleDateString('zh-CN')
      if (!groups[date]) {
        groups[date] = []
      }
      groups[date].push(notification)
    })
    return groups
  })

  // ========== WebSocket 处理器 ==========

  /**
   * 处理私信通知
   */
  function handlePrivateMessage(payload: PrivateMessageNotificationPayload) {
    const notification: NotificationItem = {
      id: payload.message_id,
      type: 'info',
      title: `来自 ${payload.sender_name} 的私信`,
      content: payload.content,
      isRead: false,
      createdAt: payload.created_at,
      data: {
        messageId: payload.message_id,
        senderId: payload.sender_id,
        senderName: payload.sender_name,
      },
    }
    addNotification(notification)
  }

  /**
   * 处理@提及通知
   */
  function handleMentioned(payload: MentionedNotificationPayload) {
    const notification: NotificationItem = {
      id: payload.message_id,
      type: 'warning',
      title: `${payload.mentioned_by_name} 提到了你`,
      content: payload.content_preview,
      isRead: false,
      createdAt: payload.created_at,
      data: {
        messageId: payload.message_id,
        roomId: payload.room_id,
        senderId: payload.mentioned_by,
        senderName: payload.mentioned_by_name,
      },
    }
    addNotification(notification)
  }

  /**
   * 处理房间邀请通知
   */
  function handleRoomInvitation(payload: RoomInvitationNotificationPayload) {
    const notification: NotificationItem = {
      id: payload.invitation_id,
      type: 'success',
      title: '房间邀请',
      content: `${payload.invited_by_name} 邀请你加入房间 "${payload.room_name}"`,
      isRead: false,
      createdAt: payload.created_at,
      data: {
        invitationId: payload.invitation_id,
        roomId: payload.room_id,
        senderId: payload.invited_by,
        senderName: payload.invited_by_name,
      },
    }
    addNotification(notification)
  }

  /**
   * 处理系统通知
   */
  function handleSystemNotification(payload: SystemNotificationPayload) {
    const typeMap: Record<string, NotificationItem['type']> = {
      new: 'info',
      important: 'warning',
      warning: 'error',
    }
    const notification: NotificationItem = {
      id: `system_${Date.now()}`,
      type: typeMap[payload.notification_type] || 'info',
      title: payload.title,
      content: payload.content,
      isRead: false,
      createdAt: payload.created_at,
      data: payload.data as Record<string, unknown> || {},
    }
    addNotification(notification)
  }

  /**
   * 处理文件上传完成通知
   */
  function handleFileUploadComplete(payload: FileUploadCompleteNotificationPayload) {
    const notification: NotificationItem = {
      id: payload.file_id,
      type: 'success',
      title: '文件上传完成',
      content: `文件 "${payload.file_name}" 上传成功`,
      isRead: false,
      createdAt: payload.uploaded_at,
      data: {
        fileId: payload.file_id,
        fileUrl: payload.file_url,
      },
    }
    addNotification(notification)
  }

  /**
   * 处理待办通知
   */
  function handlePendingAction(payload: PendingActionNotificationPayload) {
    pendingActions.value.push(payload)
    const notification: NotificationItem = {
      id: payload.notification_id,
      type: 'warning',
      title: payload.title,
      content: payload.description,
      isRead: false,
      createdAt: payload.created_at,
      data: {
        actionType: payload.action_type,
        ...(payload.data as Record<string, unknown>),
      },
    }
    addNotification(notification)
  }

  /**
   * 处理待办列表
   */
  function handlePendingActionsList(payload: PendingActionsListPayload) {
    pendingActions.value = payload.actions.map(action => ({
      notification_id: action.notification_id,
      action_type: action.action_type,
      title: action.title,
      description: action.description,
      deadline: action.deadline,
      data: action.data,
      created_at: action.created_at,
    }))
  }

  // ========== 辅助函数 ==========

  /**
   * 获取默认标题
   */
  function getDefaultTitle(notificationType: string): string {
    const titles: Record<string, string> = {
      mention: '有人提到了你',
      private_message: '新私信',
      room_invitation: '房间邀请',
      system: '系统通知',
      file_upload: '文件上传完成',
      pending_action: '待办事项',
    }
    return titles[notificationType] || '新通知'
  }

  /**
   * 添加通知到列表
   */
  function addNotification(notification: NotificationItem) {
    // 检查是否已存在
    const exists = notifications.value.some(n => n.id === notification.id)
    if (!exists) {
      notifications.value.unshift(notification)
      // 如果是未读通知，增加服务器计数
      if (!notification.isRead) {
        serverUnreadCount.value++
      }
    }
  }

  // ========== Actions ==========

  // 用于存储 watch 停止函数
  let wsConnectionWatchStop: (() => void) | null = null

  /**
   * 初始化通知系统
   * 注册 WebSocket 消息处理器
   */
  function initialize() {
    if (initialized.value) return

    // 注册服务端推送通知的处理器（仅用于实时推送新通知）
    wsService.onMessage(WSNotificationMessageType.PRIVATE_MESSAGE, handlePrivateMessage)
    wsService.onMessage(WSNotificationMessageType.MENTIONED, handleMentioned)
    wsService.onMessage(WSNotificationMessageType.ROOM_INVITATION, handleRoomInvitation)
    wsService.onMessage(WSNotificationMessageType.SYSTEM_NOTIFICATION, handleSystemNotification)
    wsService.onMessage(WSNotificationMessageType.FILE_UPLOAD_COMPLETE, handleFileUploadComplete)
    wsService.onMessage(WSNotificationMessageType.PENDING_ACTION, handlePendingAction)
    wsService.onMessage(WSNotificationMessageType.PENDING_ACTIONS_LIST, handlePendingActionsList)

    initialized.value = true

    // 监听 WebSocket 连接状态，连接成功后使用 HTTP API 获取通知列表
    const wsStore = useWebSocketStore()
    wsConnectionWatchStop = watch(
      () => wsStore.isConnected,
      (isConnected) => {
        if (isConnected) {
          // 使用 HTTP API 获取通知列表（替代 WebSocket 的离线通知）
          fetchNotifications(true, 50, 0)
          fetchPendingActions()
        }
      },
      { immediate: true }
    )
  }

  /**
   * 清理通知系统
   * 取消 WebSocket 消息处理器注册
   */
  function cleanup() {
    wsService.offMessage(WSNotificationMessageType.PRIVATE_MESSAGE, handlePrivateMessage)
    wsService.offMessage(WSNotificationMessageType.MENTIONED, handleMentioned)
    wsService.offMessage(WSNotificationMessageType.ROOM_INVITATION, handleRoomInvitation)
    wsService.offMessage(WSNotificationMessageType.SYSTEM_NOTIFICATION, handleSystemNotification)
    wsService.offMessage(WSNotificationMessageType.FILE_UPLOAD_COMPLETE, handleFileUploadComplete)
    wsService.offMessage(WSNotificationMessageType.PENDING_ACTION, handlePendingAction)
    wsService.offMessage(WSNotificationMessageType.PENDING_ACTIONS_LIST, handlePendingActionsList)

    // 停止监听 WebSocket 连接状态
    if (wsConnectionWatchStop) {
      wsConnectionWatchStop()
      wsConnectionWatchStop = null
    }

    initialized.value = false
  }

  /**
   * 将 API 通知格式转换为前端通知格式
   */
  function convertApiNotification(item: NotificationResponse): NotificationItem {
    const typeMap: Record<string, NotificationItem['type']> = {
      mention: 'warning',
      mentioned: 'warning',
      private_message: 'info',
      room_invitation: 'success',
      system_notification: 'info',
      file_upload_complete: 'success',
      pending_action: 'warning',
    }

    return {
      id: item.id,
      type: typeMap[item.notification_type] || 'info',
      title: item.title || getDefaultTitle(item.notification_type),
      content: item.content,
      isRead: item.is_read,
      createdAt: item.created_at,
      data: item.data || {},
    }
  }

  /**
   * 获取通知列表（使用 HTTP API）
   */
  async function fetchNotifications(unreadOnly: boolean = true, limit: number = 50, offset: number = 0) {
    loading.value = true
    try {
      const apiResponse = await notificationApi.getNotifications({
        unread_only: unreadOnly,
        limit,
        offset,
      })

      if (apiResponse.success && apiResponse.data) {
        const { notifications: apiNotifications, has_more, unread_count } = apiResponse.data

        // 转换并合并通知
        const convertedNotifications = apiNotifications.map(convertApiNotification)

        if (offset === 0) {
          // 首次加载，替换现有通知
          notifications.value = convertedNotifications
        } else {
          // 加载更多，追加通知
          convertedNotifications.forEach((notification: NotificationItem) => {
            const exists = notifications.value.some(n => n.id === notification.id)
            if (!exists) {
              notifications.value.push(notification)
            }
          })
        }

        // 按时间排序
        notifications.value.sort((a, b) =>
          new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime(),
        )

        hasMoreOffline.value = has_more
        // 更新服务器未读计数
        serverUnreadCount.value = unread_count
      }
    } catch (error) {
      console.error('获取通知列表失败:', error)
    } finally {
      loading.value = false
    }
  }

  /**
   * 获取离线通知（兼容旧方法，使用 HTTP API）
   */
  async function fetchOfflineNotifications(lastId?: string | null, limit: number = 50) {
    const offset = lastId
      ? notifications.value.findIndex(n => n.id === lastId) + 1
      : 0
    await fetchNotifications(true, limit, offset)
  }

  /**
   * 获取更多离线通知
   */
  function loadMoreOfflineNotifications() {
    if (!hasMoreOffline.value || notifications.value.length === 0) return
    const lastNotification = notifications.value[notifications.value.length - 1]
    if (lastNotification) {
      fetchOfflineNotifications(lastNotification.id)
    }
  }

  /**
   * 标记通知已读（使用 HTTP API）
   */
  async function markAsRead(notificationId: string) {
    // 乐观更新
    const notification = notifications.value.find(n => n.id === notificationId)
    const wasUnread = notification && !notification.isRead
    if (notification) {
      notification.isRead = true
    }

    try {
      await notificationApi.markAsRead(notificationId)
      // 更新服务器计数
      if (wasUnread && serverUnreadCount.value > 0) {
        serverUnreadCount.value--
      }
    } catch (error) {
      console.error('标记通知已读失败:', error)
      // 失败时回滚状态
      if (notification) {
        notification.isRead = false
      }
    }
  }

  /**
   * 标记所有通知已读（使用 HTTP API）
   */
  async function markAllAsRead() {
    // 乐观更新
    const previousStates = notifications.value.map(n => ({ id: n.id, isRead: n.isRead }))
    notifications.value.forEach(notification => {
      notification.isRead = true
    })
    const previousCount = serverUnreadCount.value
    serverUnreadCount.value = 0

    try {
      await notificationApi.markAllAsRead()
    } catch (error) {
      console.error('标记全部已读失败:', error)
      // 失败时回滚状态
      previousStates.forEach(({ id, isRead }) => {
        const notification = notifications.value.find(n => n.id === id)
        if (notification) {
          notification.isRead = isRead
        }
      })
      serverUnreadCount.value = previousCount
    }
  }

  /**
   * 删除通知
   */
  function deleteNotification(notificationId: string) {
    const index = notifications.value.findIndex(n => n.id === notificationId)
    if (index > -1) {
      const notification = notifications.value[index]
      const wasUnread = notification && !notification.isRead
      notifications.value.splice(index, 1)
      // 更新服务器计数
      if (wasUnread && serverUnreadCount.value > 0) {
        serverUnreadCount.value--
      }
    }
  }

  /**
   * 清空所有通知
   */
  function clearAll() {
    notifications.value = []
    serverUnreadCount.value = 0
  }

  /**
   * 获取待办列表
   */
  function fetchPendingActions(actionType?: string | null) {
    wsService.send(WSNotificationMessageType.GET_PENDING_ACTIONS, {
      action_type: actionType || null,
    })
  }

  /**
   * 响应待办通知
   */
  function respondPendingAction(
    notificationId: string,
    action: 'approve' | 'reject' | 'snooze',
    comment?: string,
  ) {
    wsService.send(WSNotificationMessageType.RESPOND_PENDING_ACTION, {
      notification_id: notificationId,
      action,
      comment,
    })
    // 从待办列表中移除
    const index = pendingActions.value.findIndex(p => p.notification_id === notificationId)
    if (index > -1) {
      pendingActions.value.splice(index, 1)
    }
  }

  /**
   * 切换通知面板显示状态
   */
  function togglePanel() {
    isPanelOpen.value = !isPanelOpen.value
  }

  /**
   * 打开通知面板
   */
  function openPanel() {
    isPanelOpen.value = true
  }

  /**
   * 关闭通知面板
   */
  function closePanel() {
    isPanelOpen.value = false
  }

  return {
    // State
    notifications,
    pendingActions,
    loading,
    initialized,
    hasMoreOffline,
    isPanelOpen,
    // Getters
    localUnreadCount,
    pendingCount,
    totalUnreadCount,
    unreadCount,
    groupedNotifications,
    // Actions
    initialize,
    cleanup,
    fetchNotifications,
    fetchOfflineNotifications,
    loadMoreOfflineNotifications,
    markAsRead,
    markAllAsRead,
    deleteNotification,
    clearAll,
    fetchPendingActions,
    respondPendingAction,
    togglePanel,
    openPanel,
    closePanel,
  }
})
