export { loadConfig, initConfig, getConfig, getDockPageConfig } from './config'

export { uiConfig } from './ui'

// 从 config.ts 重新导出类型，保持向后兼容
export type {
  UIConfig,
  AppConfig,
  SidebarItemConfig,
  ThemeConfig,
  DockItemConfig,
  DockPageConfig,
  DockConfig,
  QuickItemConfig,
  QuickChildItemConfig,
} from './config'
