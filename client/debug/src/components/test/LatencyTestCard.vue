<template>
  <n-card title="延迟测试" class="latency-test-card">
    <!-- 测试配置 -->
    <div class="test-config">
      <n-form inline :show-label="false">
        <n-form-item>
          <n-input-number
            v-model:value="config.count"
            :min="1"
            :max="100"
            placeholder="测试次数"
          >
            <template #prefix>次数</template>
          </n-input-number>
        </n-form-item>
        <n-form-item>
          <n-input-number
            v-model:value="config.interval"
            :min="100"
            :max="5000"
            :step="100"
            placeholder="间隔(ms)"
          >
            <template #prefix>间隔</template>
            <template #suffix>ms</template>
          </n-input-number>
        </n-form-item>
        <n-form-item>
          <n-checkbox v-model:checked="config.batchMode">
            批量测试所有用户
          </n-checkbox>
        </n-form-item>
      </n-form>
    </div>

    <!-- 操作按钮 -->
    <div class="action-bar">
      <n-button
        type="primary"
        :loading="loading"
        :disabled="connectedCount === 0"
        @click="handleStartTest"
      >
        <template #icon>
          <n-icon :component="Play" />
        </template>
        开始测试
      </n-button>

      <n-button
        :disabled="!loading"
        @click="handleStopTest"
      >
        <template #icon>
          <n-icon :component="Square" />
        </template>
        停止
      </n-button>

      <n-button
        :disabled="results.length === 0"
        @click="handleClear"
      >
        <template #icon>
          <n-icon :component="Trash2" />
        </template>
        清空结果
      </n-button>

      <n-button
        :disabled="results.length === 0"
        @click="handleExport"
      >
        <template #icon>
          <n-icon :component="Download" />
        </template>
        导出
      </n-button>
    </div>

    <!-- 统计结果 -->
    <div v-if="stats.count > 0" class="stats-section">
      <n-divider>测试结果</n-divider>
      <div class="stats-grid">
        <n-statistic label="测试次数" :value="stats.count" />
        <n-statistic label="最小延迟" :value="stats.min">
          <template #suffix>ms</template>
        </n-statistic>
        <n-statistic label="最大延迟" :value="stats.max">
          <template #suffix>ms</template>
        </n-statistic>
        <n-statistic label="平均延迟" :value="stats.avg">
          <template #suffix>ms</template>
        </n-statistic>
        <n-statistic label="P99延迟" :value="stats.p99">
          <template #suffix>ms</template>
        </n-statistic>
      </div>

      <!-- 延迟分布图表 -->
      <div class="latency-chart">
        <n-divider>延迟分布</n-divider>
        <div class="chart-container">
          <div
            v-for="(bar, index) in latencyDistribution"
            :key="index"
            class="chart-bar"
            :style="{ height: `${bar.percentage}%` }"
            :title="`${bar.range}: ${bar.count}次`"
          >
            <span v-if="bar.percentage > 10" class="bar-label">{{ bar.count }}</span>
          </div>
        </div>
        <div class="chart-labels">
          <span v-for="(bar, index) in latencyDistribution" :key="index" class="label">
            {{ bar.range }}
          </span>
        </div>
      </div>
    </div>

    <!-- 空状态 -->
    <n-empty v-else description="暂无测试数据">
      <template #icon>
        <n-icon :component="Activity" />
      </template>
      <template #extra>
        <n-text depth="3">点击"开始测试"测量WebSocket延迟</n-text>
      </template>
    </n-empty>

    <!-- 进度显示 -->
    <div v-if="loading && operationProgress" class="progress-section">
      <n-progress
        type="line"
        :percentage="Math.round((operationProgress.current / operationProgress.total) * 100)"
        :indicator-placement="'inside'"
        :height="20"
      >
        {{ operationProgress.current }} / {{ operationProgress.total }}
      </n-progress>
      <n-text depth="3" class="operation-text">{{ currentOperation }}</n-text>
    </div>
  </n-card>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
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
  NStatistic,
  NDivider,
  NProgress,
  useMessage,
} from 'naive-ui'
import { Play, Square, Trash2, Download, Activity } from 'lucide-vue-next'
import { useWsTestStore } from '@/store/wsTest'
import { useWsTest, type LatencyTestConfig } from '@/composables/test/useWsTest'

const message = useMessage()
const wsStore = useWsTestStore()
const wsTest = useWsTest()

// 配置
const config = ref<LatencyTestConfig>({
  count: 10,
  interval: 500,
  batchMode: true,
})

// 计算属性
const loading = computed(() => wsTest.loading.value)
const currentOperation = computed(() => wsTest.currentOperation.value)
const operationProgress = computed(() => wsTest.operationProgress.value)
const connectedCount = computed(() => wsStore.connectedCount)
const results = computed(() => wsStore.latencyResults)
const stats = computed(() => wsStore.latencyStats)

// 延迟分布数据
const latencyDistribution = computed(() => {
  if (results.value.length === 0) return []

  const ranges = [
    { min: 0, max: 50, label: '<50ms' },
    { min: 50, max: 100, label: '50-100ms' },
    { min: 100, max: 200, label: '100-200ms' },
    { min: 200, max: 500, label: '200-500ms' },
    { min: 500, max: Infinity, label: '>500ms' },
  ]

  const distribution = ranges.map(range => ({
    range: range.label,
    count: results.value.filter(r => r.latency >= range.min && r.latency < range.max).length,
  }))

  const maxCount = Math.max(...distribution.map(d => d.count), 1)

  return distribution.map(d => ({
    ...d,
    percentage: (d.count / maxCount) * 100,
  }))
})

// 操作处理
async function handleStartTest() {
  if (connectedCount.value === 0) {
    message.warning('没有已连接的用户，请先连接WebSocket')
    return
  }

  await wsTest.startLatencyTest(config.value)
}

function handleStopTest() {
  wsTest.stopLatencyTest()
  message.info('测试已停止')
}

function handleClear() {
  wsStore.clearLatencyResults()
  message.success('已清空测试结果')
}

function handleExport() {
  const data = {
    timestamp: Date.now(),
    config: config.value,
    stats: stats.value,
    results: results.value,
  }

  const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `latency-test-${Date.now()}.json`
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)

  message.success('测试结果已导出')
}
</script>

<style scoped>
.latency-test-card {
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

.stats-section {
  margin-top: 16px;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 16px;
  margin-bottom: 24px;
}

.latency-chart {
  margin-top: 24px;
}

.chart-container {
  display: flex;
  align-items: flex-end;
  justify-content: space-around;
  height: 150px;
  padding: 16px;
  background: var(--bg-default);
  border-radius: 8px;
  gap: 8px;
}

.chart-bar {
  flex: 1;
  min-width: 40px;
  background: linear-gradient(to top, #2080f0, #4098f7);
  border-radius: 4px 4px 0 0;
  transition: all 0.3s ease;
  position: relative;
  display: flex;
  align-items: flex-end;
  justify-content: center;
  padding-bottom: 4px;
}

.chart-bar:hover {
  background: linear-gradient(to top, #1060d0, #2080f0);
}

.bar-label {
  color: white;
  font-size: 12px;
  font-weight: 500;
}

.chart-labels {
  display: flex;
  justify-content: space-around;
  margin-top: 8px;
  padding: 0 16px;
}

.chart-labels .label {
  flex: 1;
  text-align: center;
  font-size: 12px;
  color: var(--text-color-3);
  min-width: 40px;
}

.progress-section {
  margin-top: 16px;
  padding-top: 16px;
  border-top: 1px solid var(--divider-color);
}

.operation-text {
  display: block;
  text-align: center;
  margin-top: 8px;
  font-size: 12px;
}

@media (max-width: 768px) {
  .stats-grid {
    grid-template-columns: repeat(3, 1fr);
  }

  .action-bar {
    justify-content: stretch;
  }

  .action-bar .n-button {
    flex: 1;
  }
}

@media (max-width: 480px) {
  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}
</style>
