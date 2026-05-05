import { computed } from 'vue'
import { useThemeStore, type ThemeType } from '@/stores/theme'

/**
 * 使用主题
 * 提供主题切换功能的组合式函数
 *
 * @example
 * ```ts
 * // 在组件中使用
 * const { effectiveTheme, isDark, isLight, toggleTheme, setTheme } = useTheme()
 *
 * // 在模板中使用
 * <div :class="{ 'dark-mode': isDark }">
 *   <button @click="toggleTheme">切换主题</button>
 * </div>
 * ```
 */
export function useTheme() {
  const themeStore = useThemeStore()

  /**
   * 当前主题设置（可能为 system）
   */
  const themeSetting = computed(() => themeStore.themeSetting)

  /**
   * 实际应用的主题（解析 system 后的结果）
   */
  const effectiveTheme = computed(() => themeStore.effectiveTheme)

  /**
   * 是否为暗色主题
   */
  const isDark = computed(() => themeStore.isDark)

  /**
   * 是否为亮色主题
   */
  const isLight = computed(() => themeStore.isLight)

  /**
   * 是否跟随系统
   */
  const isSystem = computed(() => themeStore.isSystem)

  /**
   * 设置主题
   * @param theme 主题类型
   */
  function setTheme(theme: ThemeType): void {
    themeStore.setTheme(theme)
  }

  /**
   * 切换主题
   * 按 light -> dark -> system -> light 循环切换
   */
  function toggleTheme(): void {
    themeStore.toggleTheme()
  }

  /**
   * 在亮暗之间直接切换（用于快速切换）
   */
  function toggleLightDark(): void {
    themeStore.toggleLightDark()
  }

  /**
   * 重置为系统主题
   */
  function resetToSystem(): void {
    themeStore.resetToSystem()
  }

  /**
   * 初始化主题
   * 在应用启动时调用
   */
  function initTheme(): void {
    themeStore.initTheme()
    themeStore.watchSystemThemeChange()
  }

  return {
    themeSetting,
    effectiveTheme,
    isDark,
    isLight,
    isSystem,
    setTheme,
    toggleTheme,
    toggleLightDark,
    resetToSystem,
    initTheme,
  }
}

export type { ThemeType }
