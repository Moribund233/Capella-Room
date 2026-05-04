import { computed } from 'vue'
import { useLayoutStore } from '@/store/layout'
import { useStatusBar } from '@/composables/useStatusBar'
import type { QuickConfigItem, QuickContext, UseQuickReturn } from './types'

/**
 * Layout 控制 Quick 组合式函数（通用版）
 *
 * 支持通过 layoutKey 自动绑定 LayoutStore：
 * - sidebar: 切换侧边栏
 * - footer: 切换 Footer
 *
 * @param config Quick 配置项
 * @param context Quick 上下文
 * @returns Quick 运行时接口
 *
 * @example
 * // ui.ts 配置 - 侧边栏
 * {
 *   key: 'sidebar',
 *   display: 'visible',
 *   type: 'action',
 *   icon: 'PanelLeft',
 *   iconAlt: 'PanelRight',
 *   label: '切换侧边栏',
 * }
 *
 * // ui.ts 配置 - Footer
 * {
 *   key: 'footer',
 *   display: 'visible',
 *   type: 'action',
 *   icon: 'PanelBottomClose',
 *   iconAlt: 'PanelBottomOpen',
 *   label: '切换Footer',
 * }
 */
export function useQuickLayout(config: QuickConfigItem, context: QuickContext): UseQuickReturn {
  const layoutStore = useLayoutStore()
  const { hasContent } = useStatusBar()

  /**
   * 判断是否为 Footer 模式
   * 根据 key 或 icon 名称判断
   */
  const isFooterMode = computed(() => {
    return (
      config.key === 'footer' ||
      config.layoutKey === 'footer' ||
      config.icon.toLowerCase().includes('bottom') ||
      (config.iconAlt?.toLowerCase().includes('bottom') ?? false)
    )
  })

  /**
   * 获取当前状态
   */
  const isActive = computed(() => {
    // 优先使用 layoutKey，否则根据模式判断
    const stateKey = config.layoutKey || (isFooterMode.value ? 'footer' : 'sidebar')
    return layoutStore.getState(stateKey)
  })

  /**
   * 当前图标
   * 根据状态动态切换
   * - Footer 模式：显示时 icon，隐藏时 iconAlt
   * - 侧边栏模式：打开时 iconAlt，关闭时 icon
   */
  const currentIcon = computed(() => {
    if (isFooterMode.value) {
      // Footer 模式：显示时 icon，隐藏时 iconAlt
      return isActive.value ? config.icon : config.iconAlt || config.icon
    }
    // 侧边栏模式：打开时 iconAlt，关闭时 icon
    return isActive.value ? config.iconAlt || config.icon : config.icon
  })

  /**
   * 是否禁用
   * Footer 按钮：当状态栏无内容且 Footer 未显示时禁用
   * （如果 Footer 已经显示，允许用户关闭它）
   */
  const disabled = computed(() => {
    if (isFooterMode.value) {
      return !hasContent.value && !layoutStore.isFooterVisible
    }
    return false
  })

  /**
   * 点击处理
   */
  function onClick(): void {
    // 禁用时不可点击
    if (disabled.value) return

    const stateKey = config.layoutKey || (isFooterMode.value ? 'footer' : 'sidebar')

    // 检查是否是已知的 layout key
    if (['sidebar', 'footer'].includes(stateKey)) {
      layoutStore.toggle(stateKey)
    } else {
      // 未知 key，通过 action 事件交由外部处理
      context.emitAction(config.key)
    }
  }

  /**
   * 子菜单选择处理
   */
  function onSelect(childKey: string): void {
    context.emitAction(config.key, childKey)
  }

  return {
    isActive,
    currentIcon,
    disabled,
    onClick,
    onSelect: config.type === 'menu' ? onSelect : undefined,
  }
}
