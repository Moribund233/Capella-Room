import { computed } from 'vue'
import { useUIStore } from '@/store'
import type {
  UIConfig,
  SidebarItemConfig,
  DockItemConfig,
  DockPageConfig,
  QuickItemConfig,
} from '@/config'
import * as LucideIcons from 'lucide-vue-next'
import type { FunctionalComponent } from 'vue'
import type { LucideProps } from 'lucide-vue-next'

/**
 * 图标组件类型
 */
export type IconComponent = FunctionalComponent<LucideProps>

/**
 * DockBar 项类型
 */
export interface DockBarItem {
  key: string
  label: string
  icon: IconComponent
  path: string
  disabled?: boolean
}

/**
 * 侧边栏菜单项
 */
export interface MenuItem {
  key: string
  label: string
  icon: IconComponent
  path: string
  children?: MenuItem[]
}

/**
 * 处理 logo 路径
 * 确保路径以 / 开头，以便在开发和生产环境都能正确加载
 * @param logo 原始 logo 路径
 * @returns 处理后的路径
 */
function resolveLogoPath(logo: string): string {
  // 如果路径已经是绝对路径（以 / 开头），直接返回
  if (logo.startsWith('/')) {
    return logo
  }
  // 如果路径以 http 或 https 开头，说明是外部链接，直接返回
  if (logo.startsWith('http://') || logo.startsWith('https://')) {
    return logo
  }
  // 否则添加 / 前缀
  return `/${logo}`
}

/**
 * 将配置项转换为菜单项
 */
function convertToMenuItem(item: SidebarItemConfig): MenuItem {
  // 直接使用配置中的 Lucide 图标组件名
  const iconComponent =
    (LucideIcons as unknown as Record<string, IconComponent>)[item.icon] || LucideIcons.Circle

  return {
    key: item.path.replace('/', '') || 'home',
    label: item.name,
    icon: iconComponent,
    path: item.path,
  }
}

/**
 * 将 Dock 配置项转换为 DockBar 项
 */
function convertToDockBarItem(item: DockItemConfig): DockBarItem {
  // 直接使用配置中的 Lucide 图标组件名
  const iconComponent =
    (LucideIcons as unknown as Record<string, IconComponent>)[item.icon] || LucideIcons.Circle

  return {
    key: item.key,
    label: item.label,
    icon: iconComponent,
    path: item.path,
    disabled: item.disabled,
  }
}

/**
 * 使用 UI 配置
 * @returns 配置相关的方法和状态
 */
export function useConfig() {
  const uiStore = useUIStore()

  return {
    config: computed<UIConfig>(() => uiStore.mergedConfig),
    isLoading: computed(() => false),
  }
}

/**
 * 获取应用配置
 * @returns 应用配置对象
 */
export function useAppConfig() {
  const uiStore = useUIStore()

  return computed(() => ({
    name: uiStore.appConfig.name,
    logo: resolveLogoPath(uiStore.appConfig.logo),
    version: uiStore.appConfig.version,
  })).value
}

/**
 * 获取侧边栏菜单配置
 * @returns 菜单列表
 */
export function useSidebarConfig() {
  const uiStore = useUIStore()

  const menuItems = computed<MenuItem[]>(() => uiStore.sidebarConfig.items.map(convertToMenuItem))

  return {
    menuItems,
    isLoading: computed(() => false),
  }
}

/**
 * 获取主题配置
 * @returns 主题配置对象
 */
export function useThemeConfig() {
  const uiStore = useUIStore()

  return {
    name: uiStore.themeConfig.name,
  }
}

/**
 * 获取 DockBar 配置
 * @param pagePath 页面路径（如 'example'）
 * @returns DockBar 配置
 */
export function useDockBarConfig(pagePath: string) {
  const uiStore = useUIStore()

  const dockConfig = computed<DockPageConfig | null>(() => uiStore.dockConfig[pagePath] || null)

  /**
   * DockBar 项目列表（已转换图标组件）
   */
  const dockItems = computed<DockBarItem[]>(() => {
    if (!dockConfig.value?.items) return []
    return dockConfig.value.items.map(convertToDockBarItem)
  })

  /**
   * 是否启用 DockBar
   */
  const enabled = computed(() => dockConfig.value?.enabled ?? true)

  /**
   * DockBar 位置
   */
  const position = computed(() => dockConfig.value?.position ?? 'bottom')

  /**
   * 距离边缘距离
   */
  const offset = computed(() => dockConfig.value?.offset ?? 24)

  return {
    dockConfig,
    dockItems,
    enabled,
    position,
    offset,
    isLoading: computed(() => false),
  }
}

/**
 * 获取 QuickBar 配置
 * @returns QuickBar 配置列表
 */
export function useQuickBarConfig() {
  const uiStore = useUIStore()

  const quickBarItems = computed<QuickItemConfig[]>(() => uiStore.quickBarConfig)

  return {
    quickBarItems,
    isLoading: computed(() => false),
  }
}
