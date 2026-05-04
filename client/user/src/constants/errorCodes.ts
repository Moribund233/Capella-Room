export const ErrorCodes = {
  AUTH_ERROR: '认证失败',
  TOKEN_EXPIRED: 'Token 已过期',
  TOKEN_INVALID: 'Token 无效',
  VALIDATION_ERROR: '请求参数错误',
  NOT_FOUND: '资源不存在',
  CONFLICT: '资源已存在',
  FORBIDDEN: '权限不足',
  INTERNAL_ERROR: '服务器内部错误',
} as const

export function getErrorMessage(code: string): string {
  return (ErrorCodes as Record<string, string>)[code] || '未知错误'
}
