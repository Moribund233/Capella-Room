<template>
  <div class="api-request-panel">
    <div class="panel-header">
      <div class="header-title">
        <Send class="header-icon" :size="18" />
        <span>请求配置</span>
      </div>
      <n-button type="primary" size="small" :loading="loading" :disabled="!selectedEndpoint" @click="handleSend">
        <template #icon>
          <Play :size="14" />
        </template>
        发送请求
      </n-button>
    </div>

    <div v-if="selectedEndpoint" class="panel-content">
      <!-- 端点信息 -->
      <div class="endpoint-info">
        <div class="endpoint-badge">
          <span class="method-tag" :class="`method-${selectedEndpoint.method.toLowerCase()}`">
            {{ selectedEndpoint.method }}
          </span>
          <span class="endpoint-path">{{ selectedEndpoint.path }}</span>
        </div>
        <div class="endpoint-description">
          <span class="endpoint-label">{{ selectedEndpoint.label }}</span>
          <n-tag v-if="selectedEndpoint.requiresAuth" size="small" type="warning">
            <template #icon>
              <Lock :size="12" />
            </template>
            需要认证
          </n-tag>
        </div>
      </div>

      <!-- 完整 URL 预览 -->
      <div class="url-preview">
        <div class="section-label">请求 URL</div>
        <div class="url-box">
          <code>{{ fullUrl || selectedEndpoint.path }}</code>
        </div>
      </div>

      <!-- 路径参数 -->
      <div v-if="selectedEndpoint.pathParams?.length" class="params-section">
        <div class="section-label">
          <FileKey :size="14" />
          路径参数
        </div>
        <div class="params-list">
          <div v-for="param in selectedEndpoint.pathParams" :key="param.name" class="param-item">
            <div class="param-info">
              <span class="param-name">{{ param.name }}</span>
              <span v-if="param.required" class="param-required">*</span>
            </div>
            <n-input :value="pathParams[param.name]" :placeholder="param.description" size="small"
              @update:value="(val) => emit('update:pathParams', { ...pathParams, [param.name]: val })" />
          </div>
        </div>
      </div>

      <!-- 查询参数 -->
      <div v-if="selectedEndpoint.queryParams?.length" class="params-section">
        <div class="section-label">
          <Search :size="14" />
          查询参数
        </div>
        <div class="params-list">
          <div v-for="param in selectedEndpoint.queryParams" :key="param.name" class="param-item">
            <div class="param-info">
              <span class="param-name">{{ param.name }}</span>
              <span v-if="param.required" class="param-required">*</span>
              <span v-if="param.default" class="param-default">默认: {{ param.default }}</span>
            </div>
            <n-input :value="queryParams[param.name]" :placeholder="param.description" size="small" clearable
              @update:value="(val) => emit('update:queryParams', { ...queryParams, [param.name]: val })" />
          </div>
        </div>
      </div>

      <!-- 请求头 -->
      <div class="params-section">
        <div class="section-label">
          <List :size="14" />
          请求头
          <n-button text size="tiny" @click="showHeaderEditor = !showHeaderEditor">
            {{ showHeaderEditor ? '收起' : '编辑' }}
          </n-button>
        </div>
        <div v-if="showHeaderEditor" class="headers-editor">
          <div v-for="(value, key) in customHeaders" :key="key" class="header-item">
            <n-input :value="key" disabled size="small" class="header-key" />
            <n-input :value="value" disabled size="small" class="header-value" />
          </div>
          <div class="header-item add-header">
            <n-input v-model:value="newHeaderKey" placeholder="Header 名" size="small" class="header-key" />
            <n-input v-model:value="newHeaderValue" placeholder="Header 值" size="small" class="header-value" />
            <n-button text size="small" @click="addHeader">
              <Plus :size="14" />
            </n-button>
          </div>
        </div>
        <div v-else class="headers-preview">
          <n-tag v-for="(value, key) in requestHeaders" :key="key" size="small" class="header-tag">
            {{ key }}: {{ value.substring(0, 20) }}{{ value.length > 20 ? '...' : '' }}
          </n-tag>
        </div>
      </div>

      <!-- 请求体 -->
      <div v-if="['POST', 'PUT', 'PATCH'].includes(selectedEndpoint.method)" class="params-section">
        <div class="section-label">
          <FileJson :size="14" />
          请求体 (JSON)
          <n-button text size="tiny" @click="formatBody">
            格式化
          </n-button>
        </div>
        <n-input :value="requestBody" type="textarea" :rows="8" placeholder="输入 JSON 格式的请求体..."
          class="body-editor"
          @update:value="(val) => emit('update:requestBody', val)" />
      </div>
    </div>

    <div v-else class="empty-state">
      <Terminal :size="48" class="empty-icon" />
      <p>请从左侧选择一个 API 端点</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { NButton, NInput, NTag } from 'naive-ui'
import {
  Send,
  Play,
  Lock,
  FileKey,
  Search,
  FileJson,
  Plus,
  Terminal,
  List
} from 'lucide-vue-next'
import type { ApiEndpoint } from '@/composables/test'

/**
 * API 请求面板组件
 *
 * 配置和发送 API 请求
 */

interface Props {
  /** 当前选中的端点 */
  selectedEndpoint: ApiEndpoint | null
  /** 路径参数值 */
  pathParams: Record<string, string>
  /** 查询参数值 */
  queryParams: Record<string, string>
  /** 请求体 */
  requestBody: string
  /** 自定义请求头 */
  customHeaders: Record<string, string>
  /** 完整 URL */
  fullUrl: string
  /** 请求头（包含认证） */
  requestHeaders: Record<string, string>
  /** 加载状态 */
  loading: boolean
}

const props = defineProps<Props>()

const emit = defineEmits<{
  /** 更新路径参数 */
  (e: 'update:pathParams', value: Record<string, string>): void
  /** 更新查询参数 */
  (e: 'update:queryParams', value: Record<string, string>): void
  /** 更新请求体 */
  (e: 'update:requestBody', value: string): void
  /** 更新自定义请求头 */
  (e: 'update:customHeaders', value: Record<string, string>): void
  /** 发送请求 */
  (e: 'send'): void
}>()

/** 是否显示请求头编辑器 */
const showHeaderEditor = ref(false)

/** 新请求头键 */
const newHeaderKey = ref('')

/** 新请求头值 */
const newHeaderValue = ref('')

/** 添加请求头 */
function addHeader(): void {
  if (!newHeaderKey.value.trim()) return

  const newHeaders = {
    ...props.customHeaders,
    [newHeaderKey.value.trim()]: newHeaderValue.value
  }
  emit('update:customHeaders', newHeaders)

  newHeaderKey.value = ''
  newHeaderValue.value = ''
}

/** 格式化请求体 */
function formatBody(): void {
  if (!props.requestBody.trim()) return

  try {
    const parsed = JSON.parse(props.requestBody)
    const formatted = JSON.stringify(parsed, null, 2)
    emit('update:requestBody', formatted)
  } catch {
    // 如果不是有效的 JSON，不做处理
  }
}

/** 发送请求 */
function handleSend(): void {
  emit('send')
}
</script>

<style scoped>
.api-request-panel {
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
  color: var(--primary-color);
}

.panel-content {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
}

.endpoint-info {
  margin-bottom: 16px;
  padding: 12px;
  background: var(--bg-elevated);
  border-radius: var(--radius-md);
}

.endpoint-badge {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.method-tag {
  padding: 2px 8px;
  border-radius: var(--radius-sm);
  font-size: 12px;
  font-weight: 600;
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

.endpoint-path {
  font-family: monospace;
  font-size: 13px;
  color: var(--text-secondary);
}

.endpoint-description {
  display: flex;
  align-items: center;
  gap: 8px;
}

.endpoint-label {
  font-weight: 500;
  color: var(--text-primary);
}

.url-preview {
  margin-bottom: 16px;
}

.section-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  font-weight: 500;
  color: var(--text-secondary);
  margin-bottom: 8px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.url-box {
  padding: 10px 12px;
  background: var(--bg-elevated);
  border-radius: var(--radius-md);
  font-family: monospace;
  font-size: 12px;
  color: var(--text-primary);
  word-break: break-all;
}

.params-section {
  margin-bottom: 16px;
}

.params-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.param-item {
  display: flex;
  align-items: center;
  gap: 12px;
}

.param-info {
  display: flex;
  align-items: center;
  gap: 4px;
  min-width: 100px;
}

.param-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
}

.param-required {
  color: var(--error-color);
}

.param-default {
  font-size: 11px;
  color: var(--text-tertiary);
  margin-left: 4px;
}

.headers-editor {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.header-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.header-key {
  width: 150px;
}

.header-value {
  flex: 1;
}

.headers-preview {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.header-tag {
  font-family: monospace;
  font-size: 11px;
}

.body-editor {
  font-family: monospace;
  font-size: 13px;
}

.body-editor :deep(textarea) {
  font-family: 'Fira Code', 'Consolas', monospace;
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

  .param-item {
    flex-direction: column;
    align-items: stretch;
    gap: 4px;
  }

  .param-info {
    min-width: auto;
  }

  .header-key {
    width: 100px;
  }
}
</style>
