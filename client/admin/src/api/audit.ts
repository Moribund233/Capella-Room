import { http } from './request'
import type { ApiResponse } from '@/types'

// ==================== 审计日志类型 ====================

/**
 * 审计日志
 */
export interface AuditLog {
  /** 日志ID */
  id: string
  /** 事件类型 */
  event_type: string
  /** 严重级别 */
  severity: 'info' | 'warning' | 'error' | 'critical'
  /** 操作者ID */
  actor_id: string | null
  /** 操作者名称 */
  actor_name: string | null
  /** 操作者角色 */
  actor_role: string | null
  /** 目标类型 */
  target_type: string | null
  /** 目标ID */
  target_id: string | null
  /** 操作动作 */
  action: string
  /** 描述 */
  description: string
  /** 元数据 */
  metadata: Record<string, unknown> | null
  /** 状态 */
  status: string
  /** 错误信息 */
  error_message: string | null
  /** 创建时间 */
  created_at: string
}

/**
 * 审计日志查询参数
 */
export interface AuditLogQueryParams {
  /** 事件类型 */
  event_type?: string
  /** 严重级别 */
  severity?: string
  /** 操作者ID */
  actor_id?: string
  /** 目标类型 */
  target_type?: string
  /** 状态 */
  status?: string
  /** 开始时间 */
  start_time?: string
  /** 结束时间 */
  end_time?: string
  /** 每页数量 */
  limit?: number
  /** 偏移量 */
  offset?: number
  [key: string]: unknown
}

/**
 * 审计日志列表响应
 */
export interface AuditLogListData {
  /** 日志列表 */
  logs: AuditLog[]
  /** 总条数 */
  total: number
  /** 每页数量 */
  limit: number
  /** 偏移量 */
  offset: number
}

// ==================== 安全告警类型 ====================

/**
 * 用户信息（简化）
 */
export interface AlertUserInfo {
  /** 用户ID */
  id: string
  /** 用户名 */
  username: string
  /** 头像URL */
  avatar_url: string | null
}

/**
 * 安全告警
 */
export interface AuditAlert {
  /** 告警ID */
  id: string
  /** 规则ID */
  rule_id: string | null
  /** 告警类型 */
  alert_type: string
  /** 严重级别 */
  severity: 'info' | 'warning' | 'error' | 'critical'
  /** 标题 */
  title: string
  /** 描述 */
  description: string
  /** 关联日志ID列表 */
  related_logs: string[] | null
  /** 来源IP */
  source_ip: string | null
  /** 受影响用户 */
  affected_user: AlertUserInfo | null
  /** 状态 */
  status: 'new' | 'acknowledged' | 'resolved' | 'ignored'
  /** 确认人 */
  acknowledged_by: AlertUserInfo | null
  /** 确认时间 */
  acknowledged_at: string | null
  /** 解决人 */
  resolved_by: AlertUserInfo | null
  /** 解决时间 */
  resolved_at: string | null
  /** 创建时间 */
  created_at: string
  /** 更新时间 */
  updated_at: string
}

/**
 * 告警查询参数
 */
export interface AlertQueryParams {
  /** 状态 */
  status?: string
  /** 严重级别 */
  severity?: string
  /** 告警类型 */
  alert_type?: string
  /** 受影响用户ID */
  affected_user_id?: string
  /** 开始时间 */
  start_time?: string
  /** 结束时间 */
  end_time?: string
  /** 每页数量 */
  limit?: number
  /** 偏移量 */
  offset?: number
  [key: string]: unknown
}

/**
 * 告警列表响应
 */
export interface AlertListData {
  /** 告警列表 */
  alerts: AuditAlert[]
  /** 总条数 */
  total: number
  /** 每页数量 */
  limit: number
  /** 偏移量 */
  offset: number
}

/**
 * 更新告警状态请求
 */
export interface UpdateAlertStatusRequest {
  /** 新状态 */
  status: 'acknowledged' | 'resolved' | 'ignored'
}

// ==================== 告警规则类型 ====================

/**
 * 告警规则
 */
export interface AlertRule {
  /** 规则ID */
  id: string
  /** 规则名称 */
  name: string
  /** 描述 */
  description: string | null
  /** 事件类型 */
  event_type: string | null
  /** 条件配置 */
  condition: Record<string, unknown>
  /** 严重级别 */
  severity: 'info' | 'warning' | 'error' | 'critical'
  /** 是否启用 */
  enabled: boolean
  /** 冷却时间（分钟） */
  cooldown_minutes: number
  /** 是否通知管理员 */
  notify_admins: boolean
  /** 创建时间 */
  created_at: string
  /** 更新时间 */
  updated_at: string
}

/**
 * 更新告警规则请求
 */
export interface UpdateAlertRuleRequest {
  /** 规则名称 */
  name?: string
  /** 描述 */
  description?: string | null
  /** 条件配置 */
  condition?: Record<string, unknown>
  /** 严重级别 */
  severity?: 'info' | 'warning' | 'error' | 'critical'
  /** 是否启用 */
  enabled?: boolean
  /** 冷却时间（分钟） */
  cooldown_minutes?: number
  /** 是否通知管理员 */
  notify_admins?: boolean
}

/**
 * 告警规则列表响应
 */
export interface AlertRuleListData {
  /** 规则列表 */
  rules: AlertRule[]
}

// ==================== 审计统计类型 ====================

/**
 * 严重级别统计
 */
export interface SeverityCount {
  /** 严重级别 */
  severity: string
  /** 数量 */
  count: number
}

/**
 * 事件类型统计
 */
export interface EventTypeCount {
  /** 事件类型 */
  event_type: string
  /** 数量 */
  count: number
}

/**
 * 每日统计
 */
export interface DailyCount {
  /** 日期 */
  date: string
  /** 数量 */
  count: number
}

/**
 * 审计统计信息
 */
export interface AuditStats {
  /** 总日志数 */
  total_logs: number
  /** 今日日志数 */
  today_logs: number
  /** 本周日志数 */
  week_logs: number
  /** 本月日志数 */
  month_logs: number
  /** 按严重级别统计 */
  logs_by_severity: SeverityCount[]
  /** 按事件类型统计 */
  logs_by_event_type: EventTypeCount[]
  /** 按天统计 */
  logs_by_day: DailyCount[]
  /** 告警总数 */
  alerts_count: number
  /** 新增告警数 */
  new_alerts_count: number
}

/**
 * 审计统计查询参数
 */
export interface AuditStatsQueryParams {
  /** 开始时间 */
  start_time?: string
  /** 结束时间 */
  end_time?: string
  [key: string]: unknown
}

/**
 * 导出格式
 */
export type ExportFormat = 'json' | 'csv'

/**
 * 审计日志导出参数
 */
export interface AuditExportParams {
  /** 事件类型 */
  event_type?: string
  /** 严重级别 */
  severity?: string
  /** 操作者ID */
  actor_id?: string
  /** 开始时间 */
  start_time?: string
  /** 结束时间 */
  end_time?: string
  /** 导出格式 */
  format: ExportFormat
  [key: string]: unknown
}

// ==================== API 接口 ====================

/**
 * 构建查询字符串
 * @param params 查询参数
 * @returns URL 查询字符串
 */
function buildQueryString(params: Record<string, unknown>): string {
  const queryParams = new URLSearchParams()
  Object.entries(params).forEach(([key, value]) => {
    if (value !== undefined && value !== null && value !== '') {
      queryParams.append(key, String(value))
    }
  })
  const query = queryParams.toString()
  return query ? `?${query}` : ''
}

/**
 * 审计系统 API
 */
export const auditApi = {
  /**
   * 获取审计日志列表
   * @param params 查询参数
   * @returns 审计日志列表
   */
  getAuditLogs(params: AuditLogQueryParams = {}): Promise<ApiResponse<AuditLogListData>> {
    return http.get<AuditLogListData>(`/admin/audit/logs${buildQueryString(params)}`)
  },

  /**
   * 获取审计日志详情
   * @param id 日志ID
   * @returns 审计日志详情
   */
  getAuditLogDetail(id: string): Promise<ApiResponse<AuditLog>> {
    return http.get<AuditLog>(`/admin/audit/logs/${id}`)
  },

  /**
   * 获取审计统计信息
   * @param params 查询参数
   * @returns 审计统计
   */
  getAuditStats(params: AuditStatsQueryParams = {}): Promise<ApiResponse<AuditStats>> {
    return http.get<AuditStats>(`/admin/audit/stats${buildQueryString(params)}`)
  },

  /**
   * 导出审计日志
   * @param params 导出参数
   * @returns Blob 数据
   */
  async exportAuditLogs(params: AuditExportParams): Promise<Blob> {
    const query = buildQueryString(params as Record<string, unknown>)
    const response = await fetch(
      `${import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080/api/v1'}/admin/audit/export${query}`,
      {
        method: 'GET',
        headers: {
          Authorization: `Bearer ${localStorage.getItem('access_token') || ''}`,
        },
      }
    )
    if (!response.ok) {
      throw new Error('导出失败')
    }
    return response.blob()
  },

  /**
   * 获取安全告警列表
   * @param params 查询参数
   * @returns 告警列表
   */
  getAlerts(params: AlertQueryParams = {}): Promise<ApiResponse<AlertListData>> {
    return http.get<AlertListData>(`/admin/audit/alerts${buildQueryString(params)}`)
  },

  /**
   * 更新告警状态
   * @param id 告警ID
   * @param data 状态更新请求
   * @returns 更新后的告警
   */
  updateAlertStatus(id: string, data: UpdateAlertStatusRequest): Promise<ApiResponse<AuditAlert>> {
    return http.put<AuditAlert>(`/admin/audit/alerts/${id}/status`, data)
  },

  /**
   * 获取告警规则列表
   * @returns 规则列表
   */
  getAlertRules(): Promise<ApiResponse<AlertRuleListData>> {
    return http.get<AlertRuleListData>('/admin/audit/rules')
  },

  /**
   * 更新告警规则
   * @param id 规则ID
   * @param data 规则更新请求
   * @returns 更新后的规则
   */
  updateAlertRule(id: string, data: UpdateAlertRuleRequest): Promise<ApiResponse<AlertRule>> {
    return http.put<AlertRule>(`/admin/audit/rules/${id}`, data)
  },
}
