import { computed, onMounted, getCurrentInstance } from 'vue'
import { darkTheme } from 'naive-ui'
import { useThemeStore } from '@/store'
import type { ThemeType } from '@/types'
import type { GlobalTheme } from 'naive-ui'

/**
 * 主题映射表
 * 将项目主题类型映射到 Naive UI 主题对象
 * 透明主题使用与标准主题相同的 Naive UI 主题
 */
const themeMap: Record<ThemeType, GlobalTheme | null> = {
  light: null, // Naive UI 默认就是亮色主题，传入 null
  dark: darkTheme, // Naive UI 暗色主题
  'light-transparent': null, // 透明亮色主题使用默认亮色
  'dark-transparent': darkTheme, // 透明暗色主题使用暗色
}

/**
 * 使用主题
 * 提供主题切换功能的组合式函数
 *
 * 实现原理：
 * 1. 直接使用 Naive UI 内置的 darkTheme 和 lightTheme
 * 2. 通过 n-config-provider 的 theme 属性传递主题对象
 * 3. 项目自身的样式通过 CSS 变量控制，与 Naive UI 主题分离
 *
 * @example
 * ```ts
 * // 在组件中使用
 * const { currentTheme, isDark, toggleTheme, setTheme, naiveTheme } = useTheme()
 *
 * // 在 App.vue 中配置
 * <n-config-provider :theme="naiveTheme">
 *   <RouterView />
 * </n-config-provider>
 * ```
 */
export function useTheme() {
  const themeStore = useThemeStore()

  /**
   * 当前主题
   */
  const currentTheme = computed(() => themeStore.currentTheme)

  /**
   * 是否为暗色主题
   */
  const isDark = computed(() => themeStore.isDark)

  /**
   * 是否为亮色主题
   */
  const isLight = computed(() => themeStore.isLight)

  /**
   * Naive UI 主题对象
   * 用于 n-config-provider 的 theme 属性
   */
  const naiveTheme = computed<GlobalTheme | null>(() => themeMap[themeStore.currentTheme] ?? null)

  /**
   * 设置主题
   * @param theme 主题类型
   */
  function setTheme(theme: ThemeType): void {
    themeStore.setTheme(theme)
  }

  /**
   * 切换主题
   * 在亮色和暗色之间切换
   */
  function toggleTheme(): void {
    themeStore.toggleTheme()
  }

  /**
   * 重置为系统主题
   */
  function resetToSystem(): void {
    themeStore.resetToSystem()
  }

  /**
   * 初始化主题
   * 在组件挂载时自动调用，加载保存的主题
   * 仅在组件上下文中执行，避免在非组件环境调用
   */
  if (getCurrentInstance()) {
    onMounted(() => {
      themeStore.initTheme()
    })
  }

  return {
    currentTheme,
    isDark,
    isLight,
    naiveTheme,
    setTheme,
    toggleTheme,
    resetToSystem,
  }
}
