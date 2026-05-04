<template>
  <n-card class="ws-log-panel" :bordered="false">
    <template #header>
      <div class="panel-header">
        <div class="header-left">
          <n-icon :component="Terminal" size="18" />
          <span class="title">WebSocket 日志</span>
          <n-badge :value="filteredLogs.length" :max="999" />
        </div>
        <div class="header-right">
          <n-input
            v-model:value="searchKeyword"
            placeholder="搜索日志"
            size="small"
            clearable
            style="width: 150px"
          >
            <template #prefix>
              <n-icon :component="Search" size="14" />
            </template>
          </n-input>
          <n-select
            v-model:value="filterType"
            :options="filterOptions"
            size="small"
            style="width: 120px"
            clearable
            placeholder="筛选类型"
          />
          <n-button size="small" @click="handleClear">
            <template #icon>
              <n-icon :component="Trash2" size="14" />
            </template>
            清空
          </n-button>
          <n-button size="small" @click="handleExport">
            <template #icon>
              <n-icon :component="Download" size="14" />
            </template>
            导出
          </n-button>
          <n-button size="small" @click="isExpanded = !isExpanded">
            <template #icon>
              <n-icon :component="isExpanded ? ChevronDown : ChevronUp" size="14" />
            </template>
            {{ isExpanded ? '收起' : '展开' }}
          </n-button>
        </div>
      </div>
    </template>

    <n-collapse-transition :show="isExpanded">
      <div class="log-container" ref="logContainer">
        <n-empty v-if="filteredLogs.length === 0" description="暂无日志" size="small">
          <template #icon>
            <n-icon :component="FileText" />
          </template>
        </n-empty>

        <div v-else class="log-list">
          <div
            v-for="log in filteredLogs"
            :key="log.id"
            class="log-item"
            :class="`log-${log.type}`"
          >
            <div class="log-header">
              <n-tag :type="getLogType(log.type)" size="tiny" class="log-type-tag">
                {{ log.type.toUpperCase() }}
              </n-tag>
              <span class="log-time">{{ formatTime(log.timestamp) }}</span>
              <span class="log-user">{{ log.username }}</span>
              <span v-if="log.messageType" class="log-message-type">
                [{{ log.messageType }}]
              </span>
            </div>
            <div class="log-content">{{ log.content }}</div>
            <div v-if="log.rawData" class="log-raw-data">
              <n-code :code="formatRawData(log.rawData)" language="json" :word-wrap="true" />
            </div>
          </div>
        </div>
      </div>
    </n-collapse-transition>

    <div v-if="!isExpanded" class="collapsed-hint" @click="isExpanded = true">
      <n-text depth="3">点击展开查看 {{ filteredLogs.length }} 条日志</n-text>
      <n-icon :component="ChevronUp" size="14" />
    </div>
  </n-card>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import {
  NCard,
  NBadge,
  NInput,
  NSelect,
  NButton,
  NIcon,
  NEmpty,
  NTag,
  NCode,
  NText,
  NCollapseTransition,
  useMessage,
} from 'naive-ui'
import {
  Terminal,
  Search,
  Trash2,
  Download,
  ChevronDown,
  ChevronUp,
  FileText,
} from 'lucide-vue-next'
import { useWsTestStore, type WsLogEntry } from '@/store/wsTest'

const message = useMessage()
const wsStore = useWsTestStore()

// 本地状态
const isExpanded = ref(false)
const searchKeyword = ref('')
const filterType = ref<string | null>(null)
const logContainer = ref<HTMLElement | null>(null)

// 筛选选项
const filterOptions = [
  { label: '发送', value: 'send' },
  { label: '接收', value: 'receive' },
  { label: '连接', value: 'connect' },
  { label: '断开', value: 'disconnect' },
  { label: '错误', value: 'error' },
  { label: '系统', value: 'system' },
]

// 计算属性
const logs = computed(() => wsStore.logs)

const filteredLogs = computed(() => {
  let result = logs.value

  // 按类型筛选
  if (filterType.value) {
    result = result.filter(log => log.type === filterType.value)
  }

  // 按关键词搜索
  if (searchKeyword.value) {
    const keyword = searchKeyword.value.toLowerCase()
    result = result.filter(log =>
      log.content.toLowerCase().includes(keyword) ||
      log.username.toLowerCase().includes(keyword) ||
      log.messageType?.toLowerCase().includes(keyword)
    )
  }

  // 按时间倒序
  return [...result].reverse()
})

// 获取日志类型样式
function getLogType(type: WsLogEntry['type']): 'success' | 'info' | 'warning' | 'error' | 'default' {
  const typeMap: Record<WsLogEntry['type'], 'success' | 'info' | 'warning' | 'error' | 'default'> = {
    send: 'success',
    receive: 'info',
    connect: 'success',
    disconnect: 'warning',
    error: 'error',
    system: 'default',
  }
  return typeMap[type]
}

// 格式化时间
function formatTime(timestamp: number): string {
  const date = new Date(timestamp)
  return date.toLocaleTimeString('zh-CN', {
    hour12: false,
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  })
}

// 格式化原始数据
function formatRawData(data: unknown): string {
  try {
    return JSON.stringify(data, null, 2)
  } catch {
    return String(data)
  }
}

// 清空日志
function handleClear() {
  wsStore.clearLogs()
  message.success('日志已清空')
}

// 导出日志
function handleExport() {
  const data = {
    timestamp: Date.now(),
    total: logs.value.length,
    logs: logs.value,
  }

  const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `websocket-logs-${Date.now()}.json`
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)

  message.success('日志已导出')
}

// 自动滚动到底部
watch(
  () => logs.value.length,
  () => {
    if (isExpanded.value && logContainer.value) {
      nextTick(() => {
        logContainer.value!.scrollTop = logContainer.value!.scrollHeight
      })
    }
  }
)
</script>

<style scoped>
.ws-log-panel {
  margin-top: 16px;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.title {
  font-weight: 500;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.log-container {
  max-height: 400px;
  overflow-y: auto;
  background: var(--bg-default);
  border-radius: 8px;
  padding: 12px;
}

.log-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.log-item {
  padding: 8px 12px;
  border-radius: 6px;
  background: var(--bg-container);
  border-left: 3px solid transparent;
  font-size: 13px;
}

.log-item.log-send {
  border-left-color: #18a058;
}

.log-item.log-receive {
  border-left-color: #2080f0;
}

.log-item.log-connect {
  border-left-color: #18a058;
}

.log-item.log-disconnect {
  border-left-color: #f0a020;
}

.log-item.log-error {
  border-left-color: #d03050;
  background: rgba(208, 48, 80, 0.05);
}

.log-item.log-system {
  border-left-color: #999;
}

.log-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
  flex-wrap: wrap;
}

.log-type-tag {
  font-size: 10px;
  padding: 0 4px;
  height: 18px;
}

.log-time {
  color: var(--text-color-3);
  font-size: 11px;
  font-family: monospace;
}

.log-user {
  color: var(--text-color-2);
  font-weight: 500;
}

.log-message-type {
  color: var(--primary-color);
  font-size: 11px;
}

.log-content {
  color: var(--text-color-1);
  word-break: break-word;
}

.log-raw-data {
  margin-top: 8px;
  padding: 8px;
  background: var(--bg-default);
  border-radius: 4px;
  overflow-x: auto;
}

.collapsed-hint {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 12px;
  cursor: pointer;
  border-top: 1px solid var(--divider-color);
  transition: background 0.3s;
}

.collapsed-hint:hover {
  background: var(--bg-default);
}

@media (max-width: 768px) {
  .panel-header {
    flex-direction: column;
    gap: 12px;
    align-items: flex-start;
  }

  .header-right {
    flex-wrap: wrap;
    width: 100%;
  }

  .header-right .n-input,
  .header-right .n-select {
    flex: 1;
    min-width: 120px;
  }
}
</style>
