/**
 * 监控模块类型定义
 */

/** 系统统计信息 */
export interface SystemStats {
  total_users: number
  total_rooms: number
  total_messages: number
  online_users: number
  active_connections: number
}

/** 活跃度统计信息 */
export interface ActivityStats {
  daily_active_users: number
  weekly_active_users: number
  monthly_active_users: number
  daily_messages: number
  weekly_messages: number
  monthly_messages: number
}

/** 实时监控数据 */
export interface RealtimeMetrics {
  timestamp: string
  online_users: number
  active_connections: number
  messages_per_minute: number
  cpu_usage: number
  memory_usage: number
}

/** 历史活跃度数据项 */
export interface ActivityHistoryItem {
  date: string
  active_users: number
  messages: number
}
