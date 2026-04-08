<script setup lang="ts">
/**
 * 系统监控页面
 * 实时监控系统运行状态和性能指标
 */

import { ref, onMounted, onUnmounted } from 'vue'
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { LineChart, BarChart, PieChart } from 'echarts/charts'
import {
  GridComponent,
  TooltipComponent,
  LegendComponent,
  TitleComponent,
  ToolboxComponent,
  DataZoomComponent,
} from 'echarts/components'
import VChart from 'vue-echarts'
import { getSystemStats, getActivityStats } from '@/api/monitor'
import type { SystemStats, ActivityStats } from '@/api/monitor'
import { Refresh, User, HomeFilled, ChatDotRound, CircleCheck, Connection, Clock, Warning } from '@element-plus/icons-vue'

// 注册 ECharts 组件
use([
  CanvasRenderer,
  LineChart,
  BarChart,
  PieChart,
  GridComponent,
  TooltipComponent,
  LegendComponent,
  TitleComponent,
  ToolboxComponent,
  DataZoomComponent,
])

// ==================== 响应式数据 ====================

const loading = ref(false)
const error = ref<string | null>(null)

// 系统统计数据
const systemStats = ref<SystemStats>({
  total_users: 0,
  total_rooms: 0,
  total_messages: 0,
  online_users: 0,
  active_connections: 0,
})

// 活跃度统计数据
const activityStats = ref<ActivityStats>({
  daily_active_users: 0,
  weekly_active_users: 0,
  monthly_active_users: 0,
  daily_messages: 0,
  weekly_messages: 0,
  monthly_messages: 0,
})

// 历史数据（模拟数据，实际应从后端获取）
const historyData = ref<Array<{ date: string; active_users: number; messages: number }>>([])

// 自动刷新定时器
let refreshTimer: ReturnType<typeof setInterval> | null = null

// ==================== 图表配置 ====================

// 用户活跃度趋势图配置
const activityTrendOption = ref({
  title: {
    text: '用户活跃度趋势',
    left: 'center',
    textStyle: {
      fontSize: 16,
      fontWeight: 'normal',
    },
  },
  tooltip: {
    trigger: 'axis',
    axisPointer: {
      type: 'cross',
    },
  },
  legend: {
    data: ['活跃用户', '消息数'],
    bottom: 0,
  },
  grid: {
    left: '3%',
    right: '4%',
    bottom: '15%',
    containLabel: true,
  },
  xAxis: {
    type: 'category',
    boundaryGap: false,
    data: [] as string[],
  },
  yAxis: [
    {
      type: 'value',
      name: '活跃用户',
      position: 'left',
    },
    {
      type: 'value',
      name: '消息数',
      position: 'right',
    },
  ],
  series: [
    {
      name: '活跃用户',
      type: 'line',
      smooth: true,
      data: [] as number[],
      itemStyle: { color: '#3b82f6' },
      areaStyle: {
        color: {
          type: 'linear',
          x: 0,
          y: 0,
          x2: 0,
          y2: 1,
          colorStops: [
            { offset: 0, color: 'rgba(59, 130, 246, 0.3)' },
            { offset: 1, color: 'rgba(59, 130, 246, 0.05)' },
          ],
        },
      },
    },
    {
      name: '消息数',
      type: 'line',
      smooth: true,
      yAxisIndex: 1,
      data: [] as number[],
      itemStyle: { color: '#10b981' },
      areaStyle: {
        color: {
          type: 'linear',
          x: 0,
          y: 0,
          x2: 0,
          y2: 1,
          colorStops: [
            { offset: 0, color: 'rgba(16, 185, 129, 0.3)' },
            { offset: 1, color: 'rgba(16, 185, 129, 0.05)' },
          ],
        },
      },
    },
  ],
})

// 活跃用户分布图配置
const userDistributionOption = ref({
  title: {
    text: '活跃用户分布',
    left: 'center',
    textStyle: {
      fontSize: 16,
      fontWeight: 'normal',
    },
  },
  tooltip: {
    trigger: 'item',
    formatter: '{a} <br/>{b}: {c} ({d}%)',
  },
  legend: {
    orient: 'vertical',
    left: 'left',
    top: 'center',
  },
  series: [
    {
      name: '活跃用户',
      type: 'pie',
      radius: ['40%', '70%'],
      avoidLabelOverlap: false,
      itemStyle: {
        borderRadius: 10,
        borderColor: '#fff',
        borderWidth: 2,
      },
      label: {
        show: false,
        position: 'center',
      },
      emphasis: {
        label: {
          show: true,
          fontSize: 20,
          fontWeight: 'bold',
        },
      },
      labelLine: {
        show: false,
      },
      data: [
        { value: 0, name: '日活跃用户', itemStyle: { color: '#3b82f6' } },
        { value: 0, name: '周活跃用户', itemStyle: { color: '#8b5cf6' } },
        { value: 0, name: '月活跃用户', itemStyle: { color: '#10b981' } },
      ],
    },
  ],
})

// 消息统计图配置
const messageStatsOption = ref({
  title: {
    text: '消息统计',
    left: 'center',
    textStyle: {
      fontSize: 16,
      fontWeight: 'normal',
    },
  },
  tooltip: {
    trigger: 'axis',
    axisPointer: {
      type: 'shadow',
    },
  },
  grid: {
    left: '3%',
    right: '4%',
    bottom: '3%',
    containLabel: true,
  },
  xAxis: {
    type: 'category',
    data: ['日消息', '周消息', '月消息'],
    axisTick: {
      alignWithLabel: true,
    },
  },
  yAxis: {
    type: 'value',
  },
  series: [
    {
      name: '消息数',
      type: 'bar',
      barWidth: '60%',
      data: [
        { value: 0, itemStyle: { color: '#3b82f6' } },
        { value: 0, itemStyle: { color: '#8b5cf6' } },
        { value: 0, itemStyle: { color: '#10b981' } },
      ],
    },
  ],
})

// ==================== 方法 ====================

/**
 * 获取系统统计数据
 */
async function fetchSystemStats() {
  try {
    const response = await getSystemStats()
    if (response.success && response.data) {
      systemStats.value = response.data
    }
  } catch (err) {
    console.error('获取系统统计失败:', err)
  }
}

/**
 * 获取活跃度统计数据
 */
async function fetchActivityStats() {
  try {
    const response = await getActivityStats()
    if (response.success && response.data) {
      activityStats.value = response.data
      updateCharts()
    }
  } catch (err) {
    console.error('获取活跃度统计失败:', err)
  }
}

/**
 * 生成模拟历史数据
 */
function generateHistoryData() {
  const data = []
  const today = new Date()
  for (let i = 6; i >= 0; i--) {
    const date = new Date(today)
    date.setDate(date.getDate() - i)
    data.push({
      date: date.toLocaleDateString('zh-CN', { month: 'short', day: 'numeric' }),
      active_users: Math.floor(Math.random() * 50) + 10,
      messages: Math.floor(Math.random() * 200) + 50,
    })
  }
  historyData.value = data
  updateActivityTrendChart()
}

/**
 * 更新图表数据
 */
function updateCharts() {
  // 更新用户分布图
  if (userDistributionOption.value.series && userDistributionOption.value.series[0]) {
    userDistributionOption.value.series[0].data = [
      { value: activityStats.value.daily_active_users, name: '日活跃用户', itemStyle: { color: '#3b82f6' } },
      { value: activityStats.value.weekly_active_users, name: '周活跃用户', itemStyle: { color: '#8b5cf6' } },
      { value: activityStats.value.monthly_active_users, name: '月活跃用户', itemStyle: { color: '#10b981' } },
    ]
  }

  // 更新消息统计图
  if (messageStatsOption.value.series && messageStatsOption.value.series[0]) {
    messageStatsOption.value.series[0].data = [
      { value: activityStats.value.daily_messages, itemStyle: { color: '#3b82f6' } },
      { value: activityStats.value.weekly_messages, itemStyle: { color: '#8b5cf6' } },
      { value: activityStats.value.monthly_messages, itemStyle: { color: '#10b981' } },
    ]
  }
}

/**
 * 更新活跃度趋势图
 */
function updateActivityTrendChart() {
  if (activityTrendOption.value.xAxis) {
    activityTrendOption.value.xAxis.data = historyData.value.map((item) => item.date)
  }
  if (activityTrendOption.value.series && activityTrendOption.value.series[0]) {
    activityTrendOption.value.series[0].data = historyData.value.map((item) => item.active_users)
  }
  if (activityTrendOption.value.series && activityTrendOption.value.series[1]) {
    activityTrendOption.value.series[1].data = historyData.value.map((item) => item.messages)
  }
}

/**
 * 刷新所有数据
 */
async function refreshData() {
  loading.value = true
  error.value = null
  try {
    await Promise.all([fetchSystemStats(), fetchActivityStats()])
    generateHistoryData()
  } catch (err) {
    error.value = '获取监控数据失败，请稍后重试'
    console.error(err)
  } finally {
    loading.value = false
  }
}

/**
 * 格式化数字
 */
function formatNumber(num: number): string {
  if (num >= 10000) {
    return (num / 10000).toFixed(1) + '万'
  }
  return num.toLocaleString('zh-CN')
}

// ==================== 生命周期 ====================

onMounted(() => {
  refreshData()
  // 每 30 秒自动刷新
  refreshTimer = setInterval(refreshData, 30000)
})

onUnmounted(() => {
  if (refreshTimer) {
    clearInterval(refreshTimer)
  }
})
</script>

<template>
  <div class="monitor-page">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="header-left">
        <h2 class="page-title">系统监控</h2>
        <p class="page-desc">实时监控系统运行状态和性能指标</p>
      </div>
      <button class="refresh-btn" :disabled="loading" @click="refreshData">
        <Refresh v-if="!loading" class="refresh-icon" />
        <Refresh v-else class="refresh-icon spinning" />
        {{ loading ? '刷新中...' : '刷新' }}
      </button>
    </div>

    <!-- 错误提示 -->
    <div v-if="error" class="error-alert">
      <Warning class="error-icon" />
      {{ error }}
    </div>

    <!-- 统计卡片 -->
    <div class="stats-grid">
      <div class="stat-card">
        <div class="stat-icon users">
          <User class="monitor-stat-svg-icon" />
        </div>
        <div class="stat-content">
          <div class="stat-value">{{ formatNumber(systemStats.total_users) }}</div>
          <div class="stat-label">总用户数</div>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-icon rooms">
          <HomeFilled class="monitor-stat-svg-icon" />
        </div>
        <div class="stat-content">
          <div class="stat-value">{{ formatNumber(systemStats.total_rooms) }}</div>
          <div class="stat-label">房间总数</div>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-icon messages">
          <ChatDotRound class="monitor-stat-svg-icon" />
        </div>
        <div class="stat-content">
          <div class="stat-value">{{ formatNumber(systemStats.total_messages) }}</div>
          <div class="stat-label">消息总数</div>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-icon online">
          <CircleCheck class="monitor-stat-svg-icon" />
        </div>
        <div class="stat-content">
          <div class="stat-value">{{ formatNumber(systemStats.online_users) }}</div>
          <div class="stat-label">在线用户</div>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-icon connections">
          <Connection class="monitor-stat-svg-icon" />
        </div>
        <div class="stat-content">
          <div class="stat-value">{{ formatNumber(systemStats.active_connections) }}</div>
          <div class="stat-label">活跃连接</div>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-icon daily">
          <Clock class="monitor-stat-svg-icon" />
        </div>
        <div class="stat-content">
          <div class="stat-value">{{ formatNumber(activityStats.daily_active_users) }}</div>
          <div class="stat-label">日活跃用户</div>
        </div>
      </div>
    </div>

    <!-- 图表区域 -->
    <div class="charts-grid">
      <div class="chart-card large">
        <v-chart class="chart" :option="activityTrendOption" autoresize />
      </div>
      <div class="chart-card">
        <v-chart class="chart" :option="userDistributionOption" autoresize />
      </div>
      <div class="chart-card">
        <v-chart class="chart" :option="messageStatsOption" autoresize />
      </div>
    </div>
  </div>
</template>

<style scoped>
.monitor-page {
  max-width: 1400px;
  padding: var(--spacing-6);
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: var(--spacing-6);
}

.header-left {
  flex: 1;
}

.page-title {
  font-size: var(--font-size-2xl);
  font-weight: var(--font-weight-bold);
  color: var(--text-primary);
  margin-bottom: var(--spacing-2);
}

.page-desc {
  font-size: var(--font-size-base);
  color: var(--text-secondary);
}

.refresh-btn {
  display: flex;
  align-items: center;
  gap: var(--spacing-2);
  padding: var(--spacing-2) var(--spacing-4);
  background-color: var(--primary);
  color: white;
  border: none;
  border-radius: var(--radius-md);
  font-size: var(--font-size-sm);
  font-weight: var(--font-weight-medium);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.refresh-btn:hover:not(:disabled) {
  background-color: var(--primary-hover);
}

.refresh-btn:disabled {
  opacity: 0.7;
  cursor: not-allowed;
}

.refresh-icon {
  flex-shrink: 0;
}

.refresh-icon.spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.error-alert {
  display: flex;
  align-items: center;
  gap: var(--spacing-3);
  padding: var(--spacing-4);
  background-color: var(--error-bg);
  border: 1px solid var(--error-border);
  border-radius: var(--radius-md);
  color: var(--error);
  margin-bottom: var(--spacing-6);
}

.error-icon {
  flex-shrink: 0;
}

/* 统计卡片 */
.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: var(--spacing-4);
  margin-bottom: var(--spacing-6);
}

.stat-card {
  display: flex;
  align-items: center;
  gap: var(--spacing-4);
  padding: var(--spacing-5);
  background-color: var(--bg-card);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-sm);
  transition: all var(--transition-fast);
}

.stat-card:hover {
  box-shadow: var(--shadow-md);
  transform: translateY(-2px);
}

.stat-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 48px;
  height: 48px;
  border-radius: var(--radius-lg);
  flex-shrink: 0;
}

.monitor-stat-svg-icon {
  width: 24px;
  height: 24px;
  fill: currentColor;
}

.refresh-icon {
  width: 16px;
  height: 16px;
  fill: currentColor;
}

.error-icon {
  width: 20px;
  height: 20px;
  fill: currentColor;
  flex-shrink: 0;
}

.stat-icon.users {
  background-color: rgba(59, 130, 246, 0.1);
  color: #3b82f6;
}

.stat-icon.rooms {
  background-color: rgba(139, 92, 246, 0.1);
  color: #8b5cf6;
}

.stat-icon.messages {
  background-color: rgba(16, 185, 129, 0.1);
  color: #10b981;
}

.stat-icon.online {
  background-color: rgba(245, 158, 11, 0.1);
  color: #f59e0b;
}

.stat-icon.connections {
  background-color: rgba(239, 68, 68, 0.1);
  color: #ef4444;
}

.stat-icon.daily {
  background-color: rgba(6, 182, 212, 0.1);
  color: #06b6d4;
}

.stat-content {
  flex: 1;
  min-width: 0;
}

.stat-value {
  font-size: var(--font-size-2xl);
  font-weight: var(--font-weight-bold);
  color: var(--text-primary);
  margin-bottom: var(--spacing-1);
}

.stat-label {
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
}

/* 图表区域 */
.charts-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: var(--spacing-6);
}

.chart-card {
  background-color: var(--bg-card);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-sm);
  padding: var(--spacing-5);
}

.chart-card.large {
  grid-column: span 2;
}

.chart {
  width: 100%;
  height: 300px;
}

.chart-card.large .chart {
  height: 350px;
}

/* 响应式 */
@media (max-width: 1024px) {
  .charts-grid {
    grid-template-columns: 1fr;
  }

  .chart-card.large {
    grid-column: span 1;
  }
}

@media (max-width: 640px) {
  .monitor-page {
    padding: var(--spacing-4);
  }

  .page-header {
    flex-direction: column;
    gap: var(--spacing-4);
  }

  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
  }

  .stat-card {
    padding: var(--spacing-4);
  }

  .stat-icon {
    width: 40px;
    height: 40px;
  }

  .stat-value {
    font-size: var(--font-size-xl);
  }
}
</style>
