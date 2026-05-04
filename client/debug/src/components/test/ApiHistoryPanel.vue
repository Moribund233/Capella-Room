<template>
  <div class="api-history-panel">
    <div class="panel-header">
      <div class="header-title">
        <History class="header-icon" :size="18" />
        <span>请求历史</span>
        <n-tag size="small" type="info">{{ history.length }}</n-tag>
      </div>
      <n-button v-if="history.length" text size="small" @click="clearHistory">
        <Trash2 :size="14" />
        清空
      </n-button>
    </div>

    <div v-if="history.length" class="history-list">
      <div v-for="item in history" :key="item.id" class="history-item"
        :class="{ error: item.error, active: selectedId === item.id }" @click="selectItem(item)">
        <div class="item-main">
          <div class="item-method" :class="`method-${item.endpoint.method.toLowerCase()}`">
            {{ item.endpoint.method }}
          </div>
          <div class="item-info">
            <div class="item-label">{{ item.endpoint.label }}</div>
            <div class="item-meta">
              <span class="item-time">{{ formatTime(item.timestamp) }}</span>
              <span v-if="item.duration" class="item-duration">{{ item.duration }}ms</span>
            </div>
          </div>
        </div>
        <div class="item-status">
          <n-tag v-if="item.error" size="small" type="error">错误</n-tag>
          <n-tag v-else-if="item.statusCode" size="small" :type="getStatusType(item.statusCode)">
            {{ item.statusCode }}
          </n-tag>
        </div>
      </div>
    </div>

    <div v-else class="empty-state">
      <History :size="48" class="empty-icon" />
      <p>暂无请求历史</p>
      <span class="empty-hint">发送请求后将显示在这里</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { NTag, NButton } from 'naive-ui'
import { History, Trash2 } from 'lucide-vue-next'
import type { RequestHistoryItem } from '@/composables/test'

/**
 * API 请求历史面板组件
 *
 * 展示最近的 API 请求历史记录
 */

interface Props {
  /** 历史记录列表 */
  history: RequestHistoryItem[]
}

const props = defineProps<Props>()

const emit = defineEmits<{
  /** 选择历史记录 */
  (e: 'select', item: RequestHistoryItem): void
  /** 清空历史 */
  (e: 'clear'): void
}>()

/** 当前选中的历史记录 ID */
const selectedId = ref<string | null>(null)

type TagType = 'default' | 'success' | 'info' | 'warning' | 'error' | 'primary'

/** 获取状态码对应的类型 */
function getStatusType(statusCode: number): TagType {
  if (statusCode >= 200 && statusCode < 300) return 'success'
  if (statusCode >= 300 && statusCode < 400) return 'warning'
  if (statusCode >= 400) return 'error'
  return 'default'
}

/** 格式化时间 */
function formatTime(timestamp: number): string {
  const date = new Date(timestamp)
  const now = new Date()
  const diff = now.getTime() - date.getTime()

  // 小于 1 分钟
  if (diff < 60000) {
    return '刚刚'
  }

  // 小于 1 小时
  if (diff < 3600000) {
    return `${Math.floor(diff / 60000)} 分钟前`
  }

  // 小于 24 小时
  if (diff < 86400000) {
    return `${Math.floor(diff / 3600000)} 小时前`
  }

  // 超过 24 小时，显示具体日期
  return date.toLocaleDateString('zh-CN', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  })
}

/** 选择历史记录 */
function selectItem(item: RequestHistoryItem): void {
  selectedId.value = item.id
  emit('select', item)
}

/** 清空历史 */
function clearHistory(): void {
  emit('clear')
  selectedId.value = null
}
</script>

<style scoped>
.api-history-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-container);
  border-radius: var(--radius-lg);
  overflow: hidden;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-color);
}

.header-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 500;
  color: var(--text-primary);
}

.header-icon {
  color: var(--info-color);
}

.history-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.history-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 12px;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all 0.2s;
  margin-bottom: 4px;
}

.history-item:hover {
  background: var(--bg-hover);
}

.history-item.active {
  background: var(--primary-color-light);
  border-left: 3px solid var(--primary-color);
}

.history-item.error {
  border-left: 3px solid var(--error-color);
}

.item-main {
  display: flex;
  align-items: center;
  gap: 10px;
  flex: 1;
  min-width: 0;
}

.item-method {
  padding: 2px 6px;
  border-radius: var(--radius-sm);
  font-size: 10px;
  font-weight: 600;
  min-width: 40px;
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

.item-info {
  flex: 1;
  min-width: 0;
}

.item-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.item-meta {
  display: flex;
  gap: 8px;
  margin-top: 2px;
}

.item-time,
.item-duration {
  font-size: 11px;
  color: var(--text-tertiary);
}

.item-status {
  flex-shrink: 0;
}

.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--text-tertiary);
  padding: 40px;
}

.empty-icon {
  margin-bottom: 16px;
  opacity: 0.5;
}

.empty-hint {
  font-size: 12px;
  margin-top: 8px;
}

/* 滚动条样式 */
.history-list::-webkit-scrollbar {
  width: 6px;
}

.history-list::-webkit-scrollbar-track {
  background: transparent;
}

.history-list::-webkit-scrollbar-thumb {
  background: var(--border-color);
  border-radius: 3px;
}

.history-list::-webkit-scrollbar-thumb:hover {
  background: var(--text-tertiary);
}

/* 移动端适配 */
@media (max-width: 768px) {
  .history-item {
    padding: 8px;
  }

  .item-method {
    min-width: 36px;
    font-size: 9px;
  }

  .item-label {
    font-size: 12px;
  }

  .item-meta {
    flex-direction: column;
    gap: 2px;
  }
}
</style>
