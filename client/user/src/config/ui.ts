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
  name: 'Seredeli Room',
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
      name: '房间',
      icon: 'MessageSquare',
      path: '/room/list',
    },
    {
      name: '设置',
      icon: 'Settings',
      path: '/setting',
    },
    {
      name: '调试工具',
      icon: 'Bug',
      path: '/debug',
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
 * 布局外观配置
 * 用户可自定义的布局样式设置
 * 使用 CSS 变量确保与主题一致
 */
export const layoutConfig = {
  backgroundType: 'solid' as const,
  backgroundColor: 'var(--bg-base)',
  gradientFrom: 'var(--color-primary)',
  gradientTo: 'var(--color-primary-hover)',
  backgroundImage: '',
  backgroundOpacity: 1,
  contentOpacity: 1,
  backgroundBlur: 0,
  borderRadius: 8,
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
      {
        key: 'light-transparent',
        label: '浅色透明',
        icon: 'Image',
      },
      {
        key: 'dark-transparent',
        label: '暗色透明',
        icon: 'ImagePlus',
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
 * 为主页面配置 DockBar，用于子页面导航
 * 键名为路由的第一级路径（如 'setting' 对应 /setting）
 */
export const dockConfig: DockConfig = {
  // 房间模块 - 支持动态房间ID
  room: {
    enabled: true,
    position: 'bottom',
    offset: 24,
    items: [
      {
        key: 'list',
        label: '房间列表',
        icon: 'List',
        path: '/room/list',
      },
      {
        key: 'chat',
        label: '聊天室',
        icon: 'MessageSquare',
        path: '/room/chat/:id',
        requiresParams: true,
        paramKeys: ['id'],
        missingParamsMessage: '请先选择一个房间',
      },
      {
        key: 'users',
        label: '在线用户',
        icon: 'Users',
        path: '/room/users/:id',
        requiresParams: true,
        paramKeys: ['id'],
        missingParamsMessage: '请先选择一个房间',
      },
    ],
  },
  // 设置页 - 子页面通过 DockBar 导航
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
  // Debug工具页 - 子页面通过 DockBar 导航
  debug: {
    enabled: true,
    position: 'bottom',
    offset: 24,
    items: [
      {
        key: 'multi-user',
        label: '多用户测试',
        icon: 'Users',
        path: '/debug/multi-user',
      },
      {
        key: 'websocket',
        label: 'WebSocket测试',
        icon: 'Wifi',
        path: '/debug/websocket',
      },
      {
        key: 'api',
        label: 'API测试',
        icon: 'Terminal',
        path: '/debug/api',
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
  layout: layoutConfig,
}

export default uiConfig
