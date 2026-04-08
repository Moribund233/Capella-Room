/**
 * Dashboard API 模块
 * 系统概览数据统计相关接口
 */

import { get, type ApiResponse } from './client'
import type { SystemStats, ActivityStats, HealthStatus } from '@/types/dashboard'

export type { SystemStats, ActivityStats, HealthStatus }

/**
 * 获取系统统计数据
 * @returns 系统统计信息
 */
export async function getSystemStats(): Promise<ApiResponse<SystemStats>> {
  return get<SystemStats>('/admin/stats')
}

/**
 * 获取活跃度统计数据
 * @returns 活跃度统计信息
 */
export async function getActivityStats(): Promise<ApiResponse<ActivityStats>> {
  return get<ActivityStats>('/admin/stats/activity')
}

/**
 * 获取系统健康状态
 * @returns 健康状态信息
 */
export async function getHealthStatus(): Promise<ApiResponse<HealthStatus>> {
  // 健康检查端点在 /health，不在 /api/v1 下
  // 使用 // 前缀跳过 BASE_URL 拼接
  return get<HealthStatus>('//health', { skipAuth: true })
}
