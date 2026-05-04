/**
 * WebSocket测试状态管理 Store
 * 管理多用户WebSocket连接、测试状态和日志
 */

import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { useTestUsersStore, type TestUser } from './testUsers'

/**
 * WebSocket连接状态
 */
export type WsConnectionStatus = 'disconnected' | 'connecting' | 'connected' | 'authenticated' | 'error'

/**
 * 测试用户WebSocket连接信息
 */
export interface TestWsConnection {
  /** 用户ID */
  userId: string
  /** 用户名 */
  username: string
  /** 连接状态 */
  status: WsConnectionStatus
  /** WebSocket实例 */
  ws: WebSocket | null
  /** 连接时间 */
  connectedAt: number | null
  /** 断开时间 */
  disconnectedAt: number | null
  /** 错误信息 */
  error: string | null
  /** 是否已加入房间 */
  joinedRooms: string[]
  /** 当前房间 */
  currentRoom: string | null
}

/**
 * WebSocket日志条目
 */
export interface WsLogEntry {
  /** 日志ID */
  id: string
  /** 时间戳 */
  timestamp: number
  /** 用户ID */
  userId: string
  /** 用户名 */
  username: string
  /** 日志类型 */
  type: 'send' | 'receive' | 'connect' | 'disconnect' | 'error' | 'system'
  /** 消息类型 */
  messageType?: string
  /** 日志内容 */
  content: string
  /** 原始数据 */
  rawData?: unknown
}

/**
 * 延迟测试结果
 */
export interface LatencyTestResult {
  /** 测试ID */
  id: string
  /** 用户ID */
  userId: string
  /** 用户名 */
  username: string
  /** 发送时间 */
  sentAt: number
  /** 接收时间 */
  receivedAt: number
  /** 延迟(ms) */
  latency: number
}

/**
 * 延迟测试统计
 */
export interface LatencyStats {
  /** 测试次数 */
  count: number
  /** 最小延迟 */
  min: number
  /** 最大延迟 */
  max: number
  /** 平均延迟 */
  avg: number
  /** P99延迟 */
  p99: number
}

/**
 * 稳定性测试状态
 */
export interface StabilityTest {
  /** 是否运行中 */
  isRunning: boolean
  /** 开始时间 */
  startTime: number | null
  /** 计划时长(ms) */
  duration: number
  /** 断连次数 */
  disconnectCount: number
  /** 重连次数 */
  reconnectCount: number
  /** 错误记录 */
  errors: Array<{ time: number; message: string }>
}

/**
 * 压力测试状态
 */
export interface StressTest {
  /** 是否运行中 */
  isRunning: boolean
  /** 开始时间 */
  startTime: number | null
  /** 目标消息数 */
  targetCount: number
  /** 已发送消息数 */
  sentCount: number
  /** 成功接收数 */
  receivedCount: number
  /** 失败数 */
  failedCount: number
  /** 发送速率(条/秒) */
  sendRate: number
}

const WS_BASE_URL = import.meta.env.VITE_WS_BASE_URL || 'ws://localhost:8080/ws'
const MAX_LOG_ENTRIES = 1000

export const useWsTestStore = defineStore('wsTest', () => {
  const testUsersStore = useTestUsersStore()

  // ========== State ==========
  /** WebSocket连接映射 */
  const connections = ref<Map<string, TestWsConnection>>(new Map())
  /** WebSocket日志 */
  const logs = ref<WsLogEntry[]>([])
  /** 延迟测试结果 */
  const latencyResults = ref<LatencyTestResult[]>([])
  /** 稳定性测试状态 */
  const stabilityTest = ref<StabilityTest>({
    isRunning: false,
    startTime: null,
    duration: 60000, // 默认1分钟
    disconnectCount: 0,
    reconnectCount: 0,
    errors: [],
  })
  /** 压力测试状态 */
  const stressTest = ref<StressTest>({
    isRunning: false,
    startTime: null,
    targetCount: 1000,
    sentCount: 0,
    receivedCount: 0,
    failedCount: 0,
    sendRate: 0,
  })
  /** 当前选中的房间 */
  const currentRoomId = ref<string | null>(null)
  /** 全局加载状态 */
  const loading = ref(false)
  /** 当前操作 */
  const currentOperation = ref<string>('')

  // ========== Getters ==========

  /**
   * 连接列表
   */
  const connectionList = computed(() => Array.from(connections.value.values()))

  /**
   * 已连接的连接数
   */
  const connectedCount = computed(() =>
    connectionList.value.filter(c => c.status === 'authenticated').length
  )

  /**
   * 连接统计
   */
  const connectionStats = computed(() => {
    const total = connections.value.size
    const connected = connectionList.value.filter(c => c.status === 'connected' || c.status === 'authenticated').length
    const authenticated = connectionList.value.filter(c => c.status === 'authenticated').length
    const disconnected = connectionList.value.filter(c => c.status === 'disconnected').length
    const error = connectionList.value.filter(c => c.status === 'error').length
    return { total, connected, authenticated, disconnected, error }
  })

  /**
   * 延迟测试统计
   */
  const latencyStats = computed<LatencyStats>(() => {
    const results = latencyResults.value
    if (results.length === 0) {
      return { count: 0, min: 0, max: 0, avg: 0, p99: 0 }
    }

    const latencies = results.map(r => r.latency).sort((a, b) => a - b)
    const min = latencies[0]!
    const max = latencies[latencies.length - 1]!
    const avg = latencies.reduce((a, b) => a + b, 0) / latencies.length
    const p99Index = Math.floor(latencies.length * 0.99)
    const p99 = latencies[Math.min(p99Index, latencies.length - 1)]!

    return { count: results.length, min, max, avg: Math.round(avg), p99 }
  })

  /**
   * 按用户分组的日志
   */
  const logsByUser = computed(() => {
    const grouped = new Map<string, WsLogEntry[]>()
    for (const log of logs.value) {
      const userLogs = grouped.get(log.userId) || []
      userLogs.push(log)
      grouped.set(log.userId, userLogs)
    }
    return grouped
  })

  // ========== Actions ==========

  /**
   * 生成日志ID
   */
  function generateLogId(): string {
    return `${Date.now()}_${Math.random().toString(36).substring(2, 11)}`
  }

  /**
   * 添加日志
   */
  function addLog(entry: Omit<WsLogEntry, 'id'>): void {
    const logEntry: WsLogEntry = {
      ...entry,
      id: generateLogId(),
    }
    logs.value.push(logEntry)
    // 限制日志数量
    if (logs.value.length > MAX_LOG_ENTRIES) {
      logs.value = logs.value.slice(-MAX_LOG_ENTRIES)
    }
  }

  /**
   * 清空日志
   */
  function clearLogs(): void {
    logs.value = []
  }

  /**
   * 创建WebSocket连接
   */
  function createConnection(user: TestUser): TestWsConnection {
    const existing = connections.value.get(user.id)
    if (existing) {
      return existing
    }

    const connection: TestWsConnection = {
      userId: user.id,
      username: user.username,
      status: 'disconnected',
      ws: null,
      connectedAt: null,
      disconnectedAt: null,
      error: null,
      joinedRooms: [],
      currentRoom: null,
    }
    connections.value.set(user.id, connection)
    return connection
  }

  /**
   * 连接WebSocket
   */
  async function connect(user: TestUser): Promise<boolean> {
    const connection = createConnection(user)

    if (connection.ws?.readyState === WebSocket.OPEN) {
      return true
    }

    connection.status = 'connecting'
    connection.error = null

    return new Promise((resolve) => {
      try {
        const ws = new WebSocket(WS_BASE_URL)
        connection.ws = ws

        // 连接超时处理
        const timeout = setTimeout(() => {
          connection.status = 'error'
          connection.error = '连接超时'
          ws.close()
          addLog({
            timestamp: Date.now(),
            userId: user.id,
            username: user.username,
            type: 'error',
            content: '连接超时',
          })
          resolve(false)
        }, 10000)

        ws.onopen = () => {
          clearTimeout(timeout)
          connection.status = 'connected'
          connection.connectedAt = Date.now()
          addLog({
            timestamp: Date.now(),
            userId: user.id,
            username: user.username,
            type: 'connect',
            content: 'WebSocket连接已建立',
          })

          // 发送认证消息
          ws.send(JSON.stringify({
            type: 'Auth',
            payload: { token: user.accessToken },
          }))

          addLog({
            timestamp: Date.now(),
            userId: user.id,
            username: user.username,
            type: 'send',
            messageType: 'Auth',
            content: '发送认证消息',
          })
        }

        ws.onmessage = (event) => {
          handleMessage(user.id, event.data)
        }

        ws.onclose = () => {
          clearTimeout(timeout)
          connection.status = 'disconnected'
          connection.disconnectedAt = Date.now()
          connection.ws = null

          if (stabilityTest.value.isRunning) {
            stabilityTest.value.disconnectCount++
          }

          addLog({
            timestamp: Date.now(),
            userId: user.id,
            username: user.username,
            type: 'disconnect',
            content: 'WebSocket连接已关闭',
          })
        }

        ws.onerror = (error) => {
          clearTimeout(timeout)
          connection.status = 'error'
          connection.error = '连接错误'

          if (stabilityTest.value.isRunning) {
            stabilityTest.value.errors.push({
              time: Date.now(),
              message: '连接错误',
            })
          }

          addLog({
            timestamp: Date.now(),
            userId: user.id,
            username: user.username,
            type: 'error',
            content: 'WebSocket连接错误',
            rawData: error,
          })
          resolve(false)
        }
      } catch (error) {
        connection.status = 'error'
        connection.error = error instanceof Error ? error.message : '连接失败'
        addLog({
          timestamp: Date.now(),
          userId: user.id,
          username: user.username,
          type: 'error',
          content: `连接失败: ${connection.error}`,
        })
        resolve(false)
      }
    })
  }

  /**
   * 处理收到的消息
   */
  function handleMessage(userId: string, data: string): void {
    const connection = connections.value.get(userId)
    if (!connection) return

    try {
      const message = JSON.parse(data)
      const user = testUsersStore.getUserById(userId)
      const username = user?.username || 'Unknown'

      // 处理认证结果
      if (message.type === 'AuthResult') {
        if (message.payload?.success) {
          connection.status = 'authenticated'
          // 更新store中的连接状态
          testUsersStore.updateUserConnectionStatus(userId, true)
        } else {
          connection.status = 'error'
          connection.error = message.payload?.message || '认证失败'
        }
      }

      // 处理Pong消息（延迟测试）
      if (message.type === 'Pong') {
        const pendingPing = pendingPings.get(userId)
        if (pendingPing) {
          const receivedAt = Date.now()
          const latency = receivedAt - pendingPing.sentAt

          latencyResults.value.push({
            id: generateLogId(),
            userId,
            username,
            sentAt: pendingPing.sentAt,
            receivedAt,
            latency,
          })

          pendingPings.delete(userId)
        }
      }

      // 记录日志
      addLog({
        timestamp: Date.now(),
        userId,
        username,
        type: 'receive',
        messageType: message.type,
        content: `收到 ${message.type} 消息`,
        rawData: message,
      })

      // 压力测试计数
      if (stressTest.value.isRunning && message.type === 'NewMessage') {
        stressTest.value.receivedCount++
      }
    } catch {
      addLog({
        timestamp: Date.now(),
        userId,
        username: connection.username,
        type: 'error',
        content: '消息解析失败',
        rawData: data,
      })
    }
  }

  /**
   * 断开连接
   */
  function disconnect(userId: string): void {
    const connection = connections.value.get(userId)
    if (!connection) return

    if (connection.ws) {
      connection.ws.close()
      connection.ws = null
    }

    connection.status = 'disconnected'
    connection.disconnectedAt = Date.now()
    testUsersStore.updateUserConnectionStatus(userId, false)
  }

  /**
   * 断开所有连接
   */
  function disconnectAll(): void {
    for (const userId of connections.value.keys()) {
      disconnect(userId)
    }
  }

  /**
   * 发送消息
   */
  function sendMessage(userId: string, message: unknown): boolean {
    const connection = connections.value.get(userId)
    if (!connection?.ws || connection.ws.readyState !== WebSocket.OPEN) {
      return false
    }

    try {
      connection.ws.send(JSON.stringify(message))

      const msg = message as { type: string }
      addLog({
        timestamp: Date.now(),
        userId,
        username: connection.username,
        type: 'send',
        messageType: msg.type,
        content: `发送 ${msg.type} 消息`,
        rawData: message,
      })

      return true
    } catch (error) {
      addLog({
        timestamp: Date.now(),
        userId,
        username: connection.username,
        type: 'error',
        content: '发送消息失败',
        rawData: error,
      })
      return false
    }
  }

  /**
   * 加入房间
   */
  function joinRoom(userId: string, roomId: string): void {
    const connection = connections.value.get(userId)
    if (!connection) return

    sendMessage(userId, {
      type: 'JoinRoom',
      payload: { room_id: roomId },
    })

    if (!connection.joinedRooms.includes(roomId)) {
      connection.joinedRooms.push(roomId)
    }
    connection.currentRoom = roomId
  }

  /**
   * 离开房间
   */
  function leaveRoom(userId: string, roomId: string): void {
    const connection = connections.value.get(userId)
    if (!connection) return

    sendMessage(userId, {
      type: 'LeaveRoom',
      payload: { room_id: roomId },
    })

    connection.joinedRooms = connection.joinedRooms.filter(id => id !== roomId)
    if (connection.currentRoom === roomId) {
      connection.currentRoom = null
    }
  }

  // ========== 延迟测试 ==========

  const pendingPings = new Map<string, { sentAt: number }>()

  /**
   * 发送Ping测试延迟
   */
  function sendPing(userId: string): boolean {
    const connection = connections.value.get(userId)
    if (!connection || connection.status !== 'authenticated') {
      return false
    }

    pendingPings.set(userId, { sentAt: Date.now() })

    return sendMessage(userId, { type: 'Ping' })
  }

  /**
   * 清空延迟测试结果
   */
  function clearLatencyResults(): void {
    latencyResults.value = []
    pendingPings.clear()
  }

  // ========== 稳定性测试 ==========

  /**
   * 开始稳定性测试
   */
  function startStabilityTest(duration: number): void {
    stabilityTest.value = {
      isRunning: true,
      startTime: Date.now(),
      duration,
      disconnectCount: 0,
      reconnectCount: 0,
      errors: [],
    }
  }

  /**
   * 停止稳定性测试
   */
  function stopStabilityTest(): void {
    stabilityTest.value.isRunning = false
  }

  /**
   * 记录稳定性测试重连
   */
  function recordReconnect(): void {
    if (stabilityTest.value.isRunning) {
      stabilityTest.value.reconnectCount++
    }
  }

  // ========== 压力测试 ==========

  /**
   * 开始压力测试
   */
  function startStressTest(targetCount: number): void {
    stressTest.value = {
      isRunning: true,
      startTime: Date.now(),
      targetCount,
      sentCount: 0,
      receivedCount: 0,
      failedCount: 0,
      sendRate: 0,
    }
  }

  /**
   * 停止压力测试
   */
  function stopStressTest(): void {
    stressTest.value.isRunning = false
  }

  /**
   * 记录发送消息
   */
  function recordMessageSent(): void {
    if (stressTest.value.isRunning) {
      stressTest.value.sentCount++
    }
  }

  /**
   * 记录发送失败
   */
  function recordMessageFailed(): void {
    if (stressTest.value.isRunning) {
      stressTest.value.failedCount++
    }
  }

  /**
   * 更新发送速率
   */
  function updateSendRate(rate: number): void {
    stressTest.value.sendRate = rate
  }

  /**
   * 清空压力测试数据
   */
  function clearStressTest(): void {
    stressTest.value.sentCount = 0
    stressTest.value.receivedCount = 0
    stressTest.value.failedCount = 0
    stressTest.value.sendRate = 0
  }

  // ========== 导出结果 ==========

  /**
   * 导出测试结果
   */
  function exportResults(): string {
    const results = {
      timestamp: Date.now(),
      connections: connectionStats.value,
      latency: latencyStats.value,
      stability: stabilityTest.value,
      stress: stressTest.value,
      logs: logs.value.slice(-100), // 最近100条日志
    }
    return JSON.stringify(results, null, 2)
  }

  return {
    // State
    connections,
    logs,
    latencyResults,
    stabilityTest,
    stressTest,
    currentRoomId,
    loading,
    currentOperation,
    // Getters
    connectionList,
    connectedCount,
    connectionStats,
    latencyStats,
    logsByUser,
    // Actions
    addLog,
    clearLogs,
    createConnection,
    connect,
    disconnect,
    disconnectAll,
    sendMessage,
    joinRoom,
    leaveRoom,
    sendPing,
    clearLatencyResults,
    startStabilityTest,
    stopStabilityTest,
    recordReconnect,
    startStressTest,
    stopStressTest,
    recordMessageSent,
    recordMessageFailed,
    updateSendRate,
    clearStressTest,
    exportResults,
  }
})
