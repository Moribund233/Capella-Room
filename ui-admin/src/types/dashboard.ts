/**
 * Dashboard 模块类型定义
 */

/** 系统统计数据 */
export interface SystemStats {
  /** 总用户数 */
  total_users: number
  /** 总房间数 */
  total_rooms: number
  /** 总消息数 */
  total_messages: number
  /** 在线用户数 */
  online_users: number
  /** WebSocket 活跃连接数 */
  active_connections: number
}

/** 活跃度统计数据 */
export interface ActivityStats {
  /** 日活跃用户 */
  daily_active_users: number
  /** 周活跃用户 */
  weekly_active_users: number
  /** 月活跃用户 */
  monthly_active_users: number
  /** 日消息数 */
  daily_messages: number
  /** 周消息数 */
  weekly_messages: number
  /** 月消息数 */
  monthly_messages: number
}

/** 系统健康状态 */
export interface HealthStatus {
  /** 状态: healthy/degraded/unhealthy */
  status: string
  /** 版本信息 */
  version: string
  /** 环境 */
  environment: string
  /** 时间戳 */
  timestamp: string
}

/** 历史活跃度数据项 */
export interface ActivityHistoryItem {
  date: string
  active_users: number
  messages: number
}
