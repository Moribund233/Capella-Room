/**
 * WebSocket Client
 * 负责 WebSocket 连接管理、消息收发、断线重连
 * 仅包含连接层逻辑，不包含业务逻辑
 */

import { getAccessToken } from './token'
import { getConnectionConfig, getHeartbeatConfig } from '@/config/websocketConfig'
import type {
  WebSocketMessage,
  ConnectionStatus,
  WebSocketConfig,
  WebSocketEventHandlers,
} from '@/types/websocket'

// WebSocket 基础配置
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
  private customToken: string | null = null
  // 服务端主导心跳相关
  private lastPingTime: number = 0
  private heartbeatTimeoutTimer: number | null = null
  // 认证状态管理
  private isAuthenticated: boolean = false
  private authResolve: (() => void) | null = null
  private authReject: ((error: Error) => void) | null = null
  // 连接 Promise 缓存，防止重复连接
  private connectingPromise: Promise<void> | null = null
  // 标记是否已发送认证消息
  private authSent: boolean = false

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
   * 设置自定义 token（用于多用户场景）
   */
  setToken(token: string | null): void {
    this.customToken = token
  }

  /**
   * 获取 token（优先使用自定义 token）
   */
  private getToken(): string | null {
    return this.customToken ?? getAccessToken()
  }

  // ========== 连接管理 ==========

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
   * 连接 WebSocket（认证成功后 resolve）
   */
  connect(): Promise<void> {
    console.log(`[WebSocket] connect() called, status=${this.connectionStatus}, isConnected=${this.isConnected()}, isAuthenticated=${this.isAuthenticated}`)

    // 已连接且已认证，直接返回
    if (this.isConnected() && this.isAuthenticated) {
      console.log('[WebSocket] Already connected and authenticated, returning')
      return Promise.resolve()
    }

    // 如果正在连接中，返回现有的 Promise
    if (this.connectionStatus === 'connecting' && this.connectingPromise) {
      console.log('[WebSocket] Connection already in progress, returning existing promise')
      return this.connectingPromise
    }

    // 创建新的连接 Promise
    console.log('[WebSocket] Starting new connection...')
    this.connectingPromise = this.doConnect()
    return this.connectingPromise
  }

  /**
   * 实际执行连接逻辑
   */
  private doConnect(): Promise<void> {
    return new Promise((resolve, reject) => {
      this.setConnectionStatus('connecting')
      this.isAuthenticated = false
      this.authSent = false  // 重置认证发送标记
      this.authResolve = resolve
      this.authReject = reject

      const token = this.getToken()
      if (!token) {
        this.setConnectionStatus('error')
        this.isAuthenticated = false
        this.connectingPromise = null
        reject(new Error('No access token'))
        return
      }

      // 确保 URL 不以 /ws 结尾，避免重复
      const baseUrl = WS_BASE_URL.endsWith('/ws') ? WS_BASE_URL : `${WS_BASE_URL}/ws`
      // 不再在 URL 中传递 token，改为通过消息体发送
      const wsUrl = baseUrl

      try {
        this.ws = new WebSocket(wsUrl)

        // 连接超时处理
        this.connectTimer = window.setTimeout(() => {
          if (this.connectionStatus === 'connecting') {
            this.ws?.close()
            this.setConnectionStatus('error')
            this.isAuthenticated = false
            reject(new Error('Connection timeout'))
          }
        }, this.config.connectTimeout)

        this.ws.onopen = () => {
          this.clearConnectTimer()
          // 防止重复发送认证消息
          if (this.authSent) {
            console.log('[WebSocket] Auth already sent, skipping')
            return
          }
          // 连接成功后立即发送认证消息
          // 注意：这里不 resolve，等待认证成功后再 resolve
          this.authSent = true
          this.sendInternal({ type: 'Auth', payload: { token } })
          console.log('[WebSocket] Connection opened, sent authentication')
        }

        this.ws.onclose = (event) => {
          console.log(`[WebSocket] onclose called, code=${event.code}, reason=${event.reason}, wasClean=${event.wasClean}`)
          this.handleClose()
        }

        this.ws.onerror = (error) => {
          this.clearConnectTimer()
          this.setConnectionStatus('error')
          this.isAuthenticated = false
          if (this.authReject) {
            this.authReject(new Error('WebSocket error'))
            this.authReject = null
            this.authResolve = null
          }
          this.handlers.onError?.(new Error('WebSocket error'))
        }

        this.ws.onmessage = (event) => {
          this.handleMessage(event.data)
        }
      } catch (error) {
        this.setConnectionStatus('error')
        this.isAuthenticated = false
        reject(error)
      }
    })
  }

  /**
   * 断开连接
   */
  disconnect(): void {
    console.log('[WebSocket] disconnect() called')
    console.trace('[WebSocket] disconnect trace')
    this.clearAllTimers()

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
    this.setConnectionStatus('disconnected')
    this.handlers.onDisconnect?.()
  }

  /**
   * 重新连接
   */
  async reconnect(): Promise<void> {
    this.disconnect()
    return this.connect()
  }

  // ========== 消息发送 ==========

  /**
   * 发送消息（公开方法，带认证检查）
   */
  send(message: WebSocketMessage): boolean {
    // 未认证时不允许发送业务消息（除了认证相关消息）
    if (!this.isAuthenticated && !this.isAuthMessage(message)) {
      console.error('[WebSocket] Cannot send message before authentication')
      return false
    }

    if (!this.isConnected()) {
      // 未连接时加入队列
      this.messageQueue.push(message)
      return false
    }

    return this.sendInternal(message)
  }

  /**
   * 内部发送方法（不检查认证状态）
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
      console.error('Failed to send message:', error)
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

  // ========== 事件处理 ==========

  /**
   * 更新事件处理器
   */
  setHandlers(handlers: WebSocketEventHandlers): void {
    this.handlers = { ...this.handlers, ...handlers }
  }

  /**
   * 处理连接关闭
   */
  private handleClose(): void {
    this.clearAllTimers()
    this.isAuthenticated = false
    this.authSent = false  // 重置认证发送标记
    this.connectingPromise = null  // 清理连接 Promise
    this.messageQueue = []  // 清空队列，防止重连后发送旧消息
    this.setConnectionStatus('disconnected')
    this.handlers.onDisconnect?.()

    // 自动重连
    if (this.reconnectAttempts < this.config.maxReconnectAttempts) {
      this.attemptReconnect()
    }
  }

  /**
   * 尝试重连
   */
  private attemptReconnect(): void {
    this.reconnectAttempts++
    this.setConnectionStatus('reconnecting')

    this.reconnectTimer = window.setTimeout(() => {
      this.connect().catch(() => {
        // 重连失败，继续尝试
        if (this.reconnectAttempts < this.config.maxReconnectAttempts) {
          this.attemptReconnect()
        }
      })
    }, this.config.reconnectInterval)
  }

  /**
   * 处理收到的消息
   */
  private handleMessage(data: string): void {
    try {
      const message = JSON.parse(data) as WebSocketMessage

      // 处理认证结果
      if (message.type === 'AuthResult') {
        this.handleAuthResult(message)
        return
      }

      // 处理后端发送的 Ping 消息，回复 Pong 并重置超时检测
      if (message.type === 'Ping') {
        this.handleServerPing()
        return
      }

      // 处理错误消息（包括 Token 过期）
      if (message.type === 'Error') {
        const errorMsg = message.payload as { code?: string; message?: string }
        if (errorMsg.code === 'TOKEN_EXPIRED') {
          console.error('[WebSocket] Token expired, disconnecting')
          this.isAuthenticated = false
          // 触发断开连接，前端应该重新获取 token 后重连
          this.handlers.onError?.(new Error('Token expired'))
          this.disconnect()
          return
        }
      }

      this.handlers.onMessage?.(message)
    } catch (error) {
      console.error('Failed to parse message:', error)
    }
  }

  /**
   * 处理认证结果
   */
  private handleAuthResult(message: Extract<WebSocketMessage, { type: 'AuthResult' }>): void {
    const authResult = message.payload

    if (authResult.success) {
      console.log('[WebSocket] Authentication successful')
      this.isAuthenticated = true
      this.setConnectionStatus('connected')
      this.reconnectAttempts = 0
      this.connectingPromise = null  // 清理连接 Promise
      this.startHeartbeat()
      this.flushMessageQueue()

      // 认证成功后才 resolve connect() Promise
      if (this.authResolve) {
        this.authResolve()
        this.authResolve = null
        this.authReject = null
      }

      this.handlers.onConnect?.()
    } else {
      console.error('[WebSocket] Authentication failed:', authResult.message)
      this.isAuthenticated = false
      this.setConnectionStatus('error')
      this.connectingPromise = null  // 清理连接 Promise

      // 认证失败 reject connect() Promise
      if (this.authReject) {
        this.authReject(new Error(`Authentication failed: ${authResult.message}`))
        this.authResolve = null
        this.authReject = null
      }

      this.handlers.onError?.(new Error(`Authentication failed: ${authResult.message}`))
      this.disconnect()
    }
  }

  /**
   * 处理服务端发送的 Ping
   * - 回复 Pong
   * - 重置心跳超时检测
   */
  private handleServerPing(): void {
    // 回复 Pong
    this.send({ type: 'Pong' })
    this.lastPingTime = Date.now()

    // 重置超时检测
    this.resetHeartbeatTimeout()

    // 调试日志（生产环境可移除）
    console.debug('[WebSocket] Received Ping, sent Pong')
  }

  /**
   * 重置心跳超时检测
   * 如果在超时时间内未收到下一个 Ping，则认为连接已断开
   */
  private resetHeartbeatTimeout(): void {
    // 清除旧的超时定时器
    if (this.heartbeatTimeoutTimer) {
      clearTimeout(this.heartbeatTimeoutTimer)
    }

    // 获取服务端配置的超时时间（加上缓冲时间）
    const { timeoutMs } = getHeartbeatConfig()
    // 使用服务端超时时间的 80% 作为检测阈值（留出缓冲）
    const checkInterval = Math.floor(timeoutMs * 0.8)

    // 设置新的超时检测
    this.heartbeatTimeoutTimer = window.setTimeout(() => {
      console.warn('[WebSocket] Heartbeat timeout - no Ping received from server within', checkInterval, 'ms')
      // 触发重连
      this.handleClose()
    }, checkInterval)
  }

  /**
   * 设置连接状态
   */
  private setConnectionStatus(status: ConnectionStatus): void {
    this.connectionStatus = status
  }

  // ========== 定时器管理 ==========

  /**
   * 启动心跳检测（服务端主导模式）
   * 不再主动发送 Ping，而是等待服务端的 Ping 并回复 Pong
   * 同时启动超时检测，如果在配置时间内未收到 Ping，则触发重连
   */
  private startHeartbeat(): void {
    // 服务端主导模式：不主动发送心跳，只被动响应
    // 初始化超时检测（等待第一个 Ping）
    this.resetHeartbeatTimeout()
    console.log('[WebSocket] Heartbeat monitoring started (server-led mode)')
  }

  /**
   * 清空所有定时器
   */
  private clearAllTimers(): void {
    this.clearHeartbeatTimer()
    this.clearReconnectTimer()
    this.clearConnectTimer()
    this.clearHeartbeatTimeoutTimer()
  }

  /**
   * 清空心跳超时检测定时器
   */
  private clearHeartbeatTimeoutTimer(): void {
    if (this.heartbeatTimeoutTimer) {
      clearTimeout(this.heartbeatTimeoutTimer)
      this.heartbeatTimeoutTimer = null
    }
  }

  /**
   * 清空心跳定时器
   */
  private clearHeartbeatTimer(): void {
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer)
      this.heartbeatTimer = null
    }
  }

  /**
   * 清空重连定时器
   */
  private clearReconnectTimer(): void {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer)
      this.reconnectTimer = null
    }
  }

  /**
   * 清空连接超时定时器
   */
  private clearConnectTimer(): void {
    if (this.connectTimer) {
      clearTimeout(this.connectTimer)
      this.connectTimer = null
    }
  }

  /**
   * 刷新消息队列
   */
  private flushMessageQueue(): void {
    while (this.messageQueue.length > 0 && this.isConnected()) {
      const message = this.messageQueue.shift()
      if (message) {
        this.send(message)
      }
    }
  }
}

// 导出单例
export const wsClient = new WebSocketClient()

// 导出类，允许创建多个实例
export { WebSocketClient }
