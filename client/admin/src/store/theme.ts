import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { ThemeType } from '@/types'

/**
 * 主题存储键名
 */
const THEME_KEY = 'app_theme'

/**
 * 强调色存储键名
 */
const ACCENT_COLOR_KEY = 'app_accent_color'

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
 * 获取本地存储的强调色
 * @returns 强调色值或 null
 */
function getStoredAccentColor(): string | null {
  return localStorage.getItem(ACCENT_COLOR_KEY)
}

/**
 * 生成强调色的衍生色
 * @param baseColor 基础色（hex 格式）
 * @returns 包含各种强调色变体的对象
 */
function generateAccentColors(baseColor: string): {
  primary: string
  primaryHover: string
  primaryLight: string
  primaryDark: string
} {
  // 简单的颜色变体生成（实际项目中可能需要更复杂的颜色处理库）
  return {
    primary: baseColor,
    primaryHover: baseColor,
    primaryLight: baseColor + '20', // 添加透明度
    primaryDark: baseColor,
  }
}

/**
 * 应用强调色到 CSS 变量
 * @param color 强调色值，null 表示移除自定义强调色
 */
function applyAccentColor(color: string | null): void {
  const root = document.documentElement

  if (color) {
    const colors = generateAccentColors(color)
    root.style.setProperty('--color-primary', colors.primary)
    root.style.setProperty('--color-primary-hover', colors.primaryHover)
    root.style.setProperty('--color-primary-light', colors.primaryLight)
    root.style.setProperty('--color-primary-dark', colors.primaryDark)
  } else {
    // 移除自定义 CSS 变量，恢复为主题默认值
    root.style.removeProperty('--color-primary')
    root.style.removeProperty('--color-primary-hover')
    root.style.removeProperty('--color-primary-light')
    root.style.removeProperty('--color-primary-dark')
  }
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

  link.href = `${import.meta.env.BASE_URL}Themes/${theme}.css`
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
   * 自定义强调色
   */
  const accentColor = ref<string | null>(getStoredAccentColor())

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

    // 应用保存的强调色
    if (accentColor.value) {
      applyAccentColor(accentColor.value)
    }
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

  /**
   * 设置强调色
   * @param color 强调色值（hex 格式），null 表示使用主题默认色
   * @param persist 是否持久化到本地存储，默认为 true
   */
  function setAccentColor(color: string | null, persist: boolean = true): void {
    accentColor.value = color
    applyAccentColor(color)

    if (persist) {
      if (color) {
        localStorage.setItem(ACCENT_COLOR_KEY, color)
      } else {
        localStorage.removeItem(ACCENT_COLOR_KEY)
      }
    }
  }

  /**
   * 重置强调色为默认值
   * 清除本地存储的自定义强调色
   */
  function resetAccentColor(): void {
    setAccentColor(null)
  }

  return {
    currentTheme,
    accentColor,
    isDark,
    isLight,
    initTheme,
    setTheme,
    setAccentColor,
    resetAccentColor,
    toggleTheme,
    resetToSystem,
    unloadTheme,
  }
})
