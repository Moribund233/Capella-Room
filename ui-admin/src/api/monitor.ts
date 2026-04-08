/**
 * 系统监控 API 模块
 * 提供系统统计、活跃度统计等监控相关接口
 */

import { get, type ApiResponse } from './client'
import type {
  SystemStats,
  ActivityStats,
  RealtimeMetrics,
  ActivityHistoryItem,
} from '@/types/monitor'

export type { SystemStats, ActivityStats, RealtimeMetrics, ActivityHistoryItem }

/**
 * 获取系统统计信息
 * @returns 系统统计数据
 */
export async function getSystemStats(): Promise<ApiResponse<SystemStats>> {
  return get<SystemStats>('/admin/stats')
}

/**
 * 获取活跃度统计信息
 * @returns 活跃度统计数据
 */
export async function getActivityStats(): Promise<ApiResponse<ActivityStats>> {
  return get<ActivityStats>('/admin/stats/activity')
}

/**
 * 获取历史活跃度数据（用于图表）
 * @param days 天数，默认 7 天
 * @returns 历史活跃度数据
 */
export async function getActivityHistory(
  days: number = 7
): Promise<ApiResponse<ActivityHistoryItem[]>> {
  return get<ActivityHistoryItem[]>(`/admin/stats/activity/history?days=${days}`)
}
