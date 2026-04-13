/**
 * 多用户认证 Store
 * 用于测试场景下的多用户同时在线管理
 * 使用 sessionStorage 持久化，不干扰主应用的 localStorage
 */

import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { apiClient } from '@/api/client'
import { useMultiUserWebSocketStore } from './multiUserWebSocket'
import type { User } from '@/types/api'

/**
 * 测试用户信息
 */
export interface TestUser {
  id: string
  username: string
  email: string
  role: string
  accessToken: string
  refreshToken: string
  isActive: boolean
}

/**
 * 用户凭证
 */
export interface UserCredential {
  username: string
  email: string
  password: string
}

/**
 * 登录响应
 */
interface LoginResponse {
  user: User
  access_token: string
  refresh_token: string
  expires_in: number
}

// sessionStorage 键名
const STORAGE_KEY = 'multi_user_auth_test_users'

export const useMultiUserAuthStore = defineStore('multiUserAuth', () => {
  // ========== State ==========

  /** 所有已认证的测试用户 */
  const testUsers = ref<TestUser[]>([])

  /** 当前激活的用户ID */
  const activeUserId = ref<string | null>(null)

  /** 加载状态 */
  const loading = ref(false)

  /** 是否已从 storage 恢复 */
  const isRestored = ref(false)

  // ========== Getters ==========

  /** 当前激活的用户 */
  const activeUser = computed(() => {
    if (!activeUserId.value) return null
    return testUsers.value.find(u => u.id === activeUserId.value) || null
  })

  /** 所有在线用户（已认证） */
  const onlineUsers = computed(() => testUsers.value.filter(u => u.isActive))

  /** 用户数量 */
  const userCount = computed(() => testUsers.value.length)

  // ========== Storage 操作 ==========

  /**
   * 保存用户列表到 sessionStorage
   */
  function saveToStorage() {
    try {
      const data = JSON.stringify(testUsers.value)
      sessionStorage.setItem(STORAGE_KEY, data)
    } catch (error) {
      console.error('[MultiUserAuth] 保存到 storage 失败:', error)
    }
  }

  /**
   * 从 sessionStorage 恢复用户列表
   */
  function restoreFromStorage(): boolean {
    try {
      const data = sessionStorage.getItem(STORAGE_KEY)
      if (data) {
        const users = JSON.parse(data) as TestUser[]
        testUsers.value = users
        isRestored.value = true
        console.log('[MultiUserAuth] 从 storage 恢复用户:', users.length)
        return true
      }
    } catch (error) {
      console.error('[MultiUserAuth] 从 storage 恢复失败:', error)
    }
    return false
  }

  /**
   * 清除 storage
   */
  function clearStorage() {
    try {
      sessionStorage.removeItem(STORAGE_KEY)
    } catch (error) {
      console.error('[MultiUserAuth] 清除 storage 失败:', error)
    }
  }

  // ========== Actions ==========

  /**
   * 初始化：从 storage 恢复数据
   * 在应用启动时调用
   */
  function initialize() {
    if (!isRestored.value) {
      restoreFromStorage()
    }
  }

  /**
   * 使用凭证登录用户
   * 不写入 localStorage，仅保存在内存和 sessionStorage 中
   * 认证成功后自动创建 WebSocket 连接
   * @param credential 用户凭证
   * @returns 登录成功的用户信息
   */
  async function loginUser(credential: UserCredential): Promise<TestUser> {
    loading.value = true

    try {
      console.log('[MultiUserAuth] 登录用户:', credential.email)

      const response = await apiClient.post<LoginResponse>('/api/v1/auth/login', {
        email: credential.email,
        password: credential.password,
      })

      if (!response.success) {
        throw new Error(response.message || '登录失败')
      }

      const { user, access_token, refresh_token } = response.data

      const existingIndex = testUsers.value.findIndex(u => u.id === user.id)

      const testUser: TestUser = {
        id: user.id,
        username: user.username,
        email: user.email,
        role: user.role,
        accessToken: access_token,
        refreshToken: refresh_token,
        isActive: true,
      }

      if (existingIndex >= 0) {
        testUsers.value[existingIndex] = testUser
      } else {
        testUsers.value.push(testUser)
      }

      activeUserId.value = user.id

      saveToStorage()

      // 自动创建 WebSocket 连接
      try {
        const wsStore = useMultiUserWebSocketStore()
        await wsStore.createConnection(user.id, access_token)
        console.log('[MultiUserAuth] 用户 WebSocket 连接已创建:', user.username)
      } catch (wsError) {
        console.error('[MultiUserAuth] WebSocket 连接失败:', wsError)
        // WebSocket 连接失败不影响登录成功状态
      }

      return testUser
    } finally {
      loading.value = false
    }
  }

  /**
   * 批量登录用户
   * @param credentials 用户凭证数组
   * @returns 登录结果
   */
  async function loginUsers(credentials: UserCredential[]): Promise<{
    success: TestUser[]
    failed: { credential: UserCredential; error: string }[]
  }> {
    const success: TestUser[] = []
    const failed: { credential: UserCredential; error: string }[] = []

    for (const credential of credentials) {
      try {
        const user = await loginUser(credential)
        success.push(user)
      } catch (error) {
        failed.push({
          credential,
          error: error instanceof Error ? error.message : String(error),
        })
      }
    }

    return { success, failed }
  }

  /**
   * 设置当前激活用户
   * @param userId 用户ID
   */
  function setActiveUser(userId: string | null): void {
    if (userId === null) {
      activeUserId.value = null
      return
    }

    const user = testUsers.value.find(u => u.id === userId)
    if (user) {
      testUsers.value.forEach(u => { u.isActive = false })
      user.isActive = true
      activeUserId.value = userId
      saveToStorage()
    }
  }

  /**
   * 获取用户的认证头
   * @param userId 用户ID
   * @returns Authorization 头值
   */
  function getAuthHeader(userId?: string): string | null {
    const targetUserId = userId || activeUserId.value
    if (!targetUserId) return null

    const user = testUsers.value.find(u => u.id === targetUserId)
    return user ? `Bearer ${user.accessToken}` : null
  }

  /**
   * 获取指定用户
   * @param userId 用户ID
   * @returns 用户信息
   */
  function getUser(userId: string): TestUser | undefined {
    return testUsers.value.find(u => u.id === userId)
  }

  /**
   * 更新用户 token
   * @param userId 用户ID
   * @param accessToken 新的 access token
   * @param refreshToken 新的 refresh token
   */
  function updateUserToken(userId: string, accessToken: string, refreshToken?: string): void {
    const user = testUsers.value.find(u => u.id === userId)
    if (user) {
      user.accessToken = accessToken
      if (refreshToken) {
        user.refreshToken = refreshToken
      }
      saveToStorage()
    }
  }

  /**
   * 移除用户
   * @param userId 用户ID
   */
  function removeUser(userId: string): void {
    const index = testUsers.value.findIndex(u => u.id === userId)
    if (index >= 0) {
      testUsers.value.splice(index, 1)
      if (activeUserId.value === userId) {
        activeUserId.value = null
      }
      saveToStorage()
    }
  }

  /**
   * 清空所有用户
   */
  function clearUsers(): void {
    testUsers.value = []
    activeUserId.value = null
    clearStorage()
  }

  /**
   * 以指定用户身份发送 API 请求
   * @param userId 用户ID
   * @param endpoint API 端点
   * @param config 请求配置
   * @returns 响应数据
   */
  async function requestAsUser<T>(
    userId: string,
    endpoint: string,
    config: RequestInit = {}
  ): Promise<T> {
    const user = getUser(userId)
    if (!user) {
      throw new Error('用户未找到')
    }

    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      ...((config.headers as Record<string, string>) || {}),
    }

    headers['Authorization'] = `Bearer ${user.accessToken}`

    const response = await fetch(endpoint, {
      ...config,
      headers,
    })

    const result = await response.json()

    if (!response.ok) {
      throw new Error(result.message || `HTTP ${response.status}`)
    }

    return result.data
  }

  return {
    // State
    testUsers,
    activeUserId,
    loading,
    isRestored,
    // Getters
    activeUser,
    onlineUsers,
    userCount,
    // Actions
    initialize,
    loginUser,
    loginUsers,
    setActiveUser,
    getAuthHeader,
    getUser,
    updateUserToken,
    removeUser,
    clearUsers,
    requestAsUser,
  }
})
