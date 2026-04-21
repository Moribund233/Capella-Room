import type { ApiResponse, UIConfigResponse, SaveUIConfigParams } from '@/types'

/**
 * 是否为开发环境
 */
const isDev = import.meta.env.VITE_APP_ENV === 'development'

/**
 * 模拟云端 UI 配置数据库
 * 以用户 ID 为键存储配置
 */
const mockCloudConfig: Map<string, UIConfigResponse> = new Map()

/**
 * 默认云端配置（新用户使用）
 */
const defaultCloudConfig: UIConfigResponse = {
  app: {
    name: 'SeredeliUI Cloud',
    logo: '/favicon.svg',
  },
  theme: {
    name: 'dark',
  },
  sidebar: {
    items: [
      {
        name: '首页',
        icon: 'LayoutDashboard',
        path: '/home',
      },
      {
        name: '示例',
        icon: 'FileText',
        path: '/example',
      },
      {
        name: '设置',
        icon: 'Settings',
        path: '/setting',
      },
    ],
  },
  quickBar: [
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
      key: 'user',
      display: 'dropdown',
      type: 'menu',
      icon: 'UserCircle',
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
  ],
  dock: {
    example: {
      enabled: true,
      position: 'bottom',
      offset: 24,
      items: [
        {
          key: 'overview',
          label: '概览',
          icon: 'Layout',
          path: '/example/overview',
        },
        {
          key: 'icons',
          label: '图标',
          icon: 'Smile',
          path: '/example/icons',
        },
      ],
    },
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
  },
}

/**
 * 获取当前用户 ID（从 localStorage 或 session）
 */
function getCurrentUserId(): string {
  // 从 auth store 或 localStorage 获取当前用户 ID
  const userInfo = localStorage.getItem('user_info')
  if (userInfo) {
    try {
      const user = JSON.parse(userInfo)
      return user.id || 'anonymous'
    } catch {
      return 'anonymous'
    }
  }
  return 'anonymous'
}

/**
 * 开发环境：模拟 API 实现
 * 生产环境：真实 API 调用（预留）
 */
export const uiApi = {
  /**
   * 获取用户云端 UI 配置
   * @returns 云端 UI 配置
   */
  getUserConfig(): Promise<ApiResponse<UIConfigResponse>> {
    // 开发环境使用模拟数据
    if (isDev) {
      return new Promise((resolve) => {
        setTimeout(() => {
          const userId = getCurrentUserId()
          const config = mockCloudConfig.get(userId) || defaultCloudConfig

          resolve({
            code: 200,
            data: config,
            message: '获取成功',
            success: true,
          })
        }, 300)
      })
    }

    // 生产环境：真实 API 调用（预留）
    throw new Error('Production API not implemented')
  },

  /**
   * 保存用户云端 UI 配置
   * @param params 配置参数
   * @returns 保存结果
   */
  saveUserConfig(params: SaveUIConfigParams): Promise<ApiResponse<void>> {
    // 开发环境使用模拟数据
    if (isDev) {
      return new Promise((resolve) => {
        setTimeout(() => {
          const userId = getCurrentUserId()
          const existingConfig = mockCloudConfig.get(userId) || {}

          // 合并新配置
          mockCloudConfig.set(userId, {
            ...existingConfig,
            ...params,
          })

          resolve({
            code: 200,
            data: undefined as unknown as void,
            message: '保存成功',
            success: true,
          })
        }, 300)
      })
    }

    // 生产环境：真实 API 调用（预留）
    throw new Error('Production API not implemented')
  },

  /**
   * 重置用户云端 UI 配置为默认
   * @returns 重置结果
   */
  resetUserConfig(): Promise<ApiResponse<void>> {
    // 开发环境使用模拟数据
    if (isDev) {
      return new Promise((resolve) => {
        setTimeout(() => {
          const userId = getCurrentUserId()
          mockCloudConfig.delete(userId)

          resolve({
            code: 200,
            data: undefined as unknown as void,
            message: '重置成功',
            success: true,
          })
        }, 300)
      })
    }

    // 生产环境：真实 API 调用（预留）
    throw new Error('Production API not implemented')
  },

  /**
   * 同步云端配置到本地（模拟多设备同步）
   * @returns 最新云端配置
   */
  syncConfig(): Promise<ApiResponse<UIConfigResponse>> {
    // 开发环境使用模拟数据
    if (isDev) {
      return new Promise((resolve) => {
        setTimeout(() => {
          const userId = getCurrentUserId()
          const config = mockCloudConfig.get(userId) || defaultCloudConfig

          resolve({
            code: 200,
            data: config,
            message: '同步成功',
            success: true,
          })
        }, 500)
      })
    }

    // 生产环境：真实 API 调用（预留）
    throw new Error('Production API not implemented')
  },
}
