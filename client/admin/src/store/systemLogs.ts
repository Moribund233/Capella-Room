import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type {
  LogEntry,
  LogLevel,
  LogModule,
  SubscribeLogsParams,
  LogSubscriptionConfirmedPayload,
} from '@/types'
import { useWebSocketStore } from './websocket'

/**
 * 系统日志流 Store
 *
 * 管理WebSocket系统日志流的订阅、接收和展示
 */
export const useSystemLogsStore = defineStore('systemLogs', () => {
  // ========== 状态 ==========

  /** 日志条目列表 */
  const logs = ref<LogEntry[]>([])

  /** 是否已订阅 */
  const isSubscribed = ref(false)

  /** 当前日志级别过滤 */
  const currentLevel = ref<LogLevel>('all')

  /** 当前模块过滤 */
  const currentModule = ref<LogModule>('all')

  /** 订阅确认消息 */
  const subscriptionMessage = ref<string>('')

  /** 最大保留日志条数 */
  const MAX_LOGS_COUNT = 1000

  // ========== 计算属性 ==========

  /** 过滤后的日志列表 */
  const filteredLogs = computed(() => {
    return logs.value.filter((log) => {
      const levelMatch = currentLevel.value === 'all' || log.level === currentLevel.value
      const moduleMatch = currentModule.value === 'all' || log.target.includes(currentModule.value)
      return levelMatch && moduleMatch
    })
  })

  /** 各级别日志数量 */
  const logStats = computed(() => {
    const stats = {
      error: 0,
      warn: 0,
      info: 0,
      debug: 0,
    }
    logs.value.forEach((log) => {
      if (log.level in stats) {
        stats[log.level as keyof typeof stats]++
      }
    })
    return stats
  })

  /** 日志总数 */
  const totalLogs = computed(() => logs.value.length)

  // ========== 方法 ==========

  /**
   * 添加日志条目
   * @param entry 日志条目
   */
  function addLog(entry: LogEntry): void {
    logs.value.push(entry)

    // 限制日志数量，避免内存溢出
    if (logs.value.length > MAX_LOGS_COUNT) {
      logs.value = logs.value.slice(-MAX_LOGS_COUNT)
    }
  }

  /**
   * 清空日志
   */
  function clearLogs(): void {
    logs.value = []
  }

  /**
   * 设置日志级别过滤
   * @param level 日志级别
   */
  function setLevel(level: LogLevel): void {
    currentLevel.value = level
  }

  /**
   * 设置模块过滤
   * @param module 模块
   */
  function setModule(module: LogModule): void {
    currentModule.value = module
  }

  /**
   * 订阅系统日志
   * @param params 订阅参数
   */
  function subscribe(params: SubscribeLogsParams = {}): void {
    const wsStore = useWebSocketStore()

    if (!wsStore.isAuthenticated) {
      console.error('[SystemLogs Store] WebSocket未认证，无法订阅日志')
      return
    }

    const message = {
      type: 'SubscribeLogs' as const,
      payload: {
        level: params.level || currentLevel.value,
        module: params.module || currentModule.value,
      },
    }

    wsStore.send(message)
    console.log('[SystemLogs Store] 发送订阅请求:', message)
  }

  /**
   * 取消订阅系统日志
   */
  function unsubscribe(): void {
    const wsStore = useWebSocketStore()

    if (!wsStore.isAuthenticated) {
      return
    }

    wsStore.send({ type: 'UnsubscribeLogs' })
    isSubscribed.value = false
    console.log('[SystemLogs Store] 取消订阅日志')
  }

  /**
   * 处理日志条目消息
   * @param entry 日志条目
   */
  function handleLogEntry(entry: LogEntry): void {
    addLog(entry)
  }

  /**
   * 处理订阅确认消息
   * @param payload 确认消息
   */
  function handleSubscriptionConfirmed(payload: LogSubscriptionConfirmedPayload): void {
    isSubscribed.value = payload.success
    subscriptionMessage.value = payload.message

    if (payload.success) {
      console.log('[SystemLogs Store] 日志订阅成功:', payload.message)
    } else {
      console.error('[SystemLogs Store] 日志订阅失败:', payload.message)
    }
  }

  /**
   * 初始化日志流（注册WebSocket消息处理器）
   */
  function init(): void {
    const wsStore = useWebSocketStore()

    // 注册日志条目处理器
    wsStore.on<LogEntry>('LogEntry', (entry) => {
      handleLogEntry(entry)
    })

    // 注册订阅确认处理器
    wsStore.on<LogSubscriptionConfirmedPayload>('LogSubscriptionConfirmed', (payload) => {
      handleSubscriptionConfirmed(payload)
    })

    console.log('[SystemLogs Store] 已初始化日志流处理器')
  }

  /**
   * 导出日志为JSON
   * @returns JSON字符串
   */
  function exportLogs(): string {
    return JSON.stringify(logs.value, null, 2)
  }

  /**
   * 导出日志为文本格式
   * @returns 文本内容
   */
  function exportLogsAsText(): string {
    return logs.value
      .map(
        (log) =>
          `[${new Date(log.timestamp).toLocaleString()}] [${log.level.toUpperCase()}] ${log.target}: ${log.message}`,
      )
      .join('\n')
  }

  return {
    // 状态
    logs,
    isSubscribed,
    currentLevel,
    currentModule,
    subscriptionMessage,

    // 计算属性
    filteredLogs,
    logStats,
    totalLogs,

    // 方法
    addLog,
    clearLogs,
    setLevel,
    setModule,
    subscribe,
    unsubscribe,
    init,
    exportLogs,
    exportLogsAsText,
  }
})
