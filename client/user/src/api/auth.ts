import type { ApiResponse, UserInfo, LoginParams, LoginResult, RegisterParams } from '@/types'

/**
 * 是否为开发环境
 */
const isDev = import.meta.env.VITE_APP_ENV === 'development'

/**
 * 模拟用户数据库（仅开发环境使用）
 */
const mockUsers: Map<string, { password: string; userInfo: UserInfo }> = new Map()

/**
 * 认证相关接口
 * 开发环境：模拟实现
 * 生产环境：真实 API 调用（预留）
 */
export const authApi = {
  /**
   * 用户注册
   * @param params 注册参数
   * @returns 注册结果
   */
  register(params: RegisterParams): Promise<ApiResponse<void>> {
    // 开发环境使用模拟数据
    if (isDev) {
      return new Promise((resolve) => {
        setTimeout(() => {
          if (mockUsers.has(params.username)) {
            resolve({
              code: 400,
              data: undefined as unknown as void,
              message: '用户名已存在',
              success: false,
            })
          } else {
            mockUsers.set(params.username, {
              password: params.password,
              userInfo: {
                id: Date.now().toString(),
                username: params.username,
                nickname: params.username,
              },
            })
            resolve({
              code: 200,
              data: undefined as unknown as void,
              message: '注册成功',
              success: true,
            })
          }
        }, 500)
      })
    }

    // 生产环境：真实 API 调用（预留）
    throw new Error('Production API not implemented')
  },

  /**
   * 用户登录
   * @param params 登录参数
   * @returns 登录结果
   */
  login(params: LoginParams): Promise<ApiResponse<LoginResult>> {
    // 开发环境使用模拟数据
    if (isDev) {
      return new Promise((resolve) => {
        setTimeout(() => {
          // 默认账号：admin/admin
          if (params.username === 'admin' && params.password === 'admin') {
            const userInfo: UserInfo = {
              id: '1',
              username: 'admin',
              nickname: '管理员',
              avatar: '',
              email: 'admin@example.com',
              roles: ['admin'],
            }

            // 保存用户信息到 localStorage（供 UI 配置 API 使用）
            localStorage.setItem('user_info', JSON.stringify(userInfo))

            resolve({
              code: 200,
              data: {
                token: 'mock_token_' + Date.now(),
                userInfo,
              },
              message: '登录成功',
              success: true,
            })
            return
          }

          const user = mockUsers.get(params.username)
          if (user && user.password === params.password) {
            // 保存用户信息到 localStorage（供 UI 配置 API 使用）
            localStorage.setItem('user_info', JSON.stringify(user.userInfo))

            resolve({
              code: 200,
              data: {
                token: 'mock_token_' + Date.now(),
                userInfo: user.userInfo,
              },
              message: '登录成功',
              success: true,
            })
          } else {
            resolve({
              code: 401,
              data: {} as unknown as LoginResult,
              message: '用户名或密码错误',
              success: false,
            })
          }
        }, 500)
      })
    }

    // 生产环境：真实 API 调用（预留）
    throw new Error('Production API not implemented')
  },

  /**
   * 用户登出
   * @returns 登出结果
   */
  logout(): Promise<ApiResponse<void>> {
    // 开发环境使用模拟数据
    if (isDev) {
      return new Promise((resolve) => {
        setTimeout(() => {
          resolve({
            code: 200,
            data: undefined as unknown as void,
            message: '登出成功',
            success: true,
          })
        }, 300)
      })
    }

    // 生产环境：真实 API 调用（预留）
    throw new Error('Production API not implemented')
  },

  /**
   * 获取当前用户信息
   * @returns 用户信息
   */
  getCurrentUser(): Promise<ApiResponse<UserInfo>> {
    // 开发环境使用模拟数据
    if (isDev) {
      return new Promise((resolve) => {
        setTimeout(() => {
          resolve({
            code: 200,
            data: {
              id: '1',
              username: 'admin',
              nickname: '管理员',
              avatar: '',
              email: 'admin@example.com',
              roles: ['admin'],
            },
            message: '获取成功',
            success: true,
          })
        }, 300)
      })
    }

    // 生产环境：真实 API 调用（预留）
    throw new Error('Production API not implemented')
  },
}
