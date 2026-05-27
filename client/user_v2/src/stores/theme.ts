import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { STORAGE_KEYS } from '@/constants/storageKeys'

/**
 * 主题类型
 */
export type ThemeType = 'light' | 'dark' | 'system'

/**
 * 获取本地存储的主题
 * @returns 主题类型或 null
 */
function getStoredTheme(): ThemeType | null {
  const stored = localStorage.getItem(STORAGE_KEYS.THEME)
  if (stored === 'light' || stored === 'dark' || stored === 'system') {
    return stored
  }
  return null
}

/**
 * 获取系统偏好主题
 * @returns 主题类型
 */
function getSystemTheme(): 'light' | 'dark' {
  if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {
    return 'dark'
  }
  return 'light'
}

/**
 * 应用主题到文档
 * @param theme 主题类型
 */
function applyThemeToDocument(theme: ThemeType): void {
  const effectiveTheme = theme === 'system' ? getSystemTheme() : theme
  document.documentElement.setAttribute('data-theme', effectiveTheme)
}

/**
 * 主题状态存储
 */
export const useThemeStore = defineStore(
  'theme',
  () => {
    /**
     * 当前主题设置（可能为 system）
     */
    const themeSetting = ref<ThemeType>(getStoredTheme() || 'system')

    /**
     * 实际应用的主题（解析 system 后的结果）
     */
    const effectiveTheme = computed<'light' | 'dark'>(() => {
      if (themeSetting.value === 'system') {
        return getSystemTheme()
      }
      return themeSetting.value
    })

    /**
     * 是否为暗色主题
     */
    const isDark = computed(() => effectiveTheme.value === 'dark')

    /**
     * 是否为亮色主题
     */
    const isLight = computed(() => effectiveTheme.value === 'light')

    /**
     * 是否跟随系统
     */
    const isSystem = computed(() => themeSetting.value === 'system')

    /**
     * 初始化主题
     * 应用启动时调用，加载保存的主题或系统主题
     */
    function initTheme(): void {
      const stored = getStoredTheme()
      const theme = stored || 'system'
      setTheme(theme, false)
    }

    /**
     * 设置主题
     * @param theme 主题类型
     * @param persist 是否持久化到本地存储，默认为 true
     */
    function setTheme(theme: ThemeType, persist: boolean = true): void {
      themeSetting.value = theme
      applyThemeToDocument(theme)

      if (persist) {
        localStorage.setItem(STORAGE_KEYS.THEME, theme)
      }
    }

    /**
     * 切换主题
     * 按 light -> dark -> system -> light 循环切换
     */
    function toggleTheme(): void {
      const themeMap: Record<ThemeType, ThemeType> = {
        light: 'dark',
        dark: 'system',
        system: 'light',
      }
      const newTheme = themeMap[themeSetting.value]
      setTheme(newTheme)
    }

    /**
     * 在亮暗之间直接切换（用于快速切换）
     */
    function toggleLightDark(): void {
      const newTheme = effectiveTheme.value === 'light' ? 'dark' : 'light'
      setTheme(newTheme)
    }

    /**
     * 重置为系统主题
     * 清除本地存储的主题设置
     */
    function resetToSystem(): void {
      localStorage.removeItem(STORAGE_KEYS.THEME)
      setTheme('system', false)
    }

    /**
     * 监听系统主题变化
     */
    function watchSystemThemeChange(): void {
      if (window.matchMedia) {
        const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
        mediaQuery.addEventListener('change', () => {
          if (themeSetting.value === 'system') {
            applyThemeToDocument('system')
          }
        })
      }
    }

    return {
      themeSetting,
      effectiveTheme,
      isDark,
      isLight,
      isSystem,
      initTheme,
      setTheme,
      toggleTheme,
      toggleLightDark,
      resetToSystem,
      watchSystemThemeChange,
    }
  },
  {
    persist: false,
  }
)
