/**
 * 多用户测试状态管理 Store
 * 用于管理测试用户的状态，数据存储在 sessionStorage 中避免干扰主应用
 */

import { ref, computed } from 'vue'
import { defineStore } from 'pinia'

/**
 * 测试用户状态
 */
export interface TestUser {
  /** 用户ID */
  id: string
  /** 用户名 */
  username: string
  /** 昵称 */
  nickname: string
  /** 访问令牌 */
  accessToken: string
  /** 刷新令牌 */
  refreshToken: string
  /** Token 过期时间 */
  tokenExpiry: number | null
  /** 登录状态 */
  isLoggedIn: boolean
  /** WebSocket 连接状态 */
  isConnected: boolean
  /** 创建时间 */
  createdAt: number
  /** 最后活动时间 */
  lastActivityAt: number
}

/**
 * 连接统计信息
 */
export interface ConnectionStats {
  /** 总用户数 */
  total: number
  /** 在线用户数 */
  online: number
  /** WebSocket 连接数 */
  connected: number
  /** 离线用户数 */
  offline: number
}

/**
 * 导出的用户凭据（不含敏感信息）
 */
export interface ExportedUserCredential {
  /** 用户名 */
  username: string
  /** 密码 */
  password: string
  /** 创建时间 */
  createdAt: number
}

const STORAGE_KEY = 'seredeli_test_users'

export const useTestUsersStore = defineStore('testUsers', () => {
  // ========== State ==========
  const users = ref<TestUser[]>(loadUsersFromStorage())
  const loading = ref(false)
  const currentOperation = ref<string>('')

  // ========== Getters ==========

  /**
   * 用户列表（按创建时间排序）
   */
  const userList = computed(() => {
    return [...users.value].sort((a, b) => a.createdAt - b.createdAt)
  })

  /**
   * 连接统计
   */
  const connectionStats = computed<ConnectionStats>(() => {
    const total = users.value.length
    const online = users.value.filter(u => u.isLoggedIn).length
    const connected = users.value.filter(u => u.isConnected).length
    const offline = total - online

    return { total, online, connected, offline }
  })

  /**
   * 是否为空
   */
  const isEmpty = computed(() => users.value.length === 0)

  /**
   * 是否有用户在线
   */
  const hasOnlineUsers = computed(() =>
    users.value.some(u => u.isLoggedIn)
  )

  /**
   * 获取指定用户
   */
  function getUserById(id: string): TestUser | undefined {
    return users.value.find(u => u.id === id)
  }

  // ========== Actions ==========

  /**
   * 添加用户
   */
  function addUser(user: TestUser): void {
    users.value.push(user)
    saveUsersToStorage()
  }

  /**
   * 更新用户信息
   */
  function updateUser(id: string, updates: Partial<TestUser>): void {
    const index = users.value.findIndex(u => u.id === id)
    if (index !== -1) {
      const user = users.value[index]!
      users.value[index] = { ...user, ...updates }
      saveUsersToStorage()
    }
  }

  /**
   * 更新用户登录状态
   */
  function updateUserLoginStatus(
    id: string,
    isLoggedIn: boolean,
    tokens?: { accessToken: string; refreshToken: string; expiry: number | null }
  ): void {
    const index = users.value.findIndex(u => u.id === id)
    if (index !== -1) {
      const user = users.value[index]!
      user.isLoggedIn = isLoggedIn
      user.lastActivityAt = Date.now()

      if (tokens) {
        user.accessToken = tokens.accessToken
        user.refreshToken = tokens.refreshToken
        user.tokenExpiry = tokens.expiry
      }

      saveUsersToStorage()
    }
  }

  /**
   * 更新用户 WebSocket 连接状态
   */
  function updateUserConnectionStatus(id: string, isConnected: boolean): void {
    const index = users.value.findIndex(u => u.id === id)
    if (index !== -1) {
      const user = users.value[index]!
      user.isConnected = isConnected
      user.lastActivityAt = Date.now()
      saveUsersToStorage()
    }
  }

  /**
   * 更新用户 Token
   */
  function updateUserToken(
    id: string,
    accessToken: string,
    refreshToken: string,
    expiry: number | null
  ): void {
    const index = users.value.findIndex(u => u.id === id)
    if (index !== -1) {
      const user = users.value[index]!
      user.accessToken = accessToken
      user.refreshToken = refreshToken
      user.tokenExpiry = expiry
      user.lastActivityAt = Date.now()
      saveUsersToStorage()
    }
  }

  /**
   * 移除用户
   */
  function removeUser(id: string): void {
    users.value = users.value.filter(u => u.id !== id)
    saveUsersToStorage()
  }

  /**
   * 清空所有用户
   */
  function clearAllUsers(): void {
    users.value = []
    saveUsersToStorage()
  }

  /**
   * 导出用户凭据
   * @returns 用户凭据列表
   */
  function exportUserCredentials(): ExportedUserCredential[] {
    return users.value.map(user => ({
      username: user.username,
      password: 'Test123456!',
      createdAt: user.createdAt,
    }))
  }

  /**
   * 从凭据导入用户
   * @param credentials 用户凭据列表
   * @returns 导入的用户数量
   */
  function importUserCredentials(credentials: ExportedUserCredential[]): number {
    let importedCount = 0
    for (const cred of credentials) {
      // 检查是否已存在
      const exists = users.value.some(u => u.username === cred.username)
      if (!exists) {
        const newUser: TestUser = {
          id: crypto.randomUUID(),
          username: cred.username,
          nickname: cred.username,
          accessToken: '',
          refreshToken: '',
          tokenExpiry: null,
          isLoggedIn: false,
          isConnected: false,
          createdAt: cred.createdAt || Date.now(),
          lastActivityAt: Date.now(),
        }
        users.value.push(newUser)
        importedCount++
      }
    }
    if (importedCount > 0) {
      saveUsersToStorage()
    }
    return importedCount
  }

  /**
   * 设置加载状态
   */
  function setLoading(value: boolean, operation: string = ''): void {
    loading.value = value
    currentOperation.value = operation
  }

  // ========== Storage Helpers ==========

  /**
   * 从 sessionStorage 加载用户
   */
  function loadUsersFromStorage(): TestUser[] {
    if (typeof window === 'undefined') return []

    try {
      const data = sessionStorage.getItem(STORAGE_KEY)
      if (data) {
        return JSON.parse(data)
      }
    } catch {
      // 解析失败，返回空数组
    }
    return []
  }

  /**
   * 保存用户到 sessionStorage
   */
  function saveUsersToStorage(): void {
    if (typeof window === 'undefined') return

    try {
      sessionStorage.setItem(STORAGE_KEY, JSON.stringify(users.value))
    } catch {
      // 保存失败，忽略错误
    }
  }

  return {
    // State
    users,
    loading,
    currentOperation,
    // Getters
    userList,
    connectionStats,
    isEmpty,
    hasOnlineUsers,
    getUserById,
    // Actions
    addUser,
    updateUser,
    updateUserLoginStatus,
    updateUserConnectionStatus,
    updateUserToken,
    removeUser,
    clearAllUsers,
    exportUserCredentials,
    importUserCredentials,
    setLoading,
  }
})
