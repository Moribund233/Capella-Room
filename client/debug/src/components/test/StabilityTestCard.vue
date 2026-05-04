<template>
  <n-card title="稳定性测试" class="stability-test-card">
    <!-- 测试配置 -->
    <div class="test-config">
      <n-form inline :show-label="false">
        <n-form-item>
          <n-input-number
            v-model:value="config.duration"
            :min="1"
            :max="60"
            placeholder="测试时长"
          >
            <template #prefix>时长</template>
            <template #suffix>分钟</template>
          </n-input-number>
        </n-form-item>
        <n-form-item>
          <n-checkbox v-model:checked="config.autoReconnect">
            自动重连
          </n-checkbox>
        </n-form-item>
      </n-form>
    </div>

    <!-- 操作按钮 -->
    <div class="action-bar">
      <n-button
        type="primary"
        :loading="isRunning"
        :disabled="connectedCount === 0 || isRunning"
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
        @click="handleExport"
      >
        <template #icon>
          <n-icon :component="Download" />
        </template>
        导出报告
      </n-button>
    </div>

    <!-- 测试状态 -->
    <div v-if="isRunning || hasResult" class="test-status">
      <n-divider>测试状态</n-divider>

      <!-- 进度条 -->
      <div class="progress-section">
        <div class="progress-header">
          <span>测试进度</span>
          <span>{{ elapsedTime }} / {{ formatDuration(testStatus.duration) }}</span>
        </div>
        <n-progress
          type="line"
          :percentage="progressPercentage"
          :indicator-placement="'inside'"
          :height="20"
          :status="progressStatus"
        />
      </div>

      <!-- 统计指标 -->
      <div class="stats-grid">
        <n-statistic label="断连次数" :value="testStatus.disconnectCount">
          <template #suffix>次</template>
        </n-statistic>
        <n-statistic label="重连次数" :value="testStatus.reconnectCount">
          <template #suffix>次</template>
        </n-statistic>
        <n-statistic label="错误次数" :value="testStatus.errors.length">
          <template #suffix>次</template>
        </n-statistic>
        <n-statistic label="稳定性评分" :value="stabilityScore">
          <template #suffix>分</template>
        </n-statistic>
      </div>

      <!-- 错误列表 -->
      <div v-if="testStatus.errors.length > 0" class="error-list">
        <n-divider>错误记录</n-divider>
        <n-list size="small">
          <n-list-item v-for="(error, index) in recentErrors" :key="index">
            <n-thing :title="formatTime(error.time)" :description="error.message" />
          </n-list-item>
        </n-list>
        <n-text v-if="testStatus.errors.length > 5" depth="3" class="more-errors">
          还有 {{ testStatus.errors.length - 5 }} 条错误...
        </n-text>
      </div>

      <!-- 测试结果总结 -->
      <div v-if="!isRunning && hasResult" class="result-summary">
        <n-divider>测试总结</n-divider>
        <n-alert :type="resultAlertType" :title="resultTitle">
          {{ resultDescription }}
        </n-alert>
      </div>
    </div>

    <!-- 空状态 -->
    <n-empty v-else description="暂无测试数据">
      <template #icon>
        <n-icon :component="Shield" />
      </template>
      <template #extra>
        <n-text depth="3">点击"开始测试"进行长时间稳定性测试</n-text>
      </template>
    </n-empty>
  </n-card>
</template>

<script setup lang="ts">
import { ref, computed, onUnmounted } from 'vue'
import {
  NCard,
  NForm,
  NFormItem,
  NInputNumber,
  NCheckbox,
  NButton,
  NIcon,
  NEmpty,
  NText,
  NDivider,
  NProgress,
  NStatistic,
  NList,
  NListItem,
  NThing,
  NAlert,
  useMessage,
} from 'naive-ui'
import { Play, Square, Download, Shield } from 'lucide-vue-next'
import { useWsTestStore, type StabilityTest } from '@/store/wsTest'
import { useWsTest, type StabilityTestConfig } from '@/composables/test/useWsTest'

const message = useMessage()
const wsStore = useWsTestStore()
const wsTest = useWsTest()

// 配置
const config = ref<StabilityTestConfig>({
  duration: 5,
  autoReconnect: true,
})

// 计时器
const timer = ref<number | null>(null)
const elapsedMs = ref(0)

// 计算属性
const isRunning = computed(() => wsStore.stabilityTest.isRunning)
const connectedCount = computed(() => wsStore.connectedCount)
const testStatus = computed<StabilityTest>(() => wsStore.stabilityTest)
const hasResult = computed(() =>
  !isRunning.value && (
    testStatus.value.disconnectCount > 0 ||
    testStatus.value.reconnectCount > 0 ||
    testStatus.value.errors.length > 0
  )
)

// 格式化时长
function formatDuration(ms: number): string {
  const minutes = Math.floor(ms / 60000)
  const seconds = Math.floor((ms % 60000) / 1000)
  return `${minutes}:${seconds.toString().padStart(2, '0')}`
}

// 已运行时间
const elapsedTime = computed(() => {
  if (!testStatus.value.startTime) return '0:00'
  const elapsed = isRunning.value
    ? Date.now() - testStatus.value.startTime
    : elapsedMs.value
  return formatDuration(elapsed)
})

// 进度百分比
const progressPercentage = computed(() => {
  if (!testStatus.value.startTime) return 0
  const elapsed = Date.now() - testStatus.value.startTime
  const percentage = Math.min(Math.round((elapsed / testStatus.value.duration) * 100), 100)
  return percentage
})

// 进度状态
const progressStatus = computed(() => {
  if (!isRunning.value) return 'success'
  if (testStatus.value.errors.length > 5) return 'error'
  if (testStatus.value.disconnectCount > 3) return 'warning'
  return 'info'
})

// 稳定性评分
const stabilityScore = computed(() => {
  const { disconnectCount, reconnectCount, errors } = testStatus.value
  let score = 100

  // 断连扣分
  score -= disconnectCount * 5
  // 重连扣分（比断连轻）
  score -= reconnectCount * 2
  // 错误扣分
  score -= errors.length * 3

  return Math.max(0, score)
})

// 最近的错误
const recentErrors = computed(() => {
  return testStatus.value.errors.slice(-5).reverse()
})

// 结果提示类型
const resultAlertType = computed(() => {
  if (stabilityScore.value >= 90) return 'success'
  if (stabilityScore.value >= 70) return 'warning'
  return 'error'
})

// 结果标题
const resultTitle = computed(() => {
  if (stabilityScore.value >= 90) return '测试通过'
  if (stabilityScore.value >= 70) return '测试通过（有警告）'
  return '测试未通过'
})

// 结果描述
const resultDescription = computed(() => {
  const { disconnectCount, reconnectCount, errors } = testStatus.value
  const parts: string[] = []

  if (disconnectCount === 0) {
    parts.push('连接非常稳定，没有发生断连')
  } else {
    parts.push(`发生 ${disconnectCount} 次断连`)
  }

  if (reconnectCount > 0) {
    parts.push(`自动重连 ${reconnectCount} 次`)
  }

  if (errors.length === 0) {
    parts.push('没有错误发生')
  } else {
    parts.push(`发生 ${errors.length} 个错误`)
  }

  parts.push(`稳定性评分: ${stabilityScore.value}分`)

  return parts.join('，') + '。'
})

// 格式化时间
function formatTime(timestamp: number): string {
  return new Date(timestamp).toLocaleTimeString()
}

// 操作处理
function handleStartTest() {
  if (connectedCount.value === 0) {
    message.warning('没有已连接的用户，请先连接WebSocket')
    return
  }

  elapsedMs.value = 0
  wsTest.startStabilityTest(config.value)

  // 启动计时器
  timer.value = window.setInterval(() => {
    if (testStatus.value.startTime) {
      elapsedMs.value = Date.now() - testStatus.value.startTime
    }
  }, 1000)
}

function handleStopTest() {
  if (timer.value) {
    clearInterval(timer.value)
    timer.value = null
  }

  // 保存已运行时间
  if (testStatus.value.startTime) {
    elapsedMs.value = Date.now() - testStatus.value.startTime
  }

  wsTest.stopStabilityTest()
  message.success('稳定性测试已停止')
}

function handleExport() {
  const data = {
    timestamp: Date.now(),
    config: config.value,
    result: {
      duration: elapsedMs.value,
      disconnectCount: testStatus.value.disconnectCount,
      reconnectCount: testStatus.value.reconnectCount,
      errors: testStatus.value.errors,
      stabilityScore: stabilityScore.value,
    },
  }

  const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `stability-test-${Date.now()}.json`
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)

  message.success('测试报告已导出')
}

// 清理
onUnmounted(() => {
  if (timer.value) {
    clearInterval(timer.value)
  }
})
</script>

<style scoped>
.stability-test-card {
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

.error-list {
  margin-top: 16px;
}

.more-errors {
  display: block;
  text-align: center;
  margin-top: 8px;
  font-size: 12px;
}

.result-summary {
  margin-top: 24px;
}

@media (max-width: 768px) {
  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
  }

  .action-bar {
    justify-content: stretch;
  }

  .action-bar .n-button {
    flex: 1;
  }
}
</style>
