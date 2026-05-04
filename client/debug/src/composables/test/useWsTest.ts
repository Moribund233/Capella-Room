/**
 * WebSocket测试组合式函数
 * 提供WebSocket测试的批量操作和业务逻辑
 */

import { ref, computed } from 'vue'
import { useWsTestStore, type LatencyStats } from '@/store/wsTest'
import { useTestUsersStore } from '@/store/testUsers'
import { useMessage } from 'naive-ui'

/**
 * 批量操作结果
 */
export interface BatchOperationResult {
  success: number
  failed: number
  errors: string[]
}

/**
 * 延迟测试配置
 */
export interface LatencyTestConfig {
  /** 测试次数 */
  count: number
  /** 间隔(ms) */
  interval: number
  /** 是否批量测试所有用户 */
  batchMode: boolean
}

/**
 * 稳定性测试配置
 */
export interface StabilityTestConfig {
  /** 测试时长(分钟) */
  duration: number
  /** 自动重连 */
  autoReconnect: boolean
}

/**
 * 压力测试配置
 */
export interface StressTestConfig {
  /** 并发用户数 */
  concurrentUsers: number
  /** 每个用户发送消息数 */
  messagesPerUser: number
  /** 发送间隔(ms) */
  interval: number
  /** 目标房间ID */
  roomId: string
}

/**
 * WebSocket测试组合式函数
 */
export function useWsTest() {
  const message = useMessage()
  const wsStore = useWsTestStore()
  const testUsersStore = useTestUsersStore()

  // ========== State ==========
  const isProcessing = ref(false)
  const currentOperation = ref<string>('')
  const operationProgress = ref<{ current: number; total: number } | null>(null)

  // 延迟测试定时器
  const latencyTestTimer = ref<number | null>(null)
  // 压力测试定时器
  const stressTestTimers = ref<number[]>([])

  // ========== Getters ==========

  /**
   * 连接列表
   */
  const connections = computed(() => wsStore.connectionList)

  /**
   * 连接统计
   */
  const connectionStats = computed(() => wsStore.connectionStats)

  /**
   * 延迟测试统计
   */
  const latencyStats = computed<LatencyStats>(() => wsStore.latencyStats)

  /**
   * 是否正在处理中
   */
  const loading = computed(() => wsStore.loading || isProcessing.value)

  /**
   * 已登录且未连接的用户（可用于连接）
   */
  const availableUsers = computed(() =>
    testUsersStore.userList.filter(u => u.isLoggedIn && !u.isConnected)
  )

  /**
   * 已连接的用户
   */
  const connectedUsers = computed(() =>
    testUsersStore.userList.filter(u => u.isConnected)
  )

  // ========== Actions ==========

  /**
   * 延迟函数
   */
  function delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms))
  }

  /**
   * 批量连接WebSocket
   */
  async function batchConnect(userIds?: string[]): Promise<BatchOperationResult> {
    const targetUsers = userIds
      ? testUsersStore.userList.filter(u => userIds.includes(u.id) && u.isLoggedIn && !u.isConnected)
      : availableUsers.value

    const result: BatchOperationResult = { success: 0, failed: 0, errors: [] }

    if (targetUsers.length === 0) {
      message.warning('没有可连接的用户')
      return result
    }

    isProcessing.value = true
    currentOperation.value = `连接 ${targetUsers.length} 个用户的 WebSocket`
    operationProgress.value = { current: 0, total: targetUsers.length }

    try {
      for (let i = 0; i < targetUsers.length; i++) {
        const user = targetUsers[i]!
        operationProgress.value.current = i + 1

        const success = await wsStore.connect(user)

        if (success) {
          result.success++
        } else {
          result.failed++
          result.errors.push(`用户 ${user.username} 连接失败`)
        }

        // 添加小延迟避免连接过快
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
   * 批量断开WebSocket连接
   */
  function batchDisconnect(userIds?: string[]): BatchOperationResult {
    const targetUsers = userIds
      ? testUsersStore.userList.filter(u => userIds.includes(u.id) && u.isConnected)
      : connectedUsers.value

    const result: BatchOperationResult = { success: 0, failed: 0, errors: [] }

    for (const user of targetUsers) {
      wsStore.disconnect(user.id)
      result.success++
    }

    return result
  }

  /**
   * 批量加入房间
   */
  async function batchJoinRoom(roomId: string, userIds?: string[]): Promise<BatchOperationResult> {
    const targetUsers = userIds
      ? connectedUsers.value.filter(u => userIds.includes(u.id))
      : connectedUsers.value

    const result: BatchOperationResult = { success: 0, failed: 0, errors: [] }

    if (targetUsers.length === 0) {
      message.warning('没有已连接的用户')
      return result
    }

    isProcessing.value = true
    currentOperation.value = `加入房间 ${roomId}`
    operationProgress.value = { current: 0, total: targetUsers.length }

    try {
      for (let i = 0; i < targetUsers.length; i++) {
        const user = targetUsers[i]!
        operationProgress.value.current = i + 1

        wsStore.joinRoom(user.id, roomId)
        result.success++

        if (i < targetUsers.length - 1) {
          await delay(50)
        }
      }

      wsStore.currentRoomId = roomId
    } finally {
      isProcessing.value = false
      operationProgress.value = null
      currentOperation.value = ''
    }

    return result
  }

  /**
   * 批量离开房间
   */
  async function batchLeaveRoom(roomId: string, userIds?: string[]): Promise<BatchOperationResult> {
    const targetUsers = userIds
      ? connectedUsers.value.filter(u => userIds.includes(u.id))
      : connectedUsers.value

    const result: BatchOperationResult = { success: 0, failed: 0, errors: [] }

    for (const user of targetUsers) {
      wsStore.leaveRoom(user.id, roomId)
      result.success++
      await delay(50)
    }

    return result
  }

  // ========== 延迟测试 ==========

  /**
   * 开始延迟测试
   */
  async function startLatencyTest(config: LatencyTestConfig): Promise<void> {
    const targetUsers = config.batchMode
      ? connectedUsers.value
      : connectedUsers.value.slice(0, 1)

    if (targetUsers.length === 0) {
      message.warning('没有已连接的用户，请先连接WebSocket')
      return
    }

    // 清空之前的结果
    wsStore.clearLatencyResults()

    isProcessing.value = true
    currentOperation.value = `延迟测试 (${config.count}次)`

    let completedCount = 0
    const totalCount = targetUsers.length * config.count

    try {
      for (let i = 0; i < config.count; i++) {
        for (const user of targetUsers) {
          wsStore.sendPing(user.id)
          completedCount++
          operationProgress.value = { current: completedCount, total: totalCount }
          await delay(config.interval)
        }
      }

      // 等待最后的Pong响应
      await delay(1000)

      message.success(`延迟测试完成，共收集 ${wsStore.latencyResults.length} 条数据`)
    } finally {
      isProcessing.value = false
      operationProgress.value = null
      currentOperation.value = ''
    }
  }

  /**
   * 停止延迟测试
   */
  function stopLatencyTest(): void {
    if (latencyTestTimer.value) {
      clearInterval(latencyTestTimer.value)
      latencyTestTimer.value = null
    }
    isProcessing.value = false
  }

  // ========== 稳定性测试 ==========

  /**
   * 开始稳定性测试
   */
  function startStabilityTest(config: StabilityTestConfig): void {
    const durationMs = config.duration * 60 * 1000

    wsStore.startStabilityTest(durationMs)
    message.success(`稳定性测试开始，将持续 ${config.duration} 分钟`)

    // 设置自动停止
    setTimeout(() => {
      if (wsStore.stabilityTest.isRunning) {
        stopStabilityTest()
        message.success('稳定性测试已完成')
      }
    }, durationMs)
  }

  /**
   * 停止稳定性测试
   */
  function stopStabilityTest(): void {
    wsStore.stopStabilityTest()
  }

  // ========== 压力测试 ==========

  /**
   * 开始压力测试
   */
  async function startStressTest(config: StressTestConfig): Promise<void> {
    const targetUsers = connectedUsers.value.slice(0, config.concurrentUsers)

    if (targetUsers.length === 0) {
      message.warning('没有已连接的用户，请先连接WebSocket')
      return
    }

    if (!config.roomId) {
      message.warning('请选择目标房间')
      return
    }

    // 先让所有用户加入房间
    await batchJoinRoom(config.roomId, targetUsers.map(u => u.id))

    const totalMessages = targetUsers.length * config.messagesPerUser
    wsStore.startStressTest(totalMessages)

    isProcessing.value = true
    currentOperation.value = `压力测试 (${totalMessages}条消息)`

    const startTime = Date.now()
    let sentCount = 0

    try {
      // 为每个用户创建发送定时器
      for (const user of targetUsers) {
        const timer = window.setInterval(() => {
          if (sentCount >= totalMessages || !wsStore.stressTest.isRunning) {
            return
          }

          const success = wsStore.sendMessage(user.id, {
            type: 'ChatMessage',
            payload: {
              room_id: config.roomId,
              content: `压力测试消息 ${Date.now()}`,
            },
          })

          if (success) {
            wsStore.recordMessageSent()
            sentCount++
          } else {
            wsStore.recordMessageFailed()
          }

          operationProgress.value = { current: sentCount, total: totalMessages }

          // 计算发送速率
          const elapsed = (Date.now() - startTime) / 1000
          if (elapsed > 0) {
            wsStore.updateSendRate(Math.round(sentCount / elapsed))
          }
        }, config.interval)

        stressTestTimers.value.push(timer)
      }

      // 等待所有消息发送完成
      while (sentCount < totalMessages && wsStore.stressTest.isRunning) {
        await delay(100)
      }

      // 等待响应
      await delay(2000)

      const { sentCount: finalSent, receivedCount, failedCount } = wsStore.stressTest
      const successRate = finalSent > 0 ? ((finalSent - failedCount) / finalSent * 100).toFixed(2) : '0'

      message.success(`压力测试完成: 发送${finalSent}条, 接收${receivedCount}条, 成功率${successRate}%`)
    } finally {
      stopStressTest()
      isProcessing.value = false
      operationProgress.value = null
      currentOperation.value = ''
    }
  }

  /**
   * 停止压力测试
   */
  function stopStressTest(): void {
    // 清除所有定时器
    for (const timer of stressTestTimers.value) {
      clearInterval(timer)
    }
    stressTestTimers.value = []

    wsStore.stopStressTest()
  }

  // ========== 消息类型测试 ==========

  /**
   * 发送聊天消息
   */
  function sendChatMessage(userId: string, roomId: string, content: string): boolean {
    return wsStore.sendMessage(userId, {
      type: 'ChatMessage',
      payload: {
        room_id: roomId,
        content,
      },
    })
  }

  /**
   * 发送正在输入状态
   */
  function sendTyping(userId: string, roomId: string): boolean {
    return wsStore.sendMessage(userId, {
      type: 'Typing',
      payload: {
        room_id: roomId,
      },
    })
  }

  /**
   * 发送停止输入状态
   */
  function sendStopTyping(userId: string, roomId: string): boolean {
    return wsStore.sendMessage(userId, {
      type: 'StopTyping',
      payload: {
        room_id: roomId,
      },
    })
  }

  /**
   * 发送消息已读确认
   */
  function sendMessageRead(userId: string, messageId: string): boolean {
    return wsStore.sendMessage(userId, {
      type: 'MessageRead',
      payload: {
        message_id: messageId,
      },
    })
  }

  /**
   * 发送编辑消息
   */
  function sendEditMessage(userId: string, messageId: string, newContent: string): boolean {
    return wsStore.sendMessage(userId, {
      type: 'EditMessage',
      payload: {
        message_id: messageId,
        content: newContent,
      },
    })
  }

  /**
   * 发送删除消息
   */
  function sendDeleteMessage(userId: string, messageId: string): boolean {
    return wsStore.sendMessage(userId, {
      type: 'DeleteMessage',
      payload: {
        message_id: messageId,
      },
    })
  }

  // ========== 导出结果 ==========

  /**
   * 导出测试结果
   */
  function exportResults(): void {
    const data = wsStore.exportResults()
    const blob = new Blob([data], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `websocket-test-results-${Date.now()}.json`
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)

    message.success('测试结果已导出')
  }

  /**
   * 清空所有数据
   */
  function clearAll(): void {
    wsStore.disconnectAll()
    wsStore.clearLogs()
    wsStore.clearLatencyResults()
    wsStore.clearStressTest()
    message.success('已清空所有数据')
  }

  return {
    // State
    isProcessing,
    currentOperation,
    operationProgress,
    // Getters
    connections,
    connectionStats,
    latencyStats,
    loading,
    availableUsers,
    connectedUsers,
    // Actions
    batchConnect,
    batchDisconnect,
    batchJoinRoom,
    batchLeaveRoom,
    startLatencyTest,
    stopLatencyTest,
    startStabilityTest,
    stopStabilityTest,
    startStressTest,
    stopStressTest,
    sendChatMessage,
    sendTyping,
    sendStopTyping,
    sendMessageRead,
    sendEditMessage,
    sendDeleteMessage,
    exportResults,
    clearAll,
  }
}
