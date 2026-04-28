<script setup lang="ts" generic="T extends object = Record<string, unknown>">
/**
 * DataTable - 通用响应式数据表格组件
 *
 * 基于 Naive UI 的 n-data-table 封装，提供响应式布局、分页、选择、排序等功能
 *
 * @author Your Name
 * @version 1.0.0
 */
import { computed, watch } from 'vue'
import {
  NDataTable,
  NPagination,
  NCard,
  NSpace,
  NText,
  NEmpty,
  NSkeleton,
} from 'naive-ui'
import type { DataTableColumns, DataTableCreateSummary } from 'naive-ui'
import type { VNodeChild } from 'vue'
import {
  useResponsiveTable,
  useTablePagination,
  useTableSelection,
  useTableSort,
  type TableColumn,
} from '@/composables'

/**
 * 表格行数据类型
 */
export type TableRow = Record<string, unknown>

/**
 * 组件属性定义
 */
interface Props {
  /** 表格数据 */
  data: T[]
  /** 列配置 */
  columns: TableColumn<T>[]
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
  /** 是否启用分页 */
  pagination?: boolean
  /** 分页配置 */
  paginationOptions?: {
    defaultPage?: number
    defaultPageSize?: number
    pageSizes?: number[]
    showQuickJumper?: boolean
    showSizePicker?: boolean
    simple?: boolean
  }
  /** 是否启用选择 */
  selectable?: boolean
  /** 选择配置 */
  selectionOptions?: {
    multiple?: boolean
    defaultSelected?: (string | number)[]
  }
  /** 是否启用排序 */
  sortable?: boolean
  /** 排序配置 */
  sortOptions?: {
    defaultSortKey?: string
    defaultOrder?: 'ascend' | 'descend' | false
  }
  /** 响应式配置 */
  responsive?: {
    useCardOnMobile?: boolean
    mobileMaxColumns?: number
    tabletMaxColumns?: number
  }
  /** 空数据提示 */
  emptyText?: string
  /** 是否显示统计摘要 */
  showSummary?: boolean
  /** 自定义统计摘要 */
  summary?: DataTableCreateSummary<T>
  /** 表格最大高度 */
  maxHeight?: number | string
  /** 表格高度 */
  height?: number | string
  /** 是否启用虚拟滚动 */
  virtualScroll?: boolean
  /** 是否远程分页（服务端分页） */
  remote?: boolean
  /** 总条数（远程分页时使用） */
  total?: number
}

/**
 * 组件事件定义
 */
interface Emits {
  /** 选择变化事件 */
  (e: 'selectionChange', keys: (string | number)[], rows: T[]): void
  /** 分页变化事件 */
  (e: 'pageChange', page: number, pageSize: number): void
  /** 排序变化事件 */
  (e: 'sortChange', sortKey: string | null, order: 'ascend' | 'descend' | false): void
  /** 行点击事件 */
  (e: 'rowClick', row: T, index: number): void
  /** 行双击事件 */
  (e: 'rowDblclick', row: T, index: number): void
  /** 刷新事件 */
  (e: 'refresh'): void
}

const props = withDefaults(defineProps<Props>(), {
  rowKey: 'id' as keyof T,
  loading: false,
  bordered: false,
  striped: true,
  size: 'medium',
  pagination: true,
  selectable: false,
  sortable: false,
  emptyText: '暂无数据',
  showSummary: false,
  virtualScroll: false,
  remote: false,
  total: 0,
})

const emit = defineEmits<Emits>()

// ==================== 响应式处理 ====================
const responsiveState = useResponsiveTable({
  useCardOnMobile: props.responsive?.useCardOnMobile ?? true,
  mobileMaxColumns: props.responsive?.mobileMaxColumns ?? 3,
  tabletMaxColumns: props.responsive?.tabletMaxColumns ?? 5,
})

// ==================== 分页处理 ====================
const paginationState = useTablePagination(
  props.pagination
    ? {
        defaultPage: props.paginationOptions?.defaultPage ?? 1,
        defaultPageSize: props.paginationOptions?.defaultPageSize ?? 10,
        pageSizes: props.paginationOptions?.pageSizes ?? [10, 20, 50, 100],
        showQuickJumper: props.paginationOptions?.showQuickJumper ?? true,
        showSizePicker: props.paginationOptions?.showSizePicker ?? true,
        simple: props.paginationOptions?.simple ?? false,
      }
    : undefined
)

// 监听数据变化更新总条数
watch(
  () => (props.remote ? props.total : props.data.length),
  (newTotal) => {
    if (props.pagination) {
      paginationState.setTotal(newTotal)
    }
  },
  { immediate: true }
)

// ==================== 选择处理 ====================
const selectionState = useTableSelection<T>({
  enabled: props.selectable,
  rowKey: props.rowKey,
  multiple: props.selectionOptions?.multiple ?? true,
  defaultSelected: props.selectionOptions?.defaultSelected,
})

// 监听选择变化并触发事件
watch(
  () => selectionState.selectedKeys.value,
  (newKeys) => {
    const selectedRows = props.data.filter((row) => {
      const key = getRowKey(row)
      return newKeys.includes(key)
    })
    emit('selectionChange', newKeys, selectedRows)
  }
)

// ==================== 排序处理 ====================
const sortState = useTableSort({
  enabled: props.sortable,
  defaultSortKey: props.sortOptions?.defaultSortKey,
  defaultOrder: props.sortOptions?.defaultOrder,
})

// ==================== 数据处理 ====================
const sortedData = computed(() => {
  return sortState.sortData(props.data)
})

const displayData = computed(() => {
  if (!props.pagination) {
    return sortedData.value
  }
  // 远程分页模式下，直接返回所有数据（服务端已经分页）
  if (props.remote) {
    return sortedData.value
  }
  // 本地分页模式下，对数据进行切片
  const start = paginationState.startIndex.value
  const end = paginationState.endIndex.value
  return sortedData.value.slice(start, end)
})

// ==================== 列处理 ====================
const getRowKey = (row: T): string | number => {
  const key = props.rowKey
  if (typeof key === 'function') {
    return key(row)
  }
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  return (row as any)[key] as string | number
}

const isColumnVisible = (column: TableColumn<T>): boolean => {
  if (column.visible === undefined) return true
  if (typeof column.visible === 'boolean') return column.visible
  return column.visible(responsiveState.breakpoint.value)
}

const filteredColumns = computed(() => {
  return props.columns.filter(isColumnVisible)
})

// 生成 Naive UI 列配置
const naiveColumns = computed<DataTableColumns<T>>(() => {
  const cols = filteredColumns.value.map((col) => ({
    key: col.key,
    title: col.title,
    width: col.width,
    minWidth: col.minWidth,
    fixed: col.fixed,
    align: col.align,
    className: col.className,
    sorter: props.sortable && col.sortable ? true : undefined,
    render: col.render
      ? (row: T, rowIndex: number): VNodeChild => col.render!(row, rowIndex) as VNodeChild
      : undefined,
  }))

  return cols as DataTableColumns<T>
})

// ==================== 事件处理 ====================
const handlePageChange = (page: number) => {
  paginationState.setPage(page)
  emit('pageChange', page, paginationState.pageSize.value)
}

const handlePageSizeChange = (pageSize: number) => {
  paginationState.setPageSize(pageSize)
  emit('pageChange', paginationState.page.value, pageSize)
}

const handleSorterChange = (sorter: {
  columnKey: string
  order: 'ascend' | 'descend' | false
}) => {
  sortState.handleSortChange(sorter.columnKey, sorter.order)
  emit('sortChange', sortState.sortKey.value, sortState.sortOrder.value)
}

const handleCheck = (keys: (string | number)[]) => {
  selectionState.handleSelect(keys)
}

const handleRowClick = (row: T, index: number) => {
  emit('rowClick', row, index)
}

// ==================== 移动端卡片视图 ====================
const mobileCardData = computed(() => {
  if (!responsiveState.useCardLayout.value) return []
  return displayData.value.map((row, index) => ({
    key: getRowKey(row),
    data: row,
    index,
  }))
})

// ==================== 暴露方法 ====================
defineExpose({
  /** 刷新表格 */
  refresh: () => emit('refresh'),
  /** 清空选择 */
  clearSelection: selectionState.clearSelection,
  /** 获取选中项 */
  getSelectedKeys: () => selectionState.selectedKeys.value,
  /** 设置选中项 */
  setSelectedKeys: selectionState.setSelectedKeys,
  /** 重置分页 */
  resetPagination: paginationState.resetPagination,
  /** 获取分页参数 */
  getPaginationParams: paginationState.getPaginationParams,
  /** 清空排序 */
  clearSort: sortState.clearSort,
})
</script>

<template>
  <div class="data-table-wrapper">
    <!-- 桌面端/平板端表格 -->
    <template v-if="!responsiveState.useCardLayout.value">
      <NDataTable
        :columns="naiveColumns"
        :data="displayData"
        :row-key="(row: T) => getRowKey(row)"
        :loading="loading"
        :bordered="bordered"
        :striped="striped"
        :size="size"
        :remote="false"
        :scroll-x="responsiveState.scrollConfig.value.x"
        :scroll-y="maxHeight"
        :height="height"
        :virtual-scroll="virtualScroll"
        :checked-row-keys="selectable ? selectionState.selectedKeys.value : undefined"
        :row-props="(row: T, index: number) => ({ onClick: () => handleRowClick(row, index) })"
        @update:checked-row-keys="handleCheck"
        @update:sorter="handleSorterChange"
      >
        <template #empty>
          <NEmpty :description="emptyText" />
        </template>
      </NDataTable>
    </template>

    <!-- 移动端卡片视图 -->
    <template v-else>
      <div class="mobile-card-list">
        <NCard
          v-for="item in mobileCardData"
          :key="item.key"
          class="mobile-card"
          size="small"
          :hoverable="true"
          @click="handleRowClick(item.data as T, item.index)"
        >
          <template #header>
            <div class="mobile-card-header">
              <!-- eslint-disable-next-line @typescript-eslint/no-explicit-any -->
              <NText strong>{{ filteredColumns[0] ? (item.data as any)[filteredColumns[0].key] : '' }}</NText>
            </div>
          </template>
          <div class="mobile-card-content">
            <div
              v-for="col in filteredColumns.slice(1)"
              :key="col.key"
              class="mobile-card-item"
            >
              <span class="mobile-card-label">{{ col.title }}:</span>
              <span class="mobile-card-value">
                <template v-if="col.render">
                  <component :is="() => col.render!(item.data as T, item.index)" />
                </template>
                <template v-else>
                  <!-- eslint-disable-next-line @typescript-eslint/no-explicit-any -->
                  {{ (item.data as any)[col.key] }}
                </template>
              </span>
            </div>
          </div>
        </NCard>

        <NEmpty v-if="mobileCardData.length === 0" :description="emptyText" />
      </div>
    </template>

    <!-- 加载骨架屏 -->
    <div v-if="loading && displayData.length === 0" class="skeleton-wrapper">
      <NSkeleton v-for="i in 5" :key="i" text style="width: 100%; margin-bottom: 12px" />
    </div>

    <!-- 分页器 -->
    <div v-if="pagination && !responsiveState.useCardLayout.value" class="pagination-wrapper">
      <NPagination
        v-bind="paginationState.paginationProps.value"
        @update:page="handlePageChange"
        @update:page-size="handlePageSizeChange"
      />
    </div>

    <!-- 移动端分页 -->
    <div v-if="pagination && responsiveState.useCardLayout.value" class="mobile-pagination">
      <NSpace justify="center" align="center">
        <NText depth="3">
          第 {{ paginationState.page.value }} / {{ paginationState.pageCount.value }} 页
        </NText>
      </NSpace>
      <NSpace justify="center" style="margin-top: 8px">
        <NPagination
          v-model:page="paginationState.page.value"
          :page-count="paginationState.pageCount.value"
          :simple="true"
          @update:page="handlePageChange"
        />
      </NSpace>
    </div>
  </div>
</template>

<style scoped>
.data-table-wrapper {
  width: 100%;
}

.pagination-wrapper {
  margin-top: 16px;
  display: flex;
  justify-content: flex-end;
}

.mobile-card-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.mobile-card {
  cursor: pointer;
  transition: transform 0.2s;
}

.mobile-card:hover {
  transform: translateY(-2px);
}

.mobile-card-header {
  font-size: 16px;
}

.mobile-card-content {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.mobile-card-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 4px 0;
  border-bottom: 1px solid var(--n-border-color);
}

.mobile-card-item:last-child {
  border-bottom: none;
}

.mobile-card-label {
  color: var(--n-text-color-3);
  font-size: 14px;
}

.mobile-card-value {
  color: var(--n-text-color);
  font-size: 14px;
  text-align: right;
}

.mobile-pagination {
  margin-top: 16px;
  padding: 12px;
  background: var(--n-card-color);
  border-radius: 8px;
}

.skeleton-wrapper {
  padding: 16px;
}
</style>
