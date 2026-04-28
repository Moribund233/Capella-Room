import { ref, computed } from 'vue'

/**
 * 分页参数
 */
export interface PaginationParams {
  /** 当前页码 */
  page: number
  /** 每页条数 */
  pageSize: number
}

/**
 * 分页结果
 */
export interface PaginationResult<T> {
  /** 数据列表 */
  list: T[]
  /** 总条数 */
  total: number
  /** 当前页码 */
  page: number
  /** 每页条数 */
  pageSize: number
}

/**
 * 分页配置选项
 */
export interface TablePaginationOptions {
  /** 默认页码 */
  defaultPage?: number
  /** 默认每页条数 */
  defaultPageSize?: number
  /** 每页条数选项 */
  pageSizes?: number[]
  /** 总条数 */
  total?: number
  /** 是否显示快速跳转 */
  showQuickJumper?: boolean
  /** 是否显示每页条数选择器 */
  showSizePicker?: boolean
  /** 是否简洁模式 */
  simple?: boolean
}

/**
 * 分页状态
 */
export interface TablePaginationState {
  /** 当前页码 */
  page: number
  /** 每页条数 */
  pageSize: number
  /** 总条数 */
  total: number
  /** 总页数 */
  pageCount: number
  /** 当前页起始索引 */
  startIndex: number
  /** 当前页结束索引 */
  endIndex: number
  /** 是否有上一页 */
  hasPrevPage: boolean
  /** 是否有下一页 */
  hasNextPage: boolean
}

/**
 * 使用表格分页
 *
 * 提供完整的分页逻辑和状态管理，支持响应式分页配置
 *
 * @param options - 分页配置选项
 * @returns 分页状态和方法
 *
 * @example
 * ```ts
 * const {
 *   page,
 *   pageSize,
 *   total,
 *   pageCount,
 *   setPage,
 *   setPageSize,
 *   nextPage,
 *   prevPage,
 *   resetPagination
 * } = useTablePagination({
 *   defaultPage: 1,
 *   defaultPageSize: 10,
 *   pageSizes: [10, 20, 50, 100]
 * })
 * ```
 */
export function useTablePagination(options: TablePaginationOptions = {}) {
  const {
    defaultPage = 1,
    defaultPageSize = 10,
    pageSizes = [10, 20, 50, 100],
    total = 0,
    showQuickJumper = true,
    showSizePicker = true,
    simple = false,
  } = options

  const page = ref(defaultPage)
  const pageSize = ref(defaultPageSize)
  const totalRef = ref(total)

  /**
   * 总页数
   */
  const pageCount = computed(() => {
    return Math.ceil(totalRef.value / pageSize.value)
  })

  /**
   * 当前页起始索引
   */
  const startIndex = computed(() => {
    return (page.value - 1) * pageSize.value
  })

  /**
   * 当前页结束索引
   */
  const endIndex = computed(() => {
    return Math.min(startIndex.value + pageSize.value, totalRef.value)
  })

  /**
   * 是否有上一页
   */
  const hasPrevPage = computed(() => {
    return page.value > 1
  })

  /**
   * 是否有下一页
   */
  const hasNextPage = computed(() => {
    return page.value < pageCount.value
  })

  /**
   * 分页状态
   */
  const state = computed<TablePaginationState>(() => ({
    page: page.value,
    pageSize: pageSize.value,
    total: totalRef.value,
    pageCount: pageCount.value,
    startIndex: startIndex.value,
    endIndex: endIndex.value,
    hasPrevPage: hasPrevPage.value,
    hasNextPage: hasNextPage.value,
  }))

  /**
   * 设置页码
   * @param newPage - 新页码
   */
  const setPage = (newPage: number) => {
    const targetPage = Math.max(1, Math.min(newPage, pageCount.value || 1))
    page.value = targetPage
  }

  /**
   * 设置每页条数
   * @param newPageSize - 新的每页条数
   */
  const setPageSize = (newPageSize: number) => {
    pageSize.value = newPageSize
    // 重置到第一页，避免当前页超出范围
    page.value = 1
  }

  /**
   * 下一页
   */
  const nextPage = () => {
    if (hasNextPage.value) {
      page.value++
    }
  }

  /**
   * 上一页
   */
  const prevPage = () => {
    if (hasPrevPage.value) {
      page.value--
    }
  }

  /**
   * 设置总条数
   * @param newTotal - 新的总条数
   */
  const setTotal = (newTotal: number) => {
    totalRef.value = Math.max(0, newTotal)
  }

  /**
   * 重置分页
   */
  const resetPagination = () => {
    page.value = defaultPage
    pageSize.value = defaultPageSize
  }

  /**
   * 获取当前分页参数
   */
  const getPaginationParams = (): PaginationParams => ({
    page: page.value,
    pageSize: pageSize.value,
  })

  /**
   * 分页配置（用于 Naive UI n-pagination 组件）
   */
  const paginationProps = computed(() => ({
    page: page.value,
    pageSize: pageSize.value,
    pageSizes: showSizePicker ? pageSizes : undefined,
    pageCount: pageCount.value,
    showSizePicker,
    showQuickJumper,
    simple,
  }))

  return {
    // 状态
    page,
    pageSize,
    total: totalRef,
    pageCount,
    startIndex,
    endIndex,
    hasPrevPage,
    hasNextPage,
    state,
    // 方法
    setPage,
    setPageSize,
    nextPage,
    prevPage,
    setTotal,
    resetPagination,
    getPaginationParams,
    // Naive UI 配置
    paginationProps,
  }
}

export default useTablePagination
