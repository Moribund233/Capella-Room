import { defineStore } from 'pinia'
import { ref, computed, watch } from 'vue'
import { STORAGE_KEYS } from '@/constants/storageKeys'
import { uiConfigApi, type UIConfig } from '@/api/ui-config'
import type { ThemeType } from './theme'

/**
 * 个性化配置接口
 */
export interface PersonalizationConfig {
  /** 主题设置 */
  theme: ThemeType
  /** 背景不透明度 (0-1) - 控制组件背景透明度 */
  backgroundOpacity: number
  /** 背景图片 URL */
  backgroundImage: string | null
  /** 是否启用背景图片 */
  enableBackgroundImage: boolean
  /** 强调色（主色调） */
  accentColor: string | null
  /** 是否使用自定义强调色 */
  enableCustomAccent: boolean
}

/**
 * 默认个性化配置
 */
const DEFAULT_CONFIG: PersonalizationConfig = {
  theme: 'system',
  backgroundOpacity: 1,
  backgroundImage: null,
  enableBackgroundImage: false,
  accentColor: null,
  enableCustomAccent: false,
}

/**
 * 存储键名
 */
const PERSONALIZATION_KEY = STORAGE_KEYS.THEME + '_personalization'

/**
 * 获取本地存储的个性化配置
 */
function getStoredConfig(): Partial<PersonalizationConfig> | null {
  try {
    const stored = localStorage.getItem(PERSONALIZATION_KEY)
    if (stored) {
      return JSON.parse(stored)
    }
  } catch {
    // 解析失败返回 null
  }
  return null
}

/**
 * 应用强调色到 CSS 变量
 * @param color 强调色值，null 表示移除自定义强调色
 */
function applyAccentColor(color: string | null): void {
  const root = document.documentElement

  if (color) {
    root.style.setProperty('--color-primary', color)
    root.style.setProperty('--color-primary-hover', adjustBrightness(color, 10))
    root.style.setProperty('--color-primary-pressed', adjustBrightness(color, -10))
    root.style.setProperty('--color-primary-light', hexToRgba(color, 0.15))
    root.style.setProperty('--color-primary-soft', hexToRgba(color, 0.06))
  } else {
    // 移除自定义 CSS 变量，恢复为主题默认值
    root.style.removeProperty('--color-primary')
    root.style.removeProperty('--color-primary-hover')
    root.style.removeProperty('--color-primary-pressed')
    root.style.removeProperty('--color-primary-light')
    root.style.removeProperty('--color-primary-soft')
  }
}

/**
 * 调整颜色亮度
 * @param hex 十六进制颜色
 * @param percent 调整百分比
 */
function adjustBrightness(hex: string, percent: number): string {
  const num = parseInt(hex.replace('#', ''), 16)
  const amt = Math.round(2.55 * percent)
  const R = Math.min(255, Math.max(0, (num >> 16) + amt))
  const G = Math.min(255, Math.max(0, ((num >> 8) & 0x00ff) + amt))
  const B = Math.min(255, Math.max(0, (num & 0x0000ff) + amt))
  return '#' + (0x1000000 + R * 0x10000 + G * 0x100 + B).toString(16).slice(1)
}

/**
 * 十六进制转 RGBA
 */
function hexToRgba(hex: string, alpha: number): string {
  const r = parseInt(hex.slice(1, 3), 16)
  const g = parseInt(hex.slice(3, 5), 16)
  const b = parseInt(hex.slice(5, 7), 16)
  return `rgba(${r}, ${g}, ${b}, ${alpha})`
}

/**
 * 智能透明度调整
 * 根据颜色浓度和类型动态调整透明度
 * @param baseOpacity 基础透明度 (0-1)
 * @returns 各类背景的调整系数
 */
function calculateSmartOpacity(baseOpacity: number): {
  containerOpacity: number
  elevatedOpacity: number
  lightBgOpacity: number
  darkBgOpacity: number
} {
  // 基础透明度映射到各层级
  // 当 baseOpacity = 1 时，所有层级都不透明
  // 当 baseOpacity = 0.3 时，各层级有不同的透明度

  // 容器背景（主要内容区域）- 中等透明度
  const containerOpacity = 0.3 + baseOpacity * 0.7

  //  elevated 背景（浮层、卡片）- 较高透明度保持可读性
  const elevatedOpacity = 0.5 + baseOpacity * 0.5

  // 浅色背景（success-light, warning-light 等）- 大幅降低透明度
  const lightBgOpacity = 0.2 + baseOpacity * 0.4

  // 深色背景（暗色模式下的背景）- 中等偏低透明度
  const darkBgOpacity = 0.4 + baseOpacity * 0.6

  return {
    containerOpacity: Math.min(1, containerOpacity),
    elevatedOpacity: Math.min(1, elevatedOpacity),
    lightBgOpacity: Math.min(1, lightBgOpacity),
    darkBgOpacity: Math.min(1, darkBgOpacity),
  }
}

/**
 * 应用智能透明度到 CSS 变量
 * @param opacity 基础透明度 (0-1)
 */
function applySmartTransparency(opacity: number): void {
  const root = document.documentElement
  const smartOpacity = calculateSmartOpacity(opacity)

  // 设置透明度系数 CSS 变量
  root.style.setProperty('--opacity-container', smartOpacity.containerOpacity.toFixed(2))
  root.style.setProperty('--opacity-elevated', smartOpacity.elevatedOpacity.toFixed(2))
  root.style.setProperty('--opacity-light-bg', smartOpacity.lightBgOpacity.toFixed(2))
  root.style.setProperty('--opacity-dark-bg', smartOpacity.darkBgOpacity.toFixed(2))
  root.style.setProperty('--opacity-base', opacity.toFixed(2))

  // 根据透明度调整阴影深度（透明背景需要更深的阴影来保持层次感）
  const shadowIntensity = 1 + (1 - opacity) * 0.5
  root.style.setProperty('--shadow-intensity', shadowIntensity.toFixed(2))
}

/**
 * 应用背景设置到 CSS 变量
 */
function applyBackgroundSettings(
  opacity: number,
  imageUrl: string | null,
  enableImage: boolean
): void {
  const root = document.documentElement

  // 应用智能透明度
  applySmartTransparency(opacity)

  // 应用背景图片
  if (enableImage && imageUrl) {
    root.style.setProperty('--app-background-image', `url(${imageUrl})`)
    root.style.setProperty('--app-background-size', 'cover')
    root.style.setProperty('--app-background-position', 'center')
    root.style.setProperty('--app-background-repeat', 'no-repeat')
  } else {
    root.style.removeProperty('--app-background-image')
    root.style.removeProperty('--app-background-size')
    root.style.removeProperty('--app-background-position')
    root.style.removeProperty('--app-background-repeat')
  }
}

/**
 * 将个性化配置转换为 UI 配置
 */
function toUIConfig(personalization: PersonalizationConfig): UIConfig {
  return {
    theme: {
      name: personalization.theme === 'system' ? 'auto' : personalization.theme,
    },
  }
}

/**
 * 从 UI 配置中提取个性化配置
 */
function fromUIConfig(uiConfig: UIConfig): Partial<PersonalizationConfig> {
  const result: Partial<PersonalizationConfig> = {}
  if (uiConfig.theme?.name) {
    result.theme = uiConfig.theme.name === 'auto' ? 'system' : uiConfig.theme.name
  }
  return result
}

/**
 * 个性化配置存储
 */
export const usePersonalizationStore = defineStore(
  'personalization',
  () => {
    // 从本地存储加载配置
    const stored = getStoredConfig()
    const config = ref<PersonalizationConfig>({
      ...DEFAULT_CONFIG,
      ...stored,
    })

    // 云端同步状态
    const syncing = ref(false)
    const lastSyncedAt = ref<string | null>(null)

    // 计算属性
    const theme = computed(() => config.value.theme)
    const backgroundOpacity = computed(() => config.value.backgroundOpacity)
    const backgroundImage = computed(() => config.value.backgroundImage)
    const enableBackgroundImage = computed(() => config.value.enableBackgroundImage)
    const accentColor = computed(() => config.value.accentColor)
    const enableCustomAccent = computed(() => config.value.enableCustomAccent)

    /**
     * 智能透明度计算结果
     */
    const smartOpacity = computed(() => calculateSmartOpacity(config.value.backgroundOpacity))

    /**
     * 是否有自定义背景
     */
    const hasCustomBackground = computed(
      () => config.value.enableBackgroundImage && config.value.backgroundImage
    )

    /**
     * 是否有自定义强调色
     */
    const hasCustomAccent = computed(
      () => config.value.enableCustomAccent && config.value.accentColor
    )

    /**
     * 更新配置
     */
    function updateConfig(newConfig: Partial<PersonalizationConfig>): void {
      config.value = { ...config.value, ...newConfig }
      persistConfig()
      applyConfig()
    }

    /**
     * 设置主题
     */
    function setTheme(theme: ThemeType): void {
      config.value.theme = theme
      persistConfig()
    }

    /**
     * 设置背景不透明度（智能透明度）
     */
    function setBackgroundOpacity(opacity: number): void {
      config.value.backgroundOpacity = Math.max(0.3, Math.min(1, opacity))
      persistConfig()
      applyBackgroundSettings(
        config.value.backgroundOpacity,
        config.value.backgroundImage,
        config.value.enableBackgroundImage
      )
    }

    /**
     * 设置背景图片
     */
    function setBackgroundImage(url: string | null): void {
      config.value.backgroundImage = url
      persistConfig()
      applyBackgroundSettings(
        config.value.backgroundOpacity,
        config.value.backgroundImage,
        config.value.enableBackgroundImage
      )
    }

    /**
     * 设置是否启用背景图片
     */
    function setEnableBackgroundImage(enable: boolean): void {
      config.value.enableBackgroundImage = enable
      persistConfig()
      applyBackgroundSettings(
        config.value.backgroundOpacity,
        config.value.backgroundImage,
        config.value.enableBackgroundImage
      )
    }

    /**
     * 设置强调色
     */
    function setAccentColor(color: string | null): void {
      config.value.accentColor = color
      persistConfig()
      if (config.value.enableCustomAccent) {
        applyAccentColor(color)
      }
    }

    /**
     * 设置是否启用自定义强调色
     */
    function setEnableCustomAccent(enable: boolean): void {
      config.value.enableCustomAccent = enable
      persistConfig()
      applyAccentColor(enable ? config.value.accentColor : null)
    }

    /**
     * 持久化配置到本地存储
     */
    function persistConfig(): void {
      try {
        localStorage.setItem(PERSONALIZATION_KEY, JSON.stringify(config.value))
      } catch {
        // 存储失败静默处理
      }
    }

    /**
     * 从云端同步 UI 配置
     * 加载用户保存在云端的 UI 配置并合并到本地
     */
    async function syncFromCloud(): Promise<boolean> {
      syncing.value = true
      try {
        const res = await uiConfigApi.getUIConfig()
        if (res.data) {
          const cloudConfig = fromUIConfig(res.data)
          // 合并云端配置到本地（云端优先级更高）
          if (cloudConfig.theme) {
            config.value.theme = cloudConfig.theme
          }
          persistConfig()
          lastSyncedAt.value = new Date().toISOString()
          return true
        }
        return false
      } catch (err) {
        console.warn('[Personalization] Failed to sync from cloud:', err)
        return false
      } finally {
        syncing.value = false
      }
    }

    /**
     * 同步配置到云端
     * 将本地个性化配置保存到云端
     */
    async function syncToCloud(): Promise<boolean> {
      syncing.value = true
      try {
        const uiConfig = toUIConfig(config.value)
        const res = await uiConfigApi.saveUIConfig(uiConfig)
        if (res.success) {
          lastSyncedAt.value = new Date().toISOString()
          return true
        }
        return false
      } catch (err) {
        console.warn('[Personalization] Failed to sync to cloud:', err)
        return false
      } finally {
        syncing.value = false
      }
    }

    /**
     * 重置云端 UI 配置
     */
    async function resetCloudConfig(): Promise<boolean> {
      try {
        const res = await uiConfigApi.resetUIConfig()
        return res.success
      } catch (err) {
        console.warn('[Personalization] Failed to reset cloud config:', err)
        return false
      }
    }

    /**
     * 应用配置到页面
     */
    function applyConfig(): void {
      // 应用背景设置（包含智能透明度）
      applyBackgroundSettings(
        config.value.backgroundOpacity,
        config.value.backgroundImage,
        config.value.enableBackgroundImage
      )

      // 应用强调色
      if (config.value.enableCustomAccent && config.value.accentColor) {
        applyAccentColor(config.value.accentColor)
      }
    }

    /**
     * 初始化个性化配置
     */
    function initPersonalization(): void {
      applyConfig()
    }

    /**
     * 重置为默认配置
     */
    function resetToDefault(): void {
      config.value = { ...DEFAULT_CONFIG }
      persistConfig()
      applyAccentColor(null)
      applyBackgroundSettings(1, null, false)
    }

    // 监听配置变化自动应用
    watch(
      () => config.value,
      () => {
        applyConfig()
      },
      { deep: true }
    )

    return {
      config,
      theme,
      backgroundOpacity,
      backgroundImage,
      enableBackgroundImage,
      accentColor,
      enableCustomAccent,
      smartOpacity,
      hasCustomBackground,
      hasCustomAccent,
      syncing,
      lastSyncedAt,
      updateConfig,
      setTheme,
      setBackgroundOpacity,
      setBackgroundImage,
      setEnableBackgroundImage,
      setAccentColor,
      setEnableCustomAccent,
      initPersonalization,
      resetToDefault,
      syncFromCloud,
      syncToCloud,
      resetCloudConfig,
    }
  },
  {
    persist: false,
  }
)
