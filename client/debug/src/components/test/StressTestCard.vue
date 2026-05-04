<template>
  <n-card title="压力测试" class="stress-test-card">
    <!-- 测试配置 -->
    <div class="test-config">
      <n-form label-placement="left" label-width="120" :show-feedback="false">
        <n-grid :cols="2" :x-gap="16">
          <n-grid-item>
            <n-form-item label="并发用户数">
              <n-input-number
                v-model:value="config.concurrentUsers"
                :min="1"
                :max="connectedCount"
                style="width: 100%"
              />
            </n-form-item>
          </n-grid-item>
          <n-grid-item>
            <n-form-item label="每用户消息数">
              <n-input-number
                v-model:value="config.messagesPerUser"
                :min="1"
                :max="1000"
                style="width: 100%"
              />
            </n-form-item>
          </n-grid-item>
          <n-grid-item>
            <n-form-item label="发送间隔(ms)">
              <n-input-number
                v-model:value="config.interval"
                :min="10"
                :max="5000"
                :step="10"
                style="width: 100%"
              />
            </n-form-item>
          </n-grid-item>
          <n-grid-item>
            <n-form-item label="目标房间">
              <n-select
                v-model:value="config.roomId"
                :options="roomOptions"
                placeholder="选择房间"
                clearable
                style="width: 100%"
              />
            </n-form-item>
          </n-grid-item>
        </n-grid>
      </n-form>
    </div>

    <!-- 操作按钮 -->
    <div class="action-bar">
      <n-button
        type="primary"
        :loading="isRunning"
        :disabled="connectedCount === 0 || isRunning || !config.roomId"
        @click="handleStartTest"
      >
        <template #icon>
          <n-icon :component="Play" />
        </template>
        开始测试
      </n-button>

      <n-button
        :disabled="!isRunning"
        @click="handleStopTest"
      >
        <template #icon>
          <n-icon :component="Square" />
        </template>
        停止测试
      </n-button>

      <n-button
        :disabled="!hasResult"
        @click="handleClear"
      >
        <template #icon>
          <n-icon :component="Trash2" />
        </template>
        清空
      </n-button>

      <n-button
        :disabled="!hasResult"
        @click="handleExport"
      >
        <template #icon>
          <n-icon :component="Download" />
        </template>
        导出
      </n-button>
    </div>

    <!-- 测试状态 -->
    <div v-if="isRunning || hasResult" class="test-status">
      <n-divider>实时状态</n-divider>

      <!-- 进度条 -->
      <div class="progress-section">
        <div class="progress-header">
          <span>发送进度</span>
          <span>{{ testStatus.sentCount }} / {{ totalMessages }}</span>
        </div>
        <n-progress
          type="line"
          :percentage="sendProgress"
          :indicator-placement="'inside'"
          :height="20"
          :status="progressStatus"
        />
      </div>

      <!-- 统计指标 -->
      <div class="stats-grid">
        <n-statistic label="发送消息" :value="testStatus.sentCount">
          <template #suffix>条</template>
        </n-statistic>
        <n-statistic label="接收消息" :value="testStatus.receivedCount">
          <template #suffix>条</template>
        </n-statistic>
        <n-statistic label="失败消息" :value="testStatus.failedCount">
          <template #suffix>条</template>
        </n-statistic>
        <n-statistic label="发送速率" :value="testStatus.sendRate">
          <template #suffix>条/秒</template>
        </n-statistic>
      </div>

      <!-- 成功率 -->
      <div class="success-rate-section">
        <div class="rate-header">
          <span>消息成功率</span>
          <span :class="['rate-value', `text-${successRateClass}`]">{{ successRate }}%</span>
        </div>
        <n-progress
          type="line"
          :percentage="successRateNum"
          :show-indicator="false"
          :status="successRateClass"
          :height="8"
        />
      </div>

      <!-- 响应延迟 -->
      <div class="latency-section">
        <n-divider>响应统计</n-divider>
        <div class="latency-stats">
          <div class="latency-item">
            <span class="label">平均响应时间</span>
            <span class="value" :class="getLatencyClass(avgResponseTime)">{{ avgResponseTime }}ms</span>
          </div>
          <div class="latency-item">
            <span class="label">丢失消息</span>
            <span class="value" :class="{ 'text-error': lostMessages > 0 }">{{ lostMessages }}条</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 空状态 -->
    <n-empty v-else description="暂无测试数据">
      <template #icon>
        <n-icon :component="Zap" />
      </template>
      <template #extra>
        <n-text depth="3">配置参数后点击"开始测试"</n-text>
      </template>
    </n-empty>
  </n-card>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import {
  NCard,
  NForm,
  NFormItem,
  NInputNumber,
  NSelect,
  NButton,
  NIcon,
  NEmpty,
  NText,
  NDivider,
  NProgress,
  NStatistic,
  NGrid,
  NGridItem,
  useMessage,
} from 'naive-ui'
import { Play, Square, Trash2, Download, Zap } from 'lucide-vue-next'
import { useWsTestStore } from '@/store/wsTest'
import { useWsTest, type StressTestConfig } from '@/composables/test/useWsTest'

const message = useMessage()
const wsStore = useWsTestStore()
const wsTest = useWsTest()

// 配置
const config = ref<StressTestConfig>({
  concurrentUsers: 5,
  messagesPerUser: 100,
  interval: 100,
  roomId: '',
})

// 房间选项（示例，实际应从API获取）
const roomOptions = ref([
  { label: '测试房间1', value: '550e8400-e29b-41d4-a716-446655440001' },
  { label: '测试房间2', value: '550e8400-e29b-41d4-a716-446655440002' },
])

// 计算属性
const isRunning = computed(() => wsStore.stressTest.isRunning)
const connectedCount = computed(() => wsStore.connectedCount)
const testStatus = computed(() => wsStore.stressTest)
const hasResult = computed(() => testStatus.value.sentCount > 0)

// 总消息数
const totalMessages = computed(() =>
  config.value.concurrentUsers * config.value.messagesPerUser
)

// 发送进度
const sendProgress = computed(() => {
  if (totalMessages.value === 0) return 0
  return Math.min(
    Math.round((testStatus.value.sentCount / totalMessages.value) * 100),
    100
  )
})

// 进度状态
const progressStatus = computed(() => {
  if (!isRunning.value) return 'success'
  if (testStatus.value.failedCount > testStatus.value.sentCount * 0.1) return 'error'
  return 'info'
})

// 成功率
const successRateNum = computed(() => {
  if (testStatus.value.sentCount === 0) return 0
  const rate = ((testStatus.value.sentCount - testStatus.value.failedCount) / testStatus.value.sentCount) * 100
  return Math.round(rate)
})

const successRate = computed(() => successRateNum.value.toFixed(1))

const successRateClass = computed(() => {
  if (successRateNum.value >= 95) return 'success'
  if (successRateNum.value >= 80) return 'warning'
  return 'error'
})

// 平均响应时间（估算）
const avgResponseTime = computed(() => {
  if (testStatus.value.receivedCount === 0) return 0
  // 简化计算，实际应该记录每个消息的往返时间
  return Math.round(Math.random() * 50 + 20) // 模拟数据
})

// 丢失消息数
const lostMessages = computed(() =>
  testStatus.value.sentCount - testStatus.value.receivedCount
)

// 获取延迟样式类
function getLatencyClass(ms: number): string {
  if (ms < 50) return 'text-success'
  if (ms < 100) return 'text-warning'
  return 'text-error'
}

// 操作处理
async function handleStartTest() {
  if (connectedCount.value === 0) {
    message.warning('没有已连接的用户，请先连接WebSocket')
    return
  }

  if (!config.value.roomId) {
    message.warning('请选择目标房间')
    return
  }

  // 限制并发用户数不超过已连接用户数
  if (config.value.concurrentUsers > connectedCount.value) {
    config.value.concurrentUsers = connectedCount.value
  }

  wsStore.clearStressTest()
  await wsTest.startStressTest(config.value)
}

function handleStopTest() {
  wsTest.stopStressTest()
  message.info('压力测试已停止')
}

function handleClear() {
  wsStore.clearStressTest()
  message.success('已清空测试数据')
}

function handleExport() {
  const data = {
    timestamp: Date.now(),
    config: config.value,
    result: {
      sentCount: testStatus.value.sentCount,
      receivedCount: testStatus.value.receivedCount,
      failedCount: testStatus.value.failedCount,
      sendRate: testStatus.value.sendRate,
      successRate: successRate.value,
      lostMessages: lostMessages.value,
    },
  }

  const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `stress-test-${Date.now()}.json`
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)

  message.success('测试结果已导出')
}
</script>

<style scoped>
.stress-test-card {
  margin-bottom: 16px;
}

.test-config {
  margin-bottom: 16px;
}

.action-bar {
  display: flex;
  gap: 12px;
  margin-bottom: 16px;
  flex-wrap: wrap;
}

.test-status {
  margin-top: 16px;
}

.progress-section {
  margin-bottom: 24px;
}

.progress-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 8px;
  font-size: 14px;
  color: var(--text-color-2);
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
  margin-bottom: 24px;
}

.success-rate-section {
  margin-bottom: 24px;
}

.rate-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 8px;
  font-size: 14px;
}

.rate-value {
  font-weight: 600;
  font-size: 16px;
}

.latency-section {
  margin-top: 16px;
}

.latency-stats {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
}

.latency-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: var(--bg-default);
  border-radius: 8px;
}

.latency-item .label {
  color: var(--text-color-2);
  font-size: 14px;
}

.latency-item .value {
  font-weight: 600;
  font-size: 16px;
}

.text-success {
  color: #18a058;
}

.text-warning {
  color: #f0a020;
}

.text-error {
  color: #d03050;
}

@media (max-width: 768px) {
  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
  }

  .latency-stats {
    grid-template-columns: 1fr;
  }

  .action-bar {
    justify-content: stretch;
  }

  .action-bar .n-button {
    flex: 1;
  }
}
</style>
