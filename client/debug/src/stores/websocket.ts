/**
 * WebSocket Store
 * 集中管理 WebSocket 连接，提供自动重连、消息订阅等功能
 */

import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { getAccessToken } from '@/api'

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
  const ws = ref<WebSocket | null>(null)
  const status = ref<WebSocketStatus>('disconnected')
  const logs = ref<LogEntry[]>([])
  const logSubscribed = ref(false)
  const lastError = ref<string | null>(null)
  const reconnectAttempts = ref(0)
  const reconnectTimer = ref<ReturnType<typeof setTimeout> | null>(null)

  // 聊天相关状态
  const joinedRooms = ref<string[]>([])
  const currentRoom = ref<string | null>(null)
  const chatMessages = ref<ChatMessage[]>([])
  const onlineUsers = ref<number>(0)
  const latency = ref<number | null>(null)
  const lastPingTime = ref<number | null>(null)

  // ========== Constants ==========
  const MAX_RECONNECT_ATTEMPTS = 10
  const INITIAL_RECONNECT_DELAY = 1000 // 1秒
  const MAX_RECONNECT_DELAY = 30000 // 30秒
  const MAX_LOGS = 100
  const MAX_CHAT_MESSAGES = 200

  // ========== Getters ==========
  const isConnected = computed(() => status.value === 'connected')
  const isConnecting = computed(() => status.value === 'connecting' || status.value === 'reconnecting')
  const canReconnect = computed(() => reconnectAttempts.value < MAX_RECONNECT_ATTEMPTS)

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
    if (ws.value?.readyState === WebSocket.OPEN) {
      console.log('WebSocket 已连接')
      return
    }

    if (ws.value?.readyState === WebSocket.CONNECTING) {
      console.log('WebSocket 正在连接中')
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

    status.value = reconnectAttempts.value > 0 ? 'reconnecting' : 'connecting'

    const baseUrl = import.meta.env.VITE_WS_URL || 'ws://localhost:8080'
    const wsUrl = baseUrl.endsWith('/ws') ? baseUrl : `${baseUrl}/ws`

    try {
      ws.value = new WebSocket(wsUrl)

      ws.value.onopen = () => {
        console.log('WebSocket 连接成功')
        status.value = 'connected'
        reconnectAttempts.value = 0
        lastError.value = null

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
        console.log('WebSocket 连接关闭', event.code, event.reason)
        status.value = 'disconnected'
        ws.value = null

        // 如果不是正常关闭，尝试重连
        if (!event.wasClean && canReconnect.value) {
          scheduleReconnect()
        }
      }

      ws.value.onerror = (error) => {
        console.error('WebSocket 错误:', error)
        lastError.value = '连接错误'
        status.value = 'disconnected'
      }
    } catch (error) {
      console.error('创建 WebSocket 失败:', error)
      lastError.value = '创建连接失败'
      status.value = 'disconnected'
      scheduleReconnect()
    }
  }

  /**
   * 断开 WebSocket 连接
   */
  function disconnect() {
    // 清除重连定时器
    if (reconnectTimer.value) {
      clearTimeout(reconnectTimer.value)
      reconnectTimer.value = null
    }

    // 重置重连计数
    reconnectAttempts.value = 0

    // 关闭连接
    if (ws.value) {
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
      ws.value.send(JSON.stringify(message))
      return true
    }
    console.warn('WebSocket 未连接，无法发送消息')
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
          // 认证成功后自动订阅日志
          subscribeLogs()
        }
        break

      case 'Pong':
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

      case 'RoomJoined':
        joinedRooms.value.push((msg.payload as { room_id: string }).room_id)
        break

      case 'RoomLeft':
        joinedRooms.value = joinedRooms.value.filter(
          id => id !== (msg.payload as { room_id: string }).room_id
        )
        break

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
    if (reconnectAttempts.value >= MAX_RECONNECT_ATTEMPTS) {
      console.error('达到最大重连次数，停止重连')
      lastError.value = '连接失败，请刷新页面重试'
      return
    }

    reconnectAttempts.value++

    // 指数退避算法
    const delay = Math.min(
      INITIAL_RECONNECT_DELAY * Math.pow(2, reconnectAttempts.value - 1),
      MAX_RECONNECT_DELAY
    )

    console.log(`计划 ${delay}ms 后进行第 ${reconnectAttempts.value} 次重连`)

    reconnectTimer.value = setTimeout(() => {
      console.log(`执行第 ${reconnectAttempts.value} 次重连`)
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
