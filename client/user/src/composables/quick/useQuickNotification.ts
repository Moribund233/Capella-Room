import { markRaw, ref, watch } from 'vue'
import type { QuickItem, QuickRuntimeItem } from '@/config/quick'
import { useGlobalModal } from '@/composables/useGlobalModal'
import { useNotificationStore } from '@/stores/notification'
import NotificationPanel from '@/components/quick/NotificationPanel.vue'

/**
 * 通知中心 Quick 组合式函数
 * 对接真实的 WebSocket 通知系统
 * @param config Quick 配置项
 * @returns Quick 运行时属性
 */
export function useQuickNotification(config: QuickItem): Partial<QuickRuntimeItem> {
  const { open } = useGlobalModal()
  const notificationStore = useNotificationStore()

  /**
   * 显示通知面板弹窗
   */
  function showNotificationPanel(): void {
    // 确保通知系统已初始化
    if (!notificationStore.initialized) {
      notificationStore.initialize()
    }

    // 获取离线通知
    notificationStore.fetchOfflineNotifications()

    open({
      preset: 'card',
      size: 'medium',
      component: markRaw(NotificationPanel),
      componentProps: {
        // 只传递事件处理器，数据直接从 store 获取
        onMarkAsRead: handleMarkAsRead,
        onMarkAllAsRead: handleMarkAllAsRead,
        onDelete: handleDelete,
        onClearAll: handleClearAll,
        onLoadMore: handleLoadMore,
      },
      closable: true,
      maskClosable: true,
      onClose: () => {
        // 面板关闭时的回调
      },
    })
  }

  /**
   * 切换通知面板
   */
  function togglePanel() {
    showNotificationPanel()
  }

  /**
   * 处理标记已读
   */
  function handleMarkAsRead(id: string) {
    notificationStore.markAsRead(id)
  }

  /**
   * 处理标记全部已读
   */
  function handleMarkAllAsRead() {
    notificationStore.markAllAsRead()
  }

  /**
   * 处理删除通知
   */
  function handleDelete(id: string) {
    notificationStore.deleteNotification(id)
  }

  /**
   * 处理清空所有
   */
  function handleClearAll() {
    notificationStore.clearAll()
  }

  /**
   * 加载更多离线通知
   */
  function handleLoadMore() {
    notificationStore.loadMoreOfflineNotifications()
  }

  // 创建一个响应式的 badge ref，监听 store 的变化
  const badgeRef = ref(notificationStore.totalUnreadCount)
  watch(
    () => notificationStore.totalUnreadCount,
    (newCount) => {
      badgeRef.value = newCount
    },
    { immediate: true }
  )

  return {
    isActive: false,
    currentIcon: config.icon,
    disabled: false,
    // 使用 ref 的 value，确保响应式更新
    badge: badgeRef,
    onClick: togglePanel,
  }
}
