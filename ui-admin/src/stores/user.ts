/**
 * 用户状态管理 Store
 * 使用 Pinia 管理用户认证状态和用户信息
 */
import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { login as loginApi, logout as logoutApi, refreshToken, isAdmin, isSuperAdmin } from '@/api'
import type { User, LoginRequest, UserRole } from '@/api'

/**
 * 用户 Store
 * 管理用户登录状态、Token 和用户信息
 */
export const useUserStore = defineStore('user', () => {
  // ==================== State ====================

  /** 当前用户 */
  const user = ref<User | null>(null)

  /** 访问令牌 */
  const accessToken = ref<string>('')

  /** 刷新令牌 */
  const refreshTokenValue = ref<string>('')

  /** 登录中状态 */
  const isLoading = ref(false)

  /** 错误信息 */
  const error = ref<string>('')

  /** 是否记住登录 */
  const rememberMe = ref(false)

  // ==================== Getters ====================

  /** 是否已登录 */
  const isLoggedIn = computed(() => !!accessToken.value && !!user.value)

  /** 是否管理员 */
  const isUserAdmin = computed(() => user.value ? isAdmin(user.value.role) : false)

  /** 是否超级管理员 */
  const isUserSuperAdmin = computed(() => user.value ? isSuperAdmin(user.value.role) : false)

  /** 用户显示名称 */
  const displayName = computed(() => user.value?.username || '')

  /** 用户头像 */
  const avatarUrl = computed(() => user.value?.avatar_url || '')

  // ==================== Actions ====================

  /**
   * 初始化 Store
   * 从本地存储恢复登录状态
   */
  function initialize() {
    // 优先从 localStorage 读取
    let token = localStorage.getItem('access_token')
    let refresh = localStorage.getItem('refresh_token')
    let userData = localStorage.getItem('user')

    // 如果没有，则从 sessionStorage 读取
    if (!token) {
      token = sessionStorage.getItem('access_token')
      refresh = sessionStorage.getItem('refresh_token')
      userData = sessionStorage.getItem('user')
    } else {
      rememberMe.value = true
    }

    if (token && userData) {
      try {
        accessToken.value = token
        refreshTokenValue.value = refresh || ''
        user.value = JSON.parse(userData)
      } catch {
        // 解析失败，清除存储
        clearStorage()
      }
    }
  }

  /**
   * 用户登录
   * @param credentials 登录凭证
   * @param remember 是否记住登录
   * @returns 登录是否成功
   */
  async function login(credentials: LoginRequest, remember: boolean = false): Promise<boolean> {
    isLoading.value = true
    error.value = ''
    rememberMe.value = remember

    try {
      const response = await loginApi(credentials)

      if (!response.success || !response.data) {
        error.value = response.message || '登录失败'
        return false
      }

      const { access_token, refresh_token, user: userInfo } = response.data

      // 检查是否为管理员
      if (!isAdmin(userInfo.role)) {
        error.value = '权限不足，仅管理员可登录'
        return false
      }

      // 保存登录信息
      accessToken.value = access_token
      refreshTokenValue.value = refresh_token
      user.value = userInfo

      // 存储到本地
      saveToStorage()

      return true
    } catch (err) {
      if (err instanceof Error) {
        error.value = err.message
      } else {
        error.value = '网络错误，请检查网络连接'
      }
      return false
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 用户登出
   */
  async function logout(): Promise<void> {
    try {
      // 调用登出 API（忽略错误）
      await logoutApi()
    } catch {
      // 忽略 API 错误
    } finally {
      // 清除状态
      user.value = null
      accessToken.value = ''
      refreshTokenValue.value = ''
      clearStorage()
    }
  }

  /**
   * 刷新访问令牌
   * @returns 刷新是否成功
   */
  async function doRefreshToken(): Promise<boolean> {
    if (!refreshTokenValue.value) {
      return false
    }

    try {
      const response = await refreshToken(refreshTokenValue.value)

      if (!response.success || !response.data) {
        return false
      }

      const { access_token, refresh_token } = response.data

      accessToken.value = access_token
      refreshTokenValue.value = refresh_token

      saveToStorage()

      return true
    } catch {
      // 刷新失败，清除登录状态
      logout()
      return false
    }
  }

  /**
   * 更新用户信息
   * @param userData 新的用户数据
   */
  function updateUser(userData: Partial<User>) {
    if (user.value) {
      user.value = { ...user.value, ...userData }
      saveToStorage()
    }
  }

  /**
   * 清除错误信息
   */
  function clearError() {
    error.value = ''
  }

  // ==================== Private Methods ====================

  /**
   * 保存到本地存储
   */
  function saveToStorage() {
    const storage = rememberMe.value ? localStorage : sessionStorage
    const otherStorage = rememberMe.value ? sessionStorage : localStorage

    storage.setItem('access_token', accessToken.value)
    storage.setItem('refresh_token', refreshTokenValue.value)
    storage.setItem('user', JSON.stringify(user.value))

    // 清除另一个存储中的数据
    otherStorage.removeItem('access_token')
    otherStorage.removeItem('refresh_token')
    otherStorage.removeItem('user')
  }

  /**
   * 清除本地存储
   */
  function clearStorage() {
    localStorage.removeItem('access_token')
    localStorage.removeItem('refresh_token')
    localStorage.removeItem('user')
    sessionStorage.removeItem('access_token')
    sessionStorage.removeItem('refresh_token')
    sessionStorage.removeItem('user')
  }

  // ==================== Return ====================

  return {
    // State
    user,
    accessToken,
    refreshTokenValue,
    isLoading,
    error,
    rememberMe,

    // Getters
    isLoggedIn,
    isUserAdmin,
    isUserSuperAdmin,
    displayName,
    avatarUrl,

    // Actions
    initialize,
    login,
    logout,
    doRefreshToken,
    updateUser,
    clearError,
  }
})
