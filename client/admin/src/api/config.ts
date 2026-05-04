import { http } from './request'
import type { ApiResponse } from '@/types'

/**
 * 客户端 WebSocket 配置
 */
export interface ClientWebSocketConfig {
  /** 心跳间隔（秒） */
  heartbeat_interval_secs: number
  /** 心跳超时（秒） */
  heartbeat_timeout_secs: number
  /** 认证超时（秒） */
  auth_timeout_secs: number
}

/**
 * 客户端重连配置
 */
export interface ClientReconnectConfig {
  /** 基础延迟（毫秒） */
  base_delay_ms: number
  /** 最大延迟（毫秒） */
  max_delay_ms: number
  /** 最大重连次数 */
  max_attempts: number
  /** 延迟倍数（指数退避） */
  multiplier: number
}

/**
 * 客户端上传配置
 */
export interface ClientUploadConfig {
  /** 最大文件大小（字节） */
  max_file_size: number
  /** 人类可读的最大文件大小（如 "10MB"） */
  max_file_size_human: string
}

/**
 * 客户端系统配置
 */
export interface ClientSystemConfig {
  /** 系统名称 */
  name: string
  /** 系统版本 */
  version: string
  /** 是否处于维护模式 */
  maintenance_mode: boolean
  /** 维护模式提示消息 */
  maintenance_message: string
}

/**
 * 客户端监控配置
 */
export interface ClientMonitorConfig {
  /** 监控数据刷新频率（秒） */
  refresh_interval_secs: number
}

/**
 * 客户端配置
 */
export interface ClientConfig {
  /** WebSocket 配置 */
  websocket: ClientWebSocketConfig
  /** 重连配置 */
  reconnect: ClientReconnectConfig
  /** 上传配置 */
  upload: ClientUploadConfig
  /** 系统状态 */
  system: ClientSystemConfig
  /** 监控配置 */
  monitor: ClientMonitorConfig
}

/**
 * 获取客户端配置
 * @returns 客户端配置
 */
export function getClientConfig(): Promise<ApiResponse<ClientConfig>> {
  // 客户端配置端点不在 /api/v1 下，需要移除 v1 后缀
  const baseUrl = (import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080/api/v1').replace('/api/v1', '')
  return http.get<ClientConfig>(`${baseUrl}/api/config/client`)
}

/**
 * 获取心跳超时时间（毫秒）
 * 使用服务端配置的 heartbeat_timeout_secs，并留出 80% 的缓冲时间
 * @param config 客户端配置
 * @returns 超时时间（毫秒）
 */
export function getHeartbeatTimeoutMs(config: ClientConfig | null): number {
  const timeoutSecs = config?.websocket?.heartbeat_timeout_secs ?? 90
  // 使用服务端超时时间的 80% 作为客户端检测阈值（留出缓冲）
  return Math.floor(timeoutSecs * 1000 * 0.8)
}

// ==================== 管理员配置管理类型 ====================

/**
 * 配置项数据类型
 */
export type ConfigValueType = 'string' | 'integer' | 'float' | 'boolean' | 'json'

/**
 * 配置项
 */
export interface ConfigItem {
  /** 配置键 */
  key: string
  /** 配置值 */
  value: unknown
  /** 配置值类型 */
  value_type: ConfigValueType
  /** 配置分类 */
  category: string
  /** 配置描述 */
  description: string
  /** 默认值 */
  default_value: unknown
  /** 是否可编辑 */
  is_editable: boolean
  /** 是否支持热重载 */
  is_hot_reloadable: boolean
  /** 最后更新时间 */
  updated_at?: string
}

/**
 * 配置列表查询参数
 */
export interface ConfigListParams {
  /** 配置分类 */
  category?: string
}

/**
 * 更新配置请求
 */
export interface UpdateConfigRequest {
  /** 新配置值 */
  value: unknown
}

/**
 * 配置同步状态
 */
export interface ConfigSyncStatus {
  /** 是否正在同步 */
  is_syncing: boolean
  /** 最后同步时间 */
  last_sync_at?: string
  /** 同步节点数 */
  node_count: number
  /** 同步成功节点数 */
  success_count: number
}

// ==================== 管理员配置管理 API ====================

/**
 * 配置管理 API
 */
export const configApi = {
  /**
   * 获取所有配置项
   * @param params 查询参数
   * @returns 配置项列表
   */
  getConfigs: (params?: ConfigListParams): Promise<ApiResponse<ConfigItem[]>> => {
    const queryParams = new URLSearchParams()
    if (params?.category) queryParams.append('category', params.category)

    const query = queryParams.toString()
    const url = query ? `/admin/configs?${query}` : '/admin/configs'

    return http.get<ConfigItem[]>(url)
  },

  /**
   * 获取指定配置项
   * @param key 配置键
   * @returns 配置项详情
   */
  getConfig: (key: string): Promise<ApiResponse<ConfigItem>> =>
    http.get<ConfigItem>(`/admin/configs/${key}`),

  /**
   * 更新配置项
   * @param key 配置键
   * @param data 更新数据
   * @returns 更新后的配置项
   */
  updateConfig: (key: string, data: UpdateConfigRequest): Promise<ApiResponse<ConfigItem>> =>
    http.put<ConfigItem>(`/admin/configs/${key}`, data),

  /**
   * 重置配置到默认值
   * @returns 操作结果
   */
  resetConfigs: (): Promise<ApiResponse<void>> =>
    http.post<void>('/admin/configs/reset', {}),

  /**
   * 获取配置同步状态
   * @returns 配置同步状态
   */
  getSyncStatus: (): Promise<ApiResponse<ConfigSyncStatus>> =>
    http.get<ConfigSyncStatus>('/admin/config/sync/status'),

  /**
   * 触发配置同步
   * @returns 操作结果
   */
  triggerSync: (): Promise<ApiResponse<void>> =>
    http.post<void>('/admin/config/sync', {}),
}
