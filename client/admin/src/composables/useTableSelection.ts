import { ref, computed } from 'vue'

/**
 * 选择配置选项
 */
export interface TableSelectionOptions<T = unknown> {
  /** 是否启用选择 */
  enabled?: boolean
  /** 行唯一标识字段 */
  rowKey?: keyof T | ((row: T) => string | number)
  /** 是否多选 */
  multiple?: boolean
  /** 默认选中的行 */
  defaultSelected?: (string | number)[]
}

/**
 * 选择状态
 */
export interface TableSelectionState<T = unknown> {
  /** 选中的行 keys */
  selectedKeys: (string | number)[]
  /** 选中的行数据 */
  selectedRows: T[]
  /** 是否全选 */
  isAllSelected: boolean
  /** 选中数量 */
  selectedCount: number
}

/**
 * 使用表格行选择
 *
 * 提供表格行选择功能，支持单选和多选模式
 *
 * @param options - 选择配置选项
 * @returns 选择状态和方法
 *
 * @example
 * ```ts
 * const {
 *   selectedKeys,
 *   selectedRows,
 *   isAllSelected,
 *   handleSelect,
 *   handleSelectAll,
 *   clearSelection
 * } = useTableSelection<User>({
 *   enabled: true,
 *   rowKey: 'id',
 *   multiple: true
 * })
 * ```
 */
export function useTableSelection<T extends object>(
  options: TableSelectionOptions<T> = {}
) {
  const {
    enabled = false,
    rowKey = 'id',
    multiple = true,
    defaultSelected = [],
  } = options

  const selectedKeys = ref<(string | number)[]>([...defaultSelected])

  /**
   * 获取行的唯一标识
   * @param row - 行数据
   */
  const getRowKey = (row: T): string | number => {
    if (typeof rowKey === 'function') {
      return rowKey(row)
    }
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return (row as any)[rowKey] as string | number
  }

  /**
   * 选中的行数据
   */
  const selectedRows = computed(() => {
    return [] as T[] // 实际使用时需要在组件中根据 selectedKeys 计算
  })

  /**
   * 选中数量
   */
  const selectedCount = computed(() => selectedKeys.value.length)

  /**
   * 是否全选
   */
  const isAllSelected = computed(() => {
    return false // 实际使用时需要在组件中根据数据计算
  })

  /**
   * 选择状态
   */
  const state = computed<TableSelectionState<T>>(() => ({
    selectedKeys: selectedKeys.value,
    selectedRows: selectedRows.value,
    isAllSelected: isAllSelected.value,
    selectedCount: selectedCount.value,
  }))

  /**
   * 处理行选择
   * @param keys - 选中的 keys
   */
  const handleSelect = (keys: (string | number)[]) => {
    if (!enabled) return

    if (multiple) {
      selectedKeys.value = keys
    } else {
      // 单选模式只保留最后一个选中的
      selectedKeys.value = keys.slice(-1)
    }
  }

  /**
   * 处理全选
   * @param allKeys - 所有行的 keys
   * @param selected - 是否选中
   */
  const handleSelectAll = (allKeys: (string | number)[], selected: boolean) => {
    if (!enabled) return

    if (selected) {
      selectedKeys.value = [...allKeys]
    } else {
      selectedKeys.value = []
    }
  }

  /**
   * 选中指定行
   * @param row - 行数据
   */
  const selectRow = (row: T) => {
    if (!enabled) return

    const key = getRowKey(row)
    if (multiple) {
      if (!selectedKeys.value.includes(key)) {
        selectedKeys.value.push(key)
      }
    } else {
      selectedKeys.value = [key]
    }
  }

  /**
   * 取消选中指定行
   * @param row - 行数据
   */
  const unselectRow = (row: T) => {
    if (!enabled) return

    const key = getRowKey(row)
    const index = selectedKeys.value.indexOf(key)
    if (index > -1) {
      selectedKeys.value.splice(index, 1)
    }
  }

  /**
   * 切换行选中状态
   * @param row - 行数据
   */
  const toggleRow = (row: T) => {
    if (!enabled) return

    const key = getRowKey(row)
    if (selectedKeys.value.includes(key)) {
      unselectRow(row)
    } else {
      selectRow(row)
    }
  }

  /**
   * 清空选择
   */
  const clearSelection = () => {
    selectedKeys.value = []
  }

  /**
   * 设置选中的 keys
   * @param keys - 要设置的 keys
   */
  const setSelectedKeys = (keys: (string | number)[]) => {
    selectedKeys.value = [...keys]
  }

  /**
   * 检查行是否被选中
   * @param row - 行数据
   */
  const isSelected = (row: T): boolean => {
    return selectedKeys.value.includes(getRowKey(row))
  }

  /**
   * 选择配置（用于 Naive UI n-data-table 组件）
   */
  const selectionProps = computed(() => {
    if (!enabled) return null

    return {
      type: multiple ? ('checkbox' as const) : ('radio' as const),
      options: [],
    }
  })

  return {
    // 状态
    selectedKeys,
    selectedRows,
    selectedCount,
    isAllSelected,
    state,
    // 方法
    handleSelect,
    handleSelectAll,
    selectRow,
    unselectRow,
    toggleRow,
    clearSelection,
    setSelectedKeys,
    isSelected,
    getRowKey,
    // Naive UI 配置
    selectionProps,
  }
}

export default useTableSelection
