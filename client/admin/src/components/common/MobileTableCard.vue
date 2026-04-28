<script setup lang="ts">
import { h } from 'vue'
import { NCard, NText, NEmpty, NSpace, NButton, NIcon, NDropdown } from 'naive-ui'
import { MoreHorizontal } from 'lucide-vue-next'
import type { Component } from 'vue'
import type { DropdownOption } from 'naive-ui'

/**
 * 操作按钮配置
 */
export interface MobileAction<T = unknown> {
  /** 按钮文字 */
  label: string
  /** 图标组件 */
  icon?: Component
  /** 按钮类型 */
  type?: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'error'
  /** 是否显示 */
  show?: (row: T, index: number) => boolean
  /** 点击回调 */
  onClick: (row: T, index: number) => void
}

/**
 * 列配置
 */
export interface MobileColumn<T = unknown> {
  key: string
  title: string
  render?: (row: T, index: number) => unknown
}

/**
 * 组件属性
 */
interface Props {
  /** 表格数据 */
  data: unknown[]
  /** 列配置 */
  columns: MobileColumn[]
  /** 标题列（显示在卡片头部） */
  titleColumn?: string
  /** 空数据提示 */
  emptyText?: string
  /** 操作按钮配置 */
  actions?: MobileAction[]
  /** 是否显示操作按钮区域 */
  showActions?: boolean
  /** 是否使用下拉菜单模式（节省空间） */
  dropdownMode?: boolean
  /** 下拉菜单按钮文字 */
  dropdownLabel?: string
}

const props = withDefaults(defineProps<Props>(), {
  titleColumn: '',
  emptyText: '暂无数据',
  showActions: true,
  dropdownMode: true,
  dropdownLabel: '更多',
})

const emit = defineEmits<{
  (e: 'rowClick', row: unknown, index: number): void
}>()

/**
 * 获取单元格显示值
 */
const getCellValue = (row: Record<string, unknown>, key: string): string => {
  const value = row[key]
  if (value == null) return '-'
  return String(value)
}

/**
 * 处理卡片点击
 */
const handleCardClick = (row: unknown, index: number) => {
  emit('rowClick', row, index)
}

/**
 * 检查操作按钮是否显示
 */
const isActionVisible = (action: MobileAction, row: unknown, index: number): boolean => {
  if (action.show === undefined) return true
  return action.show(row, index)
}

/**
 * 获取可见的操作按钮列表
 */
const getVisibleActions = (row: unknown, index: number): MobileAction[] => {
  if (!props.actions) return []
  return props.actions.filter(action => isActionVisible(action, row, index))
}

/**
 * 生成下拉菜单选项
 */
const getDropdownOptions = (row: unknown, index: number): DropdownOption[] => {
  const visibleActions = getVisibleActions(row, index)
  return visibleActions.map((action, actionIndex) => ({
    key: String(actionIndex),
    label: action.label,
    icon: action.icon
      ? () => h(NIcon, { component: action.icon })
      : undefined,
  }))
}

/**
 * 处理下拉菜单选择
 */
const handleDropdownSelect = (key: string, row: unknown, index: number) => {
  const visibleActions = getVisibleActions(row, index)
  const actionIndex = parseInt(key, 10)
  const action = visibleActions[actionIndex]
  if (action) {
    action.onClick(row, index)
  }
}
</script>

<template>
  <div class="mobile-table-card">
    <!-- 数据卡片列表 -->
    <template v-if="data.length > 0">
      <NCard
        v-for="(row, index) in data"
        :key="index"
        size="small"
        class="data-card"
        hoverable
        @click="handleCardClick(row, index)"
      >
        <!-- 卡片头部：显示标题列 -->
        <template v-if="titleColumn" #header>
          <div class="card-header">
            <NText strong>
              {{ getCellValue(row as Record<string, unknown>, titleColumn) }}
            </NText>
          </div>
        </template>

        <!-- 卡片内容：显示所有字段 -->
        <div class="card-content">
          <div
            v-for="col in columns.filter(c => c.key !== titleColumn)"
            :key="col.key"
            class="card-row"
          >
            <span class="card-label">{{ col.title }}:</span>
            <span class="card-value">
              <template v-if="col.render">
                <component :is="() => col.render!(row, index)" />
              </template>
              <template v-else>
                {{ getCellValue(row as Record<string, unknown>, col.key) }}
              </template>
            </span>
          </div>
        </div>

        <!-- 操作按钮区域 -->
        <template v-if="showActions && actions && actions.length > 0" #footer>
          <!-- 下拉菜单模式（节省空间） -->
          <div v-if="dropdownMode" class="action-wrapper" @click.stop>
            <NDropdown
              :options="getDropdownOptions(row, index)"
              placement="bottom-end"
              trigger="click"
              @select="(key: string) => handleDropdownSelect(key, row, index)"
            >
              <NButton size="small" quaternary>
                <template #icon>
                  <NIcon :component="MoreHorizontal" />
                </template>
                {{ dropdownLabel }}
              </NButton>
            </NDropdown>
          </div>

          <!-- 平铺按钮模式（直观展示） -->
          <NSpace v-else justify="end" size="small" @click.stop>
            <NButton
              v-for="(action, actionIndex) in getVisibleActions(row, index)"
              :key="actionIndex"
              size="small"
              :type="action.type || 'default'"
              :text="!action.label"
              @click="action.onClick(row, index)"
            >
              <template v-if="action.icon" #icon>
                <NIcon :component="action.icon" />
              </template>
              {{ action.label }}
            </NButton>
          </NSpace>
        </template>
      </NCard>
    </template>

    <!-- 空状态 -->
    <NEmpty v-else :description="emptyText" />
  </div>
</template>

<style scoped>
.mobile-table-card {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.data-card {
  cursor: pointer;
  transition: transform 0.2s ease, box-shadow 0.2s ease;
}

.data-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.card-content {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.card-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 4px 0;
  border-bottom: 1px solid var(--border-color);
}

.card-row:last-child {
  border-bottom: none;
}

.card-label {
  color: var(--text-secondary);
  font-size: 14px;
}

.card-value {
  color: var(--text-primary);
  font-size: 14px;
  font-weight: 500;
}

.action-wrapper {
  display: flex;
  justify-content: flex-end;
}

:deep(.n-card__footer) {
  padding-top: 12px;
  border-top: 1px solid var(--border-color);
}
</style>
