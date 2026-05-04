/**
 * 类型定义导出
 */

export * from './api'
export * from './websocket'
export * from './types'

// 扩展 Window 接口以支持 Naive UI 的全局属性
declare global {
  interface Window {
    $message?: import('naive-ui').MessageApi
  }
}
