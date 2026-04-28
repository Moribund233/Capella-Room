import { computed, ref, watch } from 'vue'
import {
  useResponsiveTable,
  type ResponsiveTableOptions,
  type BreakpointType,
} from './useResponsiveTable'
import {
  useTablePagination,
  type TablePaginationOptions,
  type PaginationParams,
} from './useTablePagination'
import {
  useTableSelection,
  type TableSelectionOptions,
} from './useTableSelection'
import {
  useTableSort,
  type TableSortOptions,
  type SortOrder,
} from './useTableSort'

export type { BreakpointType, PaginationParams, SortOrder }

/**
 * 表格列配置
 */
export interface TableColumn<T = unknown> {
  /** 列唯一标识 */
  key: string
  /** 列标题 */
  title: string
  /** 列宽 */
  width?: number | string
  /** 最小列宽 */
  minWidth?: number | string
  /** 是否固定列 */
  fixed?: 'left' | 'right'
  /** 是否可排序 */
  sortable?: boolean
  /** 对齐方式 */
  align?: 'left' | 'center' | 'right'
  /** 是否显示该列 */
  visible?: boolean | ((breakpoint: BreakpointType) => boolean)
  /** 列渲染函数 */
  render?: (row: T, index: number) => unknown
  /** 自定义单元格类名 */
  className?: string
}

/**
 * 数据表格配置选项
 */
export interface DataTableOptions<T extends Record<string, unknown>> {
  /** 响应式配置 */
  responsive?: ResponsiveTableOptions
  /** 分页配置 */
  pagination?: TablePaginationOptions | boolean
  /** 选择配置 */
  selection?: TableSelectionOptions<T> | boolean
  /** 排序配置 */
  sort?: TableSortOptions | boolean
  /** 行唯一标识字段 */
  rowKey?: keyof T | ((row: T) => string | number)
  /** 是否加载中 */
  loading?: boolean
  /** 是否显示边框 */
  bordered?: boolean
  /** 是否显示条纹 */
  striped?: boolean
  /** 表格大小 */
  size?: 'small' | 'medium' | 'large'
}

/**
 * 使用数据表格
 *
 * 组合响应式、分页、选择、排序功能的统一入口
 *
 * @param data - 表格数据
 * @param options - 表格配置选项
 * @returns 表格状态和操作方法
 *
 * @example
 * ```ts
 * const table = useDataTable(userList, {
 *   responsive: { useCardOnMobile: true },
 *   pagination: { defaultPageSize: 10 },
 *   selection: { enabled: true, multiple: true },
 *   sort: { enabled: true }
 * })
 * ```
 */
export function useDataTable<T extends Record<string, unknown>>(
  data: T[],
  options: DataTableOptions<T> = {}
) {
  const {
    responsive = {},
    pagination: paginationOption = true,
    selection: selectionOption = false,
    sort: sortOption = false,
    rowKey = 'id',
    loading = false,
    bordered = false,
    striped = true,
    size = 'medium',
  } = options

  // 响应式
  const responsiveState = useResponsiveTable(responsive)

  // 分页配置
  const paginationConfig = computed(() => {
    if (paginationOption === false) return false
    if (paginationOption === true) return {}
    return paginationOption
  })

  // 分页
  const paginationState = useTablePagination(
    paginationConfig.value || undefined
  )

  // 选择配置
  const selectionConfig = computed(() => {
    if (selectionOption === false) return { enabled: false }
    if (selectionOption === true) return { enabled: true, rowKey }
    return { ...selectionOption, rowKey: selectionOption.rowKey || rowKey }
  })

  // 选择
  const selectionState = useTableSelection<T>(selectionConfig.value)

  // 排序配置
  const sortConfig = computed(() => {
    if (sortOption === false) return { enabled: false }
    if (sortOption === true) return { enabled: true }
    return sortOption
  })

  // 排序
  const sortState = useTableSort(sortConfig.value)

  // 加载状态
  const isLoading = ref(loading)

  // 设置加载状态
  const setLoading = (value: boolean) => {
    isLoading.value = value
  }

  // 过滤后的数据（排序后）
  const sortedData = computed(() => {
    return sortState.sortData(data)
  })

  // 分页后的数据
  const displayData = computed(() => {
    if (paginationOption === false) {
      return sortedData.value
    }

    const start = paginationState.startIndex.value
    const end = paginationState.endIndex.value
    return sortedData.value.slice(start, end)
  })

  // 更新总条数
  watch(
    () => data.length,
    (newTotal) => {
      if (paginationOption !== false) {
        paginationState.setTotal(newTotal)
      }
    },
    { immediate: true }
  )

  // 处理表格变化（分页、排序、筛选）
  const handleChange = (params: {
    page?: number
    pageSize?: number
    sorter?: { columnKey: string; order: SortOrder }
  }) => {
    if (params.page !== undefined) {
      paginationState.setPage(params.page)
    }
    if (params.pageSize !== undefined) {
      paginationState.setPageSize(params.pageSize)
    }
    if (params.sorter) {
      sortState.handleSortChange(params.sorter.columnKey, params.sorter.order)
    }
  }

  // 刷新表格数据
  const refresh = () => {
    // 重置到第一页
    paginationState.setPage(1)
    // 触发刷新事件，由外部处理
  }

  // 重置所有状态
  const reset = () => {
    paginationState.resetPagination()
    selectionState.clearSelection()
    sortState.clearSort()
  }

  // 获取表格查询参数（用于 API 请求）
  const getQueryParams = () => {
    const params: Record<string, unknown> = {}

    if (paginationOption !== false) {
      Object.assign(params, paginationState.getPaginationParams())
    }

    const sortParams = sortState.getSortParams()
    if (sortParams) {
      Object.assign(params, sortParams)
    }

    return params
  }

  // 根据断点过滤列
  const filterColumnsByBreakpoint = (columns: TableColumn<T>[]) => {
    return columns.filter((col) => {
      if (col.visible === undefined) return true
      if (typeof col.visible === 'boolean') return col.visible
      return col.visible(responsiveState.breakpoint.value)
    })
  }

  return {
    // 响应式状态
    ...responsiveState,

    // 分页状态
    ...paginationState,

    // 选择状态
    ...selectionState,

    // 排序状态
    ...sortState,

    // 数据
    data,
    sortedData,
    displayData,

    // 加载状态
    isLoading,
    setLoading,

    // 配置
    bordered,
    striped,
    size,

    // 方法
    handleChange,
    refresh,
    reset,
    getQueryParams,
    filterColumnsByBreakpoint,
  }
}

export default useDataTable
