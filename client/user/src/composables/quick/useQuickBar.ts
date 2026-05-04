import { ref, computed } from 'vue'
import type { QuickItem, QuickRuntimeItem } from '@/config/quick'

// 导入各个 Quick 处理函数
import { useQuickTheme } from './useQuickTheme'
import { useQuickNotification } from './useQuickNotification'

/**
 * Quick 处理函数映射表
 */
const quickHandlers: Record<string, (item: QuickItem) => Partial<QuickRuntimeItem>> = {
  useQuickTheme,
  useQuickNotification,
}

/**
 * 注册 Quick 处理函数
 * @param name 处理函数名称
 * @param handler 处理函数
 */
export function registerQuickHandler(
  name: string,
  handler: (item: QuickItem) => Partial<QuickRuntimeItem>
) {
  quickHandlers[name] = handler
}

/**
 * QuickBar 核心组合式函数
 * @param config Quick 配置项列表
 */
export function useQuickBar(config: QuickItem[]) {
  const items = ref<QuickRuntimeItem[]>([])

  /**
   * 初始化 Quick 项
   */
  function initItems() {
    items.value = config.map((item) => {
      const handler = quickHandlers[item.handler]
      if (!handler) {
        console.warn(`[QuickBar] Handler not found: ${item.handler}`)
        return createDefaultItem(item)
      }

      const runtimeProps = handler(item)
      return {
        ...item,
        isActive: false,
        currentIcon: item.icon,
        disabled: false,
        onClick: () => {},
        ...runtimeProps,
      } as QuickRuntimeItem
    })
  }

  /**
   * 创建默认项（当 handler 不存在时）
   */
  function createDefaultItem(item: QuickItem): QuickRuntimeItem {
    return {
      ...item,
      isActive: false,
      currentIcon: item.icon,
      disabled: false,
      onClick: () => {
        console.warn(`[QuickBar] No handler for: ${item.key}`)
      },
    }
  }

  /**
   * 获取外显的项
   */
  const visibleItems = computed(() =>
    items.value.filter((item) => item.display === 'visible')
  )

  /**
   * 获取下拉菜单的项
   */
  const dropdownItems = computed(() =>
    items.value.filter((item) => item.display === 'dropdown')
  )

  /**
   * 更新项的徽标
   * @param key 项的 key
   * @param badge 徽标数字
   */
  function updateBadge(key: string, badge: number) {
    const item = items.value.find((i) => i.key === key)
    if (item) {
      item.badge = badge
    }
  }

  /**
   * 更新项的激活状态
   * @param key 项的 key
   * @param isActive 是否激活
   */
  function updateActive(key: string, isActive: boolean) {
    const item = items.value.find((i) => i.key === key)
    if (item) {
      item.isActive = isActive
    }
  }

  // 初始化
  initItems()

  return {
    items,
    visibleItems,
    dropdownItems,
    updateBadge,
    updateActive,
  }
}
