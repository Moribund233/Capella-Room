import { ref, computed, type VNode, type FunctionalComponent } from 'vue'
import type { LucideProps } from 'lucide-vue-next'

/**
 * 状态项类型
 */
interface StatusItem {
  /** 标签文本 */
  label?: string
  /** 值文本 */
  value?: string | number
  /** 值的样式类 */
  valueClass?: string
  /** 图标组件 */
  icon?: FunctionalComponent<LucideProps>
  /** 点击跳转链接 */
  href?: string
  /** 点击回调 */
  onClick?: () => void
}

/**
 * 状态栏内容配置
 */
interface StatusBarContent {
  /** 显示的文本内容 */
  text?: string
  /** 状态项数组 */
  items?: StatusItem[]
  /** 自定义渲染函数 */
  render?: () => VNode | VNode[]
}

/**
 * 状态栏状态
 */
const content = ref<StatusBarContent | null>(null)
const visible = ref(true)

/**
 * 状态栏配置
 */
const config = ref({
  speed: 30,
  pauseDuration: 2000,
})

/**
 * 是否有内容
 */
const hasContent = computed(() => {
  if (!content.value) return false
  return !!(content.value.text || (content.value.items && content.value.items.length > 0) || content.value.render)
})

/**
 * 设置状态栏内容
 * @param newContent 状态栏内容配置
 */
function setContent(newContent: StatusBarContent | null): void {
  content.value = newContent
}

/**
 * 设置状态栏文本
 * @param text 文本内容
 */
function setText(text: string): void {
  content.value = { text }
}

/**
 * 设置状态栏项目
 * @param items 状态项数组
 */
function setItems(items: StatusItem[]): void {
  content.value = { items }
}

/**
 * 清除状态栏内容
 */
function clear(): void {
  content.value = null
}

/**
 * 显示状态栏
 */
function show(): void {
  visible.value = true
}

/**
 * 隐藏状态栏
 */
function hide(): void {
  visible.value = false
}

/**
 * 切换状态栏显示
 */
function toggle(): void {
  visible.value = !visible.value
}

/**
 * 更新配置
 * @param newConfig 部分配置
 */
function updateConfig(newConfig: Partial<typeof config.value>): void {
  config.value = { ...config.value, ...newConfig }
}

/**
 * StatusBar 组合式函数
 * 用于管理页面底部状态栏的内容和显示状态
 *
 * @example
 * // 设置文本
 * useStatusBar().setText('系统运行正常')
 *
 * @example
 * // 设置状态项
 * useStatusBar().setItems([
 *   { label: '在线用户', value: 100, icon: Users },
 *   { label: '延迟', value: '24ms' },
 * ])
 *
 * @example
 * // 设置完整内容
 * useStatusBar().setContent({
 *   items: [...],
 *   render: () => h('span', '自定义内容')
 * })
 *
 * @example
 * // 页面卸载时清除
 * onUnmounted(() => {
 *   useStatusBar().clear()
 * })
 */
export function useStatusBar() {
  return {
    // 状态
    content: computed(() => content.value),
    visible: computed(() => visible.value),
    hasContent,
    config: computed(() => config.value),
    // 方法
    setContent,
    setText,
    setItems,
    clear,
    show,
    hide,
    toggle,
    updateConfig,
  }
}

export type { StatusItem, StatusBarContent }
