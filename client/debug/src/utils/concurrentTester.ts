/**
 * 并发测试工具
 * 用于模拟多用户并发场景
 */

import { login, register, joinRoom, leaveRoom, getRooms } from '@/api'
import { WebSocketClient } from '@/api/websocket'
import type { TestDataSet } from './testDataGenerator'

/** 虚拟用户 */
export interface VirtualUser {
  id: string
  username: string
  email: string
  password: string
  token?: string
  wsClient?: WebSocketClient
  joinedRooms: Set<string>
  messagesSent: number
  messagesReceived: number
  errors: string[]
  isActive: boolean
}

/** 并发测试配置 */
export interface ConcurrentTestConfig {
  userCount: number
  roomsPerUser: number
  messagesPerUser: number
  concurrency: number
  rampUpTime: number
  duration: number
}

/** 并发测试任务 */
export interface ConcurrentTask {
  id: string
  name: string
  execute: (user: VirtualUser) => Promise<void>
}

/** 并发测试结果 */
export interface ConcurrentTestResult {
  startTime: Date
  endTime: Date
  duration: number
  totalUsers: number
  activeUsers: number
  totalMessages: number
  messagesPerSecond: number
  averageLatency: number
  maxLatency: number
  minLatency: number
  errors: Array<{
    userId: string
    error: string
    timestamp: Date
  }>
  userResults: Array<{
    userId: string
    messagesSent: number
    messagesReceived: number
    errors: number
    avgLatency: number
  }>
}

/** 并发测试器 */
export class ConcurrentTester {
  private users: Map<string, VirtualUser> = new Map()
  private isRunning = false
  private abortController: AbortController | null = null
  private results: ConcurrentTestResult | null = null
  private messageLatencies: number[] = []

  /**
   * 创建虚拟用户
   * @param count 用户数量
   * @param prefix 用户名前缀
   * @returns 虚拟用户列表
   */
  async createVirtualUsers(count: number, prefix: string = 'concurrent'): Promise<VirtualUser[]> {
    const users: VirtualUser[] = []

    for (let i = 0; i < count; i++) {
      const userId = `${prefix}_${Date.now()}_${i}`
      const username = `${prefix}_user_${i.toString().padStart(3, '0')}`
      const email = `${username}@concurrent.test`
      const password = 'Concurrent@123456'

      users.push({
        id: userId,
        username,
        email,
        password,
        joinedRooms: new Set(),
        messagesSent: 0,
        messagesReceived: 0,
        errors: [],
        isActive: false,
      })
    }

    return users
  }

  /**
   * 注册用户
   * @param user 虚拟用户
   */
  async registerUser(user: VirtualUser): Promise<void> {
    try {
      const result = await register({
        username: user.username,
        email: user.email,
        password: user.password,
      })
      user.token = result.access_token
    } catch (error) {
      user.errors.push(`Register failed: ${error instanceof Error ? error.message : String(error)}`)
      throw error
    }
  }

  /**
   * 登录用户
   * @param user 虚拟用户
   */
  async loginUser(user: VirtualUser): Promise<void> {
    try {
      const result = await login({
        email: user.email,
        password: user.password,
      })
      user.token = result.access_token
    } catch (error) {
      user.errors.push(`Login failed: ${error instanceof Error ? error.message : String(error)}`)
      throw error
    }
  }

  /**
   * 连接 WebSocket
   * @param user 虚拟用户
   */
  async connectWebSocket(user: VirtualUser): Promise<void> {
    if (!user.token) {
      throw new Error('User not authenticated')
    }

    return new Promise((resolve, reject) => {
      const wsClient = new WebSocketClient(
        {},
        {
          onConnect: () => {
            user.wsClient = wsClient
            user.isActive = true
            resolve()
          },
          onError: (error) => {
            user.errors.push(`WebSocket error: ${error.message}`)
            reject(error)
          },
          onMessage: () => {
            user.messagesReceived++
          },
        }
      )

      wsClient.connect().catch(reject)
    })
  }

  /**
   * 断开 WebSocket
   * @param user 虚拟用户
   */
  disconnectWebSocket(user: VirtualUser): void {
    if (user.wsClient) {
      user.wsClient.disconnect()
      user.wsClient = undefined
      user.isActive = false
    }
  }

  /**
   * 加入房间
   * @param user 虚拟用户
   * @param roomId 房间ID
   */
  async joinRoom(user: VirtualUser, roomId: string): Promise<void> {
    try {
      if (user.wsClient) {
        user.wsClient.joinRoom(roomId)
        user.joinedRooms.add(roomId)
      } else {
        await joinRoom(roomId)
        user.joinedRooms.add(roomId)
      }
    } catch (error) {
      user.errors.push(`Join room failed: ${error instanceof Error ? error.message : String(error)}`)
      throw error
    }
  }

  /**
   * 离开房间
   * @param user 虚拟用户
   * @param roomId 房间ID
   */
  async leaveRoom(user: VirtualUser, roomId: string): Promise<void> {
    try {
      if (user.wsClient) {
        user.wsClient.leaveRoom(roomId)
        user.joinedRooms.delete(roomId)
      } else {
        await leaveRoom(roomId)
        user.joinedRooms.delete(roomId)
      }
    } catch (error) {
      user.errors.push(`Leave room failed: ${error instanceof Error ? error.message : String(error)}`)
      throw error
    }
  }

  /**
   * 发送消息
   * @param user 虚拟用户
   * @param roomId 房间ID
   * @param content 消息内容
   * @returns 延迟（毫秒）
   */
  async sendMessage(user: VirtualUser, roomId: string, content: string): Promise<number> {
    const startTime = Date.now()

    try {
      if (user.wsClient) {
        user.wsClient.sendChatMessage(roomId, content)
      }
      user.messagesSent++

      const latency = Date.now() - startTime
      this.messageLatencies.push(latency)
      return latency
    } catch (error) {
      user.errors.push(`Send message failed: ${error instanceof Error ? error.message : String(error)}`)
      throw error
    }
  }

  /**
   * 执行并发测试
   * @param config 测试配置
   * @param testData 测试数据
   * @returns 测试结果
   */
  async runConcurrentTest(
    config: ConcurrentTestConfig,
    testData: TestDataSet
  ): Promise<ConcurrentTestResult> {
    if (this.isRunning) {
      throw new Error('Another concurrent test is already running')
    }

    this.isRunning = true
    this.abortController = new AbortController()
    this.messageLatencies = []

    const startTime = new Date()
    const errors: Array<{ userId: string; error: string; timestamp: Date }> = []

    try {
      // 创建虚拟用户
      const virtualUsers = await this.createVirtualUsers(config.userCount)
      virtualUsers.forEach((user) => this.users.set(user.id, user))

      // 分批注册用户（斜坡启动）
      const batchSize = config.concurrency
      const delayPerBatch = config.rampUpTime / (config.userCount / batchSize)

      for (let i = 0; i < virtualUsers.length; i += batchSize) {
        if (this.abortController.signal.aborted) break

        const batch = virtualUsers.slice(i, i + batchSize)
        await Promise.all(
          batch.map(async (user) => {
            try {
              await this.registerUser(user)
              await this.loginUser(user)
              await this.connectWebSocket(user)
            } catch (error) {
              errors.push({
                userId: user.id,
                error: error instanceof Error ? error.message : String(error),
                timestamp: new Date(),
              })
            }
          })
        )

        await this.sleep(delayPerBatch)
      }

      // 获取可用房间
      const rooms = await getRooms()
      const roomIds = rooms.map((r) => r.id).filter((id): id is string => id !== undefined)

      // 用户加入房间
      const activeUsers = Array.from(this.users.values()).filter((u) => u.isActive)
      await Promise.all(
        activeUsers.map(async (user) => {
          for (let i = 0; i < config.roomsPerUser && i < roomIds.length; i++) {
            const roomId = roomIds[i]
            if (!roomId) continue
            try {
              await this.joinRoom(user, roomId)
            } catch (error) {
              errors.push({
                userId: user.id,
                error: error instanceof Error ? error.message : String(error),
                timestamp: new Date(),
              })
            }
          }
        })
      )

      // 执行消息发送测试
      const testEndTime = Date.now() + config.duration
      const messagePromises: Promise<void>[] = []

      activeUsers.forEach((user) => {
        const promise = (async () => {
          while (Date.now() < testEndTime && !this.abortController?.signal.aborted) {
            if (user.joinedRooms.size > 0) {
              const roomId = Array.from(user.joinedRooms)[0]
              if (!roomId) continue
              const content = testData.messages[Math.floor(Math.random() * testData.messages.length)]
              if (!content) continue

              try {
                await this.sendMessage(user, roomId, content)
              } catch (error) {
                errors.push({
                  userId: user.id,
                  error: error instanceof Error ? error.message : String(error),
                  timestamp: new Date(),
                })
              }
            }

            // 随机延迟，模拟真实用户行为
            await this.sleep(Math.random() * 1000 + 500)
          }
        })()

        messagePromises.push(promise)
      })

      await Promise.all(messagePromises)

      // 清理
      activeUsers.forEach((user) => {
        this.disconnectWebSocket(user)
      })

      const endTime = new Date()
      const duration = endTime.getTime() - startTime.getTime()

      // 计算统计结果
      const totalMessages = activeUsers.reduce((sum, u) => sum + u.messagesSent, 0)
      const avgLatency =
        this.messageLatencies.length > 0
          ? this.messageLatencies.reduce((a, b) => a + b, 0) / this.messageLatencies.length
          : 0

      this.results = {
        startTime,
        endTime,
        duration,
        totalUsers: config.userCount,
        activeUsers: activeUsers.length,
        totalMessages,
        messagesPerSecond: totalMessages / (duration / 1000),
        averageLatency: avgLatency,
        maxLatency: Math.max(...this.messageLatencies, 0),
        minLatency: Math.min(...this.messageLatencies, Infinity),
        errors,
        userResults: activeUsers.map((user) => ({
          userId: user.id,
          messagesSent: user.messagesSent,
          messagesReceived: user.messagesReceived,
          errors: user.errors.length,
          avgLatency: avgLatency,
        })),
      }

      return this.results
    } finally {
      this.isRunning = false
      this.cleanup()
    }
  }

  /**
   * 停止测试
   */
  stop(): void {
    if (this.abortController) {
      this.abortController.abort()
    }
    this.isRunning = false
  }

  /**
   * 清理资源
   */
  private cleanup(): void {
    this.users.forEach((user) => {
      this.disconnectWebSocket(user)
    })
    this.users.clear()
  }

  /**
   * 延迟函数
   * @param ms 毫秒
   * @returns Promise
   */
  private sleep(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms))
  }

  /**
   * 获取测试结果
   * @returns 测试结果
   */
  getResults(): ConcurrentTestResult | null {
    return this.results
  }

  /**
   * 获取活跃用户数
   * @returns 活跃用户数
   */
  getActiveUserCount(): number {
    return Array.from(this.users.values()).filter((u) => u.isActive).length
  }
}

/**
 * 创建默认并发测试配置
 * @returns 默认配置
 */
export function createDefaultConcurrentConfig(): ConcurrentTestConfig {
  return {
    userCount: 10,
    roomsPerUser: 2,
    messagesPerUser: 50,
    concurrency: 5,
    rampUpTime: 5000,
    duration: 30000,
  }
}

/**
 * 创建压力测试配置
 * @returns 压力测试配置
 */
export function createStressTestConfig(): ConcurrentTestConfig {
  return {
    userCount: 100,
    roomsPerUser: 3,
    messagesPerUser: 100,
    concurrency: 20,
    rampUpTime: 20000,
    duration: 60000,
  }
}

export default ConcurrentTester
