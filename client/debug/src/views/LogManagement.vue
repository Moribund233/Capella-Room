<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useMessage } from 'naive-ui'
import { FileText, Wifi, WifiOff, RefreshCw, Settings } from 'lucide-vue-next'
import { useWebSocketStore, type LogEntry } from '@/stores/websocket'
import LogPanel from '@/components/log/LogPanel.vue'
import LogFilter from '@/components/log/LogFilter.vue'
import LogStats from '@/components/log/LogStats.vue'

/**
 * 日志管理页
 * 提供实时日志查看、过滤、统计功能
 */

const message = useMessage()
const wsStore = useWebSocketStore()

// ========== 状态 ==========
const levelFilter = ref('all')
const moduleFilter = ref('all')
const searchKeyword = ref('')
const autoScroll = ref(true)
const selectedLog = ref<LogEntry | null>(null)
const showDetailModal = ref(false)

// ========== 计算属性 ==========
const isConnected = computed(() => wsStore.isConnected)
const isSubscribed = computed(() => wsStore.logSubscribed)

// ========== 方法 ==========

/**
 * 连接 WebSocket
 */
function connectWebSocket() {
  if (!wsStore.isConnected && !wsStore.isConnecting) {
    wsStore.connect()
  }
}

/**
 * 订阅日志
 */
function subscribeLogs() {
  if (wsStore.isConnected) {
    const success = wsStore.subscribeLogs(levelFilter.value, moduleFilter.value)
    if (success) {
      message.success('已订阅日志流')
    } else {
      message.error('订阅日志流失败')
    }
  } else {
    message.warning('请先连接 WebSocket')
  }
}

/**
 * 取消订阅日志
 */
function unsubscribeLogs() {
  const success = wsStore.unsubscribeLogs()
  if (success) {
    message.info('已取消订阅日志流')
  }
}

/**
 * 处理日志点击
 */
function handleLogClick(log: LogEntry) {
  selectedLog.value = log
  showDetailModal.value = true
}

/**
 * 清空日志
 */
function clearLogs() {
  wsStore.clearLogs()
  message.success('日志已清空')
}

/**
 * 导出日志
 */
function exportLogs() {
  const logs = wsStore.logs
  if (logs.length === 0) {
    message.warning('没有日志可导出')
    return
  }

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

  message.success('日志已导出')
}

/**
 * 格式化 JSON 字段
 */
function formatJsonFields(fields: Record<string, unknown> | undefined): string {
  if (!fields || Object.keys(fields).length === 0) {
    return '无额外字段'
  }
  return JSON.stringify(fields, null, 2)
}

// ========== 生命周期 ==========
onMounted(() => {
  // 如果未连接，先建立连接
  if (!wsStore.isConnected && !wsStore.isConnecting) {
    connectWebSocket()
  }
  // 如果已连接但未订阅，自动订阅日志
  if (wsStore.isConnected && !wsStore.logSubscribed) {
    subscribeLogs()
  }
})

// 监听连接状态变化
watch(() => wsStore.isConnected, (connected) => {
  if (connected && !wsStore.logSubscribed) {
    // 连接成功后自动订阅日志
    setTimeout(() => {
      subscribeLogs()
    }, 500)
  }
})
</script>

<template>
  <div class="page-container">
    <!-- 页面标题 -->
    <div class="page-header">
      <h1 class="page-title">
        <FileText
          class="icon-lg"
          style="display: inline; vertical-align: middle; margin-right: 8px"
        />
        日志管理
      </h1>
      <p class="page-subtitle">实时查看和分析系统日志</p>
    </div>

    <!-- 控制栏 -->
    <n-card style="margin-bottom: var(--space-lg)">
      <n-space align="center" justify="space-between">
        <n-space align="center">
          <n-tag :type="isConnected ? 'success' : 'error'" size="small">
            <template #icon>
              <n-icon>
                <component :is="isConnected ? Wifi : WifiOff" />
              </n-icon>
            </template>
            {{ isConnected ? '已连接' : '未连接' }}
          </n-tag>
          <n-tag :type="isSubscribed ? 'success' : 'default'" size="small">
            {{ isSubscribed ? '已订阅' : '未订阅' }}
          </n-tag>
          <n-text depth="3">
            共 {{ wsStore.logs.length }} 条日志
          </n-text>
        </n-space>

        <n-space align="center">
          <n-button
            v-if="!isConnected"
            type="primary"
            size="small"
            @click="connectWebSocket"
          >
            <template #icon>
              <n-icon><Wifi /></n-icon>
            </template>
            连接
          </n-button>

          <n-button
            v-if="isConnected && !isSubscribed"
            type="primary"
            size="small"
            @click="subscribeLogs"
          >
            <template #icon>
              <n-icon><RefreshCw /></n-icon>
            </template>
            订阅日志
          </n-button>

          <n-button
            v-if="isSubscribed"
            size="small"
            @click="unsubscribeLogs"
          >
            取消订阅
          </n-button>

          <n-divider vertical />

          <n-button size="small" @click="exportLogs">
            导出日志
          </n-button>

          <n-button size="small" type="error" ghost @click="clearLogs">
            清空日志
          </n-button>

          <n-switch v-model:value="autoScroll" size="small">
            <template #checked>自动滚动</template>
            <template #unchecked>手动滚动</template>
          </n-switch>
        </n-space>
      </n-space>
    </n-card>

    <!-- 主布局 -->
    <div class="log-management-layout">
      <!-- 左侧：日志列表 -->
      <div class="log-section">
        <LogPanel
          v-model:level-filter="levelFilter"
          v-model:module-filter="moduleFilter"
          v-model:search-keyword="searchKeyword"
          :auto-scroll="autoScroll"
          @log-click="handleLogClick"
        />
      </div>

      <!-- 右侧：过滤器和统计 -->
      <div class="sidebar-section">
        <LogFilter
          v-model:level="levelFilter"
          v-model:module="moduleFilter"
          v-model:keyword="searchKeyword"
        />
        <LogStats />
      </div>
    </div>

    <!-- 日志详情弹窗 -->
    <n-modal
      v-model:show="showDetailModal"
      title="日志详情"
      preset="card"
      style="width: 600px; max-width: 90vw"
    >
      <template v-if="selectedLog">
        <n-descriptions bordered :column="1" size="small">
          <n-descriptions-item label="时间">
            {{ new Date(selectedLog.timestamp).toLocaleString('zh-CN') }}
          </n-descriptions-item>
          <n-descriptions-item label="级别">
            <n-tag :type="selectedLog.level.toLowerCase() === 'error' ? 'error' : selectedLog.level.toLowerCase() === 'warn' ? 'warning' : 'info'">
              {{ selectedLog.level.toUpperCase() }}
            </n-tag>
          </n-descriptions-item>
          <n-descriptions-item label="模块">
            <n-tag>{{ selectedLog.target }}</n-tag>
          </n-descriptions-item>
          <n-descriptions-item label="消息">
            {{ selectedLog.message }}
          </n-descriptions-item>
          <n-descriptions-item v-if="selectedLog.fields && Object.keys(selectedLog.fields).length > 0" label="额外字段">
            <n-code :code="formatJsonFields(selectedLog.fields)" language="json" />
          </n-descriptions-item>
        </n-descriptions>
      </template>
    </n-modal>
  </div>
</template>

<style scoped>
.log-management-layout {
  display: flex;
  gap: var(--space-lg);
  height: calc(100vh - 280px);
  min-height: 500px;
}

.log-section {
  flex: 1;
  min-width: 0;
  background: #fff;
  border-radius: 8px;
  overflow: hidden;
}

.sidebar-section {
  flex: 0 0 280px;
  display: flex;
  flex-direction: column;
  gap: var(--space-lg);
  overflow-y: auto;
}

.sidebar-section > * {
  flex-shrink: 0;
}

/* 移动端适配 */
@media screen and (max-width: 1024px) {
  .log-management-layout {
    flex-direction: column;
    height: auto;
  }

  .log-section {
    height: 500px;
  }

  .sidebar-section {
    flex: 1 1 100%;
    flex-direction: row;
    flex-wrap: wrap;
  }

  .sidebar-section > * {
    flex: 1;
    min-width: 250px;
  }
}

@media screen and (max-width: 767px) {
  .sidebar-section {
    flex-direction: column;
  }

  .sidebar-section > * {
    min-width: 100%;
  }
}
</style>
