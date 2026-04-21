/**
 * WebSocket Client
 * 负责 WebSocket 连接管理、消息收发、断线重连
 */

import { getAccessToken } from './token'
import { getConnectionConfig, calculateReconnectDelay } from '@/config/websocketConfig'
import type {
  WebSocketMessage,
  ConnectionStatus,
  WebSocketConfig,
  WebSocketEventHandlers,
} from '@/types/websocket'

const WS_BASE_URL = import.meta.env.VITE_WS_URL || 'ws://localhost:8080'

class WebSocketClient {
  private ws: WebSocket | null = null
  private config: Required<WebSocketConfig>
  private handlers: WebSocketEventHandlers
  private connectionStatus: ConnectionStatus = 'disconnected'
  private reconnectAttempts = 0
  private heartbeatTimer: number | null = null
  private reconnectTimer: number | null = null
  private connectTimer: number | null = null
  private messageQueue: WebSocketMessage[] = []
  private isAuthenticated = false
  private authResolve: (() => void) | null = null
  private authReject: ((error: Error) => void) | null = null
  private connectingPromise: Promise<void> | null = null
  private authSent = false

  constructor(config: WebSocketConfig = {}, handlers: WebSocketEventHandlers = {}) {
    const defaultConfig = getConnectionConfig()
    this.config = {
      maxReconnectAttempts: config.maxReconnectAttempts ?? defaultConfig.maxReconnectAttempts,
      reconnectInterval: config.reconnectInterval ?? defaultConfig.reconnectInterval,
      heartbeatInterval: config.heartbeatInterval ?? defaultConfig.heartbeatInterval,
      connectTimeout: config.connectTimeout ?? defaultConfig.connectTimeout,
    }
    this.handlers = handlers
  }

  /**
   * 获取当前连接状态
   */
  getStatus(): ConnectionStatus {
    return this.connectionStatus
  }

  /**
   * 是否已连接
   */
  isConnected(): boolean {
    return this.connectionStatus === 'connected' && this.ws?.readyState === WebSocket.OPEN
  }

  /**
   * 连接 WebSocket
   */
  connect(): Promise<void> {
    console.log(`[WebSocket] connect() called, status=${this.connectionStatus}, isConnected=${this.isConnected()}, isAuthenticated=${this.isAuthenticated}`)

    if (this.isConnected() && this.isAuthenticated) {
      console.log('[WebSocket] Already connected and authenticated')
      return Promise.resolve()
    }

    if (this.connectionStatus === 'connecting' && this.connectingPromise) {
      console.log('[WebSocket] Connection already in progress')
      return this.connectingPromise
    }

    const token = getAccessToken()
    if (!token) {
      return Promise.reject(new Error('未登录'))
    }

    this.connectionStatus = 'connecting'
    this.isAuthenticated = false
    this.authSent = false

    this.connectingPromise = new Promise((resolve, reject) => {
      this.authResolve = resolve
      this.authReject = reject

      try {
        // 不再在 URL 中传递 token，改为通过消息体发送
        const wsUrl = WS_BASE_URL.endsWith('/ws') ? WS_BASE_URL : `${WS_BASE_URL}/ws`
        this.ws = new WebSocket(wsUrl)

        this.connectTimer = window.setTimeout(() => {
          if (this.connectionStatus === 'connecting') {
            this.cleanup()
            this.connectionStatus = 'disconnected'
            reject(new Error('连接超时'))
          }
        }, this.config.connectTimeout)

        this.ws.onopen = () => {
          console.log('[WebSocket] 连接已建立，发送认证消息')
          // 防止重复发送认证消息
          if (this.authSent) {
            console.log('[WebSocket] Auth already sent, skipping')
            return
          }
          // 连接成功后立即发送认证消息
          this.authSent = true
          this.sendInternal({ type: 'Auth', payload: { token } })
        }

        this.ws.onmessage = (event) => {
          this.handleMessage(event.data)
        }

        this.ws.onclose = () => {
          console.log('[WebSocket] 连接已关闭')
          this.cleanup()
          this.connectionStatus = 'disconnected'
          this.isAuthenticated = false
          this.authSent = false
          this.handlers.onDisconnect?.()
          this.scheduleReconnect()
        }

        this.ws.onerror = (error) => {
          console.error('[WebSocket] 连接错误:', error)
          this.handlers.onError?.(new Error('WebSocket 连接错误'))
          if (this.connectionStatus === 'connecting') {
            reject(new Error('连接失败'))
          }
        }
      } catch (error) {
        reject(error instanceof Error ? error : new Error('连接失败'))
      }
    })

    return this.connectingPromise
  }

  /**
   * 处理收到的消息
   */
  private handleMessage(data: string): void {
    try {
      const message: WebSocketMessage = JSON.parse(data)

      // 处理认证结果
      if (message.type === 'AuthResult') {
        this.clearConnectTimer()
        const result = message.payload as { success: boolean; message?: string }
        if (result.success) {
          console.log('[WebSocket] 认证成功')
          this.isAuthenticated = true
          this.connectionStatus = 'connected'
          this.reconnectAttempts = 0
          this.handlers.onConnect?.()
          this.authResolve?.()
          this.startHeartbeat()
          this.flushMessageQueue()
        } else {
          console.error('[WebSocket] 认证失败:', result.message)
          this.cleanup()
          this.connectionStatus = 'disconnected'
          this.authReject?.(new Error(result.message || '认证失败'))
        }
        return
      }

      // 处理心跳响应
      if (message.type === 'Pong') {
        return
      }

      // 处理错误消息
      if (message.type === 'Error') {
        console.error('[WebSocket] 收到错误:', message.payload)
      }

      // 传递给业务层处理
      this.handlers.onMessage?.(message)
    } catch (error) {
      console.error('[WebSocket] 消息解析失败:', error)
    }
  }

  /**
   * 发送消息
   */
  send(message: WebSocketMessage): boolean {
    // 未认证时不允许发送业务消息（除了认证相关消息）
    if (!this.isAuthenticated && !this.isAuthMessage(message)) {
      console.error('[WebSocket] 认证前无法发送消息')
      return false
    }

    if (!this.isConnected()) {
      this.messageQueue.push(message)
      return false
    }

    return this.sendInternal(message)
  }

  /**
   * 内部发送方法
   */
  private sendInternal(message: WebSocketMessage): boolean {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
      this.messageQueue.push(message)
      return false
    }

    try {
      this.ws.send(JSON.stringify(message))
      return true
    } catch (error) {
      console.error('[WebSocket] 发送消息失败:', error)
      this.messageQueue.push(message)
      return false
    }
  }

  /**
   * 检查是否为认证相关消息
   */
  private isAuthMessage(message: WebSocketMessage): boolean {
    return message.type === 'Auth' || message.type === 'Reconnect'
  }

  /**
   * 更新事件处理器
   */
  setHandlers(handlers: WebSocketEventHandlers): void {
    this.handlers = { ...this.handlers, ...handlers }
  }

  /**
   * 断开连接
   */
  disconnect(): void {
    console.log('[WebSocket] disconnect() called')
    this.cleanup()

    if (this.ws) {
      this.ws.onclose = null
      this.ws.onerror = null
      this.ws.onmessage = null
      this.ws.onopen = null

      if (this.ws.readyState === WebSocket.OPEN || this.ws.readyState === WebSocket.CONNECTING) {
        this.ws.close()
      }

      this.ws = null
    }

    this.isAuthenticated = false
    this.authSent = false
    this.connectingPromise = null
    this.connectionStatus = 'disconnected'
    this.handlers.onDisconnect?.()
  }

  /**
   * 清理资源
   */
  private cleanup(): void {
    this.clearConnectTimer()
    this.clearHeartbeatTimer()
    this.clearReconnectTimer()
  }

  /**
   * 清除连接超时定时器
   */
  private clearConnectTimer(): void {
    if (this.connectTimer) {
      clearTimeout(this.connectTimer)
      this.connectTimer = null
    }
  }

  /**
   * 清除心跳定时器
   */
  private clearHeartbeatTimer(): void {
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer)
      this.heartbeatTimer = null
    }
  }

  /**
   * 清除重连定时器
   */
  private clearReconnectTimer(): void {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer)
      this.reconnectTimer = null
    }
  }

  /**
   * 启动心跳
   */
  private startHeartbeat(): void {
    this.clearHeartbeatTimer()
    this.heartbeatTimer = window.setInterval(() => {
      this.sendInternal({ type: 'Ping' })
    }, this.config.heartbeatInterval)
  }

  /**
   * 刷新消息队列
   */
  private flushMessageQueue(): void {
    while (this.messageQueue.length > 0) {
      const message = this.messageQueue.shift()
      if (message) {
        this.sendInternal(message)
      }
    }
  }

  /**
   * 安排重连
   */
  private scheduleReconnect(): void {
    if (this.reconnectAttempts >= this.config.maxReconnectAttempts) {
      console.log('[WebSocket] 达到最大重连次数')
      return
    }

    this.reconnectAttempts++
    const delay = calculateReconnectDelay(this.reconnectAttempts)
    console.log(`[WebSocket] ${delay}ms 后尝试重连...`)

    this.reconnectTimer = window.setTimeout(() => {
      this.connect().catch(() => {
        // 重连失败，继续等待下一次
      })
    }, delay)
  }
}

// 单例实例
let wsClient: WebSocketClient | null = null

/**
 * 获取 WebSocket 客户端实例
 */
export function getWebSocketClient(
  config?: WebSocketConfig,
  handlers?: WebSocketEventHandlers
): WebSocketClient {
  if (!wsClient) {
    wsClient = new WebSocketClient(config, handlers)
  } else if (handlers) {
    wsClient.setHandlers(handlers)
  }
  return wsClient
}

/**
 * 重置 WebSocket 客户端（用于登出等场景）
 */
export function resetWebSocketClient(): void {
  if (wsClient) {
    wsClient.disconnect()
    wsClient = null
  }
}

export { WebSocketClient }
export type { WebSocketMessage, ConnectionStatus, WebSocketConfig, WebSocketEventHandlers }
