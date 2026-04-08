/**
 * 审计系统 API 模块
 * 提供审计日志、安全告警、告警规则相关接口
 */

import { get, post, put, type ApiResponse } from './client'
import type {
  AuditLog,
  AuditLogQuery,
  AuditLogListResponse,
  AuditStats,
  SecurityAlert,
  AlertQuery,
  AlertListResponse,
  UpdateAlertStatusRequest,
  AlertRule,
  UpdateAlertRuleRequest,
  AlertRuleListResponse,
  AuditLogExportQuery,
} from '@/types'

export type {
  AuditLog,
  AuditLogQuery,
  AuditLogListResponse,
  AuditStats,
  SecurityAlert,
  AlertQuery,
  AlertListResponse,
  UpdateAlertStatusRequest,
  AlertRule,
  UpdateAlertRuleRequest,
  AlertRuleListResponse,
  AuditLogExportQuery,
}

/**
 * 查询审计日志列表
 * @param query 查询参数
 * @returns 日志列表和总数
 */
export async function listAuditLogs(
  query: AuditLogQuery = {}
): Promise<ApiResponse<AuditLogListResponse>> {
  const params = new URLSearchParams()
  if (query.event_type) params.append('event_type', query.event_type)
  if (query.severity) params.append('severity', query.severity)
  if (query.actor_id) params.append('actor_id', query.actor_id)
  if (query.target_id) params.append('target_id', query.target_id)
  if (query.target_type) params.append('target_type', query.target_type)
  if (query.status) params.append('status', query.status)
  if (query.start_time) params.append('start_time', query.start_time)
  if (query.end_time) params.append('end_time', query.end_time)
  if (query.limit) params.append('limit', query.limit.toString())
  if (query.offset) params.append('offset', query.offset.toString())

  const queryString = params.toString()
  return get<AuditLogListResponse>(`/admin/audit/logs${queryString ? `?${queryString}` : ''}`)
}

/**
 * 获取单条审计日志详情
 * @param logId 日志ID
 * @returns 日志详情
 */
export async function getAuditLogDetail(logId: string): Promise<ApiResponse<AuditLog>> {
  return get<AuditLog>(`/admin/audit/logs/${logId}`)
}

/**
 * 获取审计统计信息
 * @param startTime 开始时间
 * @param endTime 结束时间
 * @returns 统计数据
 */
export async function getAuditStats(
  startTime?: string,
  endTime?: string
): Promise<ApiResponse<AuditStats>> {
  const params = new URLSearchParams()
  if (startTime) params.append('start_time', startTime)
  if (endTime) params.append('end_time', endTime)

  const queryString = params.toString()
  return get<AuditStats>(`/admin/audit/stats${queryString ? `?${queryString}` : ''}`)
}

/**
 * 导出审计日志
 * @param query 导出参数
 * @returns Blob 数据
 */
export async function exportAuditLogs(query: AuditLogExportQuery): Promise<Blob> {
  const params = new URLSearchParams()
  params.append('format', query.format)
  if (query.event_type) params.append('event_type', query.event_type)
  if (query.severity) params.append('severity', query.severity)
  if (query.start_time) params.append('start_time', query.start_time)
  if (query.end_time) params.append('end_time', query.end_time)

  const baseUrl = import.meta.env.VITE_API_BASE_URL || '/api/v1'
  const url = `${baseUrl}/admin/audit/export?${params.toString()}`

  const response = await fetch(url, {
    method: 'GET',
    headers: {
      Authorization: `Bearer ${localStorage.getItem('access_token') || sessionStorage.getItem('access_token') || ''}`,
    },
  })

  if (!response.ok) {
    throw new Error('导出失败')
  }

  return response.blob()
}

/**
 * 查询安全告警列表
 * @param query 查询参数
 * @returns 告警列表和总数
 */
export async function listAlerts(
  query: AlertQuery = {}
): Promise<ApiResponse<AlertListResponse>> {
  const params = new URLSearchParams()
  if (query.severity) params.append('severity', query.severity)
  if (query.status) params.append('status', query.status)
  if (query.start_time) params.append('start_time', query.start_time)
  if (query.end_time) params.append('end_time', query.end_time)
  if (query.limit) params.append('limit', query.limit.toString())
  if (query.offset) params.append('offset', query.offset.toString())

  const queryString = params.toString()
  return get<AlertListResponse>(`/admin/audit/alerts${queryString ? `?${queryString}` : ''}`)
}

/**
 * 更新告警状态
 * @param alertId 告警ID
 * @param request 状态更新请求
 * @returns 更新后的告警
 */
export async function updateAlertStatus(
  alertId: string,
  request: UpdateAlertStatusRequest
): Promise<ApiResponse<SecurityAlert>> {
  return put<SecurityAlert>(`/admin/audit/alerts/${alertId}/status`, request)
}

/**
 * 获取告警规则列表
 * @returns 规则列表
 */
export async function listAlertRules(): Promise<ApiResponse<AlertRuleListResponse>> {
  return get<AlertRuleListResponse>('/admin/audit/rules')
}

/**
 * 更新告警规则
 * @param ruleId 规则ID
 * @param request 规则更新请求
 * @returns 更新后的规则
 */
export async function updateAlertRule(
  ruleId: string,
  request: UpdateAlertRuleRequest
): Promise<ApiResponse<AlertRule>> {
  return put<AlertRule>(`/admin/audit/rules/${ruleId}`, request)
}

/**
 * 清理过期审计日志
 * @param days 保留天数
 * @param archive 是否归档
 * @param archiveDir 归档目录
 * @returns 清理的日志数量
 */
export async function cleanupAuditLogs(
  days: number,
  archive?: boolean,
  archiveDir?: string
): Promise<ApiResponse<{ deleted: number }>> {
  return post<{ deleted: number }>('/admin/audit/cleanup', {
    days,
    archive,
    archive_dir: archiveDir,
  })
}
