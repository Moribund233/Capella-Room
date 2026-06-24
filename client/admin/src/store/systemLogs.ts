import { defineStore } from 'pinia'
import { ref, computed, watch } from 'vue'
import type {
  LogEntry,
  LogLevel,
  LogModule,
  SubscribeLogsParams,
  LogSubscriptionConfirmedPayload,
} from '@/types'
import { useWebSocketStore } from './websocket'

const STORAGE_KEY = 'capella-room:system-logs'
const MAX_LOGS_CACHE = 1000
const DEFAULT_PAGE_SIZE = 50

/**
 * 日志流 Store
 *
 * localStorage 持久化缓存 + 分页
 */
export const useSystemLogsStore = defineStore('systemLogs', () => {
  // ========== 缓存层 ==========

  function loadCache(): LogEntry[] {
    try {
      const raw = localStorage.getItem(STORAGE_KEY)
      return raw ? JSON.parse(raw) : []
    } catch {
      return []
    }
  }

  function saveCache(logs: LogEntry[]) {
    try {
      const trimmed = logs.length > MAX_LOGS_CACHE
        ? logs.slice(-MAX_LOGS_CACHE)
        : logs
      localStorage.setItem(STORAGE_KEY, JSON.stringify(trimmed))
    } catch {
      // localStorage 满或不可用时静默忽略
    }
  }

  // ========== 状态 ==========

  const logs = ref<LogEntry[]>(loadCache())
  const isSubscribed = ref(false)
  const currentLevel = ref<LogLevel>('all')
  const currentModule = ref<LogModule>('all')
  const subscriptionMessage = ref<string>('')

  /** 分页 */
  const pageSize = ref(DEFAULT_PAGE_SIZE)
  const currentPage = ref(1)

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let logEntryHandler: ((payload: any) => void) | null = null
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let subscriptionConfirmedHandler: ((payload: any) => void) | null = null

  // ========== 计算属性 ==========

  const filteredLogs = computed(() => {
    return logs.value.filter((log) => {
      const levelMatch = currentLevel.value === 'all' || log.level === currentLevel.value
      const moduleMatch = currentModule.value === 'all' || log.target.includes(currentModule.value)
      return levelMatch && moduleMatch
    })
  })

  /** 分页后的当前页日志 */
  const pagedLogs = computed(() => {
    const start = (currentPage.value - 1) * pageSize.value
    return filteredLogs.value.slice(start, start + pageSize.value)
  })

  /** 总页数 */
  const totalPages = computed(() =>
    Math.max(1, Math.ceil(filteredLogs.value.length / pageSize.value)),
  )

  /** 是否在最新页（自动跟随） */
  const isOnLatestPage = computed(() =>
    currentPage.value >= totalPages.value,
  )

  const logStats = computed(() => {
    const stats = { error: 0, warn: 0, info: 0, debug: 0 }
    logs.value.forEach((log) => {
      if (log.level in stats) {
        stats[log.level as keyof typeof stats]++
      }
    })
    return stats
  })

  const totalLogs = computed(() => logs.value.length)

  // ========== 方法 ==========

  function addLog(entry: LogEntry): void {
    const wasOnLatest = isOnLatestPage.value

    logs.value.push(entry)
    if (logs.value.length > MAX_LOGS_CACHE) {
      logs.value = logs.value.slice(-MAX_LOGS_CACHE)
    }
    saveCache(logs.value)

    // 如果在最新页，新日志到达后自动翻到最新
    if (wasOnLatest) {
      currentPage.value = totalPages.value
    }
  }

  function clearLogs(): void {
    logs.value = []
    currentPage.value = 1
    saveCache(logs.value)
  }

  function setLevel(level: LogLevel): void {
    currentLevel.value = level
    currentPage.value = 1
  }

  function setModule(module: LogModule): void {
    currentModule.value = module
    currentPage.value = 1
  }

  function goToPage(page: number): void {
    currentPage.value = Math.max(1, Math.min(page, totalPages.value))
  }

  function goToLatest(): void {
    currentPage.value = totalPages.value
  }

  function subscribe(params: SubscribeLogsParams = {}): void {
    const wsStore = useWebSocketStore()
    if (!wsStore.isAuthenticated) {
      console.error('[SystemLogs Store] WebSocket未认证，无法订阅日志')
      return
    }
    wsStore.send({
      type: 'SubscribeLogs' as const,
      payload: {
        level: params.level || currentLevel.value,
        module: params.module || currentModule.value,
      },
    })
  }

  function unsubscribe(): void {
    const wsStore = useWebSocketStore()
    if (!wsStore.isAuthenticated) return
    wsStore.send({ type: 'UnsubscribeLogs' })
    isSubscribed.value = false
  }

  function handleLogEntry(entry: LogEntry): void {
    addLog(entry)
  }

  function handleSubscriptionConfirmed(payload: LogSubscriptionConfirmedPayload): void {
    isSubscribed.value = payload.success
    subscriptionMessage.value = payload.message
  }

  function init(): void {
    const wsStore = useWebSocketStore()
    destroy()

    logEntryHandler = (entry: LogEntry) => handleLogEntry(entry)
    wsStore.on<LogEntry>('LogEntry', logEntryHandler)

    subscriptionConfirmedHandler = (payload: LogSubscriptionConfirmedPayload) =>
      handleSubscriptionConfirmed(payload)
    wsStore.on<LogSubscriptionConfirmedPayload>(
      'LogSubscriptionConfirmed',
      subscriptionConfirmedHandler,
    )

    // 首次打开自动跳到最新页
    goToLatest()

    if (wsStore.isAuthenticated) {
      subscribe()
    } else {
      const stopWatch = watch(
        () => wsStore.isAuthenticated,
        (authenticated) => {
          if (authenticated) {
            subscribe()
            stopWatch()
          }
        },
      )
    }
  }

  function destroy(): void {
    const wsStore = useWebSocketStore()
    if (logEntryHandler) {
      wsStore.off('LogEntry', logEntryHandler)
      logEntryHandler = null
    }
    if (subscriptionConfirmedHandler) {
      wsStore.off('LogSubscriptionConfirmed', subscriptionConfirmedHandler)
      subscriptionConfirmedHandler = null
    }
  }

  function exportLogs(): string {
    return JSON.stringify(logs.value, null, 2)
  }

  function exportLogsAsText(): string {
    return logs.value
      .map(
        (log) =>
          `[${new Date(log.timestamp).toLocaleString()}] [${log.level.toUpperCase()}] ${log.target}: ${log.message}`,
      )
      .join('\n')
  }

  return {
    logs,
    isSubscribed,
    currentLevel,
    currentModule,
    subscriptionMessage,
    pageSize,
    currentPage,

    filteredLogs,
    pagedLogs,
    totalPages,
    isOnLatestPage,
    logStats,
    totalLogs,

    addLog,
    clearLogs,
    setLevel,
    setModule,
    goToPage,
    goToLatest,
    subscribe,
    unsubscribe,
    init,
    destroy,
    exportLogs,
    exportLogsAsText,
  }
})
