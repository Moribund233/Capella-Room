/**
 * QuickBar 配置文件
 * 定义快捷按钮的配置项
 */
import type { Ref, ComputedRef } from 'vue'

/**
 * Quick 子菜单项
 */
export interface QuickChildItem {
  /** 唯一标识 */
  key: string
  /** 显示标签 */
  label: string
  /** 图标名（element-plus 图标名） */
  icon?: string
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
 * Quick 按钮类型
 * - action: 直接执行动作
 * - menu: 有子菜单
 */
export type QuickItemType = 'action' | 'menu'

/**
 * Quick 配置项
 */
export interface QuickItem {
  /** 唯一标识 */
  key: string
  /** 显示模式：外显或下拉菜单 */
  display: QuickDisplayMode
  /** 类型：直接执行或有子菜单 */
  type: QuickItemType
  /** 主图标名（element-plus 图标名） */
  icon: string
  /** 替代图标名（用于状态切换，如主题明暗图标） */
  iconAlt?: string
  /** 显示标签 */
  label: string
  /** 子菜单项（type为menu时有效） */
  children?: QuickChildItem[]
  /** 关联的处理函数名称 */
  handler: string
}

/**
 * Quick 运行时项（组件内部使用）
 */
export interface QuickRuntimeItem extends QuickItem {
  /** 当前是否激活（支持响应式） */
  isActive: boolean | ComputedRef<boolean>
  /** 当前显示的图标（根据状态动态切换，支持响应式） */
  currentIcon: string | ComputedRef<string>
  /** 是否禁用 */
  disabled: boolean
  /** 点击处理函数 */
  onClick: () => void
  /** 子菜单选择处理函数 */
  onSelect?: (childKey: string) => void
  /** 徽标数字（支持响应式 Ref） */
  badge?: number | Ref<number>
  /** 允许扩展其他属性 */
  [key: string]: unknown
}

/**
 * QuickBar 配置
 */
export const quickBarConfig: QuickItem[] = [
  {
    key: 'personalization',
    display: 'visible',
    type: 'action',
    icon: 'Brush',
    label: '个性化',
    handler: 'useQuickPersonalization',
  },
  {
    key: 'notification',
    display: 'visible',
    type: 'action',
    icon: 'Bell',
    label: '通知',
    handler: 'useQuickNotification',
  },
]
