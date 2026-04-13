/**
 * 配置模块统一导出
 */

export {
  // 初始化函数
  initWebSocketConfig,
  reloadWebSocketConfig,

  // 配置获取函数
  getServerConfig,
  getConnectionConfig,
  getReconnectStrategy,
  getHeartbeatConfig,

  // 工具函数
  calculateReconnectDelay,
  onConfigChange,

  // 状态查询
  isConfigInitialized,
  hasServerConfig,
  getInitError,

  // Vue 组合式函数
  useWebSocketConfig,

  // 默认值
  DEFAULT_CONNECTION_CONFIG,
  DEFAULT_RECONNECT_STRATEGY,

  // 类型
  type WebSocketConnectionConfig,
  type ReconnectStrategyConfig,
} from './websocketConfig'

// 默认导出
export { default } from './websocketConfig'
