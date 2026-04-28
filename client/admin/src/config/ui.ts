/**
 * UI 配置文件
 * 取代静态的 ui.json 配置，提供类型安全的配置对象
 */

import type {
  AppConfig,
  SidebarItemConfig,
  QuickItemConfig,
  ThemeConfig,
  DockConfig,
} from '@/types'

/**
 * 应用基础配置
 */
export const appConfig: AppConfig = {
  name: '聊天室管理后台',
  logo: '/favicon.svg',
  version: '1.0.0',
}

/**
 * 侧边栏菜单配置
 */
export const sidebarConfig: { items: SidebarItemConfig[] } = {
  items: [
    {
      name: '首页',
      icon: 'LayoutDashboard',
      path: '/home',
    },
    {
      name: '用户管理',
      icon: 'Users',
      path: '/users',
    },
    {
      name: '房间管理',
      icon: 'MessageSquare',
      path: '/rooms',
    },
    {
      name: '消息管理',
      icon: 'Mail',
      path: '/messages',
    },
    {
      name: '系统设置',
      icon: 'Settings',
      path: '/setting',
    },
  ],
}

/**
 * 主题配置
 */
export const themeConfig: ThemeConfig = {
  name: 'dark',
}

/**
 * QuickBar 快捷栏配置
 */
export const quickBarConfig: QuickItemConfig[] = [
  {
    key: 'sidebar',
    display: 'visible',
    type: 'action',
    icon: 'PanelLeft',
    iconAlt: 'PanelRight',
    label: '切换侧边栏',
  },
  {
    key: 'footer',
    display: 'visible',
    type: 'action',
    icon: 'PanelBottomClose',
    iconAlt: 'PanelBottomOpen',
    label: '切换Footer',
  },
  {
    key: 'theme',
    display: 'visible',
    type: 'menu',
    icon: 'Sun',
    iconAlt: 'Moon',
    label: '主题',
    children: [
      {
        key: 'light',
        label: '浅色模式',
        icon: 'Sun',
      },
      {
        key: 'dark',
        label: '夜间模式',
        icon: 'Moon',
      },
    ],
  },
  {
    key: 'personalization',
    display: 'dropdown',
    type: 'menu',
    icon: 'Palette',
    label: '个性化',
    children: [
      {
        key: 'sidebar',
        label: '侧边栏',
        icon: 'PanelLeft',
      },
      {
        key: 'header',
        label: '顶部',
        icon: 'PanelTop',
      },
      {
        key: 'footer',
        label: '底部',
        icon: 'PanelBottom',
      },
    ],
  },
  {
    key: 'user',
    display: 'dropdown',
    type: 'menu',
    icon: 'UserCircle',
    iconAlt: 'User',
    label: '用户中心',
    children: [
      {
        key: 'profile',
        label: '用户详情',
        icon: 'User',
      },
      {
        key: 'about',
        label: '关于',
        icon: 'Info',
      },
      {
        key: 'help',
        label: '帮助',
        icon: 'HelpCircle',
      },
      {
        key: 'logout',
        label: '登出',
        icon: 'LogOut',
      },
    ],
  },
]

/**
 * DockBar 页面级配置
 */
export const dockConfig: DockConfig = {
  setting: {
    enabled: true,
    position: 'bottom',
    offset: 24,
    items: [
      {
        key: 'ui',
        label: '界面设置',
        icon: 'Palette',
        path: '/setting/ui',
      },
    ],
  },
}

/**
 * 完整的 UI 配置对象
 * 整合所有配置模块
 */
export const uiConfig = {
  app: appConfig,
  sidebar: sidebarConfig,
  theme: themeConfig,
  quickBar: quickBarConfig,
  dock: dockConfig,
}

export default uiConfig
