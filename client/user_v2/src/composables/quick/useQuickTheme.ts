import { computed } from 'vue'
import { useThemeStore } from '@/stores/theme'
import type { QuickItem, QuickRuntimeItem } from '@/config/quick'

/**
 * 主题切换 Quick 组合式函数
 * @param config Quick 配置项
 * @returns Quick 运行时属性
 */
export function useQuickTheme(config: QuickItem): Partial<QuickRuntimeItem> {
  const themeStore = useThemeStore()

  /**
   * 切换主题
   * 在亮色和暗色之间直接切换
   */
  function toggleTheme() {
    themeStore.toggleLightDark()
  }

  // 当前图标根据主题状态切换
  const currentIcon = computed(() => {
    return themeStore.isDark ? (config.iconAlt || config.icon) : config.icon
  })

  return {
    isActive: themeStore.isDark,
    currentIcon: currentIcon.value,
    disabled: false,
    onClick: toggleTheme,
  }
}
