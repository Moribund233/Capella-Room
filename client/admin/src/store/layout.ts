import { defineStore } from 'pinia'
import { ref, computed, type Ref } from 'vue'

/**
 * 布局样式配置接口
 */
export interface LayoutStyles {
  /** 侧边栏宽度（展开状态） */
  sidebarWidthExpanded: number
  /** 侧边栏宽度（折叠状态） */
  sidebarWidthCollapsed: number
  /** 侧边栏高度比例 (0-1)，1 表示全高，小于 1 表示紧凑型 */
  sidebarHeightRatio: number
  /** 头部高度 */
  headerHeight: number
  /** 头部宽度比例 (0-1)，1 表示全宽，小于 1 表示紧凑型 */
  headerWidthRatio: number
  /** 底部高度 */
  footerHeight: number
  /** 底部宽度比例 (0-1)，1 表示全宽，小于 1 表示紧凑型 */
  footerWidthRatio: number
  /** 侧边栏圆角 */
  sidebarBorderRadius: number
  /** 头部圆角 */
  headerBorderRadius: number
  /** 底部圆角 */
  footerBorderRadius: number
  /** 侧边栏透明度 (0-1) */
  sidebarOpacity: number
  /** 头部透明度 (0-1) */
  headerOpacity: number
  /** 底部透明度 (0-1) */
  footerOpacity: number
  /** 背景图片 URL */
  backgroundImage: string
  /** 背景图片不透明度 (0-1) */
  backgroundOpacity: number
}

/**
 * 默认布局样式
 */
const defaultLayoutStyles: LayoutStyles = {
  sidebarWidthExpanded: 240,
  sidebarWidthCollapsed: 64,
  sidebarHeightRatio: 1,
  headerHeight: 64,
  headerWidthRatio: 1,
  footerHeight: 48,
  footerWidthRatio: 1,
  sidebarBorderRadius: 0,
  headerBorderRadius: 0,
  footerBorderRadius: 0,
  sidebarOpacity: 1,
  headerOpacity: 1,
  footerOpacity: 1,
  backgroundImage: '',
  backgroundOpacity: 0.15,
}

/**
 * 布局状态存储
 * 集中管理所有 Layout 相关状态
 */
export const useLayoutStore = defineStore('layout', () => {
  // ========== 状态定义 ==========

  /** 侧边栏折叠状态（桌面端） */
  const isSidebarCollapsed = ref(false)

  /** 移动端菜单打开状态 */
  const isMobileMenuOpen = ref(false)

  /** Footer 显示状态 */
  const isFooterVisible = ref(false)

  /** StatusBar 显示状态 */
  const isStatusBarVisible = ref(false)

  /** StatusBar 是否有内容 */
  const hasStatusBarContent = ref(false)

  /** 响应式断点 */
  const isDesktop = ref(window.innerWidth > 1024)
  const isTablet = ref(window.innerWidth >= 768 && window.innerWidth <= 1024)
  const isMobile = ref(window.innerWidth < 768)

  /** 布局样式配置 */
  const layoutStyles = ref<LayoutStyles>({ ...defaultLayoutStyles })

  // ========== 计算属性 ==========

  /**
   * 侧边栏是否打开（跨端统一状态）
   */
  const isSidebarOpen = computed(() => {
    if (isMobile.value || isTablet.value) {
      return isMobileMenuOpen.value
    }
    return !isSidebarCollapsed.value
  })

  /**
   * 当前侧边栏宽度（用于 DockBar 定位）
   */
  const sidebarWidth = computed(() => {
    if (isMobile.value) return '0px'

    const widthVar = isDesktop.value
      ? isSidebarCollapsed.value
        ? '--sidebar-width-collapsed'
        : '--sidebar-width-expanded'
      : isSidebarCollapsed.value
        ? '--sidebar-width-tablet-collapsed'
        : '--sidebar-width-tablet-expanded'

    return `var(${widthVar})`
  })

  /**
   * CSS 变量样式对象
   * 用于动态更新布局组件样式
   */
  const cssVariables = computed(() => {
    const styles = layoutStyles.value
    const { sidebarHeightRatio, headerWidthRatio, footerWidthRatio } = styles

    // 计算紧凑型布局的偏移量，使其居中
    // 侧边栏需要减去 header 和 footer 的高度
    const sidebarTopOffset = `calc(var(--header-height) + (100vh - var(--header-height) - var(--footer-height)) * (1 - ${sidebarHeightRatio}) / 2)`
    const sidebarHeight = `calc((100vh - var(--header-height) - var(--footer-height)) * ${sidebarHeightRatio})`

    const headerMargin = `calc(100vw * (1 - ${headerWidthRatio}) / 2)`
    const headerWidth = `calc(100vw * ${headerWidthRatio})`

    const footerMargin = `calc(100vw * (1 - ${footerWidthRatio}) / 2)`
    const footerWidth = `calc(100vw * ${footerWidthRatio})`

    return {
      '--sidebar-width-expanded': `${styles.sidebarWidthExpanded}px`,
      '--sidebar-width-collapsed': `${styles.sidebarWidthCollapsed}px`,
      '--sidebar-width-tablet-expanded': `${Math.max(160, styles.sidebarWidthExpanded - 40)}px`,
      '--sidebar-width-tablet-collapsed': `${styles.sidebarWidthCollapsed}px`,
      '--sidebar-height-ratio': String(styles.sidebarHeightRatio),
      '--sidebar-top-offset': sidebarTopOffset,
      '--sidebar-compact-height': sidebarHeight,
      '--header-height': `${styles.headerHeight}px`,
      '--header-height-mobile': `${Math.max(48, styles.headerHeight - 8)}px`,
      '--header-width-ratio': String(styles.headerWidthRatio),
      '--header-margin': headerMargin,
      '--header-compact-width': headerWidth,
      '--footer-height': `${styles.footerHeight}px`,
      '--footer-width-ratio': String(styles.footerWidthRatio),
      '--footer-margin': footerMargin,
      '--footer-compact-width': footerWidth,
      '--sidebar-border-radius': `${styles.sidebarBorderRadius}px`,
      '--header-border-radius': `${styles.headerBorderRadius}px`,
      '--footer-border-radius': `${styles.footerBorderRadius}px`,
      '--sidebar-opacity': String(styles.sidebarOpacity),
      '--header-opacity': String(styles.headerOpacity),
      '--footer-opacity': String(styles.footerOpacity),
    } as Record<string, string>
  })

  // ========== Actions ==========

  /**
   * 切换侧边栏
   * 桌面端：折叠/展开
   * 移动端/平板端：打开/关闭浮层
   */
  function toggleSidebar(): void {
    if (isMobile.value || isTablet.value) {
      isMobileMenuOpen.value = !isMobileMenuOpen.value
    } else {
      isSidebarCollapsed.value = !isSidebarCollapsed.value
    }
  }

  /**
   * 关闭移动端菜单
   */
  function closeMobileMenu(): void {
    isMobileMenuOpen.value = false
  }

  /**
   * 切换 Footer 显隐
   */
  function toggleFooter(): void {
    isFooterVisible.value = !isFooterVisible.value
  }

  /**
   * 设置 StatusBar 内容状态
   * @param hasContent 是否有内容
   */
  function setStatusBarContent(hasContent: boolean): void {
    hasStatusBarContent.value = hasContent
    // 如果没有内容，自动隐藏 StatusBar
    if (!hasContent) {
      isStatusBarVisible.value = false
    }
  }

  /**
   * 设置 StatusBar 显示状态
   * @param visible 是否显示
   */
  function setStatusBarVisible(visible: boolean): void {
    isStatusBarVisible.value = visible && hasStatusBarContent.value
  }

  /**
   * 切换 StatusBar 显示状态
   */
  function toggleStatusBar(): void {
    if (hasStatusBarContent.value) {
      isStatusBarVisible.value = !isStatusBarVisible.value
    }
  }

  /**
   * 更新响应式断点
   */
  function updateBreakpoint(): void {
    const width = window.innerWidth
    isDesktop.value = width > 1024
    isTablet.value = width >= 768 && width <= 1024
    isMobile.value = width < 768
  }

  /**
   * 更新布局样式
   * @param styles 部分样式配置
   */
  function updateLayoutStyles(styles: Partial<LayoutStyles>): void {
    layoutStyles.value = { ...layoutStyles.value, ...styles }
  }

  /**
   * 重置布局样式为默认值
   */
  function resetLayoutStyles(): void {
    layoutStyles.value = { ...defaultLayoutStyles }
  }

  // ========== 状态查询（供 Quick 组合式函数使用）==========

  /**
   * 获取指定 key 的状态
   * @param key - 'sidebar' | 'footer'
   */
  function getState(key: string): boolean {
    const stateMap: Record<string, Ref<boolean>> = {
      sidebar: isSidebarOpen,
      footer: isFooterVisible,
    }
    return stateMap[key]?.value ?? false
  }

  /**
   * 切换指定 key 的状态
   * @param key - 'sidebar' | 'footer'
   */
  function toggle(key: string): void {
    const toggleMap: Record<string, () => void> = {
      sidebar: toggleSidebar,
      footer: toggleFooter,
    }
    toggleMap[key]?.()
  }

  return {
    // 状态
    isSidebarCollapsed,
    isMobileMenuOpen,
    isFooterVisible,
    isStatusBarVisible,
    hasStatusBarContent,
    isDesktop,
    isTablet,
    isMobile,
    layoutStyles,
    // 计算属性
    isSidebarOpen,
    sidebarWidth,
    cssVariables,
    // Actions
    toggleSidebar,
    closeMobileMenu,
    toggleFooter,
    setStatusBarContent,
    setStatusBarVisible,
    toggleStatusBar,
    updateBreakpoint,
    updateLayoutStyles,
    resetLayoutStyles,
    // 通用接口
    getState,
    toggle,
  }
})

export type LayoutStore = ReturnType<typeof useLayoutStore>
