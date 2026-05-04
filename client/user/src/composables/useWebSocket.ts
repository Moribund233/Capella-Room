import { onMounted, onUnmounted, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useWebSocketStore } from '@/stores/websocket'
import { useAuthStore } from '@/stores/auth'
import type { MessageHandler } from '@/types/websocket'

export function useWebSocket() {
  const wsStore = useWebSocketStore()
  const authStore = useAuthStore()
  const { isAuthenticated } = storeToRefs(authStore)
  const { connectionState, isConnected, isConnecting } = storeToRefs(wsStore)

  let stopAuthWatcher: (() => void) | null = null
  const subscribedTypes = new Map<string, Set<MessageHandler<unknown>>>()

  function connect() {
    wsStore.connect()
  }

  function disconnect() {
    wsStore.disconnect()
  }

  function send(type: string, payload?: unknown) {
    wsStore.send(type, payload)
  }

  function onMessage<T>(type: string, handler: MessageHandler<T>) {
    wsStore.onMessage(type, handler)

    if (!subscribedTypes.has(type)) {
      subscribedTypes.set(type, new Set())
    }
    subscribedTypes.get(type)!.add(handler as MessageHandler<unknown>)
  }

  function offMessage<T>(type: string, handler: MessageHandler<T>) {
    wsStore.offMessage(type, handler)
    subscribedTypes.get(type)?.delete(handler as MessageHandler<unknown>)
  }

  function cleanupSubscriptions() {
    for (const [type, handlers] of subscribedTypes) {
      for (const handler of handlers) {
        wsStore.offMessage(type, handler)
      }
    }
    subscribedTypes.clear()
  }

  // 监听认证状态，自动连接/断开
  function setupAuthWatcher() {
    stopAuthWatcher = watch(isAuthenticated, (authenticated) => {
      if (authenticated) {
        connect()
      } else {
        disconnect()
      }
    }, { immediate: true })
  }

  // 组件挂载时启动监听
  onMounted(() => {
    setupAuthWatcher()
  })

  // 组件卸载时清理
  onUnmounted(() => {
    stopAuthWatcher?.()
    cleanupSubscriptions()
    disconnect()
  })

  return {
    connectionState,
    isConnected,
    isConnecting,
    connect,
    disconnect,
    send,
    onMessage,
    offMessage,
  }
}
