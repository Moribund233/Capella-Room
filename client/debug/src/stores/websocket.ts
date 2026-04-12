/**
 * WebSocket Store
 * 集中管理 WebSocket 连接，提供自动重连、消息订阅等功能
 */

import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { getAccessToken } from '@/api'
import { getClientConfig } from '@/api/system'
import type { ClientConfig } from '@/types/config'

export type WebSocketStatus = 'connecting' | 'connected' | 'disconnected' | 'reconnecting'

export interface LogEntry {
  level: string
  target: string
  message: string
  timestamp: string
  fields?: Record<string, unknown>
}

interface MessageHandler {
  type: string
  callback: (payload: unknown) => void
}

// 聊天消息
export interface ChatMessage {
  id: string
  type: 'sent' | 'received' | 'system'
  content: string
  time: string
  sender?: string
  roomId?: string
}

// 服务端配置（动态获取）
let serverConfig: ClientConfig | null = null

export const useWebSocketStore = defineStore('websocket', () => {
  // ========== State ==========
  const ws = ref<WebSocket | null>(null)
  const status = ref<WebSocketStatus>('disconnected')
  const logs = ref<LogEntry[]>([])
  const logSubscribed = ref(false)
  const lastError = ref<string | null>(null)
  const reconnectAttempts = ref(0)
  const reconnectTimer = ref<ReturnType<typeof setTimeout> | null>(null)
  const heartbeatTimer = ref<ReturnType<typeof setInterval> | null>(null)
  const lastPongTime = ref<number>(Date.now())

  // 聊天相关状态
  const joinedRooms = ref<string[]>([])
  const currentRoom = ref<string | null>(null)
  const chatMessages = ref<ChatMessage[]>([])
  const onlineUsers = ref<number>(0)
  const latency = ref<number | null>(null)
  const lastPingTime = ref<number | null>(null)

  // ========== Constants ==========
  // 从服务端配置获取，如未获取则使用默认值
  const getHeartbeatInterval = () => (serverConfig?.websocket.heartbeat_interval_secs ?? 30) * 1000
  const getHeartbeatTimeout = () => (serverConfig?.websocket.heartbeat_timeout_secs ?? 90) * 1000
  const getReconnectBaseDelay = () => serverConfig?.reconnect.base_delay_ms ?? 1000
  const getReconnectMaxDelay = () => serverConfig?.reconnect.max_delay_ms ?? 30000
  const getMaxReconnectAttempts = () => serverConfig?.reconnect.max_attempts ?? 10

  const MAX_LOGS = 100
  const MAX_CHAT_MESSAGES = 200

  // ========== Getters ==========
  const isConnected = computed(() => status.value === 'connected')
  const isConnecting = computed(() => status.value === 'connecting' || status.value === 'reconnecting')
  const canReconnect = computed(() => reconnectAttempts.value < getMaxReconnectAttempts())

  // 消息处理器列表
  const messageHandlers = ref<MessageHandler[]>([])

  // ========== Actions ==========

  /**
   * 注册消息处理器
   */
  function onMessage(type: string, callback: (payload: unknown) => void) {
    const handler: MessageHandler = { type, callback }
    messageHandlers.value.push(handler)

    // 返回取消订阅函数
    return () => {
      const index = messageHandlers.value.indexOf(handler)
      if (index > -1) {
        messageHandlers.value.splice(index, 1)
      }
    }
  }

  /**
   * 启动心跳保活
   */
  function startHeartbeat() {
    // 清除旧的心跳定时器
    if (heartbeatTimer.value) {
      clearInterval(heartbeatTimer.value)
      heartbeatTimer.value = null
    }

    // 重置最后pong时间
    lastPongTime.value = Date.now()

    const heartbeatInterval = getHeartbeatInterval()
    const heartbeatTimeout = getHeartbeatTimeout()

    // 启动心跳定时器
    heartbeatTimer.value = setInterval(() => {
      if (ws.value?.readyState === WebSocket.OPEN) {
        // 检查是否超时未收到pong
        const now = Date.now()
        if (now - lastPongTime.value > heartbeatTimeout) {
          console.warn(`心跳超时(${heartbeatTimeout}ms)，关闭连接并重连`)
          ws.value.close(1000, 'Heartbeat timeout')
          return
        }

        // 发送ping
        lastPingTime.value = now
        send({ type: 'Ping' })
      }
    }, heartbeatInterval)
  }

  /**
   * 停止心跳
   */
  function stopHeartbeat() {
    if (heartbeatTimer.value) {
      clearInterval(heartbeatTimer.value)
      heartbeatTimer.value = null
    }
  }

  /**
   * 连接到 WebSocket
   */
  function connect() {
    // 如果已经连接，不再重复连接
    if (ws.value?.readyState === WebSocket.OPEN) {
      console.log('WebSocket 已连接，跳过')
      return
    }

    // 如果正在连接中，等待
    if (ws.value?.readyState === WebSocket.CONNECTING) {
      console.log('WebSocket 正在连接中，跳过')
      return
    }

    const token = getAccessToken()
    if (!token) {
      console.warn('未登录，无法连接 WebSocket')
      lastError.value = '未登录'
      return
    }

    // 清除之前的重连定时器
    if (reconnectTimer.value) {
      clearTimeout(reconnectTimer.value)
      reconnectTimer.value = null
    }

    // 设置状态
    const wasDisconnected = status.value === 'disconnected'
    status.value = reconnectAttempts.value > 0 ? 'reconnecting' : 'connecting'

    // 如果是首次连接或完全断开后重连，重置joinedRooms
    if (wasDisconnected && reconnectAttempts.value === 0) {
      joinedRooms.value = []
    }

    const baseUrl = import.meta.env.VITE_WS_URL || 'ws://localhost:8080'
    const wsUrl = baseUrl.endsWith('/ws') ? baseUrl : `${baseUrl}/ws`

    console.log(`[WebSocket] 正在连接到: ${wsUrl}`)

    try {
      ws.value = new WebSocket(wsUrl)

      ws.value.onopen = () => {
        console.log('[WebSocket] 连接成功')
        status.value = 'connected'
        reconnectAttempts.value = 0
        lastError.value = null
        lastPongTime.value = Date.now()

        // 启动心跳
        startHeartbeat()

        // 发送认证消息
        send({
          type: 'Auth',
          payload: { token }
        })
      }

      ws.value.onmessage = (event) => {
        try {
          const msg = JSON.parse(event.data)
          handleMessage(msg)
        } catch (e) {
          console.error('解析 WebSocket 消息失败:', e)
        }
      }

      ws.value.onclose = (event) => {
        console.log('[WebSocket] 连接关闭', event.code, event.reason, 'wasClean:', event.wasClean)

        // 停止心跳
        stopHeartbeat()

        // 如果已经是断开状态，不重复处理
        if (status.value === 'disconnected') {
          return
        }

        status.value = 'disconnected'

        // 清理 WebSocket 引用（但保留重连逻辑）
        const shouldReconnect = canReconnect.value

        // 延迟清理引用，避免竞态条件
        setTimeout(() => {
          if (ws.value?.readyState === WebSocket.CLOSED) {
            ws.value = null
          }
        }, 100)

        // 如果不是正常关闭且可以重连，安排重连
        // 1000 = 正常关闭, 1001 = 离开页面
        const isNormalClose = event.code === 1000 || event.code === 1001
        if (!isNormalClose && shouldReconnect) {
          scheduleReconnect()
        }
      }

      ws.value.onerror = (error) => {
        console.error('[WebSocket] 错误:', error)
        lastError.value = '连接错误'
        // 注意：onerror 后通常会触发 onclose，所以不重连
      }
    } catch (error) {
      console.error('[WebSocket] 创建连接失败:', error)
      lastError.value = '创建连接失败'
      status.value = 'disconnected'
      scheduleReconnect()
    }
  }

  /**
   * 断开 WebSocket 连接
   */
  function disconnect() {
    console.log('[WebSocket] 主动断开连接')

    // 停止心跳
    stopHeartbeat()

    // 清除重连定时器
    if (reconnectTimer.value) {
      clearTimeout(reconnectTimer.value)
      reconnectTimer.value = null
    }

    // 重置重连计数
    reconnectAttempts.value = 0

    // 关闭连接（正常关闭，不重连）
    if (ws.value) {
      // 先移除事件处理器，避免触发重连
      ws.value.onclose = null
      ws.value.onerror = null
      ws.value.close(1000, '主动断开')
      ws.value = null
    }

    status.value = 'disconnected'
    logSubscribed.value = false
    joinedRooms.value = []
    currentRoom.value = null
  }

  /**
   * 发送消息
   */
  function send(message: { type: string; payload?: unknown }): boolean {
    if (ws.value?.readyState === WebSocket.OPEN) {
      try {
        ws.value.send(JSON.stringify(message))
        return true
      } catch (e) {
        console.error('[WebSocket] 发送消息失败:', e)
        return false
      }
    }
    console.warn('[WebSocket] 未连接，无法发送消息:', message.type)
    return false
  }

  /**
   * 订阅日志
   */
  function subscribeLogs(level: string = 'all', module: string = 'all'): boolean {
    if (send({
      type: 'SubscribeLogs',
      payload: { level, module }
    })) {
      logSubscribed.value = true
      return true
    }
    return false
  }

  /**
   * 取消订阅日志
   */
  function unsubscribeLogs(): boolean {
    if (send({ type: 'UnsubscribeLogs' })) {
      logSubscribed.value = false
      return true
    }
    return false
  }

  /**
   * 加入房间
   */
  function joinRoom(roomId: string): boolean {
    return send({
      type: 'JoinRoom',
      payload: { room_id: roomId }
    })
  }

  /**
   * 离开房间
   */
  function leaveRoom(roomId: string): boolean {
    return send({
      type: 'LeaveRoom',
      payload: { room_id: roomId }
    })
  }

  /**
   * 发送聊天消息
   */
  function sendChatMessage(roomId: string, content: string, replyTo?: string): boolean {
    return send({
      type: 'ChatMessage',
      payload: {
        room_id: roomId,
        content,
        reply_to: replyTo || null
      }
    })
  }

  /**
   * 发送正在输入状态
   */
  function sendTyping(roomId: string): boolean {
    return send({
      type: 'Typing',
      payload: { room_id: roomId }
    })
  }

  /**
   * 发送停止输入状态
   */
  function sendStopTyping(roomId: string): boolean {
    return send({
      type: 'StopTyping',
      payload: { room_id: roomId }
    })
  }

  /**
   * 发送 Ping
   */
  function ping(): boolean {
    lastPingTime.value = Date.now()
    return send({ type: 'Ping' })
  }

  /**
   * 获取在线用户
   */
  function getOnlineUsers(): boolean {
    return send({ type: 'GetOnlineUsers' })
  }

  /**
   * 添加聊天消息
   */
  function addChatMessage(msg: ChatMessage) {
    chatMessages.value.push(msg)
    // 限制消息数量
    if (chatMessages.value.length > MAX_CHAT_MESSAGES) {
      chatMessages.value = chatMessages.value.slice(-MAX_CHAT_MESSAGES)
    }
  }

  /**
   * 清空聊天消息
   */
  function clearChatMessages() {
    chatMessages.value = []
  }

  /**
   * 处理收到的消息
   */
  function handleMessage(msg: { type: string; payload?: unknown }) {
    // 调试日志
    console.log('[WebSocket] 收到消息:', msg.type)

    // 触发注册的处理器
    messageHandlers.value
      .filter(h => h.type === msg.type)
      .forEach(h => {
        try {
          h.callback(msg.payload)
        } catch (e) {
          console.error('消息处理器执行失败:', e)
        }
      })

    // 处理内置消息类型
    switch (msg.type) {
      case 'AuthResult':
        if ((msg.payload as { success: boolean }).success) {
          console.log('[WebSocket] 认证成功')
          // 认证成功后自动订阅日志
          subscribeLogs()
        } else {
          console.error('[WebSocket] 认证失败:', msg.payload)
        }
        break

      case 'Ping':
        // 收到服务端心跳，回复 Pong
        send({ type: 'Pong' })
        lastPongTime.value = Date.now()
        break

      case 'Pong':
        lastPongTime.value = Date.now()
        if (lastPingTime.value) {
          latency.value = Date.now() - lastPingTime.value
          lastPingTime.value = null
        }
        break

      case 'LogEntry':
        addLog(msg.payload as LogEntry)
        break

      case 'LogSubscriptionConfirmed':
        logSubscribed.value = (msg.payload as { success: boolean }).success
        break

      case 'RoomJoined': {
        const roomId = (msg.payload as { room_id: string }).room_id
        if (!joinedRooms.value.includes(roomId)) {
          joinedRooms.value.push(roomId)
        }
        console.log('[WebSocket] 已加入房间:', roomId)
        break
      }

      case 'RoomLeft': {
        const roomId = (msg.payload as { room_id: string }).room_id
        joinedRooms.value = joinedRooms.value.filter(id => id !== roomId)
        console.log('[WebSocket] 已离开房间:', roomId)
        break
      }

      case 'NewMessage': {
        const payload = msg.payload as {
          message_id: string
          room_id: string
          sender_name: string
          content: string
          created_at: string
        }
        addChatMessage({
          id: payload.message_id,
          type: 'received',
          content: payload.content,
          time: new Date(payload.created_at).toLocaleTimeString(),
          sender: payload.sender_name,
          roomId: payload.room_id
        })
        break
      }

      case 'GlobalOnlineUsers':
        onlineUsers.value = (msg.payload as { total: number }).total
        break
    }
  }

  /**
   * 添加日志条目
   */
  function addLog(entry: LogEntry) {
    logs.value.unshift(entry)
    // 限制日志数量
    if (logs.value.length > MAX_LOGS) {
      logs.value = logs.value.slice(0, MAX_LOGS)
    }
  }

  /**
   * 清空日志
   */
  function clearLogs() {
    logs.value = []
  }

  /**
   * 安排重连
   */
  function scheduleReconnect() {
    const maxReconnectAttempts = getMaxReconnectAttempts()
    if (reconnectAttempts.value >= maxReconnectAttempts) {
      console.error('[WebSocket] 达到最大重连次数，停止重连')
      lastError.value = '连接失败，请刷新页面重试'
      return
    }

    reconnectAttempts.value++

    // 指数退避算法
    const baseDelay = getReconnectBaseDelay()
    const maxDelay = getReconnectMaxDelay()
    const delay = Math.min(
      baseDelay * Math.pow(2, reconnectAttempts.value - 1),
      maxDelay
    )

    console.log(`[WebSocket] 计划 ${delay}ms 后进行第 ${reconnectAttempts.value} 次重连`)

    reconnectTimer.value = setTimeout(() => {
      console.log(`[WebSocket] 执行第 ${reconnectAttempts.value} 次重连`)
      connect()
    }, delay)
  }

  return {
    // State
    status,
    logs,
    logSubscribed,
    lastError,
    reconnectAttempts,
    joinedRooms,
    currentRoom,
    chatMessages,
    onlineUsers,
    latency,
    lastPingTime,

    // Getters
    isConnected,
    isConnecting,
    canReconnect,
    ws,

    // Actions
    connect,
    disconnect,
    send,
    subscribeLogs,
    unsubscribeLogs,
    onMessage,
    clearLogs,
    // 聊天相关
    joinRoom,
    leaveRoom,
    sendChatMessage,
    sendTyping,
    sendStopTyping,
    ping,
    getOnlineUsers,
    addChatMessage,
    clearChatMessages
  }
})

/**
 * 初始化 WebSocket 配置
 * 从服务端获取配置并应用到 WebSocket Store
 * 应在应用启动时调用
 */
export async function initWebSocketConfig(): Promise<void> {
  try {
    const config = await getClientConfig()
    serverConfig = config
    console.log('[WebSocket] 已加载服务端配置:', config)
  } catch (error) {
    console.warn('[WebSocket] 无法获取服务端配置，使用默认值:', error)
    // 使用默认值，serverConfig 保持为 null
  }
}
