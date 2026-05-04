import { ref, computed, markRaw } from 'vue'
import type { QuickItem, QuickRuntimeItem } from '@/config/quick'
import { useGlobalModal } from '@/composables/useGlobalModal'
import NotificationPanel from '@/components/quick/NotificationPanel.vue'

/**
 * 通知项接口
 */
interface Notification {
  id: string
  title: string
  content: string
  type: 'info' | 'success' | 'warning' | 'error'
  isRead: boolean
  createdAt: string
}

/**
 * 通知中心 Quick 组合式函数
 * @param config Quick 配置项
 * @returns Quick 运行时属性
 */
export function useQuickNotification(config: QuickItem): Partial<QuickRuntimeItem> {
  const { open, close } = useGlobalModal()

  // 未读消息数量
  const unreadCount = ref(config.badge || 0)

  // 是否显示通知面板
  const isPanelOpen = ref(false)

  // 加载状态
  const loading = ref(false)

  // 通知列表（模拟数据，实际应从API获取）
  const notifications = ref<Notification[]>([
    {
      id: '1',
      title: '系统维护通知',
      content: '系统将于今晚 02:00-04:00 进行例行维护，期间服务可能暂时不可用。',
      type: 'info',
      isRead: false,
      createdAt: new Date(Date.now() - 1000 * 60 * 30).toISOString(), // 30分钟前
    },
    {
      id: '2',
      title: '房间创建成功',
      content: '您创建的 "周末游戏局" 房间已成功创建，快邀请好友加入吧！',
      type: 'success',
      isRead: false,
      createdAt: new Date(Date.now() - 1000 * 60 * 60 * 2).toISOString(), // 2小时前
    },
    {
      id: '3',
      title: '新消息提醒',
      content: '张三在 "技术交流" 房间中提到了你。',
      type: 'warning',
      isRead: true,
      createdAt: new Date(Date.now() - 1000 * 60 * 60 * 24).toISOString(), // 1天前
    },
  ])

  // 计算未读数量
  function updateUnreadCount() {
    unreadCount.value = notifications.value.filter(n => !n.isRead).length
  }

  /**
   * 显示通知面板弹窗
   */
  function showNotificationPanel(): void {
    isPanelOpen.value = true
    open({
      preset: 'card',
      size: 'medium',
      component: markRaw(NotificationPanel),
      componentProps: {
        notifications: notifications.value,
        loading: loading.value,
        onMarkAsRead: handleMarkAsRead,
        onMarkAllAsRead: handleMarkAllAsRead,
        onDelete: handleDelete,
        onClearAll: handleClearAll,
        onOpenSettings: handleOpenSettings,
      },
      closable: true,
      maskClosable: true,
      onClose: () => {
        isPanelOpen.value = false
      },
    })
  }

  /**
   * 切换通知面板
   */
  function togglePanel() {
    if (isPanelOpen.value) {
      close()
      isPanelOpen.value = false
    } else {
      showNotificationPanel()
    }
  }

  /**
   * 处理标记已读
   */
  function handleMarkAsRead(id: string) {
    const notification = notifications.value.find(n => n.id === id)
    if (notification) {
      notification.isRead = true
      updateUnreadCount()
    }
  }

  /**
   * 处理标记全部已读
   */
  function handleMarkAllAsRead() {
    notifications.value.forEach(n => n.isRead = true)
    unreadCount.value = 0
  }

  /**
   * 处理删除通知
   */
  function handleDelete(id: string) {
    const index = notifications.value.findIndex(n => n.id === id)
    if (index > -1) {
      notifications.value.splice(index, 1)
      updateUnreadCount()
    }
  }

  /**
   * 处理清空所有
   */
  function handleClearAll() {
    notifications.value = []
    unreadCount.value = 0
  }

  /**
   * 处理打开设置
   */
  function handleOpenSettings() {
    // 可以打开通知设置弹窗
    console.log('打开通知设置')
  }

  /**
   * 添加新通知（供外部调用）
   */
  function addNotification(notification: Omit<Notification, 'id' | 'createdAt'>) {
    const newNotification: Notification = {
      ...notification,
      id: Date.now().toString(),
      createdAt: new Date().toISOString(),
    }
    notifications.value.unshift(newNotification)
    updateUnreadCount()
  }

  return {
    isActive: isPanelOpen.value,
    currentIcon: config.icon,
    disabled: false,
    badge: unreadCount.value,
    onClick: togglePanel,
    // 暴露额外方法供外部使用
    addNotification,
    notifications,
    unreadCount: computed(() => unreadCount.value),
  }
}
