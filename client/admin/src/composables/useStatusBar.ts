import { ref, computed, type VNode } from 'vue'

/**
 * StatusBar 内容项类型
 * 支持字符串和 VNode（用于链接等简单动态内容）
 * 不支持组件类型
 */
export type StatusBarContent = string | VNode | (string | VNode)[]

/**
 * StatusBar 状态存储（全局单例）
 */
const statusBarContent = ref<StatusBarContent>('')
const statusBarVisible = ref(false)

/**
 * 使用 StatusBar
 * 提供页面级状态栏内容设置功能
 *
 * @example
 * // 在页面组件中使用
 * const { setContent, clearContent } = useStatusBar()
 *
 * // 设置简单文本
 * setContent('系统状态: 正常运行')
 *
 * // 设置带链接的内容
 * setContent([
 *   '发现新版本 ',
 *   h('a', { href: '#', onClick: handleUpdate }, '立即更新')
 * ])
 *
 * // 页面卸载时清除
 * onUnmounted(() => {
 *   clearContent()
 * })
 */
export function useStatusBar() {
  /**
   * 当前内容
   */
  const content = computed(() => statusBarContent.value)

  /**
   * 是否有内容
   */
  const hasContent = computed(() => {
    if (!statusBarContent.value) return false
    if (typeof statusBarContent.value === 'string') {
      return statusBarContent.value.trim().length > 0
    }
    return true
  })

  /**
   * 是否可见
   */
  const visible = computed(() => statusBarVisible.value && hasContent.value)

  /**
   * 设置状态栏内容
   * @param content 内容（字符串或 VNode）
   */
  function setContent(content: StatusBarContent): void {
    statusBarContent.value = content
    statusBarVisible.value = true
  }

  /**
   * 清除状态栏内容
   */
  function clearContent(): void {
    statusBarContent.value = ''
    statusBarVisible.value = false
  }

  /**
   * 显示状态栏（如果已有内容）
   */
  function show(): void {
    if (hasContent.value) {
      statusBarVisible.value = true
    }
  }

  /**
   * 隐藏状态栏（不清除内容）
   */
  function hide(): void {
    statusBarVisible.value = false
  }

  /**
   * 切换显示状态
   */
  function toggle(): void {
    statusBarVisible.value = !statusBarVisible.value
  }

  return {
    // 状态
    content,
    hasContent,
    visible,
    // 方法
    setContent,
    clearContent,
    show,
    hide,
    toggle,
  }
}

/**
 * 获取 StatusBar 全局状态（用于组件内部）
 */
export function getStatusBarState() {
  return {
    content: computed(() => statusBarContent.value),
    hasContent: computed(() => {
      if (!statusBarContent.value) return false
      if (typeof statusBarContent.value === 'string') {
        return statusBarContent.value.trim().length > 0
      }
      return true
    }),
    visible: statusBarVisible,
  }
}

/**
 * 设置 StatusBar 可见性（用于组件内部）
 */
export function setStatusBarVisible(value: boolean): void {
  statusBarVisible.value = value
}
