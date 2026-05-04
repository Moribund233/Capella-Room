<template>
  <div class="api-endpoint-select">
    <div class="search-box">
      <n-input v-model:value="searchQuery" placeholder="搜索 API 端点..." clearable>
        <template #prefix>
          <Search class="search-icon" :size="16" />
        </template>
      </n-input>
    </div>

    <div class="endpoint-list">
      <div v-for="category in filteredCategories" :key="category" class="category-group">
        <div class="category-header" @click="toggleCategory(category)">
          <ChevronRight v-if="collapsedCategories.has(category)" :size="16" />
          <ChevronDown v-else :size="16" />
          <span class="category-name">{{ category }}</span>
          <n-tag size="small" :type="getCategoryType(category)">
            {{ groupedEndpoints[category]?.length || 0 }}
          </n-tag>
        </div>

        <div v-show="!collapsedCategories.has(category)" class="category-items">
          <div v-for="endpoint in groupedEndpoints[category]" :key="endpoint.key"
            class="endpoint-item"
            :class="{ active: selectedEndpoint?.key === endpoint.key }"
            @click="selectEndpoint(endpoint)">
            <div class="endpoint-method" :class="`method-${endpoint.method.toLowerCase()}`">
              {{ endpoint.method }}
            </div>
            <div class="endpoint-info">
              <div class="endpoint-label">{{ endpoint.label }}</div>
              <div class="endpoint-path">{{ endpoint.path }}</div>
            </div>
            <Lock v-if="endpoint.requiresAuth" class="auth-icon" :size="14" />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { NInput, NTag } from 'naive-ui'
import { Search, ChevronRight, ChevronDown, Lock } from 'lucide-vue-next'
import type { ApiEndpoint } from '@/composables/test'

/**
 * API 端点选择器组件
 *
 * 按分类展示所有可用的 API 端点，支持搜索和折叠
 */

interface Props {
  /** 所有端点 */
  endpoints: ApiEndpoint[]
  /** 按分类分组的端点 */
  groupedEndpoints: Record<string, ApiEndpoint[]>
  /** 当前选中的端点 */
  selectedEndpoint: ApiEndpoint | null
}

const props = defineProps<Props>()

const emit = defineEmits<{
  /** 选择端点 */
  (e: 'select', endpoint: ApiEndpoint): void
}>()

/** 搜索关键词 */
const searchQuery = ref('')

/** 折叠的分类集合 */
const collapsedCategories = ref<Set<string>>(new Set())

/** 过滤后的端点 */
const filteredEndpoints = computed(() => {
  if (!searchQuery.value.trim()) {
    return props.endpoints
  }
  const query = searchQuery.value.toLowerCase()
  return props.endpoints.filter(endpoint =>
    endpoint.label.toLowerCase().includes(query) ||
    endpoint.path.toLowerCase().includes(query) ||
    endpoint.method.toLowerCase().includes(query)
  )
})

/** 过滤后的分类 */
const filteredCategories = computed(() => {
  const categories = new Set(filteredEndpoints.value.map(e => e.category))
  return Array.from(categories)
})

type TagType = 'default' | 'success' | 'info' | 'warning' | 'error' | 'primary'

/** 获取分类标签类型 */
function getCategoryType(category: string): TagType {
  const typeMap: Record<string, TagType> = {
    '认证': 'success',
    '用户': 'info',
    '房间': 'warning',
    '消息': 'error'
  }
  return typeMap[category] || 'default'
}

/** 切换分类折叠状态 */
function toggleCategory(category: string): void {
  const newSet = new Set(collapsedCategories.value)
  if (newSet.has(category)) {
    newSet.delete(category)
  } else {
    newSet.add(category)
  }
  collapsedCategories.value = newSet
}

/** 选择端点 */
function selectEndpoint(endpoint: ApiEndpoint): void {
  emit('select', endpoint)
}
</script>

<style scoped>
.api-endpoint-select {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-container);
  border-radius: var(--radius-lg);
  overflow: hidden;
}

.search-box {
  padding: 12px;
  border-bottom: 1px solid var(--border-color);
}

.search-icon {
  color: var(--text-tertiary);
}

.endpoint-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.category-group {
  margin-bottom: 8px;
}

.category-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  cursor: pointer;
  border-radius: var(--radius-md);
  transition: background-color 0.2s;
  user-select: none;
}

.category-header:hover {
  background: var(--bg-hover);
}

.category-name {
  flex: 1;
  font-weight: 500;
  color: var(--text-primary);
}

.category-items {
  padding-left: 8px;
}

.endpoint-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all 0.2s;
  margin-bottom: 4px;
}

.endpoint-item:hover {
  background: var(--bg-hover);
}

.endpoint-item.active {
  background: var(--primary-color-light);
  border-left: 3px solid var(--primary-color);
}

.endpoint-method {
  padding: 2px 6px;
  border-radius: var(--radius-sm);
  font-size: 11px;
  font-weight: 600;
  min-width: 48px;
  text-align: center;
  text-transform: uppercase;
}

.method-get {
  background: #10b981;
  color: white;
}

.method-post {
  background: #3b82f6;
  color: white;
}

.method-put {
  background: #f59e0b;
  color: white;
}

.method-delete {
  background: #ef4444;
  color: white;
}

.method-patch {
  background: #8b5cf6;
  color: white;
}

.endpoint-info {
  flex: 1;
  min-width: 0;
}

.endpoint-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.endpoint-path {
  font-size: 11px;
  color: var(--text-tertiary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  font-family: monospace;
}

.auth-icon {
  color: var(--warning-color);
}

/* 滚动条样式 */
.endpoint-list::-webkit-scrollbar {
  width: 6px;
}

.endpoint-list::-webkit-scrollbar-track {
  background: transparent;
}

.endpoint-list::-webkit-scrollbar-thumb {
  background: var(--border-color);
  border-radius: 3px;
}

.endpoint-list::-webkit-scrollbar-thumb:hover {
  background: var(--text-tertiary);
}

/* 移动端适配 */
@media (max-width: 768px) {
  .endpoint-item {
    padding: 8px;
  }

  .endpoint-method {
    min-width: 40px;
    font-size: 10px;
  }

  .endpoint-label {
    font-size: 12px;
  }

  .endpoint-path {
    font-size: 10px;
  }
}
</style>
