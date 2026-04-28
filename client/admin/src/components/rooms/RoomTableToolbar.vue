<script setup lang="ts">
import { NButton, NSpace, NText } from 'naive-ui'
import { Plus, Trash2 } from 'lucide-vue-next'

/**
 * 组件属性
 */
interface Props {
  /** 选中数量 */
  selectedCount: number
  /** 总数 */
  total: number
  /** 加载状态 */
  loading?: boolean
}

withDefaults(defineProps<Props>(), {
  loading: false,
})

/**
 * 组件事件
 */
interface Emits {
  /** 新增 */
  (e: 'add'): void
  /** 批量删除 */
  (e: 'batchDelete'): void
}

const emit = defineEmits<Emits>()

/**
 * 处理新增
 */
const handleAdd = () => {
  emit('add')
}

/**
 * 处理批量删除
 */
const handleBatchDelete = () => {
  emit('batchDelete')
}
</script>

<template>
  <div class="room-table-toolbar">
    <div class="toolbar-left">
      <NSpace align="center">
        <NButton type="primary" :loading="loading" @click="handleAdd">
          <template #icon>
            <Plus :size="16" />
          </template>
          新增房间
        </NButton>
        <NButton
          v-if="selectedCount > 0"
          type="error"
          :loading="loading"
          @click="handleBatchDelete"
        >
          <template #icon>
            <Trash2 :size="16" />
          </template>
          批量删除 ({{ selectedCount }})
        </NButton>
      </NSpace>
    </div>
    <div class="toolbar-right">
      <NText type="secondary">共 {{ total }} 个房间</NText>
    </div>
  </div>
</template>

<style scoped>
.room-table-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 12px;
}

.toolbar-left,
.toolbar-right {
  display: flex;
  align-items: center;
}

@media screen and (max-width: 768px) {
  .room-table-toolbar {
    flex-direction: column;
    align-items: flex-start;
  }

  .toolbar-right {
    width: 100%;
    justify-content: flex-end;
  }
}
</style>
