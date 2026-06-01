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

    <!-- 增强统计区域 -->
    <div class="section-title">
      <h2>深度分析</h2>
    </div>

    <!-- 用户增长趋势 -->
    <div class="charts-grid full-width">
      <chart-card title="用户增长趋势" subtitle="近30天新用户注册量" :loading="userGrowthLoading">
        <area-chart
          :x-axis="userGrowthDates"
          :series="userGrowthSeries"
          y-axis-name="新用户数"
          :smooth="true"
        />
      </chart-card>
    </div>

    <!-- 用户行为与房间统计 -->
    <div class="charts-grid">
      <chart-card title="用户行为指标" subtitle="人均活跃度统计" :loading="userBehaviorLoading">
        <div class="behavior-stats">
          <div class="behavior-item">
            <div class="behavior-label">人均消息数</div>
            <div class="behavior-value">{{ formatNumber(userBehaviorStats?.avg_messages_per_user) }}</div>
          </div>
          <div class="behavior-item">
            <div class="behavior-label">人均房间数</div>
            <div class="behavior-value">{{ formatNumber(userBehaviorStats?.avg_rooms_per_user) }}</div>
          </div>
          <div class="behavior-item">
            <div class="behavior-label">今日活跃用户</div>
            <div class="behavior-value">{{ userBehaviorStats?.active_users_today || 0 }}</div>
          </div>
          <div class="behavior-item">
            <div class="behavior-label">本周活跃用户</div>
            <div class="behavior-value">{{ userBehaviorStats?.active_users_this_week || 0 }}</div>
          </div>
        </div>
      </chart-card>

      <chart-card title="房间类型分布" subtitle="公开/私有/私聊房间占比" :loading="roomStatsLoading">
        <pie-chart :data="roomTypeData" type="pie" />
      </chart-card>
    </div>

    <!-- 消息类型与时间分布 -->
    <div class="charts-grid">
      <chart-card title="消息类型分布" subtitle="各类消息数量统计" :loading="messageTypeLoading">
        <pie-chart :data="messageTypeData" type="doughnut" />
      </chart-card>

      <chart-card title="消息时间分布" subtitle="24小时消息发送热度" :loading="hourlyLoading">
        <bar-chart
          :x-axis="hourlyLabels"
          :series="hourlySeries"
          y-axis-name="消息数"
          :show-legend="false"
        />
      </chart-card>
    </div>

    <!-- 房间活跃度排行 -->
    <div class="charts-grid full-width">
      <chart-card title="房间活跃度排行" subtitle="Top 10 最活跃房间" :loading="roomActivityLoading">
        <bar-chart
          :x-axis="roomActivityNames"
          :series="roomActivitySeries"
          y-axis-name="消息数"
          :horizontal="true"
          :show-legend="false"
        />
      </chart-card>
    </div>

    <!-- 好友关系与安全统计 -->
    <div class="charts-grid">
      <chart-card title="好友关系统计" subtitle="社交网络数据分析" :loading="friendStatsLoading">
        <div class="friend-stats">
          <div class="friend-item">
            <div class="friend-label">总好友关系</div>
            <div class="friend-value">{{ friendStats?.total_friendships || 0 }}</div>
          </div>
          <div class="friend-item">
            <div class="friend-label">待处理申请</div>
            <div class="friend-value">{{ friendStats?.pending_requests || 0 }}</div>
          </div>
          <div class="friend-item">
            <div class="friend-label">人均好友数</div>
            <div class="friend-value">{{ formatNumber(friendStats?.avg_friends_per_user) }}</div>
          </div>
          <div class="friend-item">
            <div class="friend-label">申请接受率</div>
            <div class="friend-value">{{ formatPercent(friendStats?.request_accept_rate) }}</div>
          </div>
        </div>
      </chart-card>

      <chart-card title="安全告警统计" subtitle="系统安全状态监控" :loading="securityStatsLoading">
        <div class="security-stats">
          <div class="security-item" :class="{ warning: (securityStats?.failed_logins_today || 0) > 0 }">
            <div class="security-label">今日登录失败</div>
            <div class="security-value">{{ securityStats?.failed_logins_today || 0 }}</div>
          </div>
          <div class="security-item" :class="{ danger: (securityStats?.pending_alerts || 0) > 0 }">
            <div class="security-label">待处理告警</div>
            <div class="security-value">{{ securityStats?.pending_alerts || 0 }}</div>
          </div>
          <div class="security-item">
            <div class="security-label">今日告警数</div>
            <div class="security-value">{{ securityStats?.alerts_today || 0 }}</div>
          </div>
          <div class="security-item">
            <div class="security-label">本周审计日志</div>
            <div class="security-value">{{ securityStats?.audit_logs_this_week || 0 }}</div>
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
import { ChartCard, BarChart, PieChart, AreaChart } from '@/components/common/charts'
import type { LineSeries, BarSeries, PieDataItem } from '@/components/common/charts'
import {
  getSystemStats,
  getActivityStats,
  getPerformanceStats,
  getUserGrowthStats,
  getUserBehaviorStats,
  getFriendStats,
  getRoomActivityRanking,
  getRoomStats,
  getMessageTypeStats,
  getMessageHourlyDistribution,
  getSecurityStats,
  type SystemStatsOverview,
  type ActivityStats,
  type PerformanceStats,
  type UserGrowthStats,
  type UserBehaviorStats,
  type FriendStats,
  type RoomActivity,
  type RoomStats,
  type MessageTypeStats,
  type MessageHourlyDistribution,
  type SecurityStats,
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

// ==================== 增强统计数据 ====================

/**
 * 用户增长统计数据
 */
const userGrowthStats = ref<UserGrowthStats | null>(null)
const userGrowthLoading = ref(false)

/**
 * 用户行为统计数据
 */
const userBehaviorStats = ref<UserBehaviorStats | null>(null)
const userBehaviorLoading = ref(false)

/**
 * 好友关系统计数据
 */
const friendStats = ref<FriendStats | null>(null)
const friendStatsLoading = ref(false)

/**
 * 房间活跃度排行
 */
const roomActivityList = ref<RoomActivity[]>([])
const roomActivityLoading = ref(false)

/**
 * 房间统计数据
 */
const roomStats = ref<RoomStats | null>(null)
const roomStatsLoading = ref(false)

/**
 * 消息类型统计数据
 */
const messageTypeStats = ref<MessageTypeStats | null>(null)
const messageTypeLoading = ref(false)

/**
 * 消息时间分布数据
 */
const hourlyDistribution = ref<MessageHourlyDistribution[]>([])
const hourlyLoading = ref(false)

/**
 * 安全告警统计数据
 */
const securityStats = ref<SecurityStats | null>(null)
const securityStatsLoading = ref(false)

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

// ==================== 增强统计计算属性 ====================

/**
 * 用户增长趋势日期
 */
const userGrowthDates = computed(() => {
  return userGrowthStats.value?.growth_by_day.map(item => item.date) || []
})

/**
 * 用户增长趋势数据
 */
const userGrowthSeries = computed<LineSeries[]>(() => [
  {
    name: '新用户',
    data: userGrowthStats.value?.growth_by_day.map(item => item.count) || [],
    smooth: true,
    area: true,
  },
])

/**
 * 房间类型分布数据
 */
const roomTypeData = computed<PieDataItem[]>(() => [
  { name: '公开房间', value: roomStats.value?.public_rooms || 0 },
  { name: '私有房间', value: roomStats.value?.private_rooms || 0 },
  { name: '私聊房间', value: roomStats.value?.direct_rooms || 0 },
])

/**
 * 消息类型分布数据
 */
const messageTypeData = computed<PieDataItem[]>(() => [
  { name: '文本消息', value: messageTypeStats.value?.text_messages || 0 },
  { name: '图片消息', value: messageTypeStats.value?.image_messages || 0 },
  { name: '文件消息', value: messageTypeStats.value?.file_messages || 0 },
  { name: '系统消息', value: messageTypeStats.value?.system_messages || 0 },
  { name: '回复消息', value: messageTypeStats.value?.reply_messages || 0 },
])

/**
 * 消息时间分布标签（0-23小时）
 */
const hourlyLabels = computed(() => {
  const labels: string[] = []
  for (let i = 0; i < 24; i++) {
    labels.push(`${i}:00`)
  }
  return labels
})

/**
 * 消息时间分布数据
 */
const hourlySeries = computed<BarSeries[]>(() => {
  const data = new Array(24).fill(0)
  hourlyDistribution.value.forEach(item => {
    if (item.hour >= 0 && item.hour < 24) {
      data[item.hour] = item.count
    }
  })
  return [{ name: '消息数', data }]
})

/**
 * 房间活跃度排行名称
 */
const roomActivityNames = computed(() => {
  return roomActivityList.value.map(item => item.name)
})

/**
 * 房间活跃度排行数据
 */
const roomActivitySeries = computed<BarSeries[]>(() => [
  {
    name: '消息数',
    data: roomActivityList.value.map(item => item.message_count),
  },
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
 * 格式化数字（保留两位小数）
 * @param value 数字值
 * @returns 格式化后的字符串
 */
function formatNumber(value: number | undefined): string {
  if (value === undefined || value === null) return '0'
  return value.toFixed(2)
}

/**
 * 格式化百分比
 * @param value 百分比值（0-100）
 * @returns 格式化后的字符串
 */
function formatPercent(value: number | undefined): string {
  if (value === undefined || value === null) return '0%'
  return `${value.toFixed(1)}%`
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

  // 并行加载增强统计数据
  await Promise.all([
    loadUserGrowthStats(),
    loadUserBehaviorStats(),
    loadFriendStats(),
    loadRoomActivityRanking(),
    loadRoomStats(),
    loadMessageTypeStats(),
    loadHourlyDistribution(),
    loadSecurityStats(),
  ])
}

/**
 * 加载用户增长统计
 */
async function loadUserGrowthStats() {
  userGrowthLoading.value = true
  try {
    const res = await getUserGrowthStats(30)
    if (res.success && res.data) {
      userGrowthStats.value = res.data
    }
  } catch (error) {
    console.error('加载用户增长统计失败:', error)
  } finally {
    userGrowthLoading.value = false
  }
}

/**
 * 加载用户行为统计
 */
async function loadUserBehaviorStats() {
  userBehaviorLoading.value = true
  try {
    const res = await getUserBehaviorStats()
    if (res.success && res.data) {
      userBehaviorStats.value = res.data
    }
  } catch (error) {
    console.error('加载用户行为统计失败:', error)
  } finally {
    userBehaviorLoading.value = false
  }
}

/**
 * 加载好友关系统计
 */
async function loadFriendStats() {
  friendStatsLoading.value = true
  try {
    const res = await getFriendStats()
    if (res.success && res.data) {
      friendStats.value = res.data
    }
  } catch (error) {
    console.error('加载好友关系统计失败:', error)
  } finally {
    friendStatsLoading.value = false
  }
}

/**
 * 加载房间活跃度排行
 */
async function loadRoomActivityRanking() {
  roomActivityLoading.value = true
  try {
    const res = await getRoomActivityRanking(10)
    if (res.success && res.data) {
      roomActivityList.value = res.data
    }
  } catch (error) {
    console.error('加载房间活跃度排行失败:', error)
  } finally {
    roomActivityLoading.value = false
  }
}

/**
 * 加载房间统计
 */
async function loadRoomStats() {
  roomStatsLoading.value = true
  try {
    const res = await getRoomStats()
    if (res.success && res.data) {
      roomStats.value = res.data
    }
  } catch (error) {
    console.error('加载房间统计失败:', error)
  } finally {
    roomStatsLoading.value = false
  }
}

/**
 * 加载消息类型统计
 */
async function loadMessageTypeStats() {
  messageTypeLoading.value = true
  try {
    const res = await getMessageTypeStats()
    if (res.success && res.data) {
      messageTypeStats.value = res.data
    }
  } catch (error) {
    console.error('加载消息类型统计失败:', error)
  } finally {
    messageTypeLoading.value = false
  }
}

/**
 * 加载消息时间分布
 */
async function loadHourlyDistribution() {
  hourlyLoading.value = true
  try {
    const res = await getMessageHourlyDistribution()
    if (res.success && res.data) {
      hourlyDistribution.value = res.data
    }
  } catch (error) {
    console.error('加载消息时间分布失败:', error)
  } finally {
    hourlyLoading.value = false
  }
}

/**
 * 加载安全告警统计
 */
async function loadSecurityStats() {
  securityStatsLoading.value = true
  try {
    const res = await getSecurityStats()
    if (res.success && res.data) {
      securityStats.value = res.data
    }
  } catch (error) {
    console.error('加载安全告警统计失败:', error)
  } finally {
    securityStatsLoading.value = false
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

/* 章节标题 */
.section-title {
  margin: 32px 0 16px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--border-color);
}

.section-title h2 {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-color-base);
  margin: 0;
}

/* 全宽图表 */
.charts-grid.full-width {
  grid-template-columns: 1fr;
}

/* 用户行为统计 */
.behavior-stats {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
  padding: 16px;
}

.behavior-item {
  text-align: center;
  padding: 20px;
  background: var(--color-secondary);
  border-radius: 8px;
  transition: transform 0.2s;
}

.behavior-item:hover {
  transform: translateY(-2px);
}

.behavior-label {
  font-size: 13px;
  color: var(--text-color-secondary);
  margin-bottom: 8px;
}

.behavior-value {
  font-size: 24px;
  font-weight: 600;
  color: var(--color-primary);
}

/* 好友关系统计 */
.friend-stats {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
  padding: 16px;
}

.friend-item {
  text-align: center;
  padding: 20px;
  background: var(--color-secondary);
  border-radius: 8px;
  transition: transform 0.2s;
}

.friend-item:hover {
  transform: translateY(-2px);
}

.friend-label {
  font-size: 13px;
  color: var(--text-color-secondary);
  margin-bottom: 8px;
}

.friend-value {
  font-size: 24px;
  font-weight: 600;
  color: var(--color-primary);
}

/* 安全告警统计 */
.security-stats {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
  padding: 16px;
}

.security-item {
  text-align: center;
  padding: 20px;
  background: var(--color-secondary);
  border-radius: 8px;
  transition: all 0.2s;
}

.security-item:hover {
  transform: translateY(-2px);
}

.security-item.warning {
  background: rgba(255, 152, 0, 0.1);
  border: 1px solid rgba(255, 152, 0, 0.3);
}

.security-item.warning .security-value {
  color: #ff9800;
}

.security-item.danger {
  background: rgba(244, 67, 54, 0.1);
  border: 1px solid rgba(244, 67, 54, 0.3);
}

.security-item.danger .security-value {
  color: #f44336;
}

.security-label {
  font-size: 13px;
  color: var(--text-color-secondary);
  margin-bottom: 8px;
}

.security-value {
  font-size: 24px;
  font-weight: 600;
  color: var(--color-primary);
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

  .behavior-stats,
  .friend-stats,
  .security-stats {
    grid-template-columns: 1fr;
  }
}
</style>
