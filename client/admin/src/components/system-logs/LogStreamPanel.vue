<template>
  <div class="log-stream-panel">
    <!-- 工具栏 -->
    <div class="log-toolbar">
      <div class="log-filters">
        <n-select
          v-model:value="systemLogsStore.currentLevel"
          :options="levelOptions"
          size="small"
          style="width: 120px"
          @update:value="handleLevelChange"
        />
        <n-select
          v-model:value="systemLogsStore.currentModule"
          :options="moduleOptions"
          size="small"
          style="width: 140px"
          @update:value="handleModuleChange"
        />
      </div>
      <div class="log-actions">
        <n-tag :type="subscriptionStatusType" size="small">
          {{ subscriptionStatusText }}
        </n-tag>
        <n-button size="small" @click="handleClear">
          <template #icon>
            <n-icon><trash-2 /></n-icon>
          </template>
          清空
        </n-button>
        <n-button size="small" @click="handleExport">
          <template #icon>
            <n-icon><download /></n-icon>
          </template>
          导出
        </n-button>
        <n-button
          size="small"
          :type="systemLogsStore.isSubscribed ? 'error' : 'primary'"
          @click="handleToggleSubscription"
        >
          <template #icon>
            <n-icon>
              <pause v-if="systemLogsStore.isSubscribed" />
              <play v-else />
            </n-icon>
          </template>
          {{ systemLogsStore.isSubscribed ? '停止' : '开始' }}
        </n-button>
      </div>
    </div>

    <!-- 统计信息 -->
    <div class="log-stats">
      <n-space>
        <n-tag size="small">总数: {{ systemLogsStore.totalLogs }}</n-tag>
        <n-tag size="small" type="error">错误: {{ systemLogsStore.logStats.error }}</n-tag>
        <n-tag size="small" type="warning">警告: {{ systemLogsStore.logStats.warn }}</n-tag>
        <n-tag size="small" type="info">信息: {{ systemLogsStore.logStats.info }}</n-tag>
        <n-tag size="small" type="success">调试: {{ systemLogsStore.logStats.debug }}</n-tag>
      </n-space>
    </div>

    <!-- 日志列表 -->
    <div ref="logContainerRef" class="log-container">
      <div
        v-for="(log, index) in systemLogsStore.filteredLogs"
        :key="index"
        class="log-entry"
        :class="`log-${log.level}`"
      >
        <div class="log-header">
          <span class="log-time">{{ formatTime(log.timestamp) }}</span>
          <n-tag :type="getLevelTagType(log.level) as any" size="tiny" class="log-level">
            {{ log.level.toUpperCase() }}
          </n-tag>
          <span class="log-target">{{ log.target }}</span>
        </div>
        <div class="log-message">{{ log.message }}</div>
        <pre v-if="log.fields" class="log-fields">{{ JSON.stringify(log.fields, null, 2) }}</pre>
      </div>
      <div v-if="systemLogsStore.filteredLogs.length === 0" class="log-empty">
        <n-empty description="暂无日志" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue'
import {
  NSelect,
  NButton,
  NTag,
  NIcon,
  NSpace,
  NEmpty,
  useMessage,
} from 'naive-ui'
import {
  Trash2,
  Download,
  Play,
  Pause,
} from 'lucide-vue-next'
import { useSystemLogsStore } from '@/store'
import type { LogLevel, LogModule } from '@/types'

const message = useMessage()
const systemLogsStore = useSystemLogsStore()
const logContainerRef = ref<HTMLElement>()

// 级别选项
const levelOptions = [
  { label: '全部级别', value: 'all' },
  { label: '错误', value: 'error' },
  { label: '警告', value: 'warn' },
  { label: '信息', value: 'info' },
  { label: '调试', value: 'debug' },
]

// 模块选项
const moduleOptions = [
  { label: '全部模块', value: 'all' },
  { label: 'WebSocket', value: 'websocket' },
  { label: '房间', value: 'room' },
  { label: '消息', value: 'message' },
  { label: '性能', value: 'performance' },
]

// 订阅状态
const subscriptionStatusText = computed(() => {
  if (systemLogsStore.isSubscribed) {
    return '已连接'
  }
  return '未连接'
})

const subscriptionStatusType = computed(() => {
  if (systemLogsStore.isSubscribed) {
    return 'success'
  }
  return 'default'
})

// 获取级别标签类型
function getLevelTagType(level: LogLevel): string {
  const typeMap: Record<string, string> = {
    error: 'error',
    warn: 'warning',
    info: 'info',
    debug: 'success',
  }
  return typeMap[level] || 'default'
}

// 格式化时间
function formatTime(timestamp: string): string {
  const date = new Date(timestamp)
  return date.toLocaleTimeString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    fractionalSecondDigits: 3,
  })
}

// 处理级别变更
function handleLevelChange(level: LogLevel) {
  systemLogsStore.setLevel(level)
  if (systemLogsStore.isSubscribed) {
    // 重新订阅以应用新过滤条件
    systemLogsStore.subscribe({ level, module: systemLogsStore.currentModule })
  }
}

// 处理模块变更
function handleModuleChange(module: LogModule) {
  systemLogsStore.setModule(module)
  if (systemLogsStore.isSubscribed) {
    // 重新订阅以应用新过滤条件
    systemLogsStore.subscribe({ level: systemLogsStore.currentLevel, module })
  }
}

// 处理清空
function handleClear() {
  systemLogsStore.clearLogs()
  message.success('日志已清空')
}

// 处理导出
function handleExport() {
  const content = systemLogsStore.exportLogsAsText()
  const blob = new Blob([content], { type: 'text/plain' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `system-logs-${new Date().toISOString().slice(0, 19).replace(/:/g, '-')}.txt`
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
  message.success('日志已导出')
}

// 处理订阅切换
function handleToggleSubscription() {
  if (systemLogsStore.isSubscribed) {
    systemLogsStore.unsubscribe()
    message.info('已停止日志流')
  } else {
    systemLogsStore.subscribe({
      level: systemLogsStore.currentLevel,
      module: systemLogsStore.currentModule,
    })
    message.info('已开始日志流')
  }
}

// 自动滚动到底部
watch(
  () => systemLogsStore.logs.length,
  () => {
    nextTick(() => {
      if (logContainerRef.value) {
        logContainerRef.value.scrollTop = logContainerRef.value.scrollHeight
      }
    })
  },
)

onMounted(() => {
  // 初始化日志流处理器
  systemLogsStore.init()
})

onUnmounted(() => {
  // 组件卸载时取消订阅
  systemLogsStore.unsubscribe()
})
</script>

<style scoped lang="scss">
.log-stream-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-color);
  border-radius: 8px;
  overflow: hidden;
}

.log-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-color);
  gap: 16px;
  flex-wrap: wrap;
}

.log-filters {
  display: flex;
  gap: 8px;
}

.log-actions {
  display: flex;
  gap: 8px;
  align-items: center;
}

.log-stats {
  padding: 8px 16px;
  border-bottom: 1px solid var(--border-color);
  background: var(--bg-color-secondary);
}

.log-container {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 13px;
  line-height: 1.5;
}

.log-entry {
  margin-bottom: 8px;
  padding: 8px 12px;
  border-radius: 4px;
  border-left: 3px solid transparent;

  &.log-error {
    background: rgba(255, 77, 79, 0.05);
    border-left-color: #ff4d4f;
  }

  &.log-warn {
    background: rgba(250, 173, 20, 0.05);
    border-left-color: #faad14;
  }

  &.log-info {
    background: rgba(24, 144, 255, 0.05);
    border-left-color: #1890ff;
  }

  &.log-debug {
    background: rgba(82, 196, 26, 0.05);
    border-left-color: #52c41a;
  }
}

.log-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.log-time {
  color: var(--text-color-secondary);
  font-size: 12px;
  min-width: 80px;
}

.log-level {
  min-width: 50px;
  text-align: center;
}

.log-target {
  color: var(--text-color-secondary);
  font-size: 12px;
}

.log-message {
  color: var(--text-color);
  word-break: break-all;
}

.log-fields {
  margin-top: 4px;
  padding: 8px;
  background: var(--bg-color-secondary);
  border-radius: 4px;
  font-size: 12px;
  overflow-x: auto;
}

.log-empty {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 200px;
}
</style>
