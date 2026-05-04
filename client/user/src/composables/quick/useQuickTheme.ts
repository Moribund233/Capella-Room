import { ref, computed } from 'vue'
import type { QuickItem, QuickRuntimeItem } from '@/config/quick'

/**
 * 主题切换 Quick 组合式函数
 * @param config Quick 配置项
 * @returns Quick 运行时属性
 */
export function useQuickTheme(config: QuickItem): Partial<QuickRuntimeItem> {
  // 当前主题模式
  const isDark = ref(false)

  // 从 localStorage 读取主题设置
  if (typeof window !== 'undefined') {
    const savedTheme = localStorage.getItem('theme')
    isDark.value = savedTheme === 'dark' ||
      (!savedTheme && window.matchMedia('(prefers-color-scheme: dark)').matches)

    // 初始化时应用主题
    applyTheme(isDark.value)
  }

  /**
   * 应用主题
   */
  function applyTheme(dark: boolean) {
    if (typeof document === 'undefined') return

    const html = document.documentElement
    if (dark) {
      html.classList.add('dark')
    } else {
      html.classList.remove('dark')
    }
  }

  /**
   * 切换主题
   */
  function toggleTheme() {
    isDark.value = !isDark.value
    applyTheme(isDark.value)

    // 保存到 localStorage
    if (typeof window !== 'undefined') {
      localStorage.setItem('theme', isDark.value ? 'dark' : 'light')
    }
  }

  // 当前图标根据主题状态切换
  const currentIcon = computed(() => {
    return isDark.value ? (config.iconAlt || config.icon) : config.icon
  })

  return {
    isActive: isDark.value,
    currentIcon: currentIcon.value,
    disabled: false,
    onClick: toggleTheme,
  }
}
