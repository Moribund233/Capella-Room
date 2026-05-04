import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { authApi, isAdmin } from '@/api'
import { useWebSocketStore } from './websocket'
import type { UserInfo, LoginResult, UserRole } from '@/types'

/**
 * Token 存储键名
 */
const ACCESS_TOKEN_KEY = 'access_token'
const REFRESH_TOKEN_KEY = 'refresh_token'
const USER_INFO_KEY = 'user_info'

/**
 * 认证状态存储
 */
export const useAuthStore = defineStore('auth', () => {
  /**
   * 访问令牌
   */
  const accessToken = ref<string | null>(localStorage.getItem(ACCESS_TOKEN_KEY))

  /**
   * 刷新令牌
   */
  const refreshToken = ref<string | null>(localStorage.getItem(REFRESH_TOKEN_KEY))

  /**
   * 用户信息
   */
  const userInfo = ref<UserInfo | null>(null)

  /**
   * 是否已登录
   */
  const isLoggedIn = computed(() => !!accessToken.value && !!userInfo.value)

  /**
   * 是否管理员（admin 或 super_admin）
   */
  const isAdminRole = computed(() => isAdmin(userInfo.value?.role))

  /**
   * 是否超级管理员
   */
  const isSuperAdminRole = computed(() => userInfo.value?.role === 'super_admin')

  /**
   * 用户角色
   */
  const userRole = computed<UserRole | undefined>(() => userInfo.value?.role)

  /**
   * 设置令牌
   * @param result 登录结果
   */
  function setTokens(result: LoginResult) {
    accessToken.value = result.access_token
    refreshToken.value = result.refresh_token
    localStorage.setItem(ACCESS_TOKEN_KEY, result.access_token)
    localStorage.setItem(REFRESH_TOKEN_KEY, result.refresh_token)
  }

  /**
   * 清除认证信息
   */
  function clearAuth() {
    accessToken.value = null
    refreshToken.value = null
    userInfo.value = null
    localStorage.removeItem(ACCESS_TOKEN_KEY)
    localStorage.removeItem(REFRESH_TOKEN_KEY)
    localStorage.removeItem(USER_INFO_KEY)
    // 断开 WebSocket 连接
    const wsStore = useWebSocketStore()
    wsStore.disconnect()
  }

  /**
   * 设置用户信息
   * @param info 用户信息
   */
  function setUserInfo(info: UserInfo) {
    userInfo.value = info
    localStorage.setItem(USER_INFO_KEY, JSON.stringify(info))
  }

  /**
   * 从本地存储恢复用户信息
   */
  function restoreUserInfo() {
    const stored = localStorage.getItem(USER_INFO_KEY)
    if (stored) {
      try {
        userInfo.value = JSON.parse(stored)
      } catch {
        userInfo.value = null
      }
    }
  }

  /**
   * 登录
   * @param email 邮箱
   * @param password 密码
   * @returns 登录是否成功
   */
  async function login(email: string, password: string): Promise<{ success: boolean; message: string }> {
    try {
      const response = await authApi.login({ email, password })

      if (!response.success || !response.data) {
        return { success: false, message: response.message || '登录失败' }
      }

      const { user } = response.data

      // 检查是否为管理员
      if (!isAdmin(user.role)) {
        return { success: false, message: '权限不足：仅管理员可登录管理后台' }
      }

      // 保存认证信息
      setTokens(response.data)
      setUserInfo(user)

      // 建立 WebSocket 连接
      const wsStore = useWebSocketStore()
      wsStore.connect(response.data.access_token)

      return { success: true, message: '登录成功' }
    } catch (error) {
      const message = error instanceof Error ? error.message : '登录失败，请检查网络连接'
      return { success: false, message }
    }
  }

  /**
   * 刷新访问令牌
   * @returns 刷新是否成功
   */
  async function refreshAccessToken(): Promise<boolean> {
    const currentRefreshToken = refreshToken.value
    if (!currentRefreshToken) {
      return false
    }

    try {
      const response = await authApi.refreshToken(currentRefreshToken)
      if (response.success && response.data) {
        setTokens(response.data)
        return true
      }
    } catch {
      // 刷新失败，清除认证信息
      clearAuth()
    }
    return false
  }

  /**
   * 获取当前用户信息
   * @returns 获取是否成功
   */
  async function fetchCurrentUser(): Promise<boolean> {
    try {
      const response = await authApi.getCurrentUser()
      if (response.success && response.data) {
        setUserInfo(response.data)
        return true
      }
    } catch {
      // 获取失败
    }
    return false
  }

  /**
   * 退出登录
   */
  async function logout() {
    clearAuth()
  }

  /**
   * 初始化认证状态（应用启动时调用）
   */
  async function initAuth(): Promise<boolean> {
    restoreUserInfo()

    if (!accessToken.value) {
      return false
    }

    // 验证令牌有效性并获取最新用户信息
    const success = await fetchCurrentUser()
    if (success && accessToken.value) {
      // 建立 WebSocket 连接
      const wsStore = useWebSocketStore()
      wsStore.connect(accessToken.value)
      return true
    }

    // 获取失败，尝试刷新令牌
    const refreshed = await refreshAccessToken()
    if (refreshed && accessToken.value) {
      const wsStore = useWebSocketStore()
      wsStore.connect(accessToken.value)
    }

    return refreshed
  }

  return {
    accessToken,
    refreshToken,
    userInfo,
    isLoggedIn,
    isAdminRole,
    isSuperAdminRole,
    userRole,
    setTokens,
    clearAuth,
    setUserInfo,
    restoreUserInfo,
    login,
    refreshAccessToken,
    fetchCurrentUser,
    logout,
    initAuth,
  }
})
