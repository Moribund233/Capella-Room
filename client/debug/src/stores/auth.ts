/**
 * 认证状态管理 Store
 */

import { ref, computed, onMounted, onUnmounted } from 'vue'
import { defineStore } from 'pinia'
import { useRouter } from 'vue-router'
import { useMessage } from 'naive-ui'
import type { User, LoginRequest, RegisterRequest } from '@/types/api'
import {
  login as loginApi,
  register as registerApi,
  logout as logoutApi,
  getCurrentUser,
  getStoredUser,
  isAuthenticated as checkIsAuthenticated,
} from '@/api/auth'
import { TOKEN_EXPIRED_EVENT } from '@/api/client'

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

  /**
   * 处理 Token 过期
   * 清除认证状态并重定向到登录页
   */
  function handleTokenExpired(message: string = '登录已过期，请重新登录') {
    const router = useRouter()
    const messageApi = useMessage()

    // 清除用户状态
    user.value = null

    // 显示提示消息
    messageApi.warning(message)

    // 重定向到登录页
    router.push('/login')
  }

  /**
   * 初始化 Token 过期监听
   * 在组件挂载时调用
   */
  function initTokenExpiredListener() {
    const handleExpired = (event: CustomEvent<{ message: string }>) => {
      handleTokenExpired(event.detail.message)
    }

    window.addEventListener(TOKEN_EXPIRED_EVENT, handleExpired as EventListener)

    // 返回清理函数，用于组件卸载时移除监听
    return () => {
      window.removeEventListener(TOKEN_EXPIRED_EVENT, handleExpired as EventListener)
    }
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
    handleTokenExpired,
    initTokenExpiredListener,
  }
})
