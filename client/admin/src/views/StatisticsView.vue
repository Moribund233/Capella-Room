<template>
  <div class="statistics-view">
    <!-- 页面标题 -->
    <div class="page-header">
      <h1 class="page-title">系统统计</h1>
      <n-button @click="refreshData" :loading="loading" type="primary" secondary>
        <template #icon>
          <RefreshCw :size="16" />
        </template>
        刷新数据
      </n-button>
    </div>

    <!-- 统计卡片区域 -->
    <div class="stats-grid">
      <n-card class="stat-card" :bordered="false">
        <n-statistic label="总用户数" :value="systemStats?.total_users || 0">
          <template #prefix>
            <Users class="stat-icon" :size="20" />
          </template>
        </n-statistic>
      </n-card>
      <n-card class="stat-card" :bordered="false">
        <n-statistic label="总房间数" :value="systemStats?.total_rooms || 0">
          <template #prefix>
            <MessageSquare class="stat-icon" :size="20" />
          </template>
        </n-statistic>
      </n-card>
      <n-card class="stat-card" :bordered="false">
        <n-statistic label="总消息数" :value="systemStats?.total_messages || 0">
          <template #prefix>
            <Mail class="stat-icon" :size="20" />
          </template>
        </n-statistic>
      </n-card>
      <n-card class="stat-card" :bordered="false">
        <n-statistic label="当前在线" :value="systemStats?.online_users || 0">
          <template #prefix>
            <UserCheck class="stat-icon" :size="20" />
          </template>
        </n-statistic>
      </n-card>
    </div>

    <!-- 活跃度统计卡片 -->
    <div class="stats-grid secondary">
      <n-card class="stat-card" :bordered="false">
        <n-statistic label="日活跃用户(DAU)" :value="activityStats?.daily_active_users || 0">
          <template #prefix>
            <Activity class="stat-icon" :size="18" />
          </template>
        </n-statistic>
      </n-card>
      <n-card class="stat-card" :bordered="false">
        <n-statistic label="周活跃用户(WAU)" :value="activityStats?.weekly_active_users || 0">
          <template #prefix>
            <Calendar class="stat-icon" :size="18" />
          </template>
        </n-statistic>
      </n-card>
      <n-card class="stat-card" :bordered="false">
        <n-statistic label="月活跃用户(MAU)" :value="activityStats?.monthly_active_users || 0">
          <template #prefix>
            <TrendingUp class="stat-icon" :size="18" />
          </template>
        </n-statistic>
      </n-card>
      <n-card class="stat-card" :bordered="false">
        <n-statistic label="今日消息数" :value="activityStats?.daily_messages || 0">
          <template #prefix>
            <MessageCircle class="stat-icon" :size="18" />
          </template>
        </n-statistic>
      </n-card>
    </div>

    <!-- 图表区域 -->
    <div class="charts-grid">
      <!-- 用户活跃度趋势 -->
      <chart-card title="用户活跃度" subtitle="DAU / WAU / MAU 对比">
        <bar-chart :x-axis="activityLabels" :series="activitySeries" y-axis-name="用户数" />
      </chart-card>

      <!-- 消息量统计 -->
      <chart-card title="消息量统计" subtitle="今日 / 本周 / 本月消息数">
        <bar-chart :x-axis="messageLabels" :series="messageSeries" y-axis-name="消息数" />
      </chart-card>

      <!-- 系统概览饼图 -->
      <chart-card title="系统概览" subtitle="用户、房间、消息分布">
        <pie-chart :data="overviewData" type="doughnut" />
      </chart-card>

      <!-- 性能指标 -->
      <chart-card title="性能指标" subtitle="系统运行状态">
        <div class="performance-metrics">
          <div class="metric-item">
            <span class="metric-label">活跃连接数</span>
            <span class="metric-value">{{ systemStats?.active_connections || 0 }}</span>
          </div>
          <div class="metric-item">
            <span class="metric-label">活跃房间数</span>
            <span class="metric-value">{{ performanceStats?.active_rooms || 0 }}</span>
          </div>
          <div class="metric-item">
            <span class="metric-label">总连接数</span>
            <span class="metric-value">{{ performanceStats?.total_connections || 0 }}</span>
          </div>
          <div class="metric-item">
            <span class="metric-label">数据更新时间</span>
            <span class="metric-value">{{ formatTime(performanceStats?.timestamp) }}</span>
          </div>
        </div>
      </chart-card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { NCard, NStatistic, NButton, useMessage } from 'naive-ui'
import {
  Users,
  UserCheck,
  MessageSquare,
  Mail,
  Activity,
  Calendar,
  TrendingUp,
  MessageCircle,
  RefreshCw,
} from 'lucide-vue-next'
import { ChartCard, BarChart, PieChart } from '@/components/common/charts'
import {
  getSystemStats,
  getActivityStats,
  getPerformanceStats,
  type SystemStatsOverview,
  type ActivityStats,
  type PerformanceStats,
} from '@/api/statistics'

/**
 * 消息提示
 */
const message = useMessage()

/**
 * 加载状态
 */
const loading = ref(false)

/**
 * 系统统计概览数据
 */
const systemStats = ref<SystemStatsOverview | null>(null)

/**
 * 活跃度统计数据
 */
const activityStats = ref<ActivityStats | null>(null)

/**
 * 性能指标数据
 */
const performanceStats = ref<PerformanceStats | null>(null)

/**
 * 活跃度图表标签
 */
const activityLabels = ['日活跃', '周活跃', '月活跃']

/**
 * 活跃度图表数据
 */
const activitySeries = computed(() => [
  {
    name: '活跃用户数',
    data: [
      activityStats.value?.daily_active_users || 0,
      activityStats.value?.weekly_active_users || 0,
      activityStats.value?.monthly_active_users || 0,
    ],
  },
])

/**
 * 消息量图表标签
 */
const messageLabels = ['今日', '本周', '本月']

/**
 * 消息量图表数据
 */
const messageSeries = computed(() => [
  {
    name: '消息数',
    data: [
      activityStats.value?.daily_messages || 0,
      activityStats.value?.weekly_messages || 0,
      activityStats.value?.monthly_messages || 0,
    ],
  },
])

/**
 * 系统概览饼图数据
 */
const overviewData = computed(() => [
  { name: '用户数', value: systemStats.value?.total_users || 0 },
  { name: '房间数', value: systemStats.value?.total_rooms || 0 },
  { name: '消息数', value: systemStats.value?.total_messages || 0 },
])

/**
 * 格式化时间
 * @param timestamp ISO 8601 格式时间戳
 * @returns 格式化后的时间字符串
 */
function formatTime(timestamp: string | undefined): string {
  if (!timestamp) return '-'
  const date = new Date(timestamp)
  return date.toLocaleString('zh-CN', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
}

/**
 * 加载统计数据
 */
async function loadStats() {
  loading.value = true
  try {
    const [systemRes, activityRes, performanceRes] = await Promise.all([
      getSystemStats(),
      getActivityStats(),
      getPerformanceStats(),
    ])

    if (systemRes.success && systemRes.data) {
      systemStats.value = systemRes.data
    }
    if (activityRes.success && activityRes.data) {
      activityStats.value = activityRes.data
    }
    if (performanceRes.success && performanceRes.data) {
      performanceStats.value = performanceRes.data
    }
  } catch (error) {
    message.error('加载统计数据失败')
    console.error('加载统计数据失败:', error)
  } finally {
    loading.value = false
  }
}

/**
 * 刷新数据
 */
async function refreshData() {
  await loadStats()
  message.success('数据已刷新')
}

/**
 * 组件挂载时加载数据
 */
onMounted(() => {
  loadStats()
})
</script>

<style scoped>
.statistics-view {
  min-height: 100%;
  padding: 24px;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.page-title {
  font-size: 24px;
  font-weight: 600;
  color: var(--text-color-base);
  margin: 0;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
  margin-bottom: 16px;
}

.stats-grid.secondary {
  margin-bottom: 24px;
}

.stats-grid.secondary .stat-card {
  background: var(--color-secondary);
}

.stat-card {
  transition: var(--transition-base);
}

.stat-card:hover {
  transform: translateY(-2px);
}

.stat-icon {
  color: var(--color-primary);
  margin-right: 8px;
}

.charts-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
}

.performance-metrics {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 16px;
}

.metric-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: var(--color-secondary);
  border-radius: 8px;
}

.metric-label {
  color: var(--text-color-secondary);
  font-size: 14px;
}

.metric-value {
  color: var(--text-color-base);
  font-size: 16px;
  font-weight: 600;
}

/* 响应式适配 */
@media (max-width: 1024px) {
  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
  }

  .charts-grid {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 640px) {
  .statistics-view {
    padding: 16px;
  }

  .page-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }

  .stats-grid {
    grid-template-columns: 1fr;
  }

  .stats-grid.secondary {
    grid-template-columns: repeat(2, 1fr);
  }
}
</style>
