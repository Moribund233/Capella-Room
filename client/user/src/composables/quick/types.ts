import type { Ref } from 'vue'
import type { LayoutStore } from '@/store/layout'

/**
 * Quick 按钮显示模式
 * - visible: 外显在 header-right 中
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
 * Quick 子菜单项
 */
export interface QuickChildItem {
  /** 唯一标识 */
  key: string
  /** 显示标签 */
  label: string
  /** 图标名（lucide 图标名） */
  icon?: string
  /** 是否禁用 */
  disabled?: boolean
}

/**
 * Quick 配置项（来自 ui.ts 配置）
 */
export interface QuickConfigItem {
  /** 唯一标识 */
  key: string
  /** 显示模式：外显或下拉菜单 */
  display: QuickDisplayMode
  /** 类型：直接执行或有子菜单 */
  type: QuickItemType
  /** 主图标名（lucide 图标名） */
  icon: string
  /** 替代图标名（用于状态切换，如主题明暗图标） */
  iconAlt?: string
  /** 显示标签 */
  label: string
  /** 徽标数字 */
  badge?: string | number
  /** 子菜单项（type为menu时有效） */
  children?: QuickChildItem[]
  /**
   * 关联的 layout key
   * 用于自动绑定 LayoutStore 状态
   */
  layoutKey?: 'sidebar' | 'footer'
}

/**
 * Quick 运行时项（组件内部使用）
 */
export interface QuickRuntimeItem extends QuickConfigItem {
  /** 当前是否激活 */
  isActive: boolean
  /** 当前显示的图标（根据状态动态切换） */
  currentIcon: string
  /** 点击处理函数 */
  onClick: () => void
  /** 子菜单选择处理函数 */
  onSelect?: (childKey: string) => void
}

/**
 * Quick 组合式函数返回接口规范
 * 所有 Quick 组合式函数必须实现此接口
 */
export interface UseQuickReturn {
  /** 当前是否激活（用于图标切换） */
  isActive: Ref<boolean>
  /** 当前显示的图标（用于状态切换） */
  currentIcon: Ref<string>
  /** 点击主按钮的处理函数 */
  onClick: () => void
  /** 选择子菜单项的处理函数（可选，仅 menu 类型需要） */
  onSelect?: (childKey: string) => void
}

/**
 * Quick 组合式函数工厂类型
 */
export type QuickFactory = (config: QuickConfigItem, context: QuickContext) => UseQuickReturn

/**
 * Quick 上下文（简化版）
 * 保留 emitAction 用于非 layout 相关的自定义操作
 */
export interface QuickContext {
  /** 触发外部自定义动作 */
  emitAction: (key: string, childKey?: string) => void
  /** LayoutStore 实例（可选，用于高级自定义） */
  layoutStore?: LayoutStore
}
