import { computed} from 'vue'
import { useGlobalModal } from '@/composables/useGlobalModal'
import { useNotificationStore } from '@/store'
import NotificationPanel from '@/components/quick/NotificationPanel.vue'
import type { QuickConfigItem, QuickContext, UseQuickReturn } from './types'

/**
 * 通知中心 Quick 组合式函数
 *
 * 功能：
 * - 显示未读通知数量徽标
 * - 点击打开通知弹窗面板
 * - 支持标记所有通知为已读
 * - 集成 WebSocket 实时通知
 *
 * @param config Quick 配置项
 * @param context Quick 上下文
 * @returns Quick 运行时接口
 */
export function useQuickNotification(
  config: QuickConfigItem,
  context: QuickContext,
): UseQuickReturn {
  const { open } = useGlobalModal()
  const notificationStore = useNotificationStore()

  // 是否有未读（用于徽标显示）
  const isActive = computed(() => notificationStore.hasUnread)

  /**
   * 显示通知面板弹窗
   */
  function showNotificationPanel(): void {
    open({
      title: '通知中心',
      component: NotificationPanel,
      componentProps: {},
      preset: 'card',
      width: 520,
      closable: true,
      maskClosable: true,
    })
  }

  /**
   * 点击处理
   */
  function onClick(): void {
    showNotificationPanel()
  }

  /**
   * 子菜单选择
   */
  function onSelect(childKey: string): void {
    if (childKey === 'mark-all-read') {
      notificationStore.markAllAsRead()
      context.emitAction(config.key, childKey)
    } else {
      context.emitAction(config.key, childKey)
    }
  }

  return {
    isActive,
    currentIcon: computed(() => config.icon),
    onClick,
    onSelect: config.type === 'menu' ? onSelect : undefined,
  }
}
