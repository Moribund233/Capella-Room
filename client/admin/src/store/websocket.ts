import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type {
  WsMessage,
  WsAuthMessage,
  WsAuthResult,
  WsConnectionState,
} from '@/types'
import { getClientConfig, getHeartbeatTimeoutMs, type ClientConfig } from '@/api/config'

/**
 * WebSocket 基础 URL
 */
const WS_BASE_URL = import.meta.env.VITE_WS_BASE_URL || 'ws://localhost:8080/ws'

/**
 * 消息处理器类型
 */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
type MessageHandler = (payload: any) => void

/**
 * WebSocket Store
 *
 * 统一管理 WebSocket 连接状态、消息订阅和连接生命周期
 */
export const useWebSocketStore = defineStore('websocket', () => {
  // ========== 私有状态 ==========

  /** WebSocket 实例 */
  let ws: WebSocket | null = null

  /** 重连尝试次数 */
  let reconnectAttempts = 0

  /** 心跳超时定时器 */
  let heartbeatTimeout: number | null = null

  /** 重连定时器 */
  let reconnectTimer: number | null = null

  /** 客户端配置 */
  let clientConfig: ClientConfig | null = null

  /** 最后收到心跳时间 */
  let lastPongTime = 0

  /** 当前使用的访问令牌 */
  let currentToken: string | null = null

  /** 消息处理器映射表 */
  const messageHandlers = new Map<string, MessageHandler[]>()

  // ========== 响应式状态 ==========

  /** 连接状态 */
  const connectionState = ref<WsConnectionState>('disconnected')

  /** 是否已连接 */
  const isConnected = computed(() =>
    connectionState.value === 'connected' || connectionState.value === 'authenticated'
  )

  /** 是否已认证 */
  const isAuthenticated = computed(() => connectionState.value === 'authenticated')

  /** 是否正在连接 */
  const isConnecting = computed(() => connectionState.value === 'connecting')

  /** 是否正在重连 */
  const isReconnecting = computed(() => connectionState.value === 'reconnecting')

  /** 重连次数 */
  const reconnectCount = computed(() => reconnectAttempts)

  // ========== 私有方法 ==========

  /**
   * 获取重连配置
   */
  function getReconnectConfig() {
    return clientConfig?.reconnect
  }

  /**
   * 加载客户端配置
   */
  async function loadConfig(): Promise<void> {
    try {
      const res = await getClientConfig()
      if (res.success && res.data) {
        clientConfig = res.data
        console.log('[WebSocket Store] 配置已加载:', clientConfig)
      }
    } catch (error) {
      console.error('[WebSocket Store] 加载配置失败，使用默认值:', error)
    }
  }

  /**
   * 发送认证消息
   * @param token JWT 访问令牌
   */
  function authenticate(token: string): void {
    const authMessage: WsMessage<WsAuthMessage> = {
      type: 'Auth',
      payload: { token },
    }
    send(authMessage)
  }

  /**
   * 处理收到的消息
   * @param data 消息数据
   */
  function handleMessage(data: string): void {
    try {
      const message = JSON.parse(data) as WsMessage<unknown>
      console.log('[WebSocket Store] 收到消息:', message)

      // 处理认证结果
      if (message.type === 'AuthResult') {
        const result = message.payload as WsAuthResult
        if (result.success) {
          console.log('[WebSocket Store] 认证成功')
          connectionState.value = 'authenticated'
          startHeartbeatTimeout()
        } else {
          console.error('[WebSocket Store] 认证失败:', result.error)
          connectionState.value = 'disconnected'
        }
        return
      }

      // 处理服务端发送的 Ping - 回复 Pong
      if (message.type === 'Ping') {
        send({ type: 'Pong' })
        lastPongTime = Date.now()
        console.log('[WebSocket Store] 收到 Ping，已回复 Pong')
        return
      }

      // 处理 Pong
      if (message.type === 'Pong') {
        lastPongTime = Date.now()
        return
      }

      // 分发消息到对应的处理器
      const handlers = messageHandlers.get(message.type)
      if (handlers) {
        handlers.forEach((handler) => handler(message.payload))
      }
    } catch (error) {
      console.error('[WebSocket Store] 消息解析失败:', error)
    }
  }

  /**
   * 启动心跳超时检测
   */
  function startHeartbeatTimeout(): void {
    stopHeartbeatTimeout()

    const timeoutMs = getHeartbeatTimeoutMs(clientConfig)
    lastPongTime = Date.now()

    const checkInterval = 10000 // 每10秒检查一次
    heartbeatTimeout = window.setInterval(() => {
      const elapsed = Date.now() - lastPongTime
      if (elapsed > timeoutMs) {
        console.error(`[WebSocket Store] 心跳超时，${elapsed}ms 未收到服务端消息`)
        disconnect()
        if (currentToken) {
          attemptReconnect(currentToken)
        }
      }
    }, checkInterval)

    console.log(`[WebSocket Store] 心跳超时检测已启动，超时时间: ${timeoutMs}ms`)
  }

  /**
   * 停止心跳超时检测
   */
  function stopHeartbeatTimeout(): void {
    if (heartbeatTimeout) {
      clearInterval(heartbeatTimeout)
      heartbeatTimeout = null
    }
  }

  /**
   * 尝试重新连接
   * @param token JWT 访问令牌
   */
  function attemptReconnect(token: string): void {
    const maxAttempts = getReconnectConfig()?.max_attempts || 5

    if (reconnectAttempts >= maxAttempts) {
      console.error('[WebSocket Store] 重连次数已达上限')
      return
    }

    reconnectAttempts++
    connectionState.value = 'reconnecting'

    const baseDelay = getReconnectConfig()?.base_delay_ms || 3000
    const delay = baseDelay * reconnectAttempts

    console.log(`[WebSocket Store] ${delay}ms 后尝试第 ${reconnectAttempts} 次重连...`)

    reconnectTimer = window.setTimeout(() => {
      connect(token)
    }, delay)
  }

  // ========== 公共方法 ==========

  /**
   * 建立 WebSocket 连接
   * @param token JWT 访问令牌
   */
  async function connect(token: string): Promise<void> {
    if (ws?.readyState === WebSocket.OPEN) {
      console.log('[WebSocket Store] 连接已存在')
      return
    }

    // 确保配置已加载
    if (!clientConfig) {
      await loadConfig()
    }

    // 保存令牌用于重连
    currentToken = token

    connectionState.value = 'connecting'

    try {
      ws = new WebSocket(WS_BASE_URL)

      ws.onopen = () => {
        console.log('[WebSocket Store] 连接已建立')
        connectionState.value = 'connected'
        reconnectAttempts = 0
        authenticate(token)
      }

      ws.onmessage = (event) => {
        handleMessage(event.data)
      }

      ws.onclose = () => {
        console.log('[WebSocket Store] 连接已关闭')
        connectionState.value = 'disconnected'
        stopHeartbeatTimeout()
        if (currentToken) {
          attemptReconnect(currentToken)
        }
      }

      ws.onerror = (error) => {
        console.error('[WebSocket Store] 连接错误:', error)
        connectionState.value = 'disconnected'
      }
    } catch (error) {
      console.error('[WebSocket Store] 连接失败:', error)
      connectionState.value = 'disconnected'
      if (currentToken) {
        attemptReconnect(currentToken)
      }
    }
  }

  /**
   * 断开连接
   */
  function disconnect(): void {
    stopHeartbeatTimeout()

    if (reconnectTimer) {
      clearTimeout(reconnectTimer)
      reconnectTimer = null
    }

    if (ws) {
      ws.close()
      ws = null
    }

    currentToken = null
    connectionState.value = 'disconnected'
    console.log('[WebSocket Store] 已断开连接')
  }

  /**
   * 发送消息
   * @param message WebSocket 消息
   */
  function send<T>(message: WsMessage<T>): void {
    if (ws?.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify(message))
    } else {
      console.warn('[WebSocket Store] 连接未就绪，无法发送消息')
    }
  }

  /**
   * 订阅消息
   * @param type 消息类型
   * @param handler 消息处理器
   */
  function on<T>(type: string, handler: (payload: T) => void): void {
    if (!messageHandlers.has(type)) {
      messageHandlers.set(type, [])
    }
    messageHandlers.get(type)?.push(handler as MessageHandler)
  }

  /**
   * 取消订阅
   * @param type 消息类型
   * @param handler 消息处理器
   */
  function off<T>(type: string, handler: (payload: T) => void): void {
    const handlers = messageHandlers.get(type)
    if (handlers) {
      const index = handlers.indexOf(handler as MessageHandler)
      if (index > -1) {
        handlers.splice(index, 1)
      }
    }
  }

  /**
   * 清空所有消息处理器
   */
  function clearHandlers(): void {
    messageHandlers.clear()
  }

  // ========== 返回值 ==========

  return {
    // 状态
    connectionState,
    isConnected,
    isAuthenticated,
    isConnecting,
    isReconnecting,
    reconnectCount,

    // 方法
    connect,
    disconnect,
    send,
    on,
    off,
    clearHandlers,
    loadConfig,
  }
})
