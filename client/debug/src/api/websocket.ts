/**
 * WebSocket Client
 * 负责 WebSocket 连接管理、消息收发、断线重连
 */

import { getAccessToken } from './token'
import type {
  WebSocketMessage,
  ConnectionStatus,
  WebSocketConfig,
  WebSocketEventHandlers,
} from '@/types/websocket'

// WebSocket 基础配置
const WS_BASE_URL = import.meta.env.VITE_WS_URL || 'ws://localhost:8080'

// 默认配置
const DEFAULT_CONFIG: Required<WebSocketConfig> = {
  maxReconnectAttempts: 5,
  reconnectInterval: 3000,
  heartbeatInterval: 30000,
  connectTimeout: 10000,
}

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
  private joinedRooms: Set<string> = new Set()

  constructor(config: WebSocketConfig = {}, handlers: WebSocketEventHandlers = {}) {
    this.config = { ...DEFAULT_CONFIG, ...config }
    this.handlers = handlers
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
   * 连接 WebSocket
   */
  connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      if (this.isConnected()) {
        resolve()
        return
      }

      this.setConnectionStatus('connecting')

      const token = getAccessToken()
      if (!token) {
        this.setConnectionStatus('error')
        reject(new Error('No access token'))
        return
      }

      // 确保 URL 不以 /ws 结尾，避免重复
      const baseUrl = WS_BASE_URL.endsWith('/ws') ? WS_BASE_URL : `${WS_BASE_URL}/ws`
      const wsUrl = `${baseUrl}?token=${encodeURIComponent(token)}`

      try {
        this.ws = new WebSocket(wsUrl)

        // 连接超时处理
        this.connectTimer = window.setTimeout(() => {
          if (this.connectionStatus === 'connecting') {
            this.ws?.close()
            this.setConnectionStatus('error')
            reject(new Error('Connection timeout'))
          }
        }, this.config.connectTimeout)

        this.ws.onopen = () => {
          this.clearConnectTimer()
          this.setConnectionStatus('connected')
          this.reconnectAttempts = 0
          this.startHeartbeat()

          // 连接成功后立即发送认证消息
          const token = getAccessToken()
          if (token) {
            this.send({
              type: 'Auth',
              payload: { token }
            })
          }

          this.flushMessageQueue()
          this.handlers.onConnect?.()
          resolve()
        }

        this.ws.onclose = () => {
          this.handleClose()
        }

        this.ws.onerror = (error) => {
          this.clearConnectTimer()
          this.setConnectionStatus('error')
          this.handlers.onError?.(new Error('WebSocket error'))
          reject(error)
        }

        this.ws.onmessage = (event) => {
          this.handleMessage(event.data)
        }
      } catch (error) {
        this.setConnectionStatus('error')
        reject(error)
      }
    })
  }

  /**
   * 断开连接
   */
  disconnect(): void {
    this.clearAllTimers()
    this.joinedRooms.clear()

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
   * 发送消息
   */
  send(message: WebSocketMessage): boolean {
    if (!this.isConnected()) {
      // 未连接时加入队列
      this.messageQueue.push(message)
      return false
    }

    try {
      this.ws!.send(JSON.stringify(message))
      return true
    } catch (error) {
      console.error('Failed to send message:', error)
      this.messageQueue.push(message)
      return false
    }
  }

  /**
   * 发送认证消息
   */
  auth(token: string): boolean {
    return this.send({
      type: 'Auth',
      payload: { token },
    })
  }

  /**
   * 加入房间
   */
  joinRoom(roomId: string): boolean {
    const success = this.send({
      type: 'JoinRoom',
      payload: { room_id: roomId },
    })

    if (success) {
      this.joinedRooms.add(roomId)
    }

    return success
  }

  /**
   * 离开房间
   */
  leaveRoom(roomId: string): boolean {
    const success = this.send({
      type: 'LeaveRoom',
      payload: { room_id: roomId },
    })

    if (success) {
      this.joinedRooms.delete(roomId)
    }

    return success
  }

  /**
   * 发送聊天消息
   */
  sendChatMessage(roomId: string, content: string, replyTo?: string): boolean {
    return this.send({
      type: 'ChatMessage',
      payload: {
        room_id: roomId,
        content,
        reply_to: replyTo,
      },
    })
  }

  /**
   * 发送正在输入状态
   */
  sendTyping(roomId: string): boolean {
    return this.send({
      type: 'Typing',
      payload: { room_id: roomId },
    })
  }

  /**
   * 发送停止输入状态
   */
  sendStopTyping(roomId: string): boolean {
    return this.send({
      type: 'StopTyping',
      payload: { room_id: roomId },
    })
  }

  /**
   * 标记消息已读
   */
  markMessageRead(messageId: string): boolean {
    return this.send({
      type: 'MessageRead',
      payload: { message_id: messageId },
    })
  }

  /**
   * 编辑消息
   */
  editMessage(messageId: string, newContent: string): boolean {
    return this.send({
      type: 'EditMessage',
      payload: {
        message_id: messageId,
        new_content: newContent,
      },
    })
  }

  /**
   * 删除消息
   */
  deleteMessage(messageId: string): boolean {
    return this.send({
      type: 'DeleteMessage',
      payload: { message_id: messageId },
    })
  }

  /**
   * 更新用户状态
   */
  updateUserStatus(status: 'online' | 'away' | 'busy' | 'offline'): boolean {
    return this.send({
      type: 'UpdateStatus',
      payload: { status },
    })
  }

  /**
   * 获取在线用户列表
   */
  getOnlineUsers(): boolean {
    return this.send({
      type: 'GetOnlineUsers',
    })
  }

  /**
   * 发送心跳
   */
  ping(): boolean {
    return this.send({ type: 'Ping' })
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

      // 处理心跳响应
      if (message.type === 'Pong') {
        return
      }

      // 处理认证结果
      if (message.type === 'AuthResult') {
        if (!message.payload.success) {
          console.error('WebSocket auth failed:', message.payload.message)
          this.disconnect()
          return
        }
      }

      // 处理重连结果
      if (message.type === 'ReconnectResult') {
        if (message.payload.success && message.payload.rooms_to_rejoin) {
          // 自动重新加入房间
          message.payload.rooms_to_rejoin.forEach((roomId) => {
            this.joinRoom(roomId)
          })
        }
      }

      this.handlers.onMessage?.(message)
    } catch (error) {
      console.error('Failed to parse message:', error)
    }
  }

  /**
   * 设置连接状态
   */
  private setConnectionStatus(status: ConnectionStatus): void {
    this.connectionStatus = status
  }

  // ========== 定时器管理 ==========

  /**
   * 启动心跳
   */
  private startHeartbeat(): void {
    this.heartbeatTimer = window.setInterval(() => {
      if (this.isConnected()) {
        this.ping()
      }
    }, this.config.heartbeatInterval)
  }

  /**
   * 清空所有定时器
   */
  private clearAllTimers(): void {
    this.clearHeartbeatTimer()
    this.clearReconnectTimer()
    this.clearConnectTimer()
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
