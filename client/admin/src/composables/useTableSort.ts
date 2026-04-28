import { ref, computed } from 'vue'

/**
 * 排序顺序
 */
export type SortOrder = 'ascend' | 'descend' | false

/**
 * 排序状态
 */
export interface SortState {
  /** 排序列 key */
  columnKey: string | null
  /** 排序顺序 */
  order: SortOrder
}

/**
 * 排序配置选项
 */
export interface TableSortOptions {
  /** 是否启用排序 */
  enabled?: boolean
  /** 默认排序列 */
  defaultSortKey?: string
  /** 默认排序顺序 */
  defaultOrder?: SortOrder
  /** 是否支持多列排序 */
  multiple?: boolean
}

/**
 * 使用表格排序
 *
 * 提供表格列排序功能，支持单列和多列排序
 *
 * @param options - 排序配置选项
 * @returns 排序状态和方法
 *
 * @example
 * ```ts
 * const {
 *   sortState,
 *   sortKey,
 *   sortOrder,
 *   handleSortChange,
 *   setSort,
 *   clearSort
 * } = useTableSort({
 *   enabled: true,
 *   defaultSortKey: 'createdAt',
 *   defaultOrder: 'descend'
 * })
 * ```
 */
export function useTableSort(options: TableSortOptions = {}) {
  const {
    enabled = false,
    defaultSortKey = null,
    defaultOrder = false,
    multiple = false,
  } = options

  const sortState = ref<SortState>({
    columnKey: defaultSortKey,
    order: defaultOrder,
  })

  /**
   * 当前排序列 key
   */
  const sortKey = computed(() => sortState.value.columnKey)

  /**
   * 当前排序顺序
   */
  const sortOrder = computed(() => sortState.value.order)

  /**
   * 是否正在排序
   */
  const isSorted = computed(() => {
    return sortState.value.columnKey !== null && sortState.value.order !== false
  })

  /**
   * 处理排序变化
   * @param columnKey - 列 key
   * @param order - 排序顺序
   */
  const handleSortChange = (columnKey: string, order: SortOrder) => {
    if (!enabled) return

    if (multiple) {
      // 多列排序逻辑（暂不支持，需要更复杂的状态管理）
      sortState.value = { columnKey, order }
    } else {
      // 单列排序
      if (sortState.value.columnKey === columnKey && order === false) {
        // 点击同一列第三次，取消排序
        sortState.value = { columnKey: null, order: false }
      } else {
        sortState.value = { columnKey, order }
      }
    }
  }

  /**
   * 设置排序
   * @param columnKey - 列 key
   * @param order - 排序顺序
   */
  const setSort = (columnKey: string, order: SortOrder) => {
    if (!enabled) return

    sortState.value = { columnKey, order }
  }

  /**
   * 清空排序
   */
  const clearSort = () => {
    sortState.value = { columnKey: null, order: false }
  }

  /**
   * 获取排序参数（用于 API 请求）
   */
  const getSortParams = () => {
    if (!isSorted.value) return null

    return {
      sortBy: sortState.value.columnKey,
      sortOrder: sortState.value.order === 'ascend' ? 'asc' : 'desc',
    }
  }

  /**
   * 对数据进行本地排序
   * @param data - 原始数据
   * @param customCompare - 自定义比较函数
   */
  const sortData = <T extends object>(
    data: T[],
    customCompare?: (a: T, b: T, key: string, order: SortOrder) => number
  ): T[] => {
    if (!isSorted.value) return data

    const { columnKey, order } = sortState.value
    if (!columnKey || !order) return data

    return [...data].sort((a, b) => {
      if (customCompare) {
        return customCompare(a, b, columnKey, order)
      }

      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const aValue = (a as any)[columnKey]
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const bValue = (b as any)[columnKey]

      // 处理 null/undefined
      if (aValue == null && bValue == null) return 0
      if (aValue == null) return order === 'ascend' ? -1 : 1
      if (bValue == null) return order === 'ascend' ? 1 : -1

      // 字符串比较
      if (typeof aValue === 'string' && typeof bValue === 'string') {
        const comparison = aValue.localeCompare(bValue)
        return order === 'ascend' ? comparison : -comparison
      }

      // 数字比较
      if (typeof aValue === 'number' && typeof bValue === 'number') {
        return order === 'ascend' ? aValue - bValue : bValue - aValue
      }

      // 日期比较
      if (aValue instanceof Date && bValue instanceof Date) {
        return order === 'ascend'
          ? aValue.getTime() - bValue.getTime()
          : bValue.getTime() - aValue.getTime()
      }

      // 默认转换为字符串比较
      const aStr = String(aValue)
      const bStr = String(bValue)
      const comparison = aStr.localeCompare(bStr)
      return order === 'ascend' ? comparison : -comparison
    })
  }

  return {
    // 状态
    sortState,
    sortKey,
    sortOrder,
    isSorted,
    // 方法
    handleSortChange,
    setSort,
    clearSort,
    getSortParams,
    sortData,
  }
}

export default useTableSort
