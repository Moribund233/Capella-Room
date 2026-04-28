<script setup lang="ts">
/**
 * UserTableToolbar - 用户表格工具栏组件
 *
 * 提供新增用户、批量删除等操作按钮
 *
 */
import { NButton, NSpace, NPopconfirm } from 'naive-ui'
import { Plus, Trash2 } from 'lucide-vue-next'

/**
 * 组件属性定义
 */
interface Props {
  /** 选中的行数 */
  selectedCount?: number
  /** 总记录数 */
  total?: number
  /** 加载状态 */
  loading?: boolean
}

/**
 * 组件事件定义
 */
interface Emits {
  /** 新增用户事件 */
  (e: 'add'): void
  /** 批量删除事件 */
  (e: 'batch-delete'): void
}

const props = withDefaults(defineProps<Props>(), {
  selectedCount: 0,
  total: 0,
  loading: false,
})

// 使用 props 避免未使用警告
// eslint-disable-next-line @typescript-eslint/no-unused-vars
const _props = props

const emit = defineEmits<Emits>()

/**
 * 处理新增用户
 */
const handleAdd = () => {
  emit('add')
}

/**
 * 处理批量删除
 */
const handleBatchDelete = () => {
  emit('batch-delete')
}
</script>

<template>
  <NSpace justify="space-between" align="center" wrap>
    <NSpace>
      <NButton type="primary" :loading="loading" @click="handleAdd">
        <template #icon>
          <Plus :size="16" />
        </template>
        新增用户
      </NButton>

      <NPopconfirm
        v-if="selectedCount > 0"
        @positive-click="handleBatchDelete"
      >
        <template #trigger>
          <NButton type="error" ghost :loading="loading">
            <template #icon>
              <Trash2 :size="16" />
            </template>
            批量删除 ({{ selectedCount }})
          </NButton>
        </template>
        确定要删除选中的 {{ selectedCount }} 个用户吗？
      </NPopconfirm>
    </NSpace>

    <span class="total-text">共 {{ total }} 条记录</span>
  </NSpace>
</template>

<style scoped>
.total-text {
  font-size: 14px;
  color: var(--text-secondary);
}
</style>
