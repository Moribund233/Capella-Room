<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import {
  NCard,
  NButton,
  NSpace,
  NTag,
  NSpin,
  NAlert,
  NStatistic,
  NGrid,
  NGridItem,
  NProgress,
  NEmpty,
  useMessage,
} from 'naive-ui'
import {
  Database,
  RefreshCw,
  Activity,
  Clock,
  Layers,
  Zap,
  AlertTriangle,
  CheckCircle,
  XCircle,
  Server,
  Radio,
  MessageSquare,
  Users,
} from 'lucide-vue-next'
import { redisApi, type RedisStatus, type RedisStats } from '@/api/redis'
import { configApi, type ConfigSyncStatus } from '@/api/config'
import { useAuthStore } from '@/store'

const message = useMessage()
const authStore = useAuthStore()

/** Redis 连接状态 */
const status = ref<RedisStatus | null>(null)
/** Redis 统计信息 */
const stats = ref<RedisStats | null>(null)
/** 配置同步状态 */
const syncStatus = ref<ConfigSyncStatus | null>(null)
/** 加载状态 */
const loading = ref(false)
/** 刷新中状态 */
const refreshing = ref(false)
/** 同步中状态 */
const syncing = ref(false)
/** 自动刷新定时器 */
let autoRefreshTimer: ReturnType<typeof setInterval> | null = null

/** 是否为SuperAdmin */
const isSuperAdmin = computed(() => authStore.userInfo?.role === 'super_admin')

/** 连接状态显示 */
const connectionStatus = computed(() => {
  if (!status.value) return { text: '未知', type: 'default' as const, icon: AlertTriangle }
  if (!status.value.enabled) return { text: '未启用', type: 'warning' as const, icon: AlertTriangle }
  if (status.value.connected) {
    return { text: '已连接', type: 'success' as const, icon: CheckCircle }
  }
  return { text: '未连接', type: 'error' as const, icon: XCircle }
})

/** 运行时间格式化 */
const formatUptime = (seconds: number | undefined): string => {
  if (seconds === undefined || isNaN(seconds)) return '-'
  const days = Math.floor(seconds / 86400)
  const hours = Math.floor((seconds % 86400) / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)

  if (days > 0) return `${days}天 ${hours}小时`
  if (hours > 0) return `${hours}小时 ${minutes}分钟`
  return `${minutes}分钟`
}

/** 字节格式化 */
const formatBytes = (bytes: number | undefined): string => {
  if (bytes === undefined || isNaN(bytes)) return '-'
  if (bytes >= 1024 * 1024 * 1024) return (bytes / (1024 * 1024 * 1024)).toFixed(2) + ' GB'
  if (bytes >= 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(2) + ' MB'
  if (bytes >= 1024) return (bytes / 1024).toFixed(2) + ' KB'
  return bytes + ' B'
}

/** 数字格式化 */
const formatNumber = (num: number | undefined): string => {
  if (num === undefined || isNaN(num)) return '0'
  if (num >= 1000000) return (num / 1000000).toFixed(1) + 'M'
  if (num >= 1000) return (num / 1000).toFixed(1) + 'K'
  return String(num)
}

/**
 * 获取 Redis 状态
 */
const fetchStatus = async () => {
  try {
    const response = await redisApi.getStatus()
    if (response.success && response.data) {
      status.value = response.data
    }
  } catch (error) {
    console.error('获取 Redis 状态失败:', error)
  }
}

/**
 * 获取 Redis 统计
 */
const fetchStats = async () => {
  try {
    const response = await redisApi.getStats()
    if (response.success && response.data) {
      stats.value = response.data
    }
  } catch (error) {
    console.error('获取 Redis 统计失败:', error)
  }
}

/**
 * 获取配置同步状态
 */
const fetchSyncStatus = async () => {
  try {
    const response = await configApi.getSyncStatus()
    if (response.success && response.data) {
      syncStatus.value = response.data
    }
  } catch (error) {
    console.error('获取同步状态失败:', error)
  }
}

/**
 * 刷新所有数据
 */
const refreshAll = async () => {
  loading.value = true
  try {
    await Promise.all([fetchStatus(), fetchStats(), fetchSyncStatus()])
  } finally {
    loading.value = false
  }
}

/**
 * 刷新 Redis 连接
 */
const handleRefreshConnection = async () => {
  if (!isSuperAdmin.value) {
    message.warning('只有超级管理员可以刷新连接')
    return
  }

  refreshing.value = true
  try {
    const response = await redisApi.refreshConnection()
    if (response.success) {
      message.success('Redis 连接已刷新')
      await fetchStatus()
    } else {
      message.error(response.message || '刷新失败')
    }
  } catch (error) {
    console.error('刷新 Redis 连接失败:', error)
    message.error('刷新失败')
  } finally {
    refreshing.value = false
  }
}

/**
 * 触发配置同步
 */
const handleTriggerSync = async () => {
  if (!isSuperAdmin.value) {
    message.warning('只有超级管理员可以触发同步')
    return
  }

  syncing.value = true
  try {
    const response = await configApi.triggerSync()
    if (response.success) {
      message.success('配置同步已触发')
      await fetchSyncStatus()
    } else {
      message.error(response.message || '同步失败')
    }
  } catch (error) {
    console.error('触发同步失败:', error)
    message.error('同步失败')
  } finally {
    syncing.value = false
  }
}

/**
 * 启动自动刷新
 */
const startAutoRefresh = () => {
  autoRefreshTimer = setInterval(() => {
    fetchStatus()
    fetchStats()
  }, 30000) // 30秒刷新一次
}

/**
 * 停止自动刷新
 */
const stopAutoRefresh = () => {
  if (autoRefreshTimer) {
    clearInterval(autoRefreshTimer)
    autoRefreshTimer = null
  }
}

onMounted(() => {
  refreshAll()
  startAutoRefresh()
})

onUnmounted(() => {
  stopAutoRefresh()
})
</script>

<template>
  <div class="redis-status-page">
    <!-- 页面标题 -->
    <div class="page-header">
      <div class="header-left">
        <h2 class="page-title">Redis 状态</h2>
        <p class="page-desc">查看 Redis 连接状态和统计信息</p>
      </div>
      <NSpace>
        <NButton :loading="refreshing" @click="handleRefreshConnection">
          <template #icon>
            <RefreshCw :size="16" />
          </template>
          刷新连接
        </NButton>
        <NButton type="primary" :loading="loading" @click="refreshAll">
          <template #icon>
            <Activity :size="16" />
          </template>
          刷新数据
        </NButton>
      </NSpace>
    </div>

    <NSpin :show="loading">
      <!-- 连接状态卡片 -->
      <NCard class="status-card" :bordered="false">
        <div class="status-header">
          <div class="status-main">
            <component :is="connectionStatus.icon" :size="32" :class="`status-icon ${connectionStatus.type}`" />
            <div class="status-info">
              <h3 class="status-title">
                Redis 服务器
                <NTag :type="connectionStatus.type" size="small">{{ connectionStatus.text }}</NTag>
              </h3>
              <p v-if="status?.nodes?.[0]" class="status-detail">
                {{ status.nodes[0].address }}
                <span v-if="status.nodes[0].latency_ms !== undefined" class="latency">
                  (延迟: {{ status.nodes[0].latency_ms.toFixed(2) }}ms)
                </span>
              </p>
            </div>
          </div>
          <div v-if="stats" class="status-meta">
            <div class="meta-item">
              <Clock :size="16" />
              <span>运行 {{ formatUptime(stats.uptime_seconds) }}</span>
            </div>
          </div>
        </div>
      </NCard>

      <!-- 连接池信息 -->
      <div class="stats-flex-grid">
        <NCard class="stat-card" :bordered="false">
          <NStatistic label="连接池大小">
            <template #prefix>
              <Server :size="20" />
            </template>
            <span class="stat-value">{{ status?.pool_size || 0 }}</span>
          </NStatistic>
        </NCard>

        <NCard class="stat-card" :bordered="false">
          <NStatistic label="活跃连接">
            <template #prefix>
              <Activity :size="20" />
            </template>
            <span class="stat-value">{{ status?.active_connections || 0 }}</span>
          </NStatistic>
        </NCard>

        <NCard class="stat-card" :bordered="false">
          <NStatistic label="空闲连接">
            <template #prefix>
              <Layers :size="20" />
            </template>
            <span class="stat-value">{{ status?.idle_connections || 0 }}</span>
          </NStatistic>
        </NCard>

        <NCard class="stat-card" :bordered="false">
          <NStatistic label="集群模式">
            <template #prefix>
              <Database :size="20" />
            </template>
            <span class="stat-value">{{ status?.cluster_mode ? '是' : '否' }}</span>
          </NStatistic>
        </NCard>
      </div>

      <!-- 性能统计 -->
      <div class="stats-flex-grid">
        <NCard class="stat-card" :bordered="false">
          <NStatistic label="每秒命令">
            <template #prefix>
              <Zap :size="20" />
            </template>
            <span class="stat-value">{{ stats?.ops_per_second || 0 }}</span>
          </NStatistic>
        </NCard>

        <NCard class="stat-card" :bordered="false">
          <NStatistic label="总命令数">
            <template #prefix>
              <Layers :size="20" />
            </template>
            <span class="stat-value">{{ formatNumber(stats?.total_commands_processed) }}</span>
          </NStatistic>
        </NCard>

        <NCard class="stat-card" :bordered="false">
          <NStatistic label="命中率">
            <template #prefix>
              <CheckCircle :size="20" />
            </template>
            <span class="stat-value">{{ ((stats?.hit_rate || 0) * 100).toFixed(1) }}%</span>
          </NStatistic>
        </NCard>

        <NCard class="stat-card" :bordered="false">
          <NStatistic label="Stream消息">
            <template #prefix>
              <MessageSquare :size="20" />
            </template>
            <span class="stat-value">{{ formatNumber(stats?.stream_messages) }}</span>
          </NStatistic>
        </NCard>
      </div>

      <!-- 内存和 Pub/Sub -->
      <NGrid :cols="2" :x-gap="16" :y-gap="16" class="detail-grid">
        <NGridItem>
          <NCard title="内存使用" class="detail-card" :bordered="false">
            <div v-if="stats" class="memory-stats">
              <div class="memory-item">
                <span class="memory-label">已用内存</span>
                <span class="memory-value">{{ formatBytes(stats.memory_used) }}</span>
              </div>
              <div class="memory-item">
                <span class="memory-label">内存峰值</span>
                <span class="memory-value">{{ formatBytes(stats.memory_peak) }}</span>
              </div>
              <div class="memory-progress" v-if="stats.memory_peak > 0">
                <div class="progress-label">
                  <span>内存使用率</span>
                  <span>{{ Math.round((stats.memory_used / stats.memory_peak) * 100) }}%</span>
                </div>
                <NProgress
                  type="line"
                  :percentage="Math.min(100, Math.round((stats.memory_used / stats.memory_peak) * 100))"
                  :show-indicator="false"
                  :height="24"
                />
              </div>
            </div>
            <NEmpty v-else description="暂无数据" />
          </NCard>
        </NGridItem>

        <NGridItem>
          <NCard title="Pub/Sub 统计" class="detail-card" :bordered="false">
            <div v-if="stats" class="pubsub-stats">
              <div class="pubsub-item">
                <Radio :size="20" />
                <div class="pubsub-info">
                  <span class="pubsub-label">频道数</span>
                  <span class="pubsub-value">{{ stats.pubsub_channels }}</span>
                </div>
              </div>
              <div class="pubsub-item">
                <MessageSquare :size="20" />
                <div class="pubsub-info">
                  <span class="pubsub-label">模式数</span>
                  <span class="pubsub-value">{{ stats.pubsub_patterns }}</span>
                </div>
              </div>
              <div class="pubsub-item">
                <Users :size="20" />
                <div class="pubsub-info">
                  <span class="pubsub-label">消费者数</span>
                  <span class="pubsub-value">{{ stats.stream_consumers }}</span>
                </div>
              </div>
            </div>
            <NEmpty v-else description="暂无数据" />
          </NCard>
        </NGridItem>
      </NGrid>

      <!-- 配置同步 -->
      <NCard title="配置同步" class="sync-card" :bordered="false">
        <div v-if="syncStatus" class="sync-status">
          <div class="sync-grid">
            <div class="sync-item">
              <span class="sync-label">同步状态</span>
              <NTag :type="syncStatus.is_syncing ? 'warning' : 'success'">
                {{ syncStatus.is_syncing ? '同步中' : '已同步' }}
              </NTag>
            </div>
            <div class="sync-item">
              <span class="sync-label">节点数量</span>
              <span class="sync-value">{{ syncStatus.node_count }}</span>
            </div>
            <div class="sync-item">
              <span class="sync-label">成功节点</span>
              <span class="sync-value success">{{ syncStatus.success_count }}</span>
            </div>
            <div v-if="syncStatus.last_sync_at" class="sync-item">
              <span class="sync-label">最后同步</span>
              <span class="sync-value">{{ new Date(syncStatus.last_sync_at).toLocaleString() }}</span>
            </div>
          </div>
          <div class="sync-actions">
            <NButton
              type="primary"
              :loading="syncing"
              :disabled="!isSuperAdmin || syncStatus.is_syncing"
              @click="handleTriggerSync"
            >
              <template #icon>
                <RefreshCw :size="16" />
              </template>
              触发同步
            </NButton>
          </div>
        </div>
        <NEmpty v-else description="暂无数据" />
      </NCard>

      <!-- 提示信息 -->
      <NAlert v-if="!isSuperAdmin" type="info" class="permission-alert">
        您当前以管理员身份查看，只有超级管理员可以刷新 Redis 连接和触发配置同步。
      </NAlert>
    </NSpin>
  </div>
</template>

<style scoped>
.redis-status-page {
  padding: 16px;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 16px;
}

.header-left {
  flex: 1;
}

.page-title {
  margin: 0 0 4px;
  font-size: 20px;
  font-weight: 600;
}

.page-desc {
  margin: 0;
  color: var(--text-color-secondary);
  font-size: 14px;
}

.status-card {
  margin-bottom: 16px;
}

.status-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 16px;
}

.status-main {
  display: flex;
  align-items: center;
  gap: 16px;
}

.status-icon {
  flex-shrink: 0;
}

.status-icon.success {
  color: #18a058;
}

.status-icon.error {
  color: #d03050;
}

.status-icon.warning {
  color: #f0a020;
}

.status-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.status-title {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-detail {
  margin: 0;
  color: var(--text-color-secondary);
  font-size: 14px;
}

.latency {
  color: var(--text-color-secondary);
}

.status-meta {
  display: flex;
  gap: 16px;
  flex-wrap: wrap;
}

.meta-item {
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--text-color-secondary);
  font-size: 14px;
}

.stats-flex-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 16px;
  margin-bottom: 16px;
}

.stat-card {
  text-align: center;
  min-width: 0;
}

.stat-card :deep(.n-statistic__label) {
  font-size: 14px;
  color: var(--text-color-secondary);
}

.stat-card :deep(.n-statistic__value) {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.stat-value {
  font-size: 24px;
  font-weight: 600;
  color: var(--text-color-base);
}

.detail-grid {
  margin-bottom: 16px;
}

.detail-card :deep(.n-card-header) {
  font-weight: 600;
}

.memory-stats {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.memory-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px;
  background: var(--card-color);
  border-radius: 8px;
}

.memory-label {
  color: var(--text-color-secondary);
  font-size: 14px;
}

.memory-value {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-color-base);
}

.memory-progress {
  margin-top: 8px;
}

.progress-label {
  display: flex;
  justify-content: space-between;
  margin-bottom: 8px;
  font-size: 14px;
  color: var(--text-color-secondary);
}

.pubsub-stats {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.pubsub-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  background: var(--card-color);
  border-radius: 8px;
  color: var(--text-color-secondary);
}

.pubsub-info {
  flex: 1;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.pubsub-label {
  font-size: 14px;
  color: var(--text-color-secondary);
}

.pubsub-value {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-color-base);
}

.sync-card {
  margin-bottom: 16px;
}

.sync-card :deep(.n-card-header) {
  font-weight: 600;
}

.sync-status {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.sync-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 16px;
}

.sync-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px;
  background: var(--card-color);
  border-radius: 8px;
}

.sync-label {
  color: var(--text-color-secondary);
  font-size: 14px;
}

.sync-value {
  font-weight: 600;
  color: var(--text-color-base);
}

.sync-value.success {
  color: #18a058;
}

.sync-actions {
  display: flex;
  justify-content: flex-end;
}

.permission-alert {
  margin-top: 16px;
}

@media (max-width: 768px) {
  .page-header {
    flex-direction: column;
    gap: 12px;
  }

  .status-header {
    flex-direction: column;
    align-items: flex-start;
  }

  .sync-grid {
    grid-template-columns: 1fr;
  }
}
</style>
