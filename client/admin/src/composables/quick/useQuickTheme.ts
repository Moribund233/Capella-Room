import { computed } from 'vue'
import { useTheme } from '@/composables'
import type { QuickConfigItem, QuickContext, UseQuickReturn } from './types'

/**
 * 主题切换 Quick 组合式函数
 *
 * @param config Quick 配置项
 * @param context Quick 上下文
 * @returns Quick 运行时接口
 *
 * @example
 * // ui.ts 配置
 * {
 *   key: 'theme',
 *   display: 'dropdown',
 *   type: 'menu',
 *   icon: 'Sun',
 *   iconAlt: 'Moon',
 *   label: '主题',
 *   children: [
 *     { key: 'light', label: '浅色模式', icon: 'Sun' },
 *     { key: 'dark', label: '夜间模式', icon: 'Moon' }
 *   ]
 * }
 */
export function useQuickTheme(config: QuickConfigItem, context: QuickContext): UseQuickReturn {
  const { isDark, setTheme } = useTheme()

  /**
   * 当前图标
   * 根据当前主题状态动态切换图标
   */
  const currentIcon = computed(() => (isDark.value ? config.iconAlt || config.icon : config.icon))

  /**
   * 点击主按钮处理函数
   * 当 type 为 action 时直接切换主题
   */
  function onClick(): void {
    if (config.type === 'action') {
      // action 类型：直接切换主题
      const newTheme = isDark.value ? 'light' : 'dark'
      setTheme(newTheme)
    }
    // menu 类型：不处理，由下拉菜单触发 onSelect
  }

  /**
   * 选择子菜单项处理函数
   * 当 type 为 menu 时通过子菜单选择主题
   */
  function onSelect(childKey: string): void {
    if (childKey === 'light') {
      setTheme('light')
    } else if (childKey === 'dark') {
      setTheme('dark')
    } else {
      // 其他 key 通过 action 事件交由外部处理
      context.emitAction(config.key, childKey)
    }
  }

  return {
    isActive: isDark,
    currentIcon,
    onClick,
    onSelect: config.type === 'menu' ? onSelect : undefined,
  }
}
