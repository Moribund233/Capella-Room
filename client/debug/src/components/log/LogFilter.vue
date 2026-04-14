<script setup lang="ts">
import { computed } from 'vue'
import { Filter, Search, Layers, AlertCircle } from 'lucide-vue-next'

/**
 * 日志过滤器组件
 * 提供日志级别、模块和搜索关键词过滤功能
 */

interface Props {
  /** 当前日志级别 */
  level: string
  /** 当前模块 */
  module: string
  /** 搜索关键词 */
  keyword: string
}

const props = defineProps<Props>()

const emit = defineEmits<{
  (e: 'update:level', value: string): void
  (e: 'update:module', value: string): void
  (e: 'update:keyword', value: string): void
  (e: 'filterChange'): void
}>()

/**
 * 日志级别选项
 */
const levelOptions = [
  { label: '全部级别', value: 'all' },
  { label: '错误 (Error)', value: 'error' },
  { label: '警告 (Warn)', value: 'warn' },
  { label: '信息 (Info)', value: 'info' },
  { label: '调试 (Debug)', value: 'debug' }
]

/**
 * 模块选项（根据后端常见的 target 值）
 */
const moduleOptions = [
  { label: '全部模块', value: 'all' },
  { label: 'WebSocket', value: 'websocket' },
  { label: '房间', value: 'room' },
  { label: '消息', value: 'message' },
  { label: '性能', value: 'performance' },
  { label: '认证', value: 'auth' },
  { label: '数据库', value: 'db' },
  { label: 'HTTP', value: 'http' }
]

/**
 * 当前级别
 */
const currentLevel = computed({
  get: () => props.level,
  set: (value) => {
    emit('update:level', value)
    emit('filterChange')
  }
})

/**
 * 当前模块
 */
const currentModule = computed({
  get: () => props.module,
  set: (value) => {
    emit('update:module', value)
    emit('filterChange')
  }
})

/**
 * 搜索关键词
 */
const searchKeyword = computed({
  get: () => props.keyword,
  set: (value) => {
    emit('update:keyword', value)
  }
})

/**
 * 重置过滤器
 */
function resetFilters() {
  emit('update:level', 'all')
  emit('update:module', 'all')
  emit('update:keyword', '')
  emit('filterChange')
}
</script>

<template>
  <div class="log-filter">
    <div class="filter-header">
      <n-icon size="18"><Filter /></n-icon>
      <span class="filter-title">日志过滤</span>
    </div>

    <n-divider style="margin: 12px 0" />

    <!-- 搜索框 -->
    <div class="filter-section">
      <n-input
        v-model:value="searchKeyword"
        placeholder="搜索日志内容..."
        clearable
        @keyup.enter="$emit('filterChange')"
      >
        <template #prefix>
          <n-icon><Search /></n-icon>
        </template>
      </n-input>
    </div>

    <!-- 级别过滤 -->
    <div class="filter-section">
      <div class="filter-label">
        <n-icon size="14" depth="3"><AlertCircle /></n-icon>
        <span>日志级别</span>
      </div>
      <n-select
        v-model:value="currentLevel"
        :options="levelOptions"
        size="small"
      />
    </div>

    <!-- 模块过滤 -->
    <div class="filter-section">
      <div class="filter-label">
        <n-icon size="14" depth="3"><Layers /></n-icon>
        <span>模块</span>
      </div>
      <n-select
        v-model:value="currentModule"
        :options="moduleOptions"
        size="small"
      />
    </div>

    <n-divider style="margin: 12px 0" />

    <!-- 重置按钮 -->
    <n-button
      size="small"
      block
      ghost
      @click="resetFilters"
    >
      重置过滤器
    </n-button>
  </div>
</template>

<style scoped>
.log-filter {
  padding: 16px;
  background: #fff;
  border-radius: 8px;
}

.filter-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 500;
  color: #262626;
}

.filter-title {
  font-size: 14px;
}

.filter-section {
  margin-bottom: 16px;
}

.filter-section:last-of-type {
  margin-bottom: 0;
}

.filter-label {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 8px;
  font-size: 12px;
  color: #595959;
}
</style>
