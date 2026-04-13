/**
 * 自动认证工具
 * 用于测试场景下的多用户认证管理
 * 使用 MultiUserAuthStore，不干扰主应用的 localStorage 认证状态
 */

import { useMultiUserAuthStore, type UserCredential, type TestUser } from '@/stores/multiUserAuth'

/**
 * TOML 格式的用户凭证文件解析结果
 */
export interface CredentialFile {
  users: UserCredential[]
}

/**
 * 解析 TOML 格式的用户凭证文件内容
 * @param content TOML 文件内容
 * @returns 用户凭证数组
 */
export function parseCredentialFile(content: string): UserCredential[] {
  const users: UserCredential[] = []
  const lines = content.split('\n')

  let currentUser: Partial<UserCredential> = {}

  for (const line of lines) {
    const trimmedLine = line.trim()

    // 跳过空行和注释
    if (!trimmedLine || trimmedLine.startsWith('#')) {
      continue
    }

    // 检测新的用户块开始
    if (trimmedLine === '[[users]]') {
      if (currentUser.username && currentUser.email && currentUser.password) {
        users.push(currentUser as UserCredential)
      }
      currentUser = {}
      continue
    }

    // 解析键值对
    const match = trimmedLine.match(/^([a-zA-Z_]+)\s*=\s*"([^"]*)"$/)
    if (match) {
      const [, key, value] = match
      if (key === 'username' || key === 'email' || key === 'password') {
        currentUser[key] = value
      }
    }
  }

  // 添加最后一个用户
  if (currentUser.username && currentUser.email && currentUser.password) {
    users.push(currentUser as UserCredential)
  }

  return users
}

/**
 * 从文件加载用户凭证
 * @param file 用户上传的文件
 * @returns 用户凭证数组
 */
export async function loadCredentialsFromFile(file: File): Promise<UserCredential[]> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()

    reader.onload = (event) => {
      try {
        const content = event.target?.result as string
        const users = parseCredentialFile(content)
        resolve(users)
      } catch (error) {
        reject(new Error('解析凭证文件失败: ' + (error instanceof Error ? error.message : String(error))))
      }
    }

    reader.onerror = () => {
      reject(new Error('读取文件失败'))
    }

    reader.readAsText(file)
  })
}

/**
 * 用户认证管理器
 * 管理多个测试用户的认证状态
 * 使用 Pinia Store 实现，不干扰主应用的 localStorage
 */
export class AuthManager {
  private _store: ReturnType<typeof useMultiUserAuthStore> | null = null

  /**
   * 获取 Store 实例（延迟初始化）
   */
  private get store() {
    if (!this._store) {
      this._store = useMultiUserAuthStore()
    }
    return this._store
  }

  /**
   * 使用凭证登录用户
   * @param credential 用户凭证
   * @returns 认证后的用户信息
   */
  async authenticateUser(credential: UserCredential): Promise<TestUser> {
    return this.store.loginUser(credential)
  }

  /**
   * 批量认证用户
   * @param credentials 用户凭证数组
   * @returns 认证结果数组
   */
  async authenticateUsers(credentials: UserCredential[]): Promise<{
    success: TestUser[]
    failed: { credential: UserCredential; error: string }[]
  }> {
    return this.store.loginUsers(credentials)
  }

  /**
   * 从文件加载并认证所有用户
   * @param file TOML 凭证文件
   * @returns 认证结果
   */
  async loadAndAuthenticateFromFile(file: File): Promise<{
    success: TestUser[]
    failed: { credential: UserCredential; error: string }[]
  }> {
    const credentials = await loadCredentialsFromFile(file)
    return this.store.loginUsers(credentials)
  }

  /**
   * 获取所有已认证用户
   * @returns 已认证用户数组
   */
  getAuthenticatedUsers(): TestUser[] {
    return this.store.testUsers
  }

  /**
   * 获取指定用户
   * @param userId 用户ID
   * @returns 用户信息或 undefined
   */
  getUser(userId: string): TestUser | undefined {
    return this.store.getUser(userId)
  }

  /**
   * 设置当前用户
   * @param userId 用户ID
   */
  setCurrentUser(userId: string): void {
    this.store.setActiveUser(userId)
  }

  /**
   * 获取当前用户
   * @returns 当前用户信息
   */
  getCurrentUser(): TestUser | null {
    return this.store.activeUser
  }

  /**
   * 移除用户
   * @param userId 用户ID
   */
  removeUser(userId: string): void {
    this.store.removeUser(userId)
  }

  /**
   * 清空所有用户
   */
  clearUsers(): void {
    this.store.clearUsers()
  }

  /**
   * 获取用户数量
   * @returns 已认证用户数量
   */
  getUserCount(): number {
    return this.store.userCount
  }

  /**
   * 以指定用户身份发送 API 请求
   * @param userId 用户ID
   * @param endpoint API 端点
   * @param config 请求配置
   * @returns 响应数据
   */
  async requestAsUser<T>(userId: string, endpoint: string, config?: RequestInit): Promise<T> {
    return this.store.requestAsUser(userId, endpoint, config)
  }
}

/**
 * 创建全局认证管理器实例
 */
export const authManager = new AuthManager()

// 重新导出类型
export type { UserCredential, TestUser }
