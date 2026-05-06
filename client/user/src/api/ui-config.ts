import httpClient from '@/services/http'
import type { ApiResponse } from '@/types/api'

/**
 * 侧边栏菜单项配置
 */
export interface SidebarItemConfig {
  /** 菜单名称 */
  name: string
  /** 图标名称（Lucide 图标） */
  icon: string
  /** 路由路径 */
  path: string
}

/**
 * 侧边栏配置
 */
export interface SidebarConfig {
  /** 菜单项列表 */
  items: SidebarItemConfig[]
}

/**
 * QuickBar 子菜单项配置
 */
export interface QuickChildItemConfig {
  /** 唯一标识 */
  key: string
  /** 菜单标签 */
  label: string
  /** 图标名称 */
  icon: string
  /** 是否禁用 */
  disabled?: boolean
}

/**
 * QuickBar 角标配置
 */
export interface BadgeConfig {
  /** 角标数字 */
  count: number
  /** 最大显示数字 */
  max: number
}

/**
 * QuickBar 项配置
 */
export interface QuickBarItemConfig {
  /** 唯一标识 */
  key: string
  /** 显示方式：visible(直接显示) / dropdown(下拉菜单) */
  display: 'visible' | 'dropdown'
  /** 类型：action(点击动作) / menu(菜单) */
  type: 'action' | 'menu'
  /** 图标名称 */
  icon: string
  /** 替代图标（用于状态切换） */
  icon_alt?: string
  /** 按钮标签 */
  label: string
  /** 角标配置 */
  badge?: BadgeConfig
  /** 子菜单项 */
  children?: QuickChildItemConfig[]
}

/**
 * QuickBar 配置
 */
export interface QuickBarConfig {
  /** QuickBar 项目列表 */
  items: QuickBarItemConfig[]
}

/**
 * 主题配置
 */
export interface ThemeConfig {
  /** 主题名称 */
  name: 'light' | 'dark' | 'auto'
}

/**
 * 应用配置
 */
export interface AppConfig {
  /** 应用名称 */
  name: string
  /** Logo URL */
  logo: string
  /** 应用版本 */
  version: string
}

/**
 * 完整 UI 配置
 */
export interface UIConfig {
  /** 应用配置 */
  app?: AppConfig
  /** 主题配置 */
  theme?: ThemeConfig
  /** 侧边栏配置 */
  sidebar?: SidebarConfig
  /** QuickBar 配置 */
  quickbar?: QuickBarItemConfig[]
}

/**
 * UI 配置 API
 * 用于云端同步用户界面配置
 */
export const uiConfigApi = {
  /**
   * 获取用户 UI 配置
   * @returns UI 配置
   */
  getUIConfig(): Promise<ApiResponse<UIConfig>> {
    return httpClient.get('/ui/config')
  },

  /**
   * 保存用户 UI 配置（支持部分更新）
   * @param config 部分 UI 配置
   * @returns 保存结果
   */
  saveUIConfig(config: UIConfig): Promise<ApiResponse<{ message: string }>> {
    return httpClient.post('/ui/config', config)
  },

  /**
   * 重置用户 UI 配置
   * @returns 重置结果
   */
  resetUIConfig(): Promise<ApiResponse<{ message: string }>> {
    return httpClient.delete('/ui/config')
  },
}
