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

export const useAuthStore = defineStore('auth', () => {
  // ========== State ==========

  /** 当前用户 */
  const user = ref<User | null>(getStoredUser())

  /** 加载状态 */
  const loading = ref(false)

  /** 错误信息 */
  const error = ref<string | null>(null)

  // ========== Getters ==========

  /** 是否已登录 */
  const isAuthenticated = computed(() => checkIsAuthenticated())

  /** 是否为管理员（包含 admin 和 super_admin） */
  const isAdmin = computed(() => user.value?.role === 'admin' || user.value?.role === 'super_admin')

  /** 是否为超级管理员 */
  const isSuperAdmin = computed(() => user.value?.role === 'super_admin')

  /** 用户角色文本 */
  const roleText = computed(() => {
    switch (user.value?.role) {
      case 'super_admin':
        return '超级管理员'
      case 'admin':
        return '管理员'
      default:
        return '用户'
    }
  })

  /** 用户名 */
  const username = computed(() => user.value?.username || '')

  /** 用户头像（首字母） */
  const userAvatar = computed(() => {
    if (user.value?.username) {
      return user.value.username.charAt(0).toUpperCase()
    }
    return '?'
  })

  // ========== Actions ==========

  /**
   * 用户登录
   * @param credentials 登录凭证 (email + password)
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
   * @param data 注册信息
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

  return {
    // State
    user,
    loading,
    error,

    // Getters
    isAuthenticated,
    isAdmin,
    isSuperAdmin,
    roleText,
    username,
    userAvatar,

    // Actions
    login,
    register,
    logout,
    fetchCurrentUser,
    clearError,
  }
})
