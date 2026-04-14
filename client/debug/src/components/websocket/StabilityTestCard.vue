<script setup lang="ts">
/**
 * WebSocket 稳定性测试组件
 * 测试连接稳定性、重连机制、断线恢复等功能
 */
import { ref, computed, onUnmounted } from 'vue'
import { useWebSocketStore } from '@/stores/websocket'
import { storeToRefs } from 'pinia'
import { Play, Square, RotateCcw, Activity, Wifi, WifiOff, AlertCircle } from 'lucide-vue-next'

const wsStore = useWebSocketStore()
const { isConnected, status, reconnectAttempts } = storeToRefs(wsStore)

// ========== 状态 ==========
const isTesting = ref(false)
const testResults = ref<Array<{
  timestamp: string
  event: 'connect' | 'disconnect' | 'reconnect' | 'error'
  duration?: number
  message?: string
}>>([])
const connectionHistory = ref<Array<{
  startTime: string
  endTime?: string
  duration: number
  disconnectReason?: string
}>>([])
const currentSessionStart = ref<string | null>(null)
const manualDisconnectCount = ref(0)

// ========== 计算属性 ==========
const totalConnections = computed(() => testResults.value.filter(r => r.event === 'connect').length)
const totalDisconnections = computed(() => testResults.value.filter(r => r.event === 'disconnect').length)
const totalReconnects = computed(() => testResults.value.filter(r => r.event === 'reconnect').length)
const totalErrors = computed(() => testResults.value.filter(r => r.event === 'error').length)

const averageConnectionDuration = computed(() => {
  const completedSessions = connectionHistory.value.filter(s => s.endTime)
  if (completedSessions.length === 0) return 0
  const total = completedSessions.reduce((sum, s) => sum + s.duration, 0)
  return Math.round(total / completedSessions.length)
})

const longestConnection = computed(() => {
  const completedSessions = connectionHistory.value.filter(s => s.endTime)
  if (completedSessions.length === 0) return 0
  return Math.max(...completedSessions.map(s => s.duration))
})

const stabilityScore = computed(() => {
  if (totalConnections.value === 0) return 100
  const errorRate = totalErrors.value / totalConnections.value
  const reconnectRate = totalReconnects.value / totalConnections.value
  return Math.max(0, Math.round(100 - errorRate * 50 - reconnectRate * 30))
})

// ========== 方法 ==========
const addTestResult = (event: 'connect' | 'disconnect' | 'reconnect' | 'error', duration?: number, message?: string) => {
  testResults.value.unshift({
    timestamp: new Date().toLocaleTimeString(),
    event,
    duration,
    message
  })
  // 限制历史记录数量
  if (testResults.value.length > 50) {
    testResults.value = testResults.value.slice(0, 50)
  }
}

const startStabilityTest = () => {
  isTesting.value = true
  addTestResult('connect', undefined, '开始稳定性测试')
  
  // 如果未连接，先连接
  if (!isConnected.value) {
    wsStore.connect()
  }
}

const stopStabilityTest = () => {
  isTesting.value = false
  addTestResult('disconnect', undefined, '停止稳定性测试')
}

const resetTest = () => {
  testResults.value = []
  connectionHistory.value = []
  currentSessionStart.value = null
  manualDisconnectCount.value = 0
}

const simulateDisconnect = () => {
  manualDisconnectCount.value++
  wsStore.disconnect()
  addTestResult('disconnect', undefined, `手动断开连接 #${manualDisconnectCount.value}`)
}

const forceReconnect = () => {
  wsStore.disconnect()
  setTimeout(() => {
    wsStore.connect()
    addTestResult('reconnect', undefined, '强制重连')
  }, 1000)
}

// 监听连接状态变化
const unwatch = wsStore.$onAction(({ name, after }) => {
  after(() => {
    if (!isTesting.value) return
    
    if (name === 'connect') {
      currentSessionStart.value = new Date().toISOString()
      connectionHistory.value.push({
        startTime: new Date().toLocaleTimeString(),
        duration: 0
      })
      addTestResult('connect')
    } else if (name === 'disconnect') {
      const lastSession = connectionHistory.value[connectionHistory.value.length - 1]
      if (lastSession && !lastSession.endTime) {
        lastSession.endTime = new Date().toLocaleTimeString()
        lastSession.duration = Date.now() - new Date(lastSession.startTime).getTime()
      }
      addTestResult('disconnect')
    }
  })
})

onUnmounted(() => {
  unwatch()
})
</script>

<template>
  <n-card title="稳定性测试" class="test-card">
    <template #header-extra>
      <n-tag :type="stabilityScore >= 80 ? 'success' : stabilityScore >= 60 ? 'warning' : 'error'">
        稳定性评分: {{ stabilityScore }}
      </n-tag>
    </template>

    <!-- 统计概览 -->
    <n-grid :cols="4" :x-gap="12" :y-gap="12" style="margin-bottom: 16px;">
      <n-grid-item>
        <n-statistic label="总连接次数" :value="totalConnections">
          <template #prefix>
            <Wifi class="icon-sm" style="color: var(--success);" />
          </template>
        </n-statistic>
      </n-grid-item>
      <n-grid-item>
        <n-statistic label="总断开次数" :value="totalDisconnections">
          <template #prefix>
            <WifiOff class="icon-sm" style="color: var(--error);" />
          </template>
        </n-statistic>
      </n-grid-item>
      <n-grid-item>
        <n-statistic label="重连次数" :value="totalReconnects">
          <template #prefix>
            <RotateCcw class="icon-sm" style="color: var(--warning);" />
          </template>
        </n-statistic>
      </n-grid-item>
      <n-grid-item>
        <n-statistic label="错误次数" :value="totalErrors">
          <template #prefix>
            <AlertCircle class="icon-sm" style="color: var(--error);" />
          </template>
        </n-statistic>
      </n-grid-item>
    </n-grid>

    <!-- 连接时长统计 -->
    <n-grid :cols="3" :x-gap="12" :y-gap="12" style="margin-bottom: 16px;">
      <n-grid-item>
        <n-statistic label="平均连接时长" :value="averageConnectionDuration" suffix="ms" />
      </n-grid-item>
      <n-grid-item>
        <n-statistic label="最长连接" :value="longestConnection" suffix="ms" />
      </n-grid-item>
      <n-grid-item>
        <n-statistic label="当前重连次数" :value="reconnectAttempts" />
      </n-grid-item>
    </n-grid>

    <!-- 测试控制 -->
    <n-space style="margin-bottom: 16px;">
      <n-button
        type="primary"
        :disabled="isTesting"
        @click="startStabilityTest"
      >
        <template #icon>
          <Play class="icon-sm" />
        </template>
        开始测试
      </n-button>
      <n-button
        type="error"
        :disabled="!isTesting"
        @click="stopStabilityTest"
      >
        <template #icon>
          <Square class="icon-sm" />
        </template>
        停止测试
      </n-button>
      <n-button @click="resetTest">
        <template #icon>
          <RotateCcw class="icon-sm" />
        </template>
        重置
      </n-button>
      <n-button
        :disabled="!isConnected"
        @click="simulateDisconnect"
      >
        <template #icon>
          <WifiOff class="icon-sm" />
        </template>
        模拟断开
      </n-button>
      <n-button
        :disabled="isConnected"
        @click="forceReconnect"
      >
        <template #icon>
          <Activity class="icon-sm" />
        </template>
        强制重连
      </n-button>
    </n-space>

    <!-- 测试日志 -->
    <n-divider />
    <div class="test-logs">
      <div class="logs-header">
        <span class="logs-title">测试日志</span>
        <n-tag size="small" :type="isConnected ? 'success' : 'error'">
          {{ isConnected ? '已连接' : '未连接' }}
        </n-tag>
      </div>
      <div class="logs-content">
        <div
          v-for="(result, index) in testResults"
          :key="index"
          class="log-item"
          :class="`log-${result.event}`"
        >
          <span class="log-time">[{{ result.timestamp }}]</span>
          <n-tag size="tiny" :type="
            result.event === 'connect' ? 'success' :
            result.event === 'disconnect' ? 'error' :
            result.event === 'reconnect' ? 'warning' : 'error'
          ">
            {{ result.event === 'connect' ? '连接' :
               result.event === 'disconnect' ? '断开' :
               result.event === 'reconnect' ? '重连' : '错误' }}
          </n-tag>
          <span v-if="result.duration" class="log-duration">{{ result.duration }}ms</span>
          <span v-if="result.message" class="log-message">{{ result.message }}</span>
        </div>
        <n-empty v-if="testResults.length === 0" description="暂无测试记录" size="small" />
      </div>
    </div>
  </n-card>
</template>

<style scoped>
.test-card {
  margin-bottom: 16px;
}

.test-logs {
  background-color: var(--bg-secondary);
  border-radius: var(--radius-md);
  padding: 12px;
}

.logs-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.logs-title {
  font-weight: 500;
  color: var(--text-primary);
}

.logs-content {
  max-height: 200px;
  overflow-y: auto;
}

.log-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 0;
  font-family: monospace;
  font-size: 12px;
}

.log-time {
  color: var(--text-muted);
  min-width: 80px;
}

.log-duration {
  color: var(--info);
}

.log-message {
  color: var(--text-secondary);
}

.icon-sm {
  width: 16px;
  height: 16px;
}
</style>
