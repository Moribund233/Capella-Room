/**
 * 审计系统类型定义
 * 包含审计日志、安全告警、告警规则相关类型
 */

/** 严重级别 */
export type Severity = 'critical' | 'high' | 'medium' | 'low' | 'error' | 'warning' | 'info'

/** 操作状态 */
export type ActionStatus = 'success' | 'failure'

/** 告警状态 */
export type AlertStatus = 'new' | 'acknowledged' | 'resolved' | 'ignored'

/** 导出格式 */
export type ExportFormat = 'json' | 'csv'

/** 审计日志 */
export interface AuditLog {
  id: string
  event_type: string
  severity: Severity
  actor_id: string
  actor_type: string
  actor_name?: string
  target_id?: string
  target_type?: string
  target_name?: string
  action: string
  status: ActionStatus
  ip_address?: string
  user_agent?: string
  details?: Record<string, unknown>
  created_at: string
}

/** 审计日志查询参数 */
export interface AuditLogQuery {
  event_type?: string
  severity?: string
  actor_id?: string
  target_id?: string
  target_type?: string
  status?: string
  start_time?: string
  end_time?: string
  limit?: number
  offset?: number
}

/** 审计日志列表响应 */
export interface AuditLogListResponse {
  logs: AuditLog[]
  total: number
  limit: number
  offset: number
}

/** 审计统计 */
export interface AuditStats {
  total_logs: number
  today_logs: number
  week_logs: number
  month_logs: number
  event_type_stats: Record<string, number>
  severity_stats: Record<string, number>
}

/** 安全告警 */
export interface SecurityAlert {
  id: string
  rule_id: string
  rule_name: string
  severity: 'critical' | 'high' | 'medium' | 'low'
  status: AlertStatus
  message: string
  context: Record<string, unknown>
  triggered_at: string
  acknowledged_by?: string
  acknowledged_at?: string
  resolved_by?: string
  resolved_at?: string
}

/** 告警查询参数 */
export interface AlertQuery {
  severity?: string
  status?: string
  start_time?: string
  end_time?: string
  limit?: number
  offset?: number
}

/** 告警列表响应 */
export interface AlertListResponse {
  alerts: SecurityAlert[]
  total: number
  limit: number
  offset: number
}

/** 更新告警状态请求 */
export interface UpdateAlertStatusRequest {
  status: 'acknowledged' | 'resolved' | 'ignored'
}

/** 告警规则 */
export interface AlertRule {
  id: string
  name: string
  description?: string
  event_type: string
  condition: string
  severity: 'critical' | 'high' | 'medium' | 'low'
  enabled: boolean
  cooldown_minutes: number
  max_alerts_per_hour: number
  created_at: string
  updated_at: string
}

/** 更新告警规则请求 */
export interface UpdateAlertRuleRequest {
  enabled?: boolean
  severity?: 'critical' | 'high' | 'medium' | 'low'
  cooldown_minutes?: number
  max_alerts_per_hour?: number
}

/** 告警规则列表响应 */
export interface AlertRuleListResponse {
  rules: AlertRule[]
}

/** 导出查询参数 */
export interface AuditLogExportQuery {
  format: ExportFormat
  event_type?: string
  severity?: string
  start_time?: string
  end_time?: string
}

/** 分页参数 */
export interface PaginationParams {
  page: number
  pageSize: number
  total: number
}

/** 事件分类 */
export interface EventCategory {
  key: string
  label: string
  events: EventItem[]
}

/** 事件项 */
export interface EventItem {
  value: string
  label: string
}
