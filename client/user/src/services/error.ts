import type { App } from 'vue'

/**
 * 错误类型枚举
 */
export enum ErrorType {
  NETWORK = 'NETWORK',
  AUTH = 'AUTH',
  VALIDATION = 'VALIDATION',
  SERVER = 'SERVER',
  UNKNOWN = 'UNKNOWN',
  WEBSOCKET = 'WEBSOCKET',
}

/**
 * 应用错误类
 */
export class AppError extends Error {
  public readonly type: ErrorType
  public readonly code?: string
  public readonly details?: unknown

  constructor(message: string, type: ErrorType = ErrorType.UNKNOWN, code?: string, details?: unknown) {
    super(message)
    this.name = 'AppError'
    this.type = type
    this.code = code
    this.details = details
  }
}

/**
 * 错误处理器接口
 */
export interface ErrorHandler {
  handle(error: unknown): void
}

/**
 * 全局错误处理器
 */
class GlobalErrorHandler implements ErrorHandler {
  private handlers: Map<ErrorType, (error: AppError) => void> = new Map()
  private defaultHandler: (error: AppError) => void

  constructor() {
    this.defaultHandler = (error: AppError) => {
      console.error('[AppError]', error)
    }
  }

  /**
   * 注册特定类型的错误处理器
   */
  register(type: ErrorType, handler: (error: AppError) => void): void {
    this.handlers.set(type, handler)
  }

  /**
   * 设置默认错误处理器
   */
  setDefaultHandler(handler: (error: AppError) => void): void {
    this.defaultHandler = handler
  }

  /**
   * 处理错误
   */
  handle(error: unknown): void {
    const appError = this.normalizeError(error)
    const handler = this.handlers.get(appError.type)

    if (handler) {
      handler(appError)
    } else {
      this.defaultHandler(appError)
    }
  }

  /**
   * 标准化错误
   */
  private normalizeError(error: unknown): AppError {
    if (error instanceof AppError) {
      return error
    }

    if (error instanceof Error) {
      return new AppError(error.message, ErrorType.UNKNOWN)
    }

    if (typeof error === 'string') {
      return new AppError(error, ErrorType.UNKNOWN)
    }

    return new AppError('发生未知错误', ErrorType.UNKNOWN, undefined, error)
  }
}

export const globalErrorHandler = new GlobalErrorHandler()

/**
 * 初始化全局错误处理
 */
export function initErrorHandler(app: App): void {
  // Vue 错误处理
  app.config.errorHandler = (err, vm, info) => {
    console.error('[Vue Error]', err)
    console.error('[Component]', vm)
    console.error('[Info]', info)
    globalErrorHandler.handle(err)
  }

  // 全局未捕获的 Promise 错误
  window.addEventListener('unhandledrejection', (event) => {
    console.error('[Unhandled Promise Rejection]', event.reason)
    globalErrorHandler.handle(event.reason)
  })

  // 全局 JS 错误
  window.addEventListener('error', (event) => {
    console.error('[Global Error]', event.error)
    console.error('[Filename]', event.filename)
    console.error('[Line]', event.lineno, '[Column]', event.colno)
    globalErrorHandler.handle(event.error)
  })
}

/**
 * 网络错误处理
 */
export function handleNetworkError(error: unknown): AppError {
  if (error instanceof AppError) {
    return error
  }

  const message = error instanceof Error ? error.message : '网络请求失败'
  return new AppError(message, ErrorType.NETWORK)
}

/**
 * 认证错误处理
 */
export function handleAuthError(error: unknown): AppError {
  const message = error instanceof Error ? error.message : '认证失败'
  return new AppError(message, ErrorType.AUTH)
}
