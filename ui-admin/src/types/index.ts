/**
 * 类型定义入口文件
 * 统一导出所有类型定义
 */

export * from './api'
export * from './auth'
export * from './user'
export * from './audit'

// Dashboard 类型
export type {
  SystemStats as DashboardSystemStats,
  ActivityStats as DashboardActivityStats,
  HealthStatus,
  ActivityHistoryItem as DashboardActivityHistoryItem,
} from './dashboard'

// Monitor 类型
export type {
  SystemStats as MonitorSystemStats,
  ActivityStats as MonitorActivityStats,
  RealtimeMetrics,
  ActivityHistoryItem as MonitorActivityHistoryItem,
} from './monitor'
