/**
 * 认证状态管理 Store
 */

import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import type { User, LoginRequest, RegisterRequest } from '@/types/api'
import {
  login as loginApi,
  register as registerApi,
  logout as logoutApi,
  getCurrentUser,
  getStoredUser,
  isAuthenticated as checkIsAuthenticated,
} from '@/api/auth'
import { useWebSocketStore } from './websocket'

export const useAuthStore = defineStore('auth', () => {
  // ========== State ==========
  const user = ref<User | null>(getStoredUser())
  const loading = ref(false)
  const error = ref<string | null>(null)

  // ========== Getters ==========
  const isAuthenticated = computed(() => checkIsAuthenticated())
  const isLoggedIn = computed(() => !!user.value)
  const username = computed(() => user.value?.username || '')
  const userAvatar = computed(() => {
    if (user.value?.username) {
      return user.value.username.charAt(0).toUpperCase()
    }
    return '?'
  })

  // ========== Actions ==========

  /**
   * 用户登录
   */
  async function login(credentials: LoginRequest): Promise<boolean> {
    loading.value = true
    error.value = null

    try {
      const response = await loginApi(credentials)
      user.value = response.user
      return true
    } catch (err) {
      error.value = err instanceof Error ? err.message : '登录失败'
      return false
    } finally {
      loading.value = false
    }
  }

  /**
   * 用户注册
   */
  async function register(data: RegisterRequest): Promise<boolean> {
    loading.value = true
    error.value = null

    try {
      const response = await registerApi(data)
      user.value = response.user
      return true
    } catch (err) {
      error.value = err instanceof Error ? err.message : '注册失败'
      return false
    } finally {
      loading.value = false
    }
  }

  /**
   * 用户登出
   */
  async function logout(): Promise<void> {
    loading.value = true

    try {
      // Disconnect WebSocket before logout to ensure clean state
      const wsStore = useWebSocketStore()
      wsStore.disconnect()

      await logoutApi()
    } finally {
      user.value = null
      loading.value = false
    }
  }

  /**
   * 获取当前用户信息
   */
  async function fetchCurrentUser(): Promise<boolean> {
    if (!isAuthenticated.value) {
      return false
    }

    loading.value = true

    try {
      const currentUser = await getCurrentUser()
      user.value = currentUser
      return true
    } catch (err) {
      console.error('获取用户信息失败:', err)
      return false
    } finally {
      loading.value = false
    }
  }

  /**
   * 清除错误信息
   */
  function clearError(): void {
    error.value = null
  }

  /**
   * 处理 Token 过期
   */
  function handleTokenExpired(): void {
    user.value = null
  }

  return {
    // State
    user,
    loading,
    error,
    // Getters
    isAuthenticated,
    isLoggedIn,
    username,
    userAvatar,
    // Actions
    login,
    register,
    logout,
    fetchCurrentUser,
    clearError,
    handleTokenExpired,
  }
})
