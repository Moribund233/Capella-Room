/**
 * 多用户测试组合式函数
 * 提供批量用户创建、登录、WebSocket 连接等测试功能
 */

import { ref, computed } from 'vue'
import { useTestUsersStore, type TestUser } from '@/store/testUsers'
import { useWsTestStore } from '@/store/wsTest'
import {
  testUserRegister,
  testUserLogin,
  testRefreshToken,
  testLogout,
  generateTestUser,
  parseTokenExpiry,
  isTokenExpiringSoon,
} from '@/api/test'
import type { RegisterRequest } from '@/types/api'

/**
 * 批量操作结果
 */
export interface BatchOperationResult {
  success: number
  failed: number
  errors: string[]
}

/**
 * 多用户测试组合式函数
 */
export function useMultiUser() {
  const store = useTestUsersStore()
  const wsStore = useWsTestStore()

  // ========== State ==========
  const isProcessing = ref(false)
  const currentOperation = ref<string>('')
  const operationProgress = ref<{ current: number; total: number } | null>(null)

  // ========== Getters ==========

  /**
   * 用户列表
   */
  const users = computed(() => store.userList)

  /**
   * 连接统计
   */
  const stats = computed(() => store.connectionStats)

  /**
   * 是否正在处理中
   */
  const loading = computed(() => store.loading || isProcessing.value)

  // ========== Actions ==========

  /**
   * 批量创建测试用户
   * @param count 创建数量
   * @returns 操作结果
   */
  async function batchCreateUsers(count: number): Promise<BatchOperationResult> {
    const result: BatchOperationResult = { success: 0, failed: 0, errors: [] }

    isProcessing.value = true
    currentOperation.value = `创建 ${count} 个测试用户`
    operationProgress.value = { current: 0, total: count }

    try {
      for (let i = 0; i < count; i++) {
        operationProgress.value.current = i + 1

        const registerData: RegisterRequest = generateTestUser(i)
        const loginResponse = await testUserRegister(registerData)

        if (loginResponse) {
          const testUser: TestUser = {
            id: loginResponse.user.id,
            username: loginResponse.user.username,
            nickname: loginResponse.user.username,
            accessToken: loginResponse.access_token,
            refreshToken: loginResponse.refresh_token,
            tokenExpiry: parseTokenExpiry(loginResponse.access_token),
            isLoggedIn: true,
            isConnected: false,
            createdAt: Date.now(),
            lastActivityAt: Date.now(),
          }
          store.addUser(testUser)
          result.success++
        } else {
          result.failed++
          result.errors.push(`用户 ${i + 1} 创建失败`)
        }

        // 添加小延迟避免请求过快
        if (i < count - 1) {
          await delay(100)
        }
      }
    } finally {
      isProcessing.value = false
      operationProgress.value = null
      currentOperation.value = ''
    }

    return result
  }

  /**
   * 批量登录用户
   * @param userIds 用户ID列表，为空则登录所有离线用户
   * @returns 操作结果
   */
  async function batchLoginUsers(userIds?: string[]): Promise<BatchOperationResult> {
    const targetUsers = userIds
      ? store.userList.filter(u => userIds.includes(u.id))
      : store.userList.filter(u => !u.isLoggedIn)

    const result: BatchOperationResult = { success: 0, failed: 0, errors: [] }

    if (targetUsers.length === 0) {
      return result
    }

    isProcessing.value = true
    currentOperation.value = `登录 ${targetUsers.length} 个用户`
    operationProgress.value = { current: 0, total: targetUsers.length }

    try {
      for (let i = 0; i < targetUsers.length; i++) {
        const user = targetUsers[i]!
        operationProgress.value.current = i + 1

        const loginResponse = await testUserLogin(user.username, 'Test123456!')

        if (loginResponse) {
          store.updateUserLoginStatus(user.id, true, {
            accessToken: loginResponse.access_token,
            refreshToken: loginResponse.refresh_token,
            expiry: parseTokenExpiry(loginResponse.access_token),
          })
          result.success++
        } else {
          result.failed++
          result.errors.push(`用户 ${user.username} 登录失败`)
        }

        if (i < targetUsers.length - 1) {
          await delay(100)
        }
      }
    } finally {
      isProcessing.value = false
      operationProgress.value = null
      currentOperation.value = ''
    }

    return result
  }

  /**
   * 批量刷新 Token
   * @param userIds 用户ID列表，为空则刷新所有在线用户
   * @returns 操作结果
   */
  async function batchRefreshTokens(userIds?: string[]): Promise<BatchOperationResult> {
    const targetUsers = userIds
      ? store.userList.filter(u => userIds.includes(u.id) && u.isLoggedIn)
      : store.userList.filter(u => u.isLoggedIn)

    const result: BatchOperationResult = { success: 0, failed: 0, errors: [] }

    if (targetUsers.length === 0) {
      return result
    }

    isProcessing.value = true
    currentOperation.value = `刷新 ${targetUsers.length} 个用户的 Token`
    operationProgress.value = { current: 0, total: targetUsers.length }

    try {
      for (let i = 0; i < targetUsers.length; i++) {
        const user = targetUsers[i]!
        operationProgress.value.current = i + 1

        const tokenResponse = await testRefreshToken(user.refreshToken)

        if (tokenResponse) {
          store.updateUserToken(
            user.id,
            tokenResponse.access_token,
            tokenResponse.refresh_token,
            parseTokenExpiry(tokenResponse.access_token)
          )
          result.success++
        } else {
          result.failed++
          result.errors.push(`用户 ${user.username} Token 刷新失败`)
          // Token 刷新失败，标记为离线
          store.updateUserLoginStatus(user.id, false)
        }

        if (i < targetUsers.length - 1) {
          await delay(50)
        }
      }
    } finally {
      isProcessing.value = false
      operationProgress.value = null
      currentOperation.value = ''
    }

    return result
  }

  /**
   * 批量登出用户
   * @param userIds 用户ID列表，为空则登出所有在线用户
   * @returns 操作结果
   */
  async function batchLogoutUsers(userIds?: string[]): Promise<BatchOperationResult> {
    const targetUsers = userIds
      ? store.userList.filter(u => userIds.includes(u.id) && u.isLoggedIn)
      : store.userList.filter(u => u.isLoggedIn)

    const result: BatchOperationResult = { success: 0, failed: 0, errors: [] }

    if (targetUsers.length === 0) {
      return result
    }

    isProcessing.value = true
    currentOperation.value = `登出 ${targetUsers.length} 个用户`
    operationProgress.value = { current: 0, total: targetUsers.length }

    try {
      for (let i = 0; i < targetUsers.length; i++) {
        const user = targetUsers[i]!
        operationProgress.value.current = i + 1

        // 先断开 WebSocket 连接
        if (user.isConnected) {
          disconnectUserWebSocket(user.id)
        }

        const success = await testLogout(user.accessToken)

        if (success) {
          store.updateUserLoginStatus(user.id, false)
          result.success++
        } else {
          result.failed++
          result.errors.push(`用户 ${user.username} 登出失败`)
        }

        if (i < targetUsers.length - 1) {
          await delay(50)
        }
      }
    } finally {
      isProcessing.value = false
      operationProgress.value = null
      currentOperation.value = ''
    }

    return result
  }

  /**
   * 批量连接 WebSocket
   * @param userIds 用户ID列表，为空则连接所有在线用户
   * @returns 操作结果
   */
  async function batchConnectWebSocket(userIds?: string[]): Promise<BatchOperationResult> {
    const targetUsers = userIds
      ? store.userList.filter(u => userIds.includes(u.id) && u.isLoggedIn && !u.isConnected)
      : store.userList.filter(u => u.isLoggedIn && !u.isConnected)

    const result: BatchOperationResult = { success: 0, failed: 0, errors: [] }

    if (targetUsers.length === 0) {
      return result
    }

    isProcessing.value = true
    currentOperation.value = `连接 ${targetUsers.length} 个用户的 WebSocket`
    operationProgress.value = { current: 0, total: targetUsers.length }

    try {
      for (let i = 0; i < targetUsers.length; i++) {
        const user = targetUsers[i]!
        operationProgress.value.current = i + 1

        // 检查 token 是否即将过期
        if (isTokenExpiringSoon(user.accessToken, 1)) {
          // 先刷新 token
          const tokenResponse = await testRefreshToken(user.refreshToken)
          if (tokenResponse) {
            store.updateUserToken(
              user.id,
              tokenResponse.access_token,
              tokenResponse.refresh_token,
              parseTokenExpiry(tokenResponse.access_token)
            )
          } else {
            result.failed++
            result.errors.push(`用户 ${user.username} Token 刷新失败，跳过连接`)
            continue
          }
        }

        // 模拟 WebSocket 连接（实际实现需要 WebSocket 客户端）
        const connected = await connectUserWebSocket(user.id, user.accessToken)

        if (connected) {
          store.updateUserConnectionStatus(user.id, true)
          result.success++
        } else {
          result.failed++
          result.errors.push(`用户 ${user.username} WebSocket 连接失败`)
        }

        if (i < targetUsers.length - 1) {
          await delay(100)
        }
      }
    } finally {
      isProcessing.value = false
      operationProgress.value = null
      currentOperation.value = ''
    }

    return result
  }

  /**
   * 批量断开 WebSocket 连接
   * @param userIds 用户ID列表，为空则断开所有已连接用户
   * @returns 操作结果
   */
  function batchDisconnectWebSocket(userIds?: string[]): BatchOperationResult {
    const targetUsers = userIds
      ? store.userList.filter(u => userIds.includes(u.id) && u.isConnected)
      : store.userList.filter(u => u.isConnected)

    const result: BatchOperationResult = { success: 0, failed: 0, errors: [] }

    for (const user of targetUsers) {
      disconnectUserWebSocket(user.id)
      store.updateUserConnectionStatus(user.id, false)
      result.success++
    }

    return result
  }

  /**
   * 删除用户
   * @param userId 用户ID
   */
  async function deleteUser(userId: string): Promise<void> {
    const user = store.getUserById(userId)
    if (!user) return

    // 断开 WebSocket
    if (user.isConnected) {
      disconnectUserWebSocket(userId)
    }

    // 登出
    if (user.isLoggedIn) {
      await testLogout(user.accessToken)
    }

    // 从 store 移除
    store.removeUser(userId)
  }

  /**
   * 清空所有用户
   */
  async function clearAllUsers(): Promise<void> {
    // 先断开所有 WebSocket
    batchDisconnectWebSocket()

    // 登出所有用户
    await batchLogoutUsers()

    // 清空 store
    store.clearAllUsers()
  }

  // ========== WebSocket Helpers ==========

  /**
   * 连接用户 WebSocket
   */
  async function connectUserWebSocket(userId: string, _token: string): Promise<boolean> {
    const user = store.getUserById(userId)
    if (!user) return false

    try {
      const success = await wsStore.connect(user)
      return success
    } catch {
      return false
    }
  }

  /**
   * 断开用户 WebSocket
   */
  function disconnectUserWebSocket(userId: string): void {
    wsStore.disconnect(userId)
  }

  // ========== Utility ==========

  /**
   * 延迟函数
   */
  function delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms))
  }

  return {
    // State
    isProcessing,
    currentOperation,
    operationProgress,
    // Getters
    users,
    stats,
    loading,
    // Actions
    batchCreateUsers,
    batchLoginUsers,
    batchRefreshTokens,
    batchLogoutUsers,
    batchConnectWebSocket,
    batchDisconnectWebSocket,
    deleteUser,
    clearAllUsers,
  }
}
