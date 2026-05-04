import { buildMessage, parseMessage } from '@/types/websocket'
import type {
  WSMessageEnvelope,
  ConnectionState,
  MessageHandler,
  ConnectionStateHandler,
} from '@/types/websocket'
import type { ClientReconnectConfig } from '@/types/config'

export interface WSEventMap {
  message: WSMessageEnvelope
  'connection-state': ConnectionState
  [type: string]: unknown
}

type GenericHandler = (...args: unknown[]) => void

/** 默认重连参数（fallback） */
const DEFAULT_RECONNECT_CONFIG: ClientReconnectConfig = {
  base_delay_ms: 1000,
  max_delay_ms: 30000,
  max_attempts: 20,
  multiplier: 2,
}

class WebSocketService {
  private ws: WebSocket | null = null
  private url = ''
  private token = ''
  private reconnectAttempts = 0
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null
  private manuallyDisconnected = false
  private messageQueue: string[] = []
  private listeners = new Map<string, Set<GenericHandler>>()
  private _connectionState: ConnectionState = 'disconnected'
  private reconnectCfg: ClientReconnectConfig = { ...DEFAULT_RECONNECT_CONFIG }
  /** 外部注入的离线检查函数，返回 true 表示服务器不可达 */
  private isOfflineCheck: (() => boolean) | null = null

  get connectionState(): ConnectionState {
    return this._connectionState
  }

  /** 注入服务端下发的配置 */
  setConfig(
    reconnectCfg?: ClientReconnectConfig,
  ): void {
    if (reconnectCfg) this.reconnectCfg = reconnectCfg
  }

  /** 注入离线检查回调 */
  setOfflineCheck(fn: () => boolean): void {
    this.isOfflineCheck = fn
  }

  private setConnectionState(state: ConnectionState) {
    this._connectionState = state
    this.emit('connection-state', state)
  }

  /** 建立 WebSocket 连接 */
  connect(url: string, token: string): void {
    this.url = url
    this.token = token
    this.manuallyDisconnected = false
    this.reconnectAttempts = 0

    if (this.ws && (this.ws.readyState === WebSocket.OPEN || this.ws.readyState === WebSocket.CONNECTING)) {
      return
    }

    this.doConnect()
  }

  /** 断开 WebSocket 连接 */
  disconnect(): void {
    this.manuallyDisconnected = true
    this.reconnectAttempts = 0
    this.clearReconnectTimer()
    this.closeWs()
    this.setConnectionState('disconnected')
  }

  /** 发送消息（如果未连接则加入队列） */
  send(type: string, payload?: unknown): void {
    const message = buildMessage(type, payload)

    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(message)
    } else {
      this.messageQueue.push(message)
    }
  }

  /** 订阅消息事件 */
  on(event: string, handler: GenericHandler): void {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, new Set())
    }
    this.listeners.get(event)!.add(handler)
  }

  /** 取消订阅 */
  off(event: string, handler: GenericHandler): void {
    this.listeners.get(event)?.delete(handler)
  }

  /** 订阅特定类型的 WebSocket 消息 */
  onMessage<T>(type: string, handler: MessageHandler<T>): void {
    this.on(type, handler as GenericHandler)
  }

  /** 取消订阅特定类型的消息 */
  offMessage<T>(type: string, handler: MessageHandler<T>): void {
    this.off(type, handler as GenericHandler)
  }

  /** 订阅连接状态变化 */
  onConnectionState(handler: ConnectionStateHandler): void {
    this.on('connection-state', handler as GenericHandler)
  }

  /** 取消订阅连接状态 */
  offConnectionState(handler: ConnectionStateHandler): void {
    this.off('connection-state', handler as GenericHandler)
  }

  /** 刷新 token 并重新认证（连接不中断） */
  updateToken(newToken: string): void {
    this.token = newToken
    if (this._connectionState === 'connected') {
      // 不能直接在 WS 中更新 token，需要重连
      this.reconnectWithNewToken()
    }
  }

  // ========== 私有方法 ==========

  private doConnect(): void {
    this.closeWs()
    this.setConnectionState('connecting')

    try {
      this.ws = new WebSocket(this.url)
      this.ws.onopen = this.handleOpen.bind(this)
      this.ws.onmessage = this.handleMessage.bind(this)
      this.ws.onclose = this.handleClose.bind(this)
      this.ws.onerror = this.handleError.bind(this)
    } catch (err) {
      console.error('[WS] Failed to create WebSocket:', err)
      this.scheduleReconnect()
    }
  }

  private handleOpen(): void {
    this.reconnectAttempts = 0
    // 发送认证消息
    this.send('Auth', { token: this.token })
  }

  private handleMessage(event: MessageEvent): void {
    const data = typeof event.data === 'string' ? event.data : ''
    if (!data) return

    const msg = parseMessage(data)
    if (!msg) return

    // 处理认证结果
    if (msg.type === 'AuthResult') {
      const payload = msg.payload as { success: boolean } | undefined
      if (payload?.success) {
        this.setConnectionState('connected')
        this.flushMessageQueue()
      } else {
        console.error('[WS] Auth failed:', payload)
        this.disconnect()
      }
      this.emit(msg.type, msg.payload)
      return
    }

    // 处理心跳 - 服务端 Ping，回复 Pong
    if (msg.type === 'Ping') {
      this.send('Pong')
      return
    }

    // 处理其他业务消息
    this.emit(msg.type, msg.payload)
    // 同时触发通用 message 事件
    this.emit('message', msg)
  }

  private handleClose(event: CloseEvent): void {
    const logReason = event.reason ? `reason: ${event.reason}` : 'no reason'
    console.log(`[WS] Connection closed (code=${event.code}, ${logReason})`)

    if (!this.manuallyDisconnected) {
      this.setConnectionState('reconnecting')
      this.scheduleReconnect()
    }
  }

  private handleError(): void {
    console.error('[WS] Connection error')
    // onclose will fire after onerror, so reconnect is handled there
  }

  private scheduleReconnect(): void {
    this.clearReconnectTimer()

    // 离线模式下不再重连
    if (this.isOfflineCheck?.()) {
      console.warn('[WS] Server offline, skipping reconnect')
      this.setConnectionState('disconnected')
      return
    }

    const { max_attempts, base_delay_ms, max_delay_ms, multiplier } = this.reconnectCfg

    if (this.reconnectAttempts >= max_attempts) {
      console.error('[WS] Max reconnect attempts reached')
      this.setConnectionState('disconnected')
      return
    }

    const delay = Math.min(
      base_delay_ms * Math.pow(multiplier, this.reconnectAttempts),
      max_delay_ms,
    )

    console.log(`[WS] Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts + 1})`)
    this.reconnectAttempts++

    this.reconnectTimer = setTimeout(() => {
      if (!this.manuallyDisconnected) {
        this.doConnect()
      }
    }, delay)
  }

  private reconnectWithNewToken(): void {
    this.closeWs()
    this.setConnectionState('reconnecting')
    // Small delay before reconnecting with new token
    this.reconnectTimer = setTimeout(() => {
      if (!this.manuallyDisconnected) {
        this.doConnect()
      }
    }, 100)
  }

  private flushMessageQueue(): void {
    const queue = this.messageQueue.slice()
    this.messageQueue = []

    if (this.ws?.readyState === WebSocket.OPEN) {
      for (const msg of queue) {
        this.ws.send(msg)
      }
    }
  }

  private closeWs(): void {
    if (this.ws) {
      this.ws.onopen = null
      this.ws.onmessage = null
      this.ws.onclose = null
      this.ws.onerror = null
      if (this.ws.readyState === WebSocket.OPEN || this.ws.readyState === WebSocket.CONNECTING) {
        this.ws.close()
      }
      this.ws = null
    }
  }

  private clearReconnectTimer(): void {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer)
      this.reconnectTimer = null
    }
  }

  private emit(event: string, ...args: unknown[]): void {
    this.listeners.get(event)?.forEach((handler) => {
      try {
        handler(...args)
      } catch (err) {
        console.error(`[WS] Error in handler for "${event}":`, err)
      }
    })
  }
}

/** 全局单例 */
export const wsService = new WebSocketService()
