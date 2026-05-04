<template>
  <div class="api-response-panel">
    <div class="panel-header">
      <div class="header-title">
        <MessageSquare class="header-icon" :size="18" />
        <span>响应结果</span>
      </div>
      <div v-if="result" class="header-actions">
        <n-tag :type="getStatusType(result.statusCode)" size="small">
          {{ result.statusCode }} {{ result.statusText }}
        </n-tag>
        <n-tag v-if="result.duration" size="small" type="info">
          <Clock :size="12" />
          {{ result.duration }}ms
        </n-tag>
        <n-button text size="small" @click="copyResponse">
          <Copy :size="14" />
        </n-button>
      </div>
    </div>

    <div v-if="result" class="panel-content">
      <!-- 响应概览 -->
      <div class="response-overview">
        <div class="overview-item">
          <span class="overview-label">状态码</span>
          <span class="overview-value" :class="`status-${getStatusType(result.statusCode)}`">
            {{ result.statusCode }}
          </span>
        </div>
        <div class="overview-item">
          <span class="overview-label">响应时间</span>
          <span class="overview-value">{{ result.duration }}ms</span>
        </div>
        <div class="overview-item">
          <span class="overview-label">内容类型</span>
          <span class="overview-value">{{ result.headers['content-type'] || 'unknown' }}</span>
        </div>
      </div>

      <!-- 响应头 -->
      <div class="response-section">
        <div class="section-header" @click="showHeaders = !showHeaders">
          <ChevronDown v-if="showHeaders" :size="16" />
          <ChevronRight v-else :size="16" />
          <List :size="14" />
          <span>响应头</span>
          <n-tag size="small" type="info">{{ Object.keys(result.headers).length }}</n-tag>
        </div>
        <div v-show="showHeaders" class="section-content">
          <div v-for="(value, key) in result.headers" :key="key" class="header-row">
            <span class="header-name">{{ key }}</span>
            <span class="header-value">{{ value }}</span>
          </div>
        </div>
      </div>

      <!-- 响应体 -->
      <div class="response-section">
        <div class="section-header">
          <FileJson :size="14" />
          <span>响应体</span>
          <n-button text size="tiny" @click="copyResponse">
            <Copy :size="12" />
            复制
          </n-button>
        </div>
        <div class="section-content">
          <pre v-if="formattedData" class="response-body"><code>{{ formattedData }}</code></pre>
          <div v-else class="empty-body">无响应数据</div>
        </div>
      </div>
    </div>

    <div v-else class="empty-state">
      <MessageSquare :size="48" class="empty-icon" />
      <p>发送请求后查看响应结果</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { NTag, NButton, useMessage } from 'naive-ui'
import {
  MessageSquare,
  Clock,
  Copy,
  ChevronDown,
  ChevronRight,
  List,
  FileJson
} from 'lucide-vue-next'
import type { ApiTestResult } from '@/composables/test'

/**
 * API 响应面板组件
 *
 * 展示 API 请求的响应结果
 */

interface Props {
  /** 响应结果 */
  result: ApiTestResult | null
}

const props = defineProps<Props>()

const message = useMessage()

/** 是否显示响应头 */
const showHeaders = ref(false)

/** 格式化的响应数据 */
const formattedData = computed(() => {
  if (!props.result?.data) return null

  if (typeof props.result.data === 'string') {
    try {
      // 尝试解析为 JSON 并格式化
      const parsed = JSON.parse(props.result.data)
      return JSON.stringify(parsed, null, 2)
    } catch {
      // 如果不是 JSON，直接返回字符串
      return props.result.data
    }
  }

  // 对象类型，直接格式化
  return JSON.stringify(props.result.data, null, 2)
})

type TagType = 'default' | 'success' | 'info' | 'warning' | 'error' | 'primary'

/** 获取状态码对应的类型 */
function getStatusType(statusCode: number): TagType {
  if (statusCode >= 200 && statusCode < 300) return 'success'
  if (statusCode >= 300 && statusCode < 400) return 'warning'
  if (statusCode >= 400 && statusCode < 500) return 'error'
  if (statusCode >= 500) return 'error'
  return 'default'
}

/** 复制响应内容 */
async function copyResponse(): Promise<void> {
  if (!formattedData.value) return

  try {
    await navigator.clipboard.writeText(formattedData.value)
    message.success('已复制到剪贴板')
  } catch {
    message.error('复制失败')
  }
}
</script>

<style scoped>
.api-response-panel {
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
  color: var(--success-color);
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.panel-content {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
}

.response-overview {
  display: flex;
  gap: 16px;
  margin-bottom: 16px;
  padding: 12px;
  background: var(--bg-elevated);
  border-radius: var(--radius-md);
}

.overview-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.overview-label {
  font-size: 11px;
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.overview-value {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.overview-value.status-success {
  color: var(--success-color);
}

.overview-value.status-warning {
  color: var(--warning-color);
}

.overview-value.status-error {
  color: var(--error-color);
}

.response-section {
  margin-bottom: 16px;
}

.section-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 0;
  font-weight: 500;
  color: var(--text-secondary);
  cursor: pointer;
  user-select: none;
}

.section-header:hover {
  color: var(--text-primary);
}

.section-content {
  background: var(--bg-elevated);
  border-radius: var(--radius-md);
  overflow: hidden;
}

.header-row {
  display: flex;
  padding: 8px 12px;
  border-bottom: 1px solid var(--border-color);
  font-size: 12px;
}

.header-row:last-child {
  border-bottom: none;
}

.header-name {
  min-width: 150px;
  font-weight: 500;
  color: var(--text-secondary);
  font-family: monospace;
}

.header-value {
  flex: 1;
  color: var(--text-primary);
  word-break: break-all;
  font-family: monospace;
}

.response-body {
  margin: 0;
  padding: 12px;
  font-family: 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  line-height: 1.5;
  color: var(--text-primary);
  background: var(--bg-elevated);
  overflow-x: auto;
  white-space: pre-wrap;
  word-wrap: break-word;
}

.response-body code {
  font-family: inherit;
}

.empty-body {
  padding: 24px;
  text-align: center;
  color: var(--text-tertiary);
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

/* 滚动条样式 */
.panel-content::-webkit-scrollbar {
  width: 6px;
}

.panel-content::-webkit-scrollbar-track {
  background: transparent;
}

.panel-content::-webkit-scrollbar-thumb {
  background: var(--border-color);
  border-radius: 3px;
}

.panel-content::-webkit-scrollbar-thumb:hover {
  background: var(--text-tertiary);
}

/* 移动端适配 */
@media (max-width: 768px) {
  .panel-content {
    padding: 12px;
  }

  .response-overview {
    flex-wrap: wrap;
    gap: 12px;
  }

  .header-row {
    flex-direction: column;
    gap: 4px;
  }

  .header-name {
    min-width: auto;
  }
}
</style>
