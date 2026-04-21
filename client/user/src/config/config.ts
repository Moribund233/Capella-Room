/**
 * 配置类型定义与配置获取函数
 */

import { uiConfig } from './ui'
import type {
  UIConfig,
  AppConfig,
  SidebarItemConfig,
  QuickItemConfig,
  ThemeConfig,
  DockConfig,
  DockPageConfig,
  DockItemConfig,
  QuickChildItemConfig,
} from '@/types'

export type {
  UIConfig,
  AppConfig,
  SidebarItemConfig,
  QuickItemConfig,
  ThemeConfig,
  DockConfig,
  DockPageConfig,
  DockItemConfig,
  QuickChildItemConfig,
}

/**
 * 获取当前配置
 * @returns UIConfig 配置对象
 */
export function getConfig(): UIConfig {
  return uiConfig
}

/**
 * 获取页面级 DockBar 配置
 * @param pagePath 页面路径（如 'example'）
 * @returns DockPageConfig | null
 */
export function getDockPageConfig(pagePath: string): DockPageConfig | null {
  return uiConfig.dock[pagePath] || null
}

/**
 * 初始化配置（已废弃，保留以兼容旧代码）
 * 现在配置直接从 ui.ts 导入，无需异步加载
 * @returns UIConfig 配置对象
 * @deprecated 直接使用 getConfig() 替代
 */
export async function initConfig(): Promise<UIConfig> {
  return uiConfig
}

/**
 * 加载配置（已废弃，保留以兼容旧代码）
 * 现在配置直接从 ui.ts 导入，无需异步加载
 * @returns UIConfig 配置对象
 * @deprecated 直接使用 getConfig() 替代
 */
export async function loadConfig(): Promise<UIConfig> {
  return uiConfig
}
