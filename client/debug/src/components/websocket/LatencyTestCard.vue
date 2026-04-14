<script setup lang="ts">
/**
 * WebSocket 延迟测试组件
 * 测试连接延迟、消息往返延迟、认证延迟等
 */
import { ref, computed } from 'vue'
import { useWebSocketStore } from '@/stores/websocket'
import { storeToRefs } from 'pinia'
import { Play, Square, BarChart3, Clock, Zap, Activity } from 'lucide-vue-next'
import { getAccessToken } from '@/api'

const wsStore = useWebSocketStore()
const { isConnected, latency: storeLatency } = storeToRefs(wsStore)

// ========== 状态 ==========
const isTesting = ref(false)
const testProgress = ref(0)
const testResults = ref<Array<{
  timestamp: string
  type: 'connect' | 'ping' | 'auth'
  latency: number
  status: 'success' | 'error'
}>>([])

// 统计数据
const connectLatencies = ref<number[]>([])
const pingLatencies = ref<number[]>([])
const authLatencies = ref<number[]>([])

// ========== 计算属性 ==========
const stats = computed(() => {
  const calcStats = (arr: number[]) => {
    if (arr.length === 0) return { min: 0, max: 0, avg: 0, median: 0, p95: 0, p99: 0 }
    const sorted = [...arr].sort((a, b) => a - b)
    const min = sorted[0]
    const max = sorted[sorted.length - 1]
    const avg = Math.round(sorted.reduce((a, b) => a + b, 0) / sorted.length)
    const median = sorted[Math.floor(sorted.length / 2)]
    const p95 = sorted[Math.floor(sorted.length * 0.95)]
    const p99 = sorted[Math.floor(sorted.length * 0.99)]
    return { min, max, avg, median, p95, p99 }
  }

  return {
    connect: calcStats(connectLatencies.value),
    ping: calcStats(pingLatencies.value),
    auth: calcStats(authLatencies.value),
  }
})

const totalTests = computed(() => testResults.value.length)
const successRate = computed(() => {
  if (totalTests.value === 0) return 0
  const success = testResults.value.filter(r => r.status === 'success').length
  return Math.round((success / totalTests.value) * 100)
})

// ========== 方法 ==========
const addResult = (type: 'connect' | 'ping' | 'auth', latency: number, status: 'success' | 'error') => {
  testResults.value.unshift({
    timestamp: new Date().toLocaleTimeString(),
    type,
    latency,
    status
  })
  if (testResults.value.length > 100) {
    testResults.value = testResults.value.slice(0, 100)
  }
}

const testConnectLatency = async (): Promise<number> => {
  return new Promise((resolve) => {
    const start = performance.now()
    const ws = new WebSocket(import.meta.env.VITE_WS_URL || 'ws://localhost:8080/ws')

    ws.onopen = () => {
      const latency = Math.round(performance.now() - start)
      ws.close()
      resolve(latency)
    }

    ws.onerror = () => {
      resolve(-1)
    }

    // 超时处理
    setTimeout(() => {
      ws.close()
      resolve(-1)
    }, 10000)
  })
}

const testAuthLatency = async (): Promise<number> => {
  return new Promise((resolve) => {
    const token = getAccessToken()
    if (!token) {
      resolve(-1)
      return
    }

    const start = performance.now()
    const ws = new WebSocket(import.meta.env.VITE_WS_URL || 'ws://localhost:8080/ws')

    ws.onopen = () => {
      ws.send(JSON.stringify({ type: 'Auth', payload: { token } }))
    }

    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data)
        if (data.type === 'AuthResult') {
          const latency = Math.round(performance.now() - start)
          ws.close()
          resolve(data.payload?.success ? latency : -1)
        }
      } catch {
        // ignore
      }
    }

    ws.onerror = () => {
      resolve(-1)
    }

    setTimeout(() => {
      ws.close()
      resolve(-1)
    }, 10000)
  })
}

const testPingLatency = async (): Promise<number> => {
  return new Promise((resolve) => {
    if (!isConnected.value) {
      resolve(-1)
      return
    }

    const start = performance.now()
    const initialLatency = storeLatency.value

    // 发送 ping
    wsStore.ping()

    // 监听 latency 变化
    const checkLatency = setInterval(() => {
      if (storeLatency.value !== null && storeLatency.value !== initialLatency) {
        clearInterval(checkLatency)
        resolve(storeLatency.value)
      }

      // 超时
      if (performance.now() - start > 5000) {
        clearInterval(checkLatency)
        resolve(-1)
      }
    }, 50)
  })
}

const runLatencyTest = async (count: number = 10) => {
  isTesting.value = true
  testProgress.value = 0

  // 清空之前的结果
  connectLatencies.value = []
  pingLatencies.value = []
  authLatencies.value = []

  for (let i = 0; i < count; i++) {
    // 测试连接延迟
    const connectLatency = await testConnectLatency()
    if (connectLatency > 0) {
      connectLatencies.value.push(connectLatency)
      addResult('connect', connectLatency, 'success')
    } else {
      addResult('connect', 0, 'error')
    }

    await new Promise(r => setTimeout(r, 100))

    // 测试认证延迟
    const authLatency = await testAuthLatency()
    if (authLatency > 0) {
      authLatencies.value.push(authLatency)
      addResult('auth', authLatency, 'success')
    } else {
      addResult('auth', 0, 'error')
    }

    await new Promise(r => setTimeout(r, 100))

    // 测试 Ping 延迟（如果已连接）
    if (isConnected.value) {
      const pingLatency = await testPingLatency()
      if (pingLatency > 0) {
        pingLatencies.value.push(pingLatency)
        addResult('ping', pingLatency, 'success')
      } else {
        addResult('ping', 0, 'error')
      }
    }

    testProgress.value = Math.round(((i + 1) / count) * 100)
    await new Promise(r => setTimeout(r, 200))
  }

  isTesting.value = false
}

const stopTest = () => {
  isTesting.value = false
}

const resetTest = () => {
  testResults.value = []
  connectLatencies.value = []
  pingLatencies.value = []
  authLatencies.value = []
  testProgress.value = 0
}
</script>

<template>
  <n-card title="延迟测试" class="test-card">
    <template #header-extra>
      <n-tag :type="successRate >= 90 ? 'success' : successRate >= 70 ? 'warning' : 'error'">
        成功率: {{ successRate }}%
      </n-tag>
    </template>

    <!-- 当前延迟显示 -->
    <n-grid :cols="2" :x-gap="12" :y-gap="12" style="margin-bottom: 16px;">
      <n-grid-item>
        <n-statistic label="当前 Ping 延迟" :value="storeLatency || '-'" suffix="ms">
          <template #prefix>
            <Zap class="icon-sm" :style="{ color: storeLatency && storeLatency < 100 ? 'var(--success)' : storeLatency && storeLatency < 300 ? 'var(--warning)' : 'var(--error)' }" />
          </template>
        </n-statistic>
      </n-grid-item>
      <n-grid-item>
        <n-statistic label="测试次数" :value="totalTests" />
      </n-grid-item>
    </n-grid>

    <!-- 详细统计 -->
    <n-divider />
    <div class="stats-section">
      <div class="stats-title">
        <BarChart3 class="icon-sm" />
        延迟统计 (ms)
      </div>

      <!-- 连接延迟 -->
      <div class="stat-group">
        <div class="stat-label">连接延迟</div>
        <n-space>
          <n-tag size="small">最小: {{ stats.connect.min }}ms</n-tag>
          <n-tag size="small">平均: {{ stats.connect.avg }}ms</n-tag>
          <n-tag size="small">中位数: {{ stats.connect.median }}ms</n-tag>
          <n-tag size="small" type="info">P95: {{ stats.connect.p95 }}ms</n-tag>
          <n-tag size="small" type="warning">P99: {{ stats.connect.p99 }}ms</n-tag>
          <n-tag size="small" type="error">最大: {{ stats.connect.max }}ms</n-tag>
        </n-space>
      </div>

      <!-- 认证延迟 -->
      <div class="stat-group">
        <div class="stat-label">认证延迟</div>
        <n-space>
          <n-tag size="small">最小: {{ stats.auth.min }}ms</n-tag>
          <n-tag size="small">平均: {{ stats.auth.avg }}ms</n-tag>
          <n-tag size="small">中位数: {{ stats.auth.median }}ms</n-tag>
          <n-tag size="small" type="info">P95: {{ stats.auth.p95 }}ms</n-tag>
          <n-tag size="small" type="warning">P99: {{ stats.auth.p99 }}ms</n-tag>
          <n-tag size="small" type="error">最大: {{ stats.auth.max }}ms</n-tag>
        </n-space>
      </div>

      <!-- Ping 延迟 -->
      <div class="stat-group">
        <div class="stat-label">Ping 延迟</div>
        <n-space>
          <n-tag size="small">最小: {{ stats.ping.min }}ms</n-tag>
          <n-tag size="small">平均: {{ stats.ping.avg }}ms</n-tag>
          <n-tag size="small">中位数: {{ stats.ping.median }}ms</n-tag>
          <n-tag size="small" type="info">P95: {{ stats.ping.p95 }}ms</n-tag>
          <n-tag size="small" type="warning">P99: {{ stats.ping.p99 }}ms</n-tag>
          <n-tag size="small" type="error">最大: {{ stats.ping.max }}ms</n-tag>
        </n-space>
      </div>
    </div>

    <!-- 测试控制 -->
    <n-divider />
    <n-space style="margin-bottom: 16px;">
      <n-button
        type="primary"
        :disabled="isTesting"
        @click="runLatencyTest(10)"
      >
        <template #icon>
          <Play class="icon-sm" />
        </template>
        运行测试 (10次)
      </n-button>
      <n-button
        :disabled="isTesting"
        @click="runLatencyTest(50)"
      >
        <template #icon>
          <Activity class="icon-sm" />
        </template>
        压力测试 (50次)
      </n-button>
      <n-button
        type="error"
        :disabled="!isTesting"
        @click="stopTest"
      >
        <template #icon>
          <Square class="icon-sm" />
        </template>
        停止
      </n-button>
      <n-button @click="resetTest">
        <template #icon>
          <Clock class="icon-sm" />
        </template>
        重置
      </n-button>
    </n-space>

    <!-- 进度条 -->
    <n-progress
      v-if="isTesting"
      type="line"
      :percentage="testProgress"
      :indicator-placement="'inside'"
      processing
      style="margin-bottom: 16px;"
    />

    <!-- 测试结果 -->
    <div class="test-logs">
      <div class="logs-header">
        <span class="logs-title">测试记录</span>
      </div>
      <div class="logs-content">
        <div
          v-for="(result, index) in testResults.slice(0, 20)"
          :key="index"
          class="log-item"
        >
          <span class="log-time">[{{ result.timestamp }}]</span>
          <n-tag size="tiny" :type="result.type === 'connect' ? 'info' : result.type === 'auth' ? 'warning' : 'success'">
            {{ result.type === 'connect' ? '连接' : result.type === 'auth' ? '认证' : 'Ping' }}
          </n-tag>
          <n-tag size="tiny" :type="result.status === 'success' ? 'success' : 'error'">
            {{ result.status === 'success' ? '成功' : '失败' }}
          </n-tag>
          <span v-if="result.latency > 0" class="log-latency" :class="{
            'latency-good': result.latency < 100,
            'latency-medium': result.latency >= 100 && result.latency < 300,
            'latency-bad': result.latency >= 300
          }">
            {{ result.latency }}ms
          </span>
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

.stats-section {
  margin-bottom: 16px;
}

.stats-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 500;
  margin-bottom: 12px;
  color: var(--text-primary);
}

.stat-group {
  margin-bottom: 12px;
}

.stat-label {
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: 4px;
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

.log-latency {
  font-weight: 500;
}

.latency-good {
  color: var(--success);
}

.latency-medium {
  color: var(--warning);
}

.latency-bad {
  color: var(--error);
}

.icon-sm {
  width: 16px;
  height: 16px;
}
</style>
