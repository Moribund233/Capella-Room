import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { uiConfig as defaultUiConfig } from '@/config/ui'
import { uiApi } from '@/api'
import type {
  UIConfig,
  AppConfig,
  SidebarItemConfig,
  QuickItemConfig,
  ThemeConfig,
  DockConfig,
} from '@/types'

/**
 * 是否为开发环境
 * 开发环境：仅使用本地配置，不连接云端
 * 生产环境：支持云端配置同步
 */
const isDev = import.meta.env.VITE_APP_ENV === 'development'

/**
 * UI 配置存储
 *
 * 管理用户界面偏好设置，包括：
 * - 应用信息（名称、Logo）
 * - 主题设置
 * - 侧边栏菜单顺序和图标
 * - QuickBar 配置
 * - DockBar 配置
 *
 * 数据来源优先级：
 * 1. 本地存储（localStorage）- 当前用户偏好
 * 2. 后端云端数据（预留）- 多设备同步
 * 3. 默认配置（ui.ts）- 初始值
 */
export const useUIStore = defineStore('ui', () => {
  // ========== 状态定义 ==========

  /**
   * 本地配置覆盖（用户偏好）
   * 仅存储与默认配置不同的部分
   */
  const localConfig = ref<Partial<UIConfig> | null>(null)

  /**
   * 云端配置（预留，用于后端同步）
   */
  const cloudConfig = ref<Partial<UIConfig> | null>(null)

  /**
   * 是否正在加载云端配置
   */
  const isLoadingCloud = ref(false)

  /**
   * 最后同步时间
   */
  const lastSyncTime = ref<Date | null>(null)

  // ========== 计算属性 ==========

  /**
   * 应用配置
   * 注意：应用配置已从云端配置中移除，仅使用本地配置文件(ui.ts)中的值
   */
  const appConfig = computed<AppConfig>(() => defaultUiConfig.app)

  /**
   * 合并后的主题配置
   */
  const themeConfig = computed<ThemeConfig>({
    get: () => ({
      ...defaultUiConfig.theme,
      ...cloudConfig.value?.theme,
      ...localConfig.value?.theme,
    }),
    set: (value) => {
      localConfig.value = {
        ...localConfig.value,
        theme: value,
      }
      saveToLocalStorage()
    },
  })

  /**
   * 合并后的侧边栏配置
   */
  const sidebarConfig = computed<{ items: SidebarItemConfig[] }>({
    get: () => ({
      items:
        localConfig.value?.sidebar?.items ??
        cloudConfig.value?.sidebar?.items ??
        defaultUiConfig.sidebar.items,
    }),
    set: (value) => {
      localConfig.value = {
        ...localConfig.value,
        sidebar: value,
      }
      saveToLocalStorage()
    },
  })

  /**
   * 合并后的 QuickBar 配置
   */
  const quickBarConfig = computed<QuickItemConfig[]>({
    get: () =>
      localConfig.value?.quickBar ?? cloudConfig.value?.quickBar ?? defaultUiConfig.quickBar ?? [],
    set: (value) => {
      localConfig.value = {
        ...localConfig.value,
        quickBar: value,
      }
      saveToLocalStorage()
    },
  })

  /**
   * 合并后的 Dock 配置
   */
  const dockConfig = computed<DockConfig>({
    get: () => ({
      ...defaultUiConfig.dock,
      ...cloudConfig.value?.dock,
      ...localConfig.value?.dock,
    }),
    set: (value) => {
      localConfig.value = {
        ...localConfig.value,
        dock: value,
      }
      saveToLocalStorage()
    },
  })

  /**
   * 完整的合并配置
   * 注意：应用配置(app)已从云端配置中移除
   */
  const mergedConfig = computed<UIConfig>(() => ({
    theme: themeConfig.value,
    sidebar: sidebarConfig.value,
    quickBar: quickBarConfig.value,
    dock: dockConfig.value,
  }))

  /**
   * 是否有本地自定义配置
   */
  const hasLocalConfig = computed(() => localConfig.value !== null)

  /**
   * 是否有云端配置
   */
  const hasCloudConfig = computed(() => cloudConfig.value !== null)

  // ========== 私有方法 ==========

  /**
   * 保存到本地存储
   */
  function saveToLocalStorage(): void {
    try {
      if (localConfig.value) {
        localStorage.setItem('admin_ui_config_override', JSON.stringify(localConfig.value))
      } else {
        localStorage.removeItem('admin_ui_config_override')
      }
    } catch (error) {
      console.error('Failed to save UI config to localStorage:', error)
    }
  }

  /**
   * 从本地存储加载
   */
  function loadFromLocalStorage(): void {
    try {
      const stored = localStorage.getItem('admin_ui_config_override')
      if (stored) {
        localConfig.value = JSON.parse(stored) as Partial<UIConfig>
      }
    } catch (error) {
      console.error('Failed to load UI config from localStorage:', error)
      localConfig.value = null
    }
  }

  // ========== Actions ==========

  /**
   * 初始化配置
   * 从本地存储加载用户偏好
   */
  function initConfig(): void {
    loadFromLocalStorage()
  }

  /**
   * 更新主题配置
   * @param config 部分主题配置
   */
  function updateThemeConfig(config: Partial<ThemeConfig>): void {
    themeConfig.value = { ...themeConfig.value, ...config }
  }

  /**
   * 更新侧边栏菜单项
   * @param items 新的菜单项列表
   */
  function updateSidebarItems(items: SidebarItemConfig[]): void {
    sidebarConfig.value = { items }
  }

  /**
   * 更新 QuickBar 配置
   * @param items 新的 QuickBar 项列表
   */
  function updateQuickBarItems(items: QuickItemConfig[]): void {
    quickBarConfig.value = items
  }

  /**
   * 更新 Dock 配置
   * @param config 新的 Dock 配置
   */
  function updateDockConfig(config: DockConfig): void {
    dockConfig.value = config
  }

  /**
   * 重置所有配置为默认值
   */
  function resetToDefault(): void {
    localConfig.value = null
    cloudConfig.value = null
    saveToLocalStorage()
  }

  /**
   * 重置指定模块为默认
   * @param module 配置模块名
   */
  function resetModule(module: keyof UIConfig): void {
    if (localConfig.value) {
      const { [module]: _, ...rest } = localConfig.value
      localConfig.value = Object.keys(rest).length > 0 ? rest : null
      saveToLocalStorage()
    }
  }

  /**
   * 加载云端配置
   * 登录成功后调用，获取用户云端 UI 配置
   * 开发环境：跳过云端加载
   */
  async function loadCloudConfig(): Promise<void> {
    // 开发环境不使用云端配置
    if (isDev) {
      return
    }

    isLoadingCloud.value = true
    try {
      const response = await uiApi.getUserConfig()
      if (response.success && response.data) {
        cloudConfig.value = response.data as Partial<UIConfig>
        lastSyncTime.value = new Date()
      }
    } catch (error) {
      console.error('Failed to load cloud config:', error)
    } finally {
      isLoadingCloud.value = false
    }
  }

  /**
   * 保存到云端
   * 将本地配置同步到云端
   * 开发环境：仅保存到本地存储
   */
  async function saveToCloud(): Promise<void> {
    // 开发环境不保存到云端
    if (isDev) {
      return
    }

    try {
      if (!localConfig.value) return

      const response = await uiApi.saveUserConfig({
        theme: localConfig.value.theme,
        sidebar: localConfig.value.sidebar,
        quickBar: localConfig.value.quickBar,
        dock: localConfig.value.dock,
      })

      if (response.success) {
        lastSyncTime.value = new Date()
      }
    } catch (error) {
      console.error('Failed to save cloud config:', error)
      throw error
    }
  }

  /**
   * 登录成功后初始化
   * 加载云端配置并合并到本地
   * 开发环境：跳过云端初始化
   */
  async function initAfterLogin(): Promise<void> {
    // 开发环境不加载云端配置
    if (isDev) {
      return
    }

    await loadCloudConfig()
    // 云端配置优先级高于本地，进行合并
    if (cloudConfig.value) {
      // 合并策略：本地优先，云端补充
      // 如需云端优先，可调用 syncCloudToLocal()
    }
  }

  /**
   * 同步云端配置到本地
   * 云端配置优先级高于本地
   */
  function syncCloudToLocal(): void {
    if (cloudConfig.value) {
      // 合并云端配置到本地
      localConfig.value = {
        ...localConfig.value,
        ...cloudConfig.value,
      }
      saveToLocalStorage()
      lastSyncTime.value = new Date()
    }
  }

  // 初始化时加载本地配置
  initConfig()

  return {
    // 状态
    localConfig,
    cloudConfig,
    isLoadingCloud,
    lastSyncTime,

    // 计算属性
    appConfig,
    themeConfig,
    sidebarConfig,
    quickBarConfig,
    dockConfig,
    mergedConfig,
    hasLocalConfig,
    hasCloudConfig,

    // Actions
    initConfig,
    updateThemeConfig,
    updateSidebarItems,
    updateQuickBarItems,
    updateDockConfig,
    resetToDefault,
    resetModule,
    loadCloudConfig,
    saveToCloud,
    syncCloudToLocal,
    initAfterLogin,
  }
})
