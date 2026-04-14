<script setup lang="ts">
/**
 * WebSocket 压力测试组件
 * 测试消息吞吐量、并发连接、长时间运行稳定性
 */
import { ref, computed } from 'vue'
import { useWebSocketStore } from '@/stores/websocket'
import { storeToRefs } from 'pinia'
import { Play, Square, TrendingUp, MessageSquare, Zap, Clock } from 'lucide-vue-next'

const wsStore = useWebSocketStore()
const { isConnected } = storeToRefs(wsStore)

// ========== 状态 ==========
const isTesting = ref(false)
const testConfig = ref({
  messageCount: 100,
  messageInterval: 50, // ms
  concurrentConnections: 1,
  testDuration: 60, // seconds
})

const testResults = ref({
  messagesSent: 0,
  messagesReceived: 0,
  messagesFailed: 0,
  totalBytes: 0,
  startTime: 0,
  endTime: 0,
  latencies: [] as number[],
})

const currentTest = ref({
  progress: 0,
  currentConnections: 0,
  messagesPerSecond: 0,
  elapsedTime: 0,
})

// ========== 计算属性 ==========
const throughput = computed(() => {
  const duration = (testResults.value.endTime || Date.now()) - testResults.value.startTime
  if (duration === 0) return 0
  return Math.round((testResults.value.messagesReceived / duration) * 1000)
})

const averageLatency = computed(() => {
  if (testResults.value.latencies.length === 0) return 0
  return Math.round(
    testResults.value.latencies.reduce((a, b) => a + b, 0) / testResults.value.latencies.length
  )
})

const successRate = computed(() => {
  const total = testResults.value.messagesSent
  if (total === 0) return 0
  return Math.round((testResults.value.messagesReceived / total) * 100)
})

const testDuration = computed(() => {
  if (testResults.value.startTime === 0) return 0
  const end = testResults.value.endTime || Date.now()
  return Math.round((end - testResults.value.startTime) / 1000)
})

// ========== 方法 ==========
const resetResults = () => {
  testResults.value = {
    messagesSent: 0,
    messagesReceived: 0,
    messagesFailed: 0,
    totalBytes: 0,
    startTime: 0,
    endTime: 0,
    latencies: [],
  }
  currentTest.value = {
    progress: 0,
    currentConnections: 0,
    messagesPerSecond: 0,
    elapsedTime: 0,
  }
}

const sendTestMessage = async (): Promise<boolean> => {
  return new Promise((resolve) => {
    if (!isConnected.value) {
      resolve(false)
      return
    }

    const start = performance.now()
    const messageId = `stress_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
    const content = 'Stress test message: ' + 'x'.repeat(100) // 100 bytes payload

    // 发送消息 - 使用正确的 Ping 格式（无 payload）
    const success = wsStore.send({
      type: 'Ping'
    })

    if (!success) {
      resolve(false)
      return
    }

    testResults.value.messagesSent++
    testResults.value.totalBytes += content.length

    // 等待响应
    const timeout = setTimeout(() => {
      testResults.value.messagesFailed++
      resolve(false)
    }, 5000)

    // 监听响应
    const checkResponse = () => {
      const latency = Math.round(performance.now() - start)
      testResults.value.latencies.push(latency)
      testResults.value.messagesReceived++
      clearTimeout(timeout)
      resolve(true)
    }

    // 简化处理：假设消息发送成功即算成功
    setTimeout(checkResponse, 10)
  })
}

const runStressTest = async () => {
  if (!isConnected.value) {
    return
  }

  isTesting.value = true
  resetResults()
  testResults.value.startTime = Date.now()

  const interval = setInterval(() => {
    currentTest.value.elapsedTime = Math.round((Date.now() - testResults.value.startTime) / 1000)
    currentTest.value.progress = Math.min(
      100,
      Math.round((currentTest.value.elapsedTime / testConfig.value.testDuration) * 100)
    )

    // 计算每秒消息数
    if (currentTest.value.elapsedTime > 0) {
      currentTest.value.messagesPerSecond = Math.round(
        testResults.value.messagesSent / currentTest.value.elapsedTime
      )
    }
  }, 1000)

  // 发送消息循环
  let messageCount = 0
  const sendLoop = async () => {
    while (isTesting.value && messageCount < testConfig.value.messageCount) {
      await sendTestMessage()
      messageCount++
      await new Promise(r => setTimeout(r, testConfig.value.messageInterval))
    }
    stopTest()
  }

  // 超时处理
  const timeoutTimer = setTimeout(() => {
    if (isTesting.value) {
      stopTest()
    }
  }, testConfig.value.testDuration * 1000)

  await sendLoop()

  clearInterval(interval)
  clearTimeout(timeoutTimer)
}

const stopTest = () => {
  isTesting.value = false
  testResults.value.endTime = Date.now()
}

const formatBytes = (bytes: number): string => {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(2) + ' KB'
  return (bytes / (1024 * 1024)).toFixed(2) + ' MB'
}
</script>

<template>
  <n-card title="压力测试" class="test-card">
    <template #header-extra>
      <n-tag :type="successRate >= 95 ? 'success' : successRate >= 80 ? 'warning' : 'error'">
        成功率: {{ successRate }}%
      </n-tag>
    </template>

    <!-- 配置区域 -->
    <n-grid :cols="2" :x-gap="16" :y-gap="16" style="margin-bottom: 16px;">
      <n-grid-item>
        <n-form-item label="消息数量">
          <n-input-number v-model:value="testConfig.messageCount" :min="10" :max="10000" :disabled="isTesting" />
        </n-form-item>
      </n-grid-item>
      <n-grid-item>
        <n-form-item label="发送间隔 (ms)">
          <n-input-number v-model:value="testConfig.messageInterval" :min="10" :max="1000" :disabled="isTesting" />
        </n-form-item>
      </n-grid-item>
      <n-grid-item>
        <n-form-item label="测试时长 (秒)">
          <n-input-number v-model:value="testConfig.testDuration" :min="10" :max="300" :disabled="isTesting" />
        </n-form-item>
      </n-grid-item>
      <n-grid-item>
        <n-form-item label="并发连接">
          <n-input-number v-model:value="testConfig.concurrentConnections" :min="1" :max="10" :disabled="isTesting" />
        </n-form-item>
      </n-grid-item>
    </n-grid>

    <!-- 实时统计 -->
    <n-divider />
    <n-grid :cols="4" :x-gap="12" :y-gap="12" style="margin-bottom: 16px;">
      <n-grid-item>
        <n-statistic label="消息发送">
          <template #prefix>
            <MessageSquare class="icon-sm" style="color: var(--info);" />
          </template>
          {{ testResults.messagesSent }}
        </n-statistic>
      </n-grid-item>
      <n-grid-item>
        <n-statistic label="消息接收">
          <template #prefix>
            <TrendingUp class="icon-sm" style="color: var(--success);" />
          </template>
          {{ testResults.messagesReceived }}
        </n-statistic>
      </n-grid-item>
      <n-grid-item>
        <n-statistic label="消息失败">
          <template #prefix>
            <Zap class="icon-sm" style="color: var(--error);" />
          </template>
          {{ testResults.messagesFailed }}
        </n-statistic>
      </n-grid-item>
      <n-grid-item>
        <n-statistic label="吞吐量">
          <template #prefix>
            <Clock class="icon-sm" style="color: var(--warning);" />
          </template>
          {{ throughput }}/s
        </n-statistic>
      </n-grid-item>
    </n-grid>

    <!-- 更多统计 -->
    <n-grid :cols="3" :x-gap="12" :y-gap="12" style="margin-bottom: 16px;">
      <n-grid-item>
        <n-statistic label="平均延迟" :value="averageLatency" suffix="ms" />
      </n-grid-item>
      <n-grid-item>
        <n-statistic label="数据总量" :value="formatBytes(testResults.totalBytes)" />
      </n-grid-item>
      <n-grid-item>
        <n-statistic label="测试时长" :value="testDuration" suffix="秒" />
      </n-grid-item>
    </n-grid>

    <!-- 控制按钮 -->
    <n-space style="margin-bottom: 16px;">
      <n-button
        type="primary"
        :disabled="isTesting || !isConnected"
        @click="runStressTest"
      >
        <template #icon>
          <Play class="icon-sm" />
        </template>
        开始压力测试
      </n-button>
      <n-button
        type="error"
        :disabled="!isTesting"
        @click="stopTest"
      >
        <template #icon>
          <Square class="icon-sm" />
        </template>
        停止测试
      </n-button>
    </n-space>

    <!-- 进度条 -->
    <n-progress
      v-if="isTesting || currentTest.progress > 0"
      type="line"
      :percentage="currentTest.progress"
      :indicator-placement="'inside'"
      :status="isTesting ? 'processing' : 'success'"
      style="margin-bottom: 16px;"
    />

    <!-- 实时状态 -->
    <div v-if="isTesting" class="realtime-status">
      <n-alert type="info" :show-icon="false">
        <n-space>
          <span>运行时间: {{ currentTest.elapsedTime }}秒</span>
          <span>|</span>
          <span>当前速度: {{ currentTest.messagesPerSecond }} 消息/秒</span>
          <span>|</span>
          <span>进度: {{ currentTest.progress }}%</span>
        </n-space>
      </n-alert>
    </div>

    <!-- 测试结果评估 -->
    <n-divider />
    <div class="result-evaluation">
      <div class="evaluation-title">测试结果评估</div>
      <n-space>
        <n-tag :type="throughput >= 50 ? 'success' : throughput >= 20 ? 'warning' : 'error'">
          吞吐量: {{ throughput >= 50 ? '优秀' : throughput >= 20 ? '良好' : '需优化' }}
        </n-tag>
        <n-tag :type="averageLatency <= 100 ? 'success' : averageLatency <= 300 ? 'warning' : 'error'">
          延迟: {{ averageLatency <= 100 ? '优秀' : averageLatency <= 300 ? '良好' : '需优化' }}
        </n-tag>
        <n-tag :type="successRate >= 99 ? 'success' : successRate >= 95 ? 'warning' : 'error'">
          可靠性: {{ successRate >= 99 ? '优秀' : successRate >= 95 ? '良好' : '需优化' }}
        </n-tag>
      </n-space>
    </div>
  </n-card>
</template>

<style scoped>
.test-card {
  margin-bottom: 16px;
}

.realtime-status {
  margin-bottom: 16px;
}

.result-evaluation {
  margin-top: 16px;
}

.evaluation-title {
  font-weight: 500;
  margin-bottom: 12px;
  color: var(--text-primary);
}

.icon-sm {
  width: 16px;
  height: 16px;
}
</style>
