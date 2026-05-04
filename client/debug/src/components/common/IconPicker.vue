<template>
  <div class="icon-picker">
    <n-input v-model:value="searchQuery" placeholder="搜索图标..." clearable class="search-input" size="small">
      <template #prefix>
        <Search :size="14" />
      </template>
    </n-input>

    <n-scrollbar class="icons-container" style="max-height: 196px">
      <div class="icons-grid">
        <n-button v-for="iconName in paginatedIcons" :key="iconName" class="icon-item"
          :type="selectedIcon === iconName ? 'primary' : 'default'" quaternary @click="selectIcon(iconName)">
          <template #icon>
            <component :is="getIconComponent(iconName)" :size="18" />
          </template>
        </n-button>
      </div>
    </n-scrollbar>

    <n-pagination v-model:page="currentPage" :page-size="pageSize" :item-count="filteredIcons.length" class="pagination"
      size="small" :page-slot="5" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { NInput, NButton, NScrollbar, NPagination } from 'naive-ui'
import { Search } from 'lucide-vue-next'
import * as LucideIcons from 'lucide-vue-next'
import type { FunctionalComponent } from 'vue'
import type { LucideProps } from 'lucide-vue-next'

/**
 * 组件属性定义
 */
interface Props {
  /** 选中的图标 */
  selectedIcon?: string
}

defineProps<Props>()

/**
 * 组件事件定义
 */
interface Emits {
  /** 选择图标 */
  (e: 'select', iconName: string): void
}

const emit = defineEmits<Emits>()

/** 搜索关键词 */
const searchQuery = ref('')

/** 当前页码 */
const currentPage = ref(1)

/** 每页显示数量（小尺寸弹窗适配：4x12网格） */
const pageSize = 48

/** 获取所有 Lucide 图标名称 */
const allIconNames = computed(() => {
  return Object.keys(LucideIcons).filter(name => name !== 'createLucideIcon')
})

/** 根据搜索过滤的图标 */
const filteredIcons = computed(() => {
  if (!searchQuery.value) return allIconNames.value
  const query = searchQuery.value.toLowerCase()
  return allIconNames.value.filter(name => name.toLowerCase().includes(query))
})

/** 分页后的图标 */
const paginatedIcons = computed(() => {
  const start = (currentPage.value - 1) * pageSize
  const end = start + pageSize
  return filteredIcons.value.slice(start, end)
})

/**
 * 获取图标组件
 * @param iconName 图标名称
 * @returns 图标组件
 */
function getIconComponent(iconName: string): FunctionalComponent<LucideProps> {
  return (LucideIcons as unknown as Record<string, FunctionalComponent<LucideProps>>)[iconName]
    || LucideIcons.Circle
}

/**
 * 选择图标
 * @param iconName 图标名称
 */
function selectIcon(iconName: string): void {
  emit('select', iconName)
}
</script>

<style scoped>
.icon-picker {
  display: flex;
  flex-direction: column;
  gap: 10px;
  width: 100%;
}

.search-input {
  width: 100%;
}

.icons-container {
  height: 196px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
}

.icons-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 4px;
  padding: 8px;
}

.icon-item {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 44px;
  width: 44px;
  padding: 0;
}

.pagination {
  justify-content: center;
  margin-top: 4px;
}
</style>
