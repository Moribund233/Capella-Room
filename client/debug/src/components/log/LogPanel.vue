<script setup lang="ts">
import { ref, computed, nextTick, watch } from 'vue'
import { useWebSocketStore, type LogEntry } from '@/stores/websocket'
import { FileText, Trash2, Download, AlertCircle, Info, AlertTriangle, XCircle } from 'lucide-vue-next'

/**
 * 日志面板组件
 * 显示实时日志流，支持自动滚动、清空、导出等功能
 */

interface Props {
  /** 日志级别过滤 */
  levelFilter?: string
  /** 模块过滤 */
  moduleFilter?: string
  /** 搜索关键词 */
  searchKeyword?: string
  /** 是否自动滚动到底部 */
  autoScroll?: boolean
  /** 最大显示行数 */
  maxLines?: number
}

const props = withDefaults(defineProps<Props>(), {
  levelFilter: 'all',
  moduleFilter: 'all',
  searchKeyword: '',
  autoScroll: true,
  maxLines: 1000
})

const emit = defineEmits<{
  (e: 'logClick', log: LogEntry): void
}>()

const wsStore = useWebSocketStore()
const logContainerRef = ref<HTMLElement | null>(null)
const isPaused = ref(false)

/**
 * 过滤后的日志列表
 */
const filteredLogs = computed(() => {
  let logs = wsStore.logs

  // 级别过滤
  if (props.levelFilter && props.levelFilter !== 'all') {
    logs = logs.filter(log => log.level.toLowerCase() === props.levelFilter.toLowerCase())
  }

  // 模块过滤
  if (props.moduleFilter && props.moduleFilter !== 'all') {
    logs = logs.filter(log => log.target === props.moduleFilter)
  }

  // 搜索过滤
  if (props.searchKeyword) {
    const keyword = props.searchKeyword.toLowerCase()
    logs = logs.filter(log =>
      log.message.toLowerCase().includes(keyword) ||
      log.target.toLowerCase().includes(keyword) ||
      JSON.stringify(log.fields).toLowerCase().includes(keyword)
    )
  }

  // 限制显示数量
  return logs.slice(0, props.maxLines)
})

/**
 * 日志总数
 */
const totalLogs = computed(() => wsStore.logs.length)

/**
 * 过滤后的日志数
 */
const filteredCount = computed(() => filteredLogs.value.length)

/**
 * 获取日志级别对应的图标
 */
function getLevelIcon(level: string) {
  switch (level.toLowerCase()) {
    case 'error':
      return XCircle
    case 'warn':
      return AlertTriangle
    case 'info':
      return Info
    default:
      return FileText
  }
}

/**
 * 获取日志级别对应的颜色
 */
function getLevelColor(level: string): string {
  switch (level.toLowerCase()) {
    case 'error':
      return '#f5222d'
    case 'warn':
      return '#faad14'
    case 'info':
      return '#1890ff'
    case 'debug':
      return '#52c41a'
    default:
      return '#8c8c8c'
  }
}

/**
 * 获取日志级别对应的标签类型
 */
function getLevelType(level: string): 'error' | 'warning' | 'info' | 'success' | 'default' {
  switch (level.toLowerCase()) {
    case 'error':
      return 'error'
    case 'warn':
      return 'warning'
    case 'info':
      return 'info'
    case 'debug':
      return 'success'
    default:
      return 'default'
  }
}

/**
 * 格式化时间戳
 */
function formatTimestamp(timestamp: string): string {
  const date = new Date(timestamp)
  return date.toLocaleTimeString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    fractionalSecondDigits: 3
  })
}

/**
 * 格式化日志字段
 */
function formatFields(fields: Record<string, unknown> | undefined): string {
  if (!fields || Object.keys(fields).length === 0) {
    return ''
  }
  return JSON.stringify(fields, null, 2)
}

/**
 * 处理日志点击
 */
function handleLogClick(log: LogEntry) {
  emit('logClick', log)
}

/**
 * 清空日志
 */
function clearLogs() {
  wsStore.clearLogs()
}

/**
 * 导出日志
 */
function exportLogs() {
  const logs = filteredLogs.value
  const content = logs.map(log => {
    const fields = log.fields ? ` ${JSON.stringify(log.fields)}` : ''
    return `[${log.timestamp}] [${log.level.toUpperCase()}] [${log.target}] ${log.message}${fields}`
  }).join('\n')

  const blob = new Blob([content], { type: 'text/plain;charset=utf-8' })
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = `logs_${new Date().toISOString().slice(0, 19).replace(/:/g, '-')}.txt`
  document.body.appendChild(link)
  link.click()
  document.body.removeChild(link)
  URL.revokeObjectURL(url)
}

/**
 * 切换暂停/恢复
 */
function togglePause() {
  isPaused.value = !isPaused.value
}

/**
 * 滚动到底部
 */
async function scrollToBottom() {
  await nextTick()
  if (logContainerRef.value && props.autoScroll && !isPaused.value) {
    logContainerRef.value.scrollTop = logContainerRef.value.scrollHeight
  }
}

/**
 * 监听日志变化，自动滚动
 */
watch(() => wsStore.logs.length, () => {
  if (props.autoScroll && !isPaused.value) {
    scrollToBottom()
  }
}, { flush: 'post' })

/**
 * 监听过滤条件变化，滚动到底部
 */
watch([() => props.levelFilter, () => props.moduleFilter], () => {
  scrollToBottom()
})

defineExpose({
  clearLogs,
  exportLogs,
  scrollToBottom,
  togglePause
})
</script>

<template>
  <div class="log-panel">
    <!-- 工具栏 -->
    <div class="log-toolbar">
      <n-space align="center" justify="space-between">
        <n-space align="center">
          <n-tag size="small" :type="wsStore.logSubscribed ? 'success' : 'default'">
            {{ wsStore.logSubscribed ? '已订阅' : '未订阅' }}
          </n-tag>
          <n-text depth="3" class="log-count">
            显示 {{ filteredCount }} / {{ totalLogs }} 条日志
          </n-text>
        </n-space>

        <n-space align="center">
          <n-button
            size="small"
            :type="isPaused ? 'warning' : 'default'"
            @click="togglePause"
          >
            <template #icon>
              <n-icon>
                <component :is="isPaused ? 'Play' : 'Pause'" />
              </n-icon>
            </template>
            {{ isPaused ? '继续' : '暂停' }}
          </n-button>
          <n-button size="small" @click="exportLogs">
            <template #icon>
              <n-icon><Download /></n-icon>
            </template>
            导出
          </n-button>
          <n-button size="small" type="error" ghost @click="clearLogs">
            <template #icon>
              <n-icon><Trash2 /></n-icon>
            </template>
            清空
          </n-button>
        </n-space>
      </n-space>
    </div>

    <!-- 日志列表 -->
    <div ref="logContainerRef" class="log-container">
      <div v-if="filteredLogs.length === 0" class="log-empty">
        <n-empty description="暂无日志">
          <template #icon>
            <n-icon size="48" depth="3"><FileText /></n-icon>
          </template>
        </n-empty>
      </div>

      <div v-else class="log-list">
        <div
          v-for="(log, index) in filteredLogs"
          :key="`${log.timestamp}-${index}`"
          class="log-item"
          :class="`log-level-${log.level.toLowerCase()}`"
          @click="handleLogClick(log)"
        >
          <!-- 时间戳 -->
          <span class="log-timestamp">{{ formatTimestamp(log.timestamp) }}</span>

          <!-- 级别标签 -->
          <n-tag
            size="tiny"
            :type="getLevelType(log.level)"
            class="log-level"
          >
            <template #icon>
              <n-icon size="12">
                <component :is="getLevelIcon(log.level)" />
              </n-icon>
            </template>
            {{ log.level.toUpperCase() }}
          </n-tag>

          <!-- 模块 -->
          <n-tag size="tiny" class="log-target" :color="{ textColor: '#8c8c8c', borderColor: '#d9d9d9' }">
            {{ log.target }}
          </n-tag>

          <!-- 消息内容 -->
          <span class="log-message" :style="{ color: getLevelColor(log.level) }">
            {{ log.message }}
          </span>

          <!-- 展开字段 -->
          <n-collapse v-if="log.fields && Object.keys(log.fields).length > 0" class="log-fields">
            <n-collapse-item title="详情" name="fields">
              <pre class="log-fields-content">{{ formatFields(log.fields) }}</pre>
            </n-collapse-item>
          </n-collapse>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.log-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #fafafa;
  border-radius: 8px;
  overflow: hidden;
}

.log-toolbar {
  padding: 12px 16px;
  background: #fff;
  border-bottom: 1px solid #f0f0f0;
}

.log-count {
  font-size: 12px;
}

.log-container {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.log-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
}

.log-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.log-item {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 6px 10px;
  background: #fff;
  border-radius: 4px;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 12px;
  line-height: 1.5;
  cursor: pointer;
  transition: background-color 0.2s;
  flex-wrap: wrap;
}

.log-item:hover {
  background: #f0f5ff;
}

.log-level-error {
  border-left: 3px solid #f5222d;
}

.log-level-warn {
  border-left: 3px solid #faad14;
}

.log-level-info {
  border-left: 3px solid #1890ff;
}

.log-level-debug {
  border-left: 3px solid #52c41a;
}

.log-timestamp {
  color: #8c8c8c;
  flex-shrink: 0;
  min-width: 85px;
}

.log-level {
  flex-shrink: 0;
  min-width: 55px;
  justify-content: center;
}

.log-target {
  flex-shrink: 0;
  max-width: 120px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.log-message {
  flex: 1;
  word-break: break-all;
  min-width: 200px;
}

.log-fields {
  width: 100%;
  margin-top: 4px;
  margin-left: 0;
}

.log-fields-content {
  margin: 0;
  padding: 8px;
  background: #f5f5f5;
  border-radius: 4px;
  font-size: 11px;
  overflow-x: auto;
  white-space: pre-wrap;
  word-break: break-all;
}

/* 滚动条样式 */
.log-container::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}

.log-container::-webkit-scrollbar-track {
  background: #f1f1f1;
  border-radius: 3px;
}

.log-container::-webkit-scrollbar-thumb {
  background: #c1c1c1;
  border-radius: 3px;
}

.log-container::-webkit-scrollbar-thumb:hover {
  background: #a8a8a8;
}
</style>
