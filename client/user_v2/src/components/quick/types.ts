/**
 * QuickBar 类型定义
 */

/**
 * Quick 子菜单项
 */
export interface QuickChildItem {
  /** 唯一标识 */
  key: string
  /** 显示标签 */
  label: string
  /** 是否禁用 */
  disabled?: boolean
}

/**
 * Quick 按钮显示模式
 * - visible: 外显在快捷栏中
 * - dropdown: 聚合在下拉菜单中
 */
export type QuickDisplayMode = 'visible' | 'dropdown'

/**
 * Quick 配置项
 */
export interface QuickItem {
  /** 唯一标识 */
  key: string
  /** 显示模式：外显或下拉菜单 */
  display: QuickDisplayMode
  /** 图标组件 */
  icon: unknown
  /** 显示标签 */
  label: string
  /** 是否激活 */
  isActive?: boolean
  /** 徽章数值 */
  badge?: number
  /** 子菜单项 */
  children?: QuickChildItem[]
  /** 点击回调 */
  onClick?: () => void
  /** 子菜单选择回调 */
  onSelect?: (childKey: string) => void
}
