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
 * 有效的主题类型
 */
const VALID_THEMES: ThemeType[] = ['light', 'dark', 'light-transparent', 'dark-transparent']

/**
 * 获取本地存储的主题
 * @returns 主题类型或 null
 */
function getStoredTheme(): ThemeType | null {
  const stored = localStorage.getItem(THEME_KEY)
  if (stored && VALID_THEMES.includes(stored as ThemeType)) {
    return stored as ThemeType
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
 * 获取本地存储的强调色
 * @returns 强调色或 null
 */
function getStoredAccentColor(): string | null {
  return localStorage.getItem(ACCENT_COLOR_KEY)
}

/**
 * 生成强调色的相关色值
 * @param hex 基础色值
 * @returns 包含主色、悬停色、激活色、浅色背景的对象
 */
function generateAccentColors(hex: string): {
  primary: string
  hover: string
  active: string
  light: string
  gradient: string
} {
  // 简单的颜色处理函数
  const adjustBrightness = (color: string, amount: number): string => {
    const num = parseInt(color.replace('#', ''), 16)
    const r = Math.min(255, Math.max(0, (num >> 16) + amount))
    const g = Math.min(255, Math.max(0, ((num >> 8) & 0x00ff) + amount))
    const b = Math.min(255, Math.max(0, (num & 0x0000ff) + amount))
    return `#${((r << 16) | (g << 8) | b).toString(16).padStart(6, '0')}`
  }

  const primary = hex
  const hover = adjustBrightness(hex, 40)
  const active = adjustBrightness(hex, -20)
  const light = `${hex}14` // 约 8% 透明度
  const gradient = `linear-gradient(135deg, ${hex} 0%, ${adjustBrightness(hex, 20)} 100%)`

  return { primary, hover, active, light, gradient }
}

/**
 * 应用强调色到 CSS 变量
 * @param color 强调色值
 */
function applyAccentColor(color: string | null): void {
  if (!color) {
    // 移除自定义强调色
    document.documentElement.style.removeProperty('--color-primary')
    document.documentElement.style.removeProperty('--color-primary-hover')
    document.documentElement.style.removeProperty('--color-primary-active')
    document.documentElement.style.removeProperty('--color-primary-light')
    document.documentElement.style.removeProperty('--color-primary-gradient')
    document.documentElement.style.removeProperty('--sidebar-text-active')
    document.documentElement.style.removeProperty('--sidebar-bg-hover')
    document.documentElement.style.removeProperty('--sidebar-bg-active')
    return
  }

  const colors = generateAccentColors(color)

  // 应用到 CSS 变量
  document.documentElement.style.setProperty('--color-primary', colors.primary)
  document.documentElement.style.setProperty('--color-primary-hover', colors.hover)
  document.documentElement.style.setProperty('--color-primary-active', colors.active)
  document.documentElement.style.setProperty('--color-primary-light', colors.light)
  document.documentElement.style.setProperty('--color-primary-gradient', colors.gradient)
  document.documentElement.style.setProperty('--sidebar-text-active', colors.primary)
  document.documentElement.style.setProperty('--sidebar-bg-hover', `${colors.primary}10`)
  document.documentElement.style.setProperty('--sidebar-bg-active', `${colors.primary}1A`)
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
   * 是否为暗色主题（包括暗色透明主题）
   */
  const isDark = computed(() => currentTheme.value === 'dark' || currentTheme.value === 'dark-transparent')

  /**
   * 是否为亮色主题（包括亮色透明主题）
   */
  const isLight = computed(() => currentTheme.value === 'light' || currentTheme.value === 'light-transparent')

  /**
   * 初始化主题
   * 应用启动时调用，加载保存的主题或系统主题
   */
  function initTheme(): void {
    const stored = getStoredTheme()
    const theme = stored || getSystemTheme()
    setTheme(theme, false)
    // 应用保存的强调色
    applyAccentColor(accentColor.value)
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

    // 重新应用强调色（覆盖主题默认色）
    applyAccentColor(accentColor.value)

    // 持久化到本地存储
    if (persist) {
      localStorage.setItem(THEME_KEY, theme)
    }
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
   * 重置强调色为默认
   */
  function resetAccentColor(): void {
    setAccentColor(null)
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
    applyAccentColor(null)
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
