import { ref, computed } from 'vue'
import { defineStore } from 'pinia'

/**
 * 主题状态管理
 * 管理应用的亮色/暗色主题切换
 */

export type Theme = 'light' | 'dark'

export const useThemeStore = defineStore('theme', () => {
  /** 当前主题 */
  const theme = ref<Theme>('light')

  /** 是否暗色主题 */
  const isDark = computed(() => theme.value === 'dark')

  /**
   * 初始化主题
   * 从localStorage或系统偏好读取
   */
  const initTheme = () => {
    const savedTheme = localStorage.getItem('theme') as Theme | null
    if (savedTheme && (savedTheme === 'light' || savedTheme === 'dark')) {
      theme.value = savedTheme
    } else if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
      theme.value = 'dark'
    } else {
      theme.value = 'light'
    }
    applyTheme()
  }

  /**
   * 应用主题到DOM
   * 动态加载对应主题的CSS文件
   */
  const applyTheme = () => {
    document.documentElement.setAttribute('data-theme', theme.value)

    // 动态加载主题CSS文件
    const themeId = 'theme-style'
    let themeLink = document.getElementById(themeId) as HTMLLinkElement | null

    if (!themeLink) {
      themeLink = document.createElement('link')
      themeLink.id = themeId
      themeLink.rel = 'stylesheet'
      document.head.appendChild(themeLink)
    }

    themeLink.href = `/Themes/${theme.value}.css?v=${Date.now()}`
  }

  /**
   * 切换主题
   */
  const toggleTheme = () => {
    theme.value = theme.value === 'light' ? 'dark' : 'light'
    applyTheme()
    localStorage.setItem('theme', theme.value)
  }

  /**
   * 设置指定主题
   * @param newTheme 目标主题
   */
  const setTheme = (newTheme: Theme) => {
    theme.value = newTheme
    applyTheme()
    localStorage.setItem('theme', newTheme)
  }

  return {
    theme,
    isDark,
    initTheme,
    toggleTheme,
    setTheme,
  }
})
