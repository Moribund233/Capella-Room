/**
 * 客户端配置类型定义
 * 对应后端 /api/config/client 端点返回的数据结构
 */

/** 客户端配置 */
export interface ClientConfig {
  /** WebSocket 配置 */
  websocket: ClientWebSocketConfig
  /** 重连配置 */
  reconnect: ClientReconnectConfig
  /** 上传配置 */
  upload: ClientUploadConfig
  /** 系统状态 */
  system: ClientSystemConfig
}

/** 客户端 WebSocket 配置 */
export interface ClientWebSocketConfig {
  /** 心跳间隔（秒） */
  heartbeat_interval_secs: number
  /** 心跳超时（秒） */
  heartbeat_timeout_secs: number
  /** 认证超时（秒） */
  auth_timeout_secs: number
}

/** 客户端重连配置 */
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

/** 客户端上传配置 */
export interface ClientUploadConfig {
  /** 最大文件大小（字节） */
  max_file_size: number
  /** 人类可读的最大文件大小（如 "10MB"） */
  max_file_size_human: string
}

/** 客户端系统配置 */
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
