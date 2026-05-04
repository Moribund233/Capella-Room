/** 服务端下发的客户端配置 */
export interface ClientConfig {
  websocket: ClientWebSocketConfig
  reconnect: ClientReconnectConfig
  upload: ClientUploadConfig
  system: ClientSystemConfig
  monitor: ClientMonitorConfig
}

export interface ClientWebSocketConfig {
  heartbeat_interval_secs: number
  heartbeat_timeout_secs: number
  auth_timeout_secs: number
}

export interface ClientReconnectConfig {
  base_delay_ms: number
  max_delay_ms: number
  max_attempts: number
  multiplier: number
}

export interface ClientUploadConfig {
  max_file_size: number
  max_file_size_human: string
}

export interface ClientSystemConfig {
  name: string
  version: string
  maintenance_mode: boolean
  maintenance_message: string
}

export interface ClientMonitorConfig {
  refresh_interval_secs: number
}
