import { ref, computed, onMounted, onUnmounted } from 'vue'

/**
 * 响应式断点类型
 */
export type BreakpointType = 'mobile' | 'tablet' | 'desktop'

/**
 * 断点配置
 */
export interface BreakpointConfig {
  /** 移动端最大宽度 (px) */
  mobile: number
  /** 平板端最大宽度 (px) */
  tablet: number
}

/**
 * 默认断点配置
 */
const defaultBreakpoints: BreakpointConfig = {
  mobile: 768,
  tablet: 1024,
}

/**
 * 响应式表格配置选项
 */
export interface ResponsiveTableOptions {
  /** 断点配置 */
  breakpoints?: Partial<BreakpointConfig>
  /** 是否在移动端使用卡片布局 */
  useCardOnMobile?: boolean
  /** 移动端默认显示的最大列数 */
  mobileMaxColumns?: number
  /** 平板端默认显示的最大列数 */
  tabletMaxColumns?: number
}

/**
 * 响应式表格状态
 */
export interface ResponsiveTableState {
  /** 当前断点 */
  breakpoint: BreakpointType
  /** 是否为移动端 */
  isMobile: boolean
  /** 是否为平板端 */
  isTablet: boolean
  /** 是否为桌面端 */
  isDesktop: boolean
  /** 是否使用卡片布局 */
  useCardLayout: boolean
  /** 当前应显示的最大列数 */
  maxColumns: number
  /** 表格滚动配置 */
  scrollConfig: {
    x: number | string | undefined
    y: number | string | undefined
  }
}

/**
 * 使用响应式表格
 *
 * 提供响应式断点检测和表格布局适配功能，支持三端（移动端、平板端、桌面端）自适应
 *
 * @param options - 响应式表格配置选项
 * @returns 响应式表格状态和工具函数
 *
 * @example
 * ```ts
 * const {
 *   breakpoint,
 *   isMobile,
 *   isDesktop,
 *   useCardLayout,
 *   maxColumns,
 *   scrollConfig
 * } = useResponsiveTable({
 *   useCardOnMobile: true,
 *   mobileMaxColumns: 3
 * })
 * ```
 */
export function useResponsiveTable(options: ResponsiveTableOptions = {}) {
  const {
    breakpoints = {},
    useCardOnMobile = true,
    mobileMaxColumns = 3,
    tabletMaxColumns = 5,
  } = options

  const config: BreakpointConfig = {
    ...defaultBreakpoints,
    ...breakpoints,
  }

  const windowWidth = ref(window.innerWidth)

  /**
   * 当前断点
   */
  const breakpoint = computed<BreakpointType>(() => {
    const width = windowWidth.value
    if (width < config.mobile) return 'mobile'
    if (width < config.tablet) return 'tablet'
    return 'desktop'
  })

  /**
   * 是否为移动端
   */
  const isMobile = computed(() => breakpoint.value === 'mobile')

  /**
   * 是否为平板端
   */
  const isTablet = computed(() => breakpoint.value === 'tablet')

  /**
   * 是否为桌面端
   */
  const isDesktop = computed(() => breakpoint.value === 'desktop')

  /**
   * 是否使用卡片布局
   */
  const useCardLayout = computed(() => useCardOnMobile && isMobile.value)

  /**
   * 当前应显示的最大列数
   */
  const maxColumns = computed(() => {
    switch (breakpoint.value) {
      case 'mobile':
        return mobileMaxColumns
      case 'tablet':
        return tabletMaxColumns
      case 'desktop':
        return Infinity
      default:
        return Infinity
    }
  })

  /**
   * 表格滚动配置
   */
  const scrollConfig = computed(() => {
    switch (breakpoint.value) {
      case 'mobile':
        return {
          x: '100%',
          y: undefined,
        }
      case 'tablet':
        return {
          x: 800,
          y: undefined,
        }
      case 'desktop':
      default:
        return {
          x: undefined,
          y: undefined,
        }
    }
  })

  /**
   * 处理窗口大小变化
   */
  const handleResize = () => {
    windowWidth.value = window.innerWidth
  }

  onMounted(() => {
    window.addEventListener('resize', handleResize)
  })

  onUnmounted(() => {
    window.removeEventListener('resize', handleResize)
  })

  return {
    breakpoint,
    isMobile,
    isTablet,
    isDesktop,
    useCardLayout,
    maxColumns,
    scrollConfig,
  }
}

export default useResponsiveTable
