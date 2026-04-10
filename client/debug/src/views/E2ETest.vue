<script setup lang="ts">
import { ref, computed, onUnmounted } from 'vue'
import { useMessage } from 'naive-ui'
import {
  Play,
  Square,
  CheckCircle,
  XCircle,
  Clock,
  Users,
  FileText,
  Download,
  Trash2,
} from 'lucide-vue-next'
import { ScenarioRunner, predefinedScenarios, type ScenarioResult } from '@/utils/scenarioRunner'
import { ConcurrentTester, type ConcurrentTestResult } from '@/utils/concurrentTester'
import { generateTestDataSet, type TestDataSet } from '@/utils/testDataGenerator'

const message = useMessage()

// ========== 状态 ==========
const activeTab = ref<'scenarios' | 'concurrent' | 'report'>('scenarios')
const isRunning = ref(false)
const currentScenario = ref<string>('')
const progress = ref(0)
const logs = ref<Array<{ time: string; level: 'info' | 'success' | 'error' | 'warning'; message: string }>>([])

// 场景测试结果
const scenarioResults = ref<ScenarioResult[]>([])
const currentScenarioResult = ref<ScenarioResult | null>(null)

// 并发测试结果
const concurrentResults = ref<ConcurrentTestResult[]>([])
const currentConcurrentResult = ref<ConcurrentTestResult | null>(null)

// 测试数据
const testData = ref<TestDataSet | null>(null)

// 并发测试配置
const concurrentConfig = ref({
  userCount: 10,
  roomsPerUser: 2,
  messagesPerUser: 50,
  concurrency: 5,
  rampUpTime: 5000,
  duration: 30000,
})

// 运行器实例
let scenarioRunner: ScenarioRunner | null = null
let concurrentTester: ConcurrentTester | null = null

// ========== 计算属性 ==========
const canStart = computed(() => !isRunning.value)
const canStop = computed(() => isRunning.value)

const totalTests = computed(() => scenarioResults.value.length + concurrentResults.value.length)
const passedTests = computed(() =>
  scenarioResults.value.filter(r => r.success).length +
  concurrentResults.value.filter(r => r.errors.length === 0).length
)
const failedTests = computed(() => totalTests.value - passedTests.value)

// ========== 日志函数 ==========
const addLog = (level: 'info' | 'success' | 'error' | 'warning', msg: string) => {
  const time = new Date().toLocaleTimeString()
  logs.value.unshift({ time, level, message: msg })
  if (logs.value.length > 100) {
    logs.value.pop()
  }
}

const clearLogs = () => {
  logs.value = []
}

// ========== 场景测试 ==========
const runScenario = async (scenarioId: string) => {
  if (isRunning.value) return

  const scenario = predefinedScenarios.find(s => s.id === scenarioId)
  if (!scenario) {
    message.error('场景不存在')
    return
  }

  isRunning.value = true
  currentScenario.value = scenarioId
  progress.value = 0
  clearLogs()

  addLog('info', `开始执行场景: ${scenario.name}`)

  try {
    scenarioRunner = new ScenarioRunner()

    // 监听进度
    const totalSteps = scenario.steps.length
    let completedSteps = 0

    const result = await scenarioRunner.runScenario({
      ...scenario,
      setup: async () => {
        addLog('info', '执行场景初始化...')
      },
      teardown: async () => {
        addLog('info', '执行场景清理...')
      },
    })

    // 更新进度和日志
    result.stepResults.forEach((stepResult, index) => {
      completedSteps++
      progress.value = Math.round((completedSteps / totalSteps) * 100)

      const step = scenario.steps[index]
      if (stepResult.success) {
        addLog('success', `✓ ${step.name} (${stepResult.duration}ms)`)
      } else {
        addLog('error', `✗ ${step.name}: ${stepResult.error}`)
      }
    })

    currentScenarioResult.value = result
    scenarioResults.value.unshift(result)

    if (result.success) {
      addLog('success', `场景执行完成: ${result.summary.passed}/${result.summary.total} 步骤通过`)
      message.success('场景测试通过')
    } else {
      addLog('error', `场景执行失败: ${result.summary.failed} 个步骤失败`)
      message.error('场景测试失败')
    }
  } catch (error) {
    addLog('error', `场景执行异常: ${error instanceof Error ? error.message : String(error)}`)
    message.error('场景执行异常')
  } finally {
    isRunning.value = false
    scenarioRunner = null
  }
}

const stopScenario = () => {
  if (scenarioRunner) {
    scenarioRunner.stop()
    addLog('warning', '场景测试已停止')
  }
  isRunning.value = false
}

// ========== 并发测试 ==========
const runConcurrentTest = async () => {
  if (isRunning.value) return

  isRunning.value = true
  progress.value = 0
  clearLogs()

  addLog('info', '开始并发测试...')
  addLog('info', `配置: ${concurrentConfig.value.userCount} 用户, ${concurrentConfig.value.duration / 1000} 秒`)

  try {
    // 生成测试数据
    testData.value = generateTestDataSet(
      { prefix: 'concurrent', count: concurrentConfig.value.userCount, password: 'Test@123456' },
      { prefix: 'concurrent', count: 5, isPrivate: false },
      { count: 50, contentPool: [] }
    )

    concurrentTester = new ConcurrentTester()

    // 模拟进度更新
    const progressInterval = setInterval(() => {
      if (progress.value < 90) {
        progress.value += Math.random() * 5
      }
    }, 1000)

    const result = await concurrentTester.runConcurrentTest(concurrentConfig.value, testData.value)

    clearInterval(progressInterval)
    progress.value = 100

    currentConcurrentResult.value = result
    concurrentResults.value.unshift(result)

    addLog('success', `并发测试完成`)
    addLog('info', `总消息数: ${result.totalMessages}`)
    addLog('info', `消息速率: ${result.messagesPerSecond.toFixed(2)} msg/s`)
    addLog('info', `平均延迟: ${result.averageLatency.toFixed(2)}ms`)
    addLog('info', `错误数: ${result.errors.length}`)

    if (result.errors.length === 0) {
      message.success('并发测试通过')
    } else {
      message.warning(`并发测试完成，但有 ${result.errors.length} 个错误`)
    }
  } catch (error) {
    addLog('error', `并发测试异常: ${error instanceof Error ? error.message : String(error)}`)
    message.error('并发测试异常')
  } finally {
    isRunning.value = false
    concurrentTester = null
  }
}

const stopConcurrentTest = () => {
  if (concurrentTester) {
    concurrentTester.stop()
    addLog('warning', '并发测试已停止')
  }
  isRunning.value = false
}

// ========== 报告导出 ==========
const exportReport = () => {
  const report = {
    generatedAt: new Date().toISOString(),
    summary: {
      totalTests: totalTests.value,
      passed: passedTests.value,
      failed: failedTests.value,
      passRate: totalTests.value > 0 ? ((passedTests.value / totalTests.value) * 100).toFixed(2) + '%' : 'N/A',
    },
    scenarioResults: scenarioResults.value,
    concurrentResults: concurrentResults.value.map(r => ({
      startTime: r.startTime,
      endTime: r.endTime,
      duration: r.duration,
      totalUsers: r.totalUsers,
      activeUsers: r.activeUsers,
      totalMessages: r.totalMessages,
      messagesPerSecond: r.messagesPerSecond,
      averageLatency: r.averageLatency,
      errorCount: r.errors.length,
    })),
  }

  const blob = new Blob([JSON.stringify(report, null, 2)], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `e2e-test-report-${Date.now()}.json`
  a.click()
  URL.revokeObjectURL(url)

  message.success('报告已导出')
}

// ========== 清理 ==========
const clearResults = () => {
  scenarioResults.value = []
  concurrentResults.value = []
  currentScenarioResult.value = null
  currentConcurrentResult.value = null
  clearLogs()
  message.success('结果已清空')
}

onUnmounted(() => {
  stopScenario()
  stopConcurrentTest()
})
</script>

<template>
  <div class="page-container">
    <div class="page-header">
      <h1 class="page-title">
        <CheckCircle class="icon-lg" style="display: inline; vertical-align: middle; margin-right: 8px" />
        端到端测试
      </h1>
      <p class="page-subtitle">执行端到端场景测试和并发测试</p>
    </div>

    <!-- 统计卡片 -->
    <div class="card-grid" style="margin-bottom: var(--space-lg)">
      <n-card class="stat-card">
        <div class="stat-icon" style="background: var(--success)">
          <CheckCircle class="icon-lg" />
        </div>
        <div class="stat-content">
          <div class="stat-value">{{ passedTests }}</div>
          <div class="stat-label">通过</div>
        </div>
      </n-card>
      <n-card class="stat-card">
        <div class="stat-icon" style="background: var(--error)">
          <XCircle class="icon-lg" />
        </div>
        <div class="stat-content">
          <div class="stat-value">{{ failedTests }}</div>
          <div class="stat-label">失败</div>
        </div>
      </n-card>
      <n-card class="stat-card">
        <div class="stat-icon" style="background: var(--info)">
          <Clock class="icon-lg" />
        </div>
        <div class="stat-content">
          <div class="stat-value">{{ totalTests }}</div>
          <div class="stat-label">总计</div>
        </div>
      </n-card>
    </div>

    <!-- 标签页 -->
    <n-tabs v-model:value="activeTab" type="line" style="margin-bottom: var(--space-lg)">
      <n-tab-pane name="scenarios" tab="场景测试">
        <template #tab>
          <n-space align="center">
            <FileText class="icon-sm" />
            <span>场景测试</span>
          </n-space>
        </template>
      </n-tab-pane>
      <n-tab-pane name="concurrent" tab="并发测试">
        <template #tab>
          <n-space align="center">
            <Users class="icon-sm" />
            <span>并发测试</span>
          </n-space>
        </template>
      </n-tab-pane>
      <n-tab-pane name="report" tab="测试报告">
        <template #tab>
          <n-space align="center">
            <CheckCircle class="icon-sm" />
            <span>测试报告</span>
          </n-space>
        </template>
      </n-tab-pane>
    </n-tabs>

    <!-- 场景测试面板 -->
    <div v-if="activeTab === 'scenarios'" style="display: grid; grid-template-columns: 1fr 1fr; gap: var(--space-lg)">
      <!-- 左侧：场景列表 -->
      <n-card title="测试场景">
        <n-list>
          <n-list-item v-for="scenario in predefinedScenarios" :key="scenario.id">
            <n-thing :title="scenario.name" :description="scenario.description">
              <template #header-extra>
                <n-space>
                  <n-tag size="small">{{ scenario.steps.length }} 步骤</n-tag>
                  <n-button
                    size="small"
                    type="primary"
                    :disabled="!canStart"
                    :loading="isRunning && currentScenario === scenario.id"
                    @click="runScenario(scenario.id)"
                  >
                    <template #icon>
                      <Play class="icon-sm" />
                    </template>
                    执行
                  </n-button>
                </n-space>
              </template>
            </n-thing>
          </n-list-item>
        </n-list>
      </n-card>

      <!-- 右侧：执行状态 -->
      <div style="display: flex; flex-direction: column; gap: var(--space-lg)">
        <n-card title="执行状态">
          <n-space vertical>
            <n-progress :percentage="progress" :indicator-placement="'inside'" />
            <n-space justify="center">
              <n-button type="error" :disabled="!canStop" @click="stopScenario">
                <template #icon>
                  <Square class="icon-sm" />
                </template>
                停止
              </n-button>
              <n-button @click="clearResults">
                <template #icon>
                  <Trash2 class="icon-sm" />
                </template>
                清空
              </n-button>
            </n-space>
          </n-space>
        </n-card>

        <n-card title="执行日志" style="flex: 1">
          <template #header-extra>
            <n-button text size="small" @click="clearLogs">
              <template #icon>
                <Trash2 class="icon-sm" />
              </template>
              清空
            </n-button>
          </template>
          <div class="log-panel" style="max-height: 300px; overflow-y: auto">
            <div v-for="(log, index) in logs" :key="index" class="log-entry">
              <span class="log-time">[{{ log.time }}]</span>
              <span :class="`log-${log.level}`">[{{ log.level.toUpperCase() }}]</span>
              <span style="color: var(--text-white); margin-left: 8px">{{ log.message }}</span>
            </div>
            <div v-if="logs.length === 0" style="color: var(--text-muted); text-align: center; padding: var(--space-lg)">
              暂无日志
            </div>
          </div>
        </n-card>
      </div>
    </div>

    <!-- 并发测试面板 -->
    <div v-if="activeTab === 'concurrent'" style="display: grid; grid-template-columns: 1fr 1fr; gap: var(--space-lg)">
      <!-- 左侧：配置 -->
      <n-card title="并发测试配置">
        <n-form label-placement="left" label-width="120">
          <n-form-item label="虚拟用户数">
            <n-input-number v-model:value="concurrentConfig.userCount" :min="1" :max="1000" style="width: 100%" />
          </n-form-item>
          <n-form-item label="每用户房间数">
            <n-input-number v-model:value="concurrentConfig.roomsPerUser" :min="1" :max="10" style="width: 100%" />
          </n-form-item>
          <n-form-item label="并发数">
            <n-input-number v-model:value="concurrentConfig.concurrency" :min="1" :max="100" style="width: 100%" />
          </n-form-item>
          <n-form-item label="斜坡时间(ms)">
            <n-input-number v-model:value="concurrentConfig.rampUpTime" :min="1000" :step="1000" style="width: 100%" />
          </n-form-item>
          <n-form-item label="持续时间(ms)">
            <n-input-number v-model:value="concurrentConfig.duration" :min="5000" :step="5000" style="width: 100%" />
          </n-form-item>
        </n-form>
        <n-space justify="center" style="margin-top: var(--space-lg)">
          <n-button type="primary" :disabled="!canStart" :loading="isRunning" @click="runConcurrentTest">
            <template #icon>
              <Play class="icon-sm" />
            </template>
            开始测试
          </n-button>
          <n-button :disabled="!canStop" @click="stopConcurrentTest">
            <template #icon>
              <Square class="icon-sm" />
            </template>
            停止
          </n-button>
        </n-space>
      </n-card>

      <!-- 右侧：结果 -->
      <n-card title="测试结果">
        <div v-if="currentConcurrentResult" style="display: grid; grid-template-columns: 1fr 1fr; gap: var(--space-md)">
          <n-statistic label="总用户数" :value="currentConcurrentResult.totalUsers" />
          <n-statistic label="活跃用户数" :value="currentConcurrentResult.activeUsers" />
          <n-statistic label="总消息数" :value="currentConcurrentResult.totalMessages" />
          <n-statistic label="消息速率" :value="currentConcurrentResult.messagesPerSecond.toFixed(2)" suffix="msg/s" />
          <n-statistic label="平均延迟" :value="currentConcurrentResult.averageLatency.toFixed(2)" suffix="ms" />
          <n-statistic label="错误数" :value="currentConcurrentResult.errors.length" />
        </div>
        <div v-else style="color: var(--text-muted); text-align: center; padding: var(--space-xl)">
          暂无测试结果
        </div>
      </n-card>
    </div>

    <!-- 测试报告面板 -->
    <div v-if="activeTab === 'report'">
      <n-card title="测试报告">
        <template #header-extra>
          <n-space>
            <n-button @click="exportReport">
              <template #icon>
                <Download class="icon-sm" />
              </template>
              导出报告
            </n-button>
            <n-button @click="clearResults">
              <template #icon>
                <Trash2 class="icon-sm" />
              </template>
              清空
            </n-button>
          </n-space>
        </template>

        <!-- 场景测试结果 -->
        <n-divider title-placement="left">场景测试结果</n-divider>
        <n-list v-if="scenarioResults.length > 0">
          <n-list-item v-for="result in scenarioResults" :key="result.scenarioId">
            <n-thing>
              <template #header>
                <n-space align="center">
                  <n-icon :component="result.success ? CheckCircle : XCircle" :color="result.success ? '#18a058' : '#d03050'" />
                  <span>{{ result.scenarioId }}</span>
                  <n-tag :type="result.success ? 'success' : 'error'" size="small">
                    {{ result.summary.passed }}/{{ result.summary.total }}
                  </n-tag>
                </n-space>
              </template>
              <template #description>
                <n-space vertical size="small">
                  <span>耗时: {{ result.duration }}ms</span>
                  <span>时间: {{ result.startTime.toLocaleString() }}</span>
                </n-space>
              </template>
            </n-thing>
          </n-list-item>
        </n-list>
        <div v-else style="color: var(--text-muted); text-align: center; padding: var(--space-lg)">
          暂无场景测试结果
        </div>

        <!-- 并发测试结果 -->
        <n-divider title-placement="left">并发测试结果</n-divider>
        <n-list v-if="concurrentResults.length > 0">
          <n-list-item v-for="(result, index) in concurrentResults" :key="index">
            <n-thing>
              <template #header>
                <n-space align="center">
                  <n-icon :component="result.errors.length === 0 ? CheckCircle : XCircle" :color="result.errors.length === 0 ? '#18a058' : '#d03050'" />
                  <span>并发测试 #{{ index + 1 }}</span>
                  <n-tag :type="result.errors.length === 0 ? 'success' : 'warning'" size="small">
                    {{ result.errors.length }} 错误
                  </n-tag>
                </n-space>
              </template>
              <template #description>
                <n-space vertical size="small">
                  <span>用户数: {{ result.totalUsers }} | 消息数: {{ result.totalMessages }} | 速率: {{ result.messagesPerSecond.toFixed(2) }} msg/s</span>
                  <span>平均延迟: {{ result.averageLatency.toFixed(2) }}ms | 时间: {{ result.startTime.toLocaleString() }}</span>
                </n-space>
              </template>
            </n-thing>
          </n-list-item>
        </n-list>
        <div v-else style="color: var(--text-muted); text-align: center; padding: var(--space-lg)">
          暂无并发测试结果
        </div>
      </n-card>
    </div>
  </div>
</template>

<style scoped>
.stat-card {
  display: flex;
  align-items: center;
  gap: var(--space-md);
}

.stat-icon {
  width: 56px;
  height: 56px;
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-white);
}

.stat-content {
  flex: 1;
}

.stat-value {
  font-size: 28px;
  font-weight: 700;
  color: var(--text-primary);
  line-height: 1;
}

.stat-label {
  font-size: 14px;
  color: var(--text-secondary);
  margin-top: 4px;
}

.log-panel {
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  line-height: 1.6;
}

.log-entry {
  padding: 4px 0;
  border-bottom: 1px solid var(--border-color);
}

.log-time {
  color: var(--text-muted);
  margin-right: 8px;
}

.log-info {
  color: var(--info);
}

.log-success {
  color: var(--success);
}

.log-error {
  color: var(--error);
}

.log-warning {
  color: var(--warning);
}
</style>
