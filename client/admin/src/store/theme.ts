import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { ThemeType } from '@/types'

/**
 * 主题存储键名
 */
const THEME_KEY = 'app_theme'

/**
 * 主题样式链接 ID
 */
const THEME_LINK_ID = 'theme-style-link'

/**
 * 获取本地存储的主题
 * @returns 主题类型或 null
 */
function getStoredTheme(): ThemeType | null {
  const stored = localStorage.getItem(THEME_KEY)
  if (stored === 'light' || stored === 'dark') {
    return stored
  }
  return null
}

/**
 * 获取系统偏好主题
 * @returns 主题类型
 */
function getSystemTheme(): ThemeType {
  if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {
    return 'dark'
  }
  return 'light'
}

/**
 * 创建或更新主题样式链接
 * @param theme 主题类型
 */
function updateThemeLink(theme: ThemeType): void {
  let link = document.getElementById(THEME_LINK_ID) as HTMLLinkElement | null

  if (!link) {
    link = document.createElement('link')
    link.id = THEME_LINK_ID
    link.rel = 'stylesheet'
    document.head.appendChild(link)
  }

  link.href = `/Themes/${theme}.css`
}

/**
 * 移除主题样式链接
 */
function removeThemeLink(): void {
  const link = document.getElementById(THEME_LINK_ID)
  if (link) {
    link.remove()
  }
}

/**
 * 主题状态存储
 */
export const useThemeStore = defineStore('theme', () => {
  /**
   * 当前主题
   */
  const currentTheme = ref<ThemeType>(getStoredTheme() || getSystemTheme())

  /**
   * 是否为暗色主题
   */
  const isDark = computed(() => currentTheme.value === 'dark')

  /**
   * 是否为亮色主题
   */
  const isLight = computed(() => currentTheme.value === 'light')

  /**
   * 初始化主题
   * 应用启动时调用，加载保存的主题或系统主题
   */
  function initTheme(): void {
    const stored = getStoredTheme()
    const theme = stored || getSystemTheme()
    setTheme(theme, false)
  }

  /**
   * 设置主题
   * @param theme 主题类型
   * @param persist 是否持久化到本地存储，默认为 true
   */
  function setTheme(theme: ThemeType, persist: boolean = true): void {
    currentTheme.value = theme

    // 更新 HTML data-theme 属性
    document.documentElement.setAttribute('data-theme', theme)

    // 更新主题样式链接（完整卸载旧主题，加载新主题）
    updateThemeLink(theme)

    // 持久化到本地存储
    if (persist) {
      localStorage.setItem(THEME_KEY, theme)
    }
  }

  /**
   * 切换主题
   * 在亮色和暗色之间切换
   */
  function toggleTheme(): void {
    const newTheme = currentTheme.value === 'light' ? 'dark' : 'light'
    setTheme(newTheme)
  }

  /**
   * 重置为系统主题
   * 清除本地存储的主题设置
   */
  function resetToSystem(): void {
    localStorage.removeItem(THEME_KEY)
    const systemTheme = getSystemTheme()
    setTheme(systemTheme, false)
  }

  /**
   * 卸载主题
   * 移除主题样式链接和 data-theme 属性
   */
  function unloadTheme(): void {
    removeThemeLink()
    document.documentElement.removeAttribute('data-theme')
  }

  return {
    currentTheme,
    isDark,
    isLight,
    initTheme,
    setTheme,
    toggleTheme,
    resetToSystem,
    unloadTheme,
  }
})
