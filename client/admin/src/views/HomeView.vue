<template>
  <div class="home-view">
    <!-- 欢迎区域 -->
    <div class="welcome-section">
      <div class="welcome-content">
        <h1 class="welcome-title">欢迎回来，{{ userStore.userInfo?.username || '管理员' }}</h1>
        <p class="welcome-subtitle">今天是 {{ today }}，系统运行正常</p>
      </div>
      <n-button @click="refreshData" :loading="loading" type="primary" secondary>
        <template #icon>
          <RefreshCw :size="16" />
        </template>
        刷新
      </n-button>
    </div>

    <!-- 实时监控卡片 -->
    <div class="section-header">
      <h2 class="section-title">
        <Activity class="section-icon" :size="18" />
        实时监控
      </h2>
      <n-button text @click="$router.push('/statistics')">
        查看详细统计
        <template #icon>
          <ArrowRight :size="14" />
        </template>
      </n-button>
    </div>

    <div class="stats-grid">
      <n-card class="stat-card" :bordered="false">
        <n-statistic label="当前在线用户" :value="systemStats?.online_users || 0">
          <template #prefix>
            <Users class="stat-icon online" :size="20" />
          </template>
        </n-statistic>
        <div class="stat-trend">
          <span>WebSocket 连接: {{ systemStats?.active_connections || 0 }}</span>
        </div>
      </n-card>

      <n-card class="stat-card" :bordered="false">
        <n-statistic label="今日消息数" :value="activityStats?.daily_messages || 0">
          <template #prefix>
            <MessageCircle class="stat-icon message" :size="20" />
          </template>
        </n-statistic>
        <div class="stat-trend">
          <span>本周: {{ formatNumber(activityStats?.weekly_messages || 0) }}</span>
        </div>
      </n-card>

      <n-card class="stat-card" :bordered="false">
        <n-statistic label="活跃房间数" :value="performanceStats?.active_rooms || 0">
          <template #prefix>
            <MessageSquare class="stat-icon room" :size="20" />
          </template>
        </n-statistic>
        <div class="stat-trend">
          <span>总房间: {{ systemStats?.total_rooms || 0 }}</span>
        </div>
      </n-card>

      <n-card class="stat-card" :bordered="false">
        <n-statistic label="内存使用" :value="Math.round(monitorData?.system.memory.usage_percent || 0)">
          <template #prefix>
            <Cpu class="stat-icon system" :size="20" />
          </template>
          <template #suffix>%</template>
        </n-statistic>
        <div class="stat-trend" :class="{ up: (monitorData?.system.memory.usage_percent || 0) > 80, down: (monitorData?.system.memory.usage_percent || 0) < 50 }">
          <span>{{ formatMemory(monitorData?.system.memory.used_mb) }} / {{ formatMemory(monitorData?.system.memory.total_mb) }}</span>
        </div>
      </n-card>
    </div>

    <!-- 系统状态实时曲线 -->
    <div class="section-header">
      <h2 class="section-title">
        <TrendingUp class="section-icon" :size="18" />
        系统状态趋势
      </h2>
      <span class="chart-legend">
        <span class="legend-item memory"><span class="legend-dot"></span>内存使用率</span>
        <span class="legend-item online"><span class="legend-dot"></span>在线用户</span>
        <span class="legend-item connections"><span class="legend-dot"></span>WebSocket连接</span>
      </span>
    </div>

    <n-card class="chart-card" :bordered="false">
      <LineChart
        :x-axis="chartData.xAxis"
        :series="chartData.series"
        :loading="loading"
        :show-legend="false"
        :show-tooltip="true"
        :show-grid="true"
        height="280px"
        :option="chartOption"
      />
    </n-card>

    <!-- 快捷入口 -->
    <div class="section-header">
      <h2 class="section-title">
        <Zap class="section-icon" :size="18" />
        快捷操作
      </h2>
    </div>

    <div class="quick-actions">
      <n-card
        v-for="action in quickActions"
        :key="action.key"
        class="action-card"
        :bordered="false"
        hoverable
        @click="handleActionClick(action)"
      >
        <div class="action-content">
          <div class="action-icon-wrapper" :class="action.color">
            <component :is="action.icon" :size="24" />
          </div>
          <div class="action-info">
            <h3 class="action-title">{{ action.title }}</h3>
            <p class="action-desc">{{ action.description }}</p>
          </div>
          <ChevronRight class="action-arrow" :size="18" />
        </div>
      </n-card>
    </div>

  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { NCard, NStatistic, NButton, useMessage } from 'naive-ui'
import {
  Users,
  UserCheck,
  MessageSquare,
  MessageCircle,
  TrendingUp,
  RefreshCw,
  ArrowRight,
  Zap,
  ChevronRight,
  Cpu,
  Shield,
  Settings,
  BarChart3,
  FileText,
  Activity,
  type LucideIcon,
} from 'lucide-vue-next'
import LineChart from '@/components/common/charts/LineChart.vue'
import { useAuthStore } from '@/store'
import {
  getSystemStats,
  getActivityStats,
  getPerformanceStats,
  getMonitorData,
  type SystemStatsOverview,
  type ActivityStats,
  type PerformanceStats,
  type MonitorData,
} from '@/api/statistics'
import { getClientConfig, type ClientConfig } from '@/api/config'

/**
 * 路由实例
 */
const router = useRouter()

/**
 * 消息提示
 */
const message = useMessage()

/**
 * 认证 store
 */
const authStore = useAuthStore()

/**
 * 用户信息
 */
const userStore = authStore

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
 * 系统监控数据
 */
const monitorData = ref<MonitorData | null>(null)

/**
 * 客户端配置
 */
const clientConfig = ref<ClientConfig | null>(null)

/**
 * 自动刷新定时器
 */
let refreshTimer: ReturnType<typeof setInterval> | null = null

/**
 * 历史数据点（用于绘制曲线）
 */
interface HistoryPoint {
  time: string
  memoryPercent: number
  onlineUsers: number
  connections: number
}

const historyData = ref<HistoryPoint[]>([])
const MAX_HISTORY_POINTS = 20

/**
 * 图表数据
 */
const chartData = computed(() => {
  const xAxis = historyData.value.map((item) => item.time)
  const series = [
    {
      name: '内存使用率',
      data: historyData.value.map((item) => item.memoryPercent),
      smooth: true,
      area: true,
      yAxisIndex: 0,
    },
    {
      name: '在线用户',
      data: historyData.value.map((item) => item.onlineUsers),
      smooth: true,
      area: false,
      yAxisIndex: 1,
    },
    {
      name: 'WebSocket连接',
      data: historyData.value.map((item) => item.connections),
      smooth: true,
      area: false,
      yAxisIndex: 1,
    },
  ]
  return { xAxis, series }
})

/**
 * 图表配置（双Y轴）
 */
const chartOption = computed(() => ({
  yAxis: [
    {
      type: 'value' as const,
      name: '内存(%)',
      min: 0,
      max: 100,
      position: 'left' as const,
      axisLine: { show: true },
      axisLabel: { formatter: '{value}%' },
    },
    {
      type: 'value' as const,
      name: '数量',
      min: 0,
      position: 'right' as const,
      axisLine: { show: true },
      splitLine: { show: false },
    },
  ],
  grid: {
    left: '8%',
    right: '8%',
    bottom: '3%',
    top: '10%',
    containLabel: true,
  },
}))

/**
 * 今日日期
 */
const today = computed(() => {
  return new Date().toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
    weekday: 'long',
  })
})

/**
 * 格式化内存大小
 * @param mb 内存大小(MB)
 * @returns 格式化后的字符串
 */
function formatMemory(mb: number | undefined): string {
  if (mb === undefined) return '-'
  if (mb >= 1024) {
    return `${(mb / 1024).toFixed(1)} GB`
  }
  return `${mb} MB`
}

/**
 * 格式化数字（添加千分位）
 * @param num 数字
 * @returns 格式化后的字符串
 */
function formatNumber(num: number): string {
  return num.toLocaleString('zh-CN')
}

/**
 * 快捷操作配置
 */
interface QuickAction {
  key: string
  title: string
  description: string
  icon: LucideIcon
  color: string
  path?: string
  action?: () => void
}

const quickActions: QuickAction[] = [
  {
    key: 'users',
    title: '用户管理',
    description: '查看和管理系统用户',
    icon: UserCheck,
    color: 'blue',
    path: '/users',
  },
  {
    key: 'rooms',
    title: '房间管理',
    description: '管理聊天房间和消息',
    icon: MessageSquare,
    color: 'green',
    path: '/rooms',
  },
  {
    key: 'messages',
    title: '消息审核',
    description: '审核和处理违规消息',
    icon: Shield,
    color: 'orange',
    path: '/messages',
  },
  {
    key: 'statistics',
    title: '系统统计',
    description: '查看详细统计数据',
    icon: BarChart3,
    color: 'purple',
    path: '/statistics',
  },
  {
    key: 'audit',
    title: '审计日志',
    description: '查看系统操作记录',
    icon: FileText,
    color: 'cyan',
    path: '/audit',
  },
  {
    key: 'settings',
    title: '系统设置',
    description: '配置系统参数',
    icon: Settings,
    color: 'gray',
    path: '/setting',
  },
]

/**
 * 处理快捷操作点击
 */
function handleActionClick(action: QuickAction) {
  if (action.path) {
    router.push(action.path)
  } else if (action.action) {
    action.action()
  }
}

/**
 * 加载统计数据
 */
async function loadStats() {
  loading.value = true
  try {
    const [systemRes, activityRes, performanceRes, monitorRes] = await Promise.all([
      getSystemStats(),
      getActivityStats(),
      getPerformanceStats(),
      getMonitorData(),
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
    if (monitorRes.success && monitorRes.data) {
      monitorData.value = monitorRes.data
    }

    // 更新历史数据用于图表（使用所有可用的数据）
    if (systemRes.success && systemRes.data && monitorRes.success && monitorRes.data) {
      const now = new Date()
      const timeStr = `${now.getHours().toString().padStart(2, '0')}:${now.getMinutes().toString().padStart(2, '0')}:${now.getSeconds().toString().padStart(2, '0')}`

      historyData.value.push({
        time: timeStr,
        memoryPercent: Math.round(monitorRes.data.system.memory.usage_percent),
        onlineUsers: systemRes.data.online_users,
        connections: systemRes.data.active_connections,
      })

      // 保持最大数据点数
      if (historyData.value.length > MAX_HISTORY_POINTS) {
        historyData.value.shift()
      }
    }
  } catch (error) {
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
 * 启动自动刷新
 */
function startAutoRefresh() {
  // 清除现有定时器
  if (refreshTimer) {
    clearInterval(refreshTimer)
  }

  // 获取刷新间隔（默认30秒）
  const intervalSecs = clientConfig.value?.monitor?.refresh_interval_secs || 30
  const intervalMs = intervalSecs * 1000

  // 启动定时器
  refreshTimer = setInterval(() => {
    loadStats()
  }, intervalMs)

  console.log(`[HomeView] 自动刷新已启动，间隔: ${intervalSecs}秒`)
}

/**
 * 停止自动刷新
 */
function stopAutoRefresh() {
  if (refreshTimer) {
    clearInterval(refreshTimer)
    refreshTimer = null
    console.log('[HomeView] 自动刷新已停止')
  }
}

/**
 * 加载客户端配置
 */
async function loadClientConfig() {
  try {
    const res = await getClientConfig()
    if (res.success && res.data) {
      clientConfig.value = res.data
      // 获取配置后启动自动刷新
      startAutoRefresh()
    }
  } catch (error) {
    console.error('加载客户端配置失败:', error)
    // 使用默认配置启动自动刷新
    startAutoRefresh()
  }
}

/**
 * 组件挂载时加载数据
 */
onMounted(() => {
  loadStats()
  loadClientConfig()
})

/**
 * 组件卸载时清理
 */
onUnmounted(() => {
  stopAutoRefresh()
})
</script>

<style scoped>
.home-view {
  min-height: 100%;
  padding: 24px;
}

/* 欢迎区域 */
.welcome-section {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 32px;
}

.welcome-title {
  font-size: 24px;
  font-weight: 600;
  color: var(--text-color-base);
  margin: 0 0 8px 0;
}

.welcome-subtitle {
  font-size: 14px;
  color: var(--text-color-secondary);
  margin: 0;
}

/* 区域标题 */
.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  margin-top: 24px;
}

.section-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-color-base);
  margin: 0;
  display: flex;
  align-items: center;
  gap: 8px;
}

.section-icon {
  color: var(--color-primary);
}

/* 统计卡片 */
.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
}

.stat-card {
  transition: var(--transition-base);
}

.stat-card:hover {
  transform: translateY(-2px);
}

.stat-icon {
  margin-right: 8px;
}

.stat-icon.online {
  color: #18a058;
}

.stat-icon.message {
  color: #2080f0;
}

.stat-icon.room {
  color: #f0a020;
}

.stat-icon.system {
  color: #8a2be2;
}

.stat-trend {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-top: 8px;
  font-size: 12px;
  color: var(--text-color-secondary);
}

.stat-trend.up {
  color: #18a058;
}

.stat-trend.down {
  color: #d03050;
}

/* 图表区域 */
.chart-card {
  margin-bottom: 8px;
}

.chart-legend {
  display: flex;
  align-items: center;
  gap: 16px;
  font-size: 12px;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--text-color-secondary);
}

.legend-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.legend-item.memory .legend-dot {
  background: #8a2be2;
}

.legend-item.online .legend-dot {
  background: #18a058;
}

.legend-item.connections .legend-dot {
  background: #2080f0;
}

/* 快捷操作 */
.quick-actions {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
}

.action-card {
  cursor: pointer;
  transition: var(--transition-base);
}

.action-card:hover {
  transform: translateY(-2px);
}

.action-content {
  display: flex;
  align-items: center;
  gap: 12px;
}

.action-icon-wrapper {
  width: 48px;
  height: 48px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  flex-shrink: 0;
}

.action-icon-wrapper.blue {
  background: linear-gradient(135deg, #2080f0, #4098f7);
}

.action-icon-wrapper.green {
  background: linear-gradient(135deg, #18a058, #36ad6a);
}

.action-icon-wrapper.orange {
  background: linear-gradient(135deg, #f0a020, #f7b846);
}

.action-icon-wrapper.purple {
  background: linear-gradient(135deg, #8a2be2, #a855f7);
}

.action-icon-wrapper.cyan {
  background: linear-gradient(135deg, #08979c, #13c2c2);
}

.action-icon-wrapper.gray {
  background: linear-gradient(135deg, #5c5c5c, #8c8c8c);
}

.action-info {
  flex: 1;
  min-width: 0;
}

.action-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-color-base);
  margin: 0 0 4px 0;
}

.action-desc {
  font-size: 12px;
  color: var(--text-color-secondary);
  margin: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.action-arrow {
  color: var(--text-color-tertiary);
}

/* 响应式适配 */
@media (max-width: 1200px) {
  .quick-actions {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (max-width: 1024px) {
  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (max-width: 768px) {
  .quick-actions {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 640px) {
  .home-view {
    padding: 16px;
  }

  .welcome-section {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }

  .stats-grid {
    grid-template-columns: 1fr;
  }

  .welcome-title {
    font-size: 20px;
  }
}
</style>
