/**
 * Avatar utility functions
 * 统一的头像渐变和投影样式
 */

/** 8 种预定义渐变组合（使用 CSS 变量以支持主题切换） */
const avatarGradients = [
  'linear-gradient(135deg, var(--accent), var(--accent-pink))',
  'linear-gradient(135deg, var(--accent-blue), var(--accent))',
  'linear-gradient(135deg, var(--accent-green), var(--accent-blue))',
  'linear-gradient(135deg, var(--accent-orange), var(--accent-pink))',
  'linear-gradient(135deg, var(--accent), var(--accent-green))',
  'linear-gradient(135deg, var(--accent-pink), var(--accent-orange))',
  'linear-gradient(135deg, var(--accent-blue), var(--accent-green))',
  'linear-gradient(135deg, var(--accent-orange), var(--accent))',
]

/** 简单的字符串哈希函数 */
function hashStr(str: string): number {
  let hash = 0
  for (let i = 0; i < str.length; i++) {
    hash = ((hash << 5) - hash + str.charCodeAt(i)) | 0
  }
  return Math.abs(hash)
}

/** 根据名称获取确定性渐变 */
export function getAvatarGradient(name: string): string {
  return avatarGradients[hashStr(name) % avatarGradients.length]!
}

/** 根据名称获取单色背景（用于小尺寸头像） */
const avatarColors = [
  'var(--accent)',
  'var(--accent-pink)',
  'var(--accent-green)',
  'var(--accent-orange)',
  'var(--accent-blue)',
]

export function getAvatarColor(name: string): string {
  return avatarColors[hashStr(name) % avatarColors.length]!
}

/** 获取头像投影样式 */
export function getAvatarShadow(size: 'sm' | 'md' | 'lg' = 'md'): string {
  const shadows = {
    sm: '0 2px 8px rgba(124, 92, 252, 0.2)',
    md: '0 4px 14px rgba(124, 92, 252, 0.25)',
    lg: '0 6px 20px rgba(124, 92, 252, 0.3)',
  }
  return shadows[size]
}

/** 头像 hover 投影 */
export function getAvatarHoverShadow(size: 'sm' | 'md' | 'lg' = 'md'): string {
  const shadows = {
    sm: '0 4px 12px rgba(124, 92, 252, 0.3)',
    md: '0 6px 18px rgba(124, 92, 252, 0.35)',
    lg: '0 8px 24px rgba(124, 92, 252, 0.4)',
  }
  return shadows[size]
}

/** 头像尺寸映射 */
export const avatarSizes = {
  xs: 24,
  sm: 32,
  md: 40,
  lg: 56,
  xl: 72,
  xxl: 96,
} as const

export type AvatarSize = keyof typeof avatarSizes
