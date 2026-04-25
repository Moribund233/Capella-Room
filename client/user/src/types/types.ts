import type { MessageApi } from 'naive-ui'

/**
 * 扩展 Window 接口以支持 Naive UI 的全局属性
 */
declare global {
  interface Window {
    $message?: MessageApi
  }
}

// ==================== 基础类型 ====================

/**
 * 响应式断点类型
 */
export type Breakpoint = 'mobile' | 'tablet' | 'desktop'

/**
 * 主题类型
 */
export type ThemeType = 'light' | 'dark'

// ==================== 菜单相关类型 ====================

/**
 * 菜单项类型
 */
export interface MenuItem {
  /** 唯一标识 */
  key: string
  /** 显示文本 */
  label: string
  /** 图标组件 */
  icon?: unknown
  /** 路由路径 */
  path?: string
  /** 子菜单 */
  children?: MenuItem[]
  /** 是否禁用 */
  disabled?: boolean
}

// ==================== UI 配置相关类型 ====================

/**
 * 侧边栏菜单项配置
 */
export interface SidebarItemConfig {
  /** 菜单名称 */
  name: string
  /** 图标名称 */
  icon: string
  /** 路由路径 */
  path: string
}

/**
 * DockBar 项目配置
 */
export interface DockItemConfig {
  /** 唯一标识 */
  key: string
  /** 显示标签 */
  label: string
  /** 图标名称 */
  icon: string
  /** 路由路径（可以是模板，如 /room/chat/:id） */
  path: string
  /** 是否禁用 */
  disabled?: boolean
  /** 是否需要路由参数（如房间ID） */
  requiresParams?: boolean
  /** 参数名列表（用于从当前路由提取参数） */
  paramKeys?: string[]
  /** 缺少参数时的提示消息 */
  missingParamsMessage?: string
}

/**
 * DockBar 页面级配置
 */
export interface DockPageConfig {
  /** 是否启用 */
  enabled?: boolean
  /** 位置 */
  position?: 'bottom' | 'left' | 'right'
  /** 偏移量 */
  offset?: number
  /** 项目列表 */
  items?: DockItemConfig[]
}

/**
 * DockBar 配置
 */
export interface DockConfig {
  [pagePath: string]: DockPageConfig
}

/**
 * 应用配置
 */
export interface AppConfig {
  /** 应用名称 */
  name: string
  /** Logo 路径 */
  logo: string
  /** 版本号 */
  version: string
}

/**
 * 主题配置
 */
export interface ThemeConfig {
  /** 主题名称 */
  name: string
}

/**
 * QuickBar 子菜单项配置
 */
export interface QuickChildItemConfig {
  /** 唯一标识 */
  key: string
  /** 显示标签 */
  label: string
  /** 图标名称 */
  icon?: string
  /** 是否禁用 */
  disabled?: boolean
}

/**
 * QuickBar 项目配置
 */
export interface QuickItemConfig {
  /** 唯一标识 */
  key: string
  /** 显示方式 */
  display: 'visible' | 'dropdown'
  /** 类型 */
  type: 'action' | 'menu'
  /** 图标名称 */
  icon: string
  /** 替代图标名称 */
  iconAlt?: string
  /** 显示标签 */
  label: string
  /** 徽章 */
  badge?: string | number
  /** 子菜单 */
  children?: QuickChildItemConfig[]
}

/**
 * UI 配置
 */
export interface UIConfig {
  /** 应用配置 */
  app: AppConfig
  /** 侧边栏配置 */
  sidebar: {
    items: SidebarItemConfig[]
  }
  /** 主题配置 */
  theme: ThemeConfig
  /** Dock 配置 */
  dock: DockConfig
  /** QuickBar 配置 */
  quickBar?: QuickItemConfig[]
}

// ==================== API 请求/响应类型 ====================

/**
 * UI 配置 API 响应数据
 */
export interface UIConfigResponse {
  /** 应用配置 */
  app?: Partial<AppConfig>
  /** 主题配置 */
  theme?: Partial<ThemeConfig>
  /** 侧边栏配置 */
  sidebar?: {
    items?: SidebarItemConfig[]
  }
  /** QuickBar 配置 */
  quickBar?: QuickItemConfig[]
  /** Dock 配置 */
  dock?: DockConfig
}

/**
 * 保存 UI 配置请求参数
 */
export interface SaveUIConfigParams {
  /** 应用配置 */
  app?: Partial<AppConfig>
  /** 主题配置 */
  theme?: Partial<ThemeConfig>
  /** 侧边栏配置 */
  sidebar?: {
    items?: SidebarItemConfig[]
  }
  /** QuickBar 配置 */
  quickBar?: QuickItemConfig[]
  /** Dock 配置 */
  dock?: DockConfig
}
