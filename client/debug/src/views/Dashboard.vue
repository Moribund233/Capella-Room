<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import {
  Wifi,
  Users,
  MessageSquare,
  Server,
  Activity,
  Clock,
  Shield,
  AlertCircle
} from 'lucide-vue-next'
import {
  getSystemStatus,
  getSystemStats,
  getAdminSystemStats,
  getConnectionInfo,
  type SystemStats,
  type AdminSystemStats,
  type ConnectionInfo,
  getAccessToken
} from '@/api'
import { useWebSocketStore, type LogEntry } from '@/stores/websocket'
import { useMessage } from 'naive-ui'
import { storeToRefs } from 'pinia'

const message = useMessage()
const wsStore = useWebSocketStore()
const { status: wsStatus, logs: wsLogs, logSubscribed, isConnected: wsConnected } = storeToRefs(wsStore)

// ========== 状态 ==========
const serverStatus = ref<'healthy' | 'degraded' | 'unhealthy'>('healthy')
const stats = ref<SystemStats>({
  active_users: 0,
  total_rooms: 0,
  total_messages: 0,
  websocket_connections: 0
})
const adminStats = ref<AdminSystemStats | null>(null)
const connectionInfo = ref<ConnectionInfo | null>(null)
const loading = ref(false)
const isAdmin = ref(false)
const adminStatsError = ref<string | null>(null)

// 本地日志副本（用于显示格式化）
const recentLogs = computed(() => {
  return wsLogs.value.map((log: LogEntry) => ({
    time: new Date(log.timestamp).toLocaleTimeString('zh-CN'),
    level: log.level,
    target: log.target,
    message: log.message,
    fields: log.fields
  }))
})

// 自动刷新定时器
let refreshTimer: ReturnType<typeof setInterval> | null = null

// ========== 计算属性 ==========
const displayStats = computed(() => {
  // 如果有管理员统计，优先使用
  if (adminStats.value) {
    return {
      active_users: adminStats.value.online_users,
      total_rooms: adminStats.value.total_rooms,
      total_messages: adminStats.value.total_messages,
      websocket_connections: stats.value.websocket_connections
    }
  }
  // 否则使用基础统计
  return stats.value
})

// 统计卡片类型
interface StatCard {
  label: string
  value: string
  icon: typeof Server
  status: 'success' | 'normal' | 'warning' | 'error'
}

// ========== 统计数据卡片 ==========
const statCards = computed<StatCard[]>(() => {
  const ds = displayStats.value
  return [
    {
      label: '服务器状态',
      value: serverStatus.value === 'healthy' ? '健康' : serverStatus.value === 'degraded' ? '降级' : '异常',
      icon: Server,
      status: serverStatus.value === 'healthy' ? 'success' : serverStatus.value === 'degraded' ? 'warning' : 'error'
    },
    {
      label: 'WebSocket',
      value: wsConnected.value ? '已连接' : wsStatus.value === 'reconnecting' ? '重连中' : '未连接',
      icon: Wifi,
      status: wsConnected.value ? 'success' : wsStatus.value === 'reconnecting' ? 'warning' : 'error'
    },
    {
      label: '在线用户',
      value: String(ds.active_users),
      icon: Users,
      status: 'normal'
    },
    {
      label: '活跃房间',
      value: String(ds.total_rooms),
      icon: MessageSquare,
      status: 'normal'
    }
  ]
})

// ========== 快捷操作 ==========
const quickActions = [
  { label: '测试 WebSocket', icon: Wifi, desc: '连接并测试实时通信', route: '/websocket' },
  { label: 'API 调试', icon: Activity, desc: '测试 REST API 接口', route: '/api-test' },
  { label: '查看日志', icon: Clock, desc: '查看系统运行日志', route: '/logs' },
  { label: '性能监控', icon: Server, desc: '查看服务器性能指标', route: '/monitor' }
]

// ========== 检查用户权限 ==========
const checkUserRole = () => {
  const token = getAccessToken()
  if (!token) {
    isAdmin.value = false
    return
  }

  try {
    // 解析 JWT token 获取用户角色
    const parts = token.split('.')
    if (parts.length < 2) {
      isAdmin.value = false
      return
    }
    const base64Url = parts[1]
    if (!base64Url) {
      isAdmin.value = false
      return
    }
    const base64 = base64Url.replace(/-/g, '+').replace(/_/g, '/')
    const jsonPayload = decodeURIComponent(
      atob(base64)
        .split('')
        .map((c) => '%' + ('00' + c.charCodeAt(0).toString(16)).slice(-2))
        .join('')
    )
    const payload = JSON.parse(jsonPayload)
    isAdmin.value = payload.role === 'super_admin' || payload.role === 'admin'
  } catch (e) {
    console.error('解析 token 失败:', e)
    isAdmin.value = false
  }
}

// ========== 数据加载 ==========
const loadDashboardData = async () => {
  loading.value = true
  adminStatsError.value = null

  try {
    // 并行加载基础数据
    const [statusData, statsData, infoData] = await Promise.all([
      getSystemStatus().catch(() => null),
      getSystemStats().catch(() => null),
      getConnectionInfo().catch(() => null)
    ])

    if (statusData) {
      serverStatus.value = statusData.status
    }

    if (statsData) {
      stats.value = statsData
    }

    connectionInfo.value = infoData

    // 如果是管理员，加载详细统计
    if (isAdmin.value) {
      try {
        const adminData = await getAdminSystemStats()
        adminStats.value = adminData
      } catch (error: any) {
        console.error('加载管理员统计数据失败:', error)
        adminStatsError.value = error.response?.status === 403
          ? '权限不足，无法访问管理员统计'
          : '加载管理员统计数据失败'
        adminStats.value = null
      }
    }
  } catch (error) {
    console.error('加载仪表盘数据失败:', error)
    message.error('加载数据失败')
  } finally {
    loading.value = false
  }
}

// ========== 刷新数据 ==========
const refreshData = async () => {
  await loadDashboardData()
  message.success('数据已刷新')
}

// ========== 路由跳转 ==========
const navigateTo = (route: string) => {
  // 使用 window.location 进行跳转，因为快捷操作可能对应不同路由
  window.location.href = route
}

// ========== 初始化 ==========
onMounted(() => {
  checkUserRole()
  loadDashboardData()

  // 连接 WebSocket（如果未连接）
  if (!wsConnected.value) {
    wsStore.connect()
  }

  // 每30秒自动刷新统计数据
  refreshTimer = setInterval(loadDashboardData, 30000)
})

onUnmounted(() => {
  if (refreshTimer) {
    clearInterval(refreshTimer)
  }
  // 注意：不在组件卸载时断开 WebSocket，因为它是全局共享的
})
</script>

<template>
  <div class="page-container">
    <div class="page-header">
      <h1 class="page-title">
        <Activity class="icon-lg" style="display: inline; vertical-align: middle; margin-right: 8px" />
        调试仪表盘
      </h1>
      <p class="page-subtitle">
        Seredeli Room 后端服务调试控制台
        <n-tag v-if="isAdmin" type="success" size="small" style="margin-left: 8px">
          <template #icon>
            <Shield style="width: 12px; height: 12px" />
          </template>
          管理员
        </n-tag>
      </p>
    </div>

    <!-- 统计卡片 - Flex紧凑布局 -->
    <div class="card-flex compact" style="margin-bottom: var(--space-xl)">
      <n-card v-for="(stat, index) in statCards" :key="index" class="card-flex-item stat-card">
        <div class="stat-icon">
          <component :is="stat.icon" class="icon-lg" />
        </div>
        <div class="stat-content">
          <div class="stat-value">{{ stat.value }}</div>
          <div class="stat-label">{{ stat.label }}</div>
        </div>
      </n-card>
    </div>

    <!-- 管理员统计信息 - Flex布局 -->
    <n-card v-if="isAdmin && adminStats" title="管理员统计" style="margin-bottom: var(--space-lg)">
      <div class="admin-stats-flex">
        <div class="admin-stat-item">
          <n-statistic label="总用户数" :value="adminStats.total_users" />
        </div>
        <div class="admin-stat-item">
          <n-statistic label="总房间数" :value="adminStats.total_rooms" />
        </div>
        <div class="admin-stat-item">
          <n-statistic label="总消息数" :value="adminStats.total_messages" />
        </div>
        <div class="admin-stat-item">
          <n-statistic label="在线用户" :value="adminStats.online_users" />
        </div>
        <div class="admin-stat-item">
          <n-statistic label="今日新用户" :value="adminStats.today_new_users" />
        </div>
        <div class="admin-stat-item">
          <n-statistic label="今日消息数" :value="adminStats.today_messages" />
        </div>
      </div>
    </n-card>

    <!-- 管理员统计加载错误提示 -->
    <n-alert
      v-if="isAdmin && adminStatsError"
      type="warning"
      :title="adminStatsError"
      closable
      style="margin-bottom: var(--space-lg)"
    >
      <template #icon>
        <AlertCircle />
      </template>
      部分统计数据可能不可用，基础功能仍可正常使用。
    </n-alert>

    <!-- 非管理员提示 -->
    <n-alert
      v-if="!isAdmin"
      type="info"
      title="普通用户视图"
      style="margin-bottom: var(--space-lg)"
    >
      <template #icon>
        <Shield />
      </template>
      您当前以普通用户身份登录，部分详细统计数据需要管理员权限才能查看。
    </n-alert>

    <!-- 主内容区域 - Flex布局 -->
    <div class="dashboard-main-flex">
      <!-- 左侧：日志面板 -->
      <n-card title="系统日志" class="dashboard-log-card">
        <template #header-extra>
          <div style="display: flex; align-items: center; gap: 8px">
            <n-tag v-if="wsConnected" type="success" size="small">已连接</n-tag>
            <n-tag v-else-if="wsStatus === 'reconnecting'" type="warning" size="small">重连中({{ wsStore.reconnectAttempts }})</n-tag>
            <n-tag v-else type="error" size="small">未连接</n-tag>
            <n-tag v-if="logSubscribed" type="info" size="small">接收中</n-tag>
          </div>
        </template>
        <div class="log-panel">
          <div v-if="recentLogs.length === 0" style="text-align: center; color: var(--text-muted); padding: var(--space-lg)">
            暂无日志数据
          </div>
          <div v-for="(log, index) in recentLogs" :key="index" class="log-entry">
            <span class="log-time">[{{ log.time }}]</span>
            <span :class="`log-${log.level}`">[{{ log.level.toUpperCase() }}]</span>
            <span style="color: var(--text-secondary); margin-left: 4px">[{{ log.target }}]</span>
            <span style="color: var(--text-white); margin-left: 8px">{{ log.message }}</span>
          </div>
        </div>
      </n-card>

      <!-- 右侧：快捷操作 -->
      <n-card title="快捷操作" class="dashboard-action-card">
        <div class="quick-actions-flex">
          <n-button
            v-for="(action, index) in quickActions"
            :key="index"
            size="large"
            class="quick-action-btn"
            @click="navigateTo(action.route)"
          >
            <template #icon>
              <component :is="action.icon" class="icon-md" />
            </template>
            <div class="quick-action-content">
              <div class="quick-action-label">{{ action.label }}</div>
              <div class="quick-action-desc">{{ action.desc }}</div>
            </div>
          </n-button>
        </div>
      </n-card>
    </div>

    <!-- 连接信息 - Flex布局 -->
    <n-card title="连接信息" style="margin-top: var(--space-lg)">
      <div class="connection-info-flex">
        <div class="connection-info-item">
          <span class="connection-label">API 地址：</span>
          <span class="connection-value">{{ connectionInfo?.api_url || 'http://localhost:8080' }}</span>
        </div>
        <div class="connection-info-item">
          <span class="connection-label">WebSocket：</span>
          <span class="connection-value">{{ connectionInfo?.websocket_url || 'ws://localhost:8080/ws' }}</span>
        </div>
        <div class="connection-info-item">
          <span class="connection-label">版本：</span>
          <span class="connection-value">{{ connectionInfo?.version || 'v0.9.0' }}</span>
        </div>
      </div>
    </n-card>
  </div>
</template>

<style scoped>
/* 管理员统计 - Flex布局 */
.admin-stats-flex {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-lg);
}

.admin-stat-item {
  flex: 1 1 calc(33.333% - var(--space-lg) * 2 / 3);
  min-width: 150px;
}

/* 主内容区域 - Flex布局 */
.dashboard-main-flex {
  display: flex;
  gap: var(--space-lg);
}

.dashboard-log-card {
  flex: 2;
  min-width: 0;
}

.dashboard-action-card {
  flex: 1;
  min-width: 250px;
}

/* 快捷操作 - Flex布局 */
.quick-actions-flex {
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}

.quick-action-btn {
  justify-content: flex-start;
  height: 60px;
  width: 100%;
}

.quick-action-content {
  text-align: left;
  margin-left: var(--space-sm);
}

.quick-action-label {
  font-weight: 500;
}

.quick-action-desc {
  font-size: 12px;
  color: var(--text-secondary);
}

/* 连接信息 - Flex布局 */
.connection-info-flex {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-md) var(--space-xl);
}

.connection-info-item {
  flex: 1 1 auto;
  min-width: 200px;
}

.connection-label {
  color: var(--text-secondary);
  font-size: 14px;
}

.connection-value {
  color: var(--text-primary);
  font-size: 14px;
  font-weight: 500;
}

/* 统计卡片 - 紧凑设计 */
.stat-card {
  display: inline-flex;
  align-items: center;
  gap: var(--space-sm);
  padding: var(--space-sm) var(--space-md);
  width: auto;
}

.stat-icon {
  width: 40px;
  height: 40px;
  border-radius: var(--radius-md);
  background: var(--gradient-primary);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-white);
  flex-shrink: 0;
}

.stat-content {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

/* 移动端适配 */
@media screen and (max-width: 767px) {
  .admin-stats-flex {
    gap: var(--space-md);
  }

  .admin-stat-item {
    flex: 1 1 calc(50% - var(--space-md) / 2);
    min-width: 120px;
  }

  .dashboard-main-flex {
    flex-direction: column;
  }

  .dashboard-log-card,
  .dashboard-action-card {
    flex: 1 1 100%;
    min-width: auto;
  }

  .quick-action-btn {
    height: 50px;
  }

  .connection-info-flex {
    flex-direction: column;
    gap: var(--space-sm);
  }

  .connection-info-item {
    min-width: auto;
  }
}

/* 统计卡片值和标签 - 紧凑设计 */
.stat-value {
  font-size: 20px;
  font-weight: 600;
  color: var(--text-primary);
  line-height: 1.2;
}

.stat-label {
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.2;
}

/* 日志面板 */
.log-panel {
  background-color: var(--log-bg);
  border-radius: var(--radius-md);
  padding: var(--space-md);
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  line-height: 1.8;
  max-height: 350px;
  overflow-y: auto;
}

.log-entry {
  padding: 2px 0;
}

.log-time {
  color: var(--log-time);
  margin-right: var(--space-sm);
}

.log-info {
  color: var(--log-info);
}

.log-success {
  color: var(--log-success);
}

.log-warning {
  color: var(--log-warning);
}

.log-error {
  color: var(--log-error);
}

@media screen and (max-width: 375px) {
  .admin-stat-item {
    flex: 1 1 100%;
  }

  .stat-icon {
    width: 36px;
    height: 36px;
  }

  .stat-value {
    font-size: 18px;
  }
}
</style>
