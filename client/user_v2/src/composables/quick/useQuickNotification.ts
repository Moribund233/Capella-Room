import { markRaw, ref, watch } from 'vue'
import type { QuickItem, QuickRuntimeItem } from '@/config/quick'
import { useGlobalModal } from '@/composables/useGlobalModal'
import { useNotificationStore } from '@/stores/notification'
import NotificationContent from '@/components/notification/NotificationContent.vue'

/**
 * 通知中心 Quick 组合式函数
 * 对接真实的 WebSocket 通知系统
 */
export function useQuickNotification(config: QuickItem): Partial<QuickRuntimeItem> {
  const { open } = useGlobalModal()
  const notificationStore = useNotificationStore()

  function showNotificationPanel(): void {
    if (!notificationStore.initialized) {
      notificationStore.initialize()
    }
    notificationStore.fetchOfflineNotifications()

    open({
      preset: 'card',
      component: markRaw(NotificationContent),
      closable: true,
      maskClosable: true,
    })
  }

  function togglePanel() {
    showNotificationPanel()
  }

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
    badge: badgeRef,
    onClick: togglePanel,
  }
}
