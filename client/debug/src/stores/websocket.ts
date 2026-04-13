/**
 * WebSocket Store
 * 集中管理 WebSocket 业务逻辑和状态
 * 使用 WebSocketClient 处理连接层
 */

import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { getAccessToken } from '@/api'
import { wsClient } from '@/api/websocket'
import {
  getReconnectStrategy,
  calculateReconnectDelay,
  initWebSocketConfig as initConfig,
  isConfigInitialized,
} from '@/config/websocketConfig'
import type { WebSocketMessage } from '@/types/websocket'

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

export const useWebSocketStore = defineStore('websocket', () => {
  // ========== State ==========
  const status = ref<WebSocketStatus>('disconnected')
  const logs = ref<LogEntry[]>([])
  const logSubscribed = ref(false)
  const lastError = ref<string | null>(null)
  const reconnectAttempts = ref(0)

  // 聊天相关状态
  const joinedRooms = ref<string[]>([])
  const currentRoom = ref<string | null>(null)
  const chatMessages = ref<ChatMessage[]>([])
  const onlineUsers = ref<number>(0)
  const latency = ref<number | null>(null)
  const lastPingTime = ref<number | null>(null)
  const lastPongTime = ref<number>(Date.now())

  // ========== Constants ==========
  const MAX_LOGS = 100
  const MAX_CHAT_MESSAGES = 200

  // ========== Getters ==========
  const isConnected = computed(() => status.value === 'connected')
  const isConnecting = computed(() => status.value === 'connecting' || status.value === 'reconnecting')

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
   * 连接到 WebSocket
   */
  function connect() {
    // 如果已经连接或正在连接，跳过
    if (wsClient.isConnected() || status.value === 'connecting') {
      return
    }

    const token = getAccessToken()
    if (!token) {
      console.warn('未登录，无法连接 WebSocket')
      lastError.value = '未登录'
      return
    }

    // 设置状态
    status.value = reconnectAttempts.value > 0 ? 'reconnecting' : 'connecting'

    // 设置 WebSocketClient 事件处理器
    wsClient.setHandlers({
      onConnect: () => {
        console.log('[WebSocket] 连接成功')
        status.value = 'connected'
        reconnectAttempts.value = 0
        lastError.value = null
        lastPongTime.value = Date.now()

        // 连接成功后发送认证消息
        const token = getAccessToken()
        if (token) {
          send({
            type: 'Auth',
            payload: { token }
          })
        }
      },
      onDisconnect: () => {
        console.log('[WebSocket] 连接断开')
        status.value = 'disconnected'
        logSubscribed.value = false
      },
      onError: (error) => {
        console.error('[WebSocket] 错误:', error)
        lastError.value = '连接错误'
      },
      onMessage: (message) => {
        handleMessage(message)
      }
    })

    // 连接
    wsClient.connect().catch((error) => {
      console.error('[WebSocket] 连接失败:', error)
      status.value = 'disconnected'
      scheduleReconnect()
    })
  }

  /**
   * 断开 WebSocket 连接
   */
  function disconnect() {
    console.log('[WebSocket] 主动断开连接')

    // 重置重连计数
    reconnectAttempts.value = 0

    // 断开连接
    wsClient.disconnect()

    status.value = 'disconnected'
    logSubscribed.value = false
    joinedRooms.value = []
    currentRoom.value = null
  }

  /**
   * 发送消息
   */
  function send(message: { type: string; payload?: unknown }): boolean {
    return wsClient.send(message as WebSocketMessage)
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
  function handleMessage(msg: WebSocketMessage) {
    // 调试日志
    console.log('[WebSocket] 收到消息:', msg.type)

    // 触发注册的处理器
    messageHandlers.value
      .filter(h => h.type === msg.type)
      .forEach(h => {
        try {
          h.callback('payload' in msg ? msg.payload : undefined)
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
    const strategy = getReconnectStrategy()
    if (reconnectAttempts.value >= strategy.maxAttempts) {
      console.error('[WebSocket] 达到最大重连次数，停止重连')
      lastError.value = '连接失败，请刷新页面重试'
      return
    }

    reconnectAttempts.value++

    // 指数退避算法
    const delay = calculateReconnectDelay(reconnectAttempts.value)

    console.log(`[WebSocket] 计划 ${delay}ms 后进行第 ${reconnectAttempts.value} 次重连`)

    setTimeout(() => {
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
 * @deprecated 直接使用 @/config/websocketConfig 中的 initWebSocketConfig
 */
export async function initWebSocketConfig(): Promise<void> {
  if (!isConfigInitialized()) {
    await initConfig()
  }
}
