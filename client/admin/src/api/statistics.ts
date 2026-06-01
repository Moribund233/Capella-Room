import { http } from './request'
import type { ApiResponse } from '@/types'

// ==================== 系统统计类型 ====================

/**
 * 系统统计概览数据
 */
export interface SystemStatsOverview {
  /** 总注册用户数 */
  total_users: number
  /** 总房间数 */
  total_rooms: number
  /** 总消息数 */
  total_messages: number
  /** 当前在线用户数 */
  online_users: number
  /** 当前活跃 WebSocket 连接数 */
  active_connections: number
}

/**
 * 活跃度统计数据
 */
export interface ActivityStats {
  /** 日活跃用户（DAU） */
  daily_active_users: number
  /** 周活跃用户（WAU） */
  weekly_active_users: number
  /** 月活跃用户（MAU） */
  monthly_active_users: number
  /** 今日消息数 */
  daily_messages: number
  /** 本周消息数 */
  weekly_messages: number
  /** 本月消息数 */
  monthly_messages: number
}

/**
 * 性能指标数据
 */
export interface PerformanceStats {
  /** 总消息数 */
  total_messages: number
  /** 总连接数 */
  total_connections: number
  /** 当前在线用户数 */
  current_online_users: number
  /** 活跃房间数 */
  active_rooms: number
  /** 时间戳 */
  timestamp: string
}

// ==================== 系统监控类型 ====================

/**
 * 内存信息
 */
export interface MemoryInfo {
  /** 总内存 (MB) */
  total_mb: number
  /** 已使用内存 (MB) */
  used_mb: number
  /** 可用内存 (MB) */
  available_mb: number
  /** 使用率 (%) */
  usage_percent: number
}

/**
 * 磁盘信息
 */
export interface DiskInfo {
  /** 总空间 (GB) */
  total_gb: number
  /** 已使用空间 (GB) */
  used_gb: number
  /** 可用空间 (GB) */
  available_gb: number
  /** 使用率 (%) */
  usage_percent: number
}

/**
 * 系统监控信息
 */
export interface SystemMonitorInfo {
  /** 内存使用情况 */
  memory: MemoryInfo
  /** 磁盘使用情况 */
  disk: DiskInfo
  /** 应用进程内存占用 (MB) */
  process_memory_mb: number
}

/**
 * 数据库连接池信息
 */
export interface DatabasePoolInfo {
  /** 最大连接数 */
  max_connections: number
  /** 当前活跃连接数 */
  active_connections: number
  /** 空闲连接数 */
  idle_connections: number
  /** 等待连接的请求数 */
  waiting_requests: number
}

/**
 * 综合监控数据
 */
export interface MonitorData {
  /** 系统监控信息 */
  system: SystemMonitorInfo
  /** 数据库连接池信息 */
  database: DatabasePoolInfo
  /** 时间戳 */
  timestamp: string
}

// ==================== API 接口 ====================

/**
 * 获取系统统计概览
 * @returns 系统统计概览数据
 */
export function getSystemStats(): Promise<ApiResponse<SystemStatsOverview>> {
  return http.get<SystemStatsOverview>('/admin/stats')
}

/**
 * 获取活跃度统计
 * @returns 活跃度统计数据
 */
export function getActivityStats(): Promise<ApiResponse<ActivityStats>> {
  return http.get<ActivityStats>('/admin/stats/activity')
}

/**
 * 获取性能指标
 * @returns 性能指标数据
 */
export function getPerformanceStats(): Promise<ApiResponse<PerformanceStats>> {
  return http.get<PerformanceStats>('/admin/stats/performance')
}

/**
 * 获取系统监控数据
 * @returns 综合监控数据
 */
export function getMonitorData(): Promise<ApiResponse<MonitorData>> {
  return http.get<MonitorData>('/admin/monitor')
}

// ==================== 增强统计类型 ====================

/**
 * 用户增长统计
 */
export interface UserGrowthStats {
  new_users_today: number
  new_users_this_week: number
  new_users_this_month: number
  total_users: number
  growth_by_day: DailyUserCount[]
}

/**
 * 每日用户数量
 */
export interface DailyUserCount {
  date: string
  count: number
}

/**
 * 用户行为统计
 */
export interface UserBehaviorStats {
  avg_messages_per_user: number
  avg_rooms_per_user: number
  active_users_today: number
  active_users_this_week: number
}

/**
 * 好友关系统计
 */
export interface FriendStats {
  total_friendships: number
  pending_requests: number
  avg_friends_per_user: number
  request_accept_rate: number
}

/**
 * 房间活跃度
 */
export interface RoomActivity {
  id: string
  name: string
  member_count: number
  message_count: number
  last_message_at: string | null
}

/**
 * 房间统计概览
 */
export interface RoomStats {
  total_rooms: number
  public_rooms: number
  private_rooms: number
  direct_rooms: number
  avg_rooms_per_user: number
  avg_members_per_room: number
  empty_rooms: number
}

/**
 * 消息类型统计
 */
export interface MessageTypeStats {
  text_messages: number
  image_messages: number
  file_messages: number
  system_messages: number
  reply_messages: number
}

/**
 * 消息时间分布
 */
export interface MessageHourlyDistribution {
  hour: number
  count: number
}

/**
 * 严重级别统计
 */
export interface SeverityCount {
  severity: string
  count: number
}

/**
 * 安全统计
 */
export interface SecurityStats {
  failed_logins_today: number
  pending_alerts: number
  alerts_today: number
  alerts_by_severity: SeverityCount[]
  audit_logs_this_week: number
}

// ==================== 增强统计 API ====================

/**
 * 获取用户增长统计
 * @param days 天数范围
 * @returns 用户增长统计数据
 */
export function getUserGrowthStats(days?: number): Promise<ApiResponse<UserGrowthStats>> {
  return http.get<UserGrowthStats>('/admin/stats/users/growth', { params: { days } })
}

/**
 * 获取用户行为统计
 * @returns 用户行为统计数据
 */
export function getUserBehaviorStats(): Promise<ApiResponse<UserBehaviorStats>> {
  return http.get<UserBehaviorStats>('/admin/stats/users/behavior')
}

/**
 * 获取好友关系统计
 * @returns 好友关系统计数据
 */
export function getFriendStats(): Promise<ApiResponse<FriendStats>> {
  return http.get<FriendStats>('/admin/stats/users/friends')
}

/**
 * 获取房间活跃度排行
 * @param limit 返回数量
 * @returns 房间活跃度排行
 */
export function getRoomActivityRanking(limit?: number): Promise<ApiResponse<RoomActivity[]>> {
  return http.get<RoomActivity[]>('/admin/stats/rooms/activity', { params: { limit } })
}

/**
 * 获取房间统计概览
 * @returns 房间统计概览
 */
export function getRoomStats(): Promise<ApiResponse<RoomStats>> {
  return http.get<RoomStats>('/admin/stats/rooms/overview')
}

/**
 * 获取消息类型分布统计
 * @returns 消息类型统计数据
 */
export function getMessageTypeStats(): Promise<ApiResponse<MessageTypeStats>> {
  return http.get<MessageTypeStats>('/admin/stats/messages/types')
}

/**
 * 获取消息时间分布统计
 * @returns 消息时间分布数据
 */
export function getMessageHourlyDistribution(): Promise<ApiResponse<MessageHourlyDistribution[]>> {
  return http.get<MessageHourlyDistribution[]>('/admin/stats/messages/hourly')
}

/**
 * 获取安全告警统计
 * @returns 安全告警统计数据
 */
export function getSecurityStats(): Promise<ApiResponse<SecurityStats>> {
  return http.get<SecurityStats>('/admin/stats/security')
}
