import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { wsService } from '@/services/websocket'
import { useAuthStore } from './auth'
import { useConfigStore } from './config'
import { useConnectivityStore } from './connectivity'
import type { ConnectionState, MessageHandler } from '@/types/websocket'

export const useWebSocketStore = defineStore('websocket', () => {
  const connectionState = ref<ConnectionState>('disconnected')
  const reconnectAttempts = ref(0)
  // 标记是否已注册状态监听，避免重复注册
  let isStateHandlerRegistered = false
  // 标记是否正在连接中，避免重复连接
  let isConnectingInProgress = false

  const isConnected = computed(() => connectionState.value === 'connected')
  const isConnecting = computed(() =>
    connectionState.value === 'connecting' || connectionState.value === 'reconnecting',
  )

  // 同步 wsService 的状态变化
  function onStateChange(state: ConnectionState) {
    connectionState.value = state
    if (state === 'reconnecting') {
      reconnectAttempts.value++
    } else if (state === 'connected') {
      reconnectAttempts.value = 0
      isConnectingInProgress = false
    } else if (state === 'disconnected') {
      isConnectingInProgress = false
    }
  }

  // 注入服务端配置 + 初始化连接
  async function connect() {
    const authStore = useAuthStore()
    if (!authStore.accessToken) {
      console.warn('[WebSocketStore] No access token, skipping connection')
      return
    }

    // 如果已经连接或正在连接，不重复操作
    if (connectionState.value === 'connected') {
      console.log('[WebSocketStore] Already connected')
      return
    }
    if (isConnectingInProgress) {
      console.log('[WebSocketStore] Connection already in progress')
      return
    }

    const wsUrl = import.meta.env.VITE_WS_URL
    if (!wsUrl) {
      console.error('[WebSocketStore] VITE_WS_URL is not configured')
      return
    }

    // 离线模式下不尝试连接
    const connectivity = useConnectivityStore()
    if (connectivity.isOffline) {
      console.warn('[WebSocketStore] Server is offline, skipping WS connection')
      connectionState.value = 'disconnected'
      return
    }

    isConnectingInProgress = true

    // 确保服务端配置已加载并注入 wsService
    const configStore = useConfigStore()
    await configStore.ensureLoaded()

    // 配置加载后再次检查（可能在加载过程中被设为离线）
    if (connectivity.isOffline) {
      console.warn('[WebSocketStore] Server marked offline during config load, skipping WS connection')
      connectionState.value = 'disconnected'
      isConnectingInProgress = false
      return
    }

    wsService.setConfig(configStore.config.reconnect)
    wsService.setOfflineCheck(() => useConnectivityStore().isOffline)

    // 只注册一次状态监听
    if (!isStateHandlerRegistered) {
      wsService.onConnectionState(onStateChange)
      isStateHandlerRegistered = true
    }

    wsService.connect(wsUrl, authStore.accessToken)
  }

  // 断开连接
  function disconnect() {
    wsService.offConnectionState(onStateChange)
    wsService.disconnect()
    connectionState.value = 'disconnected'
    reconnectAttempts.value = 0
  }

  // 发送消息
  function send(type: string, payload?: unknown) {
    wsService.send(type, payload)
  }

  // 订阅消息
  function onMessage<T>(type: string, handler: MessageHandler<T>) {
    wsService.onMessage(type, handler)
  }

  // 取消订阅
  function offMessage<T>(type: string, handler: MessageHandler<T>) {
    wsService.offMessage(type, handler)
  }

  // 刷新 token 并重新认证
  function refreshToken() {
    const authStore = useAuthStore()
    if (authStore.accessToken) {
      wsService.updateToken(authStore.accessToken)
    }
  }

  return {
    connectionState,
    reconnectAttempts,
    isConnected,
    isConnecting,
    connect,
    disconnect,
    send,
    onMessage,
    offMessage,
    refreshToken,
  }
})
