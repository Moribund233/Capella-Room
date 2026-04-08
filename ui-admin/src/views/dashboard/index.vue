<script setup lang="ts">
/**
 * 仪表盘页面
 * 系统概览和数据统计
 */

import { ref, onMounted, onUnmounted } from 'vue'
import { getSystemStats, getActivityStats, getHealthStatus } from '@/api'
import type { SystemStats, ActivityStats, HealthStatus } from '@/api/dashboard'
import { ApiError } from '@/api/client'
import { User, CircleCheck, OfficeBuilding, ChatDotRound, WarnTriangleFilled, InfoFilled, Connection } from '@element-plus/icons-vue'

/** 系统统计数据 */
const systemStats = ref<SystemStats | null>(null)
/** 活跃度统计数据 */
const activityStats = ref<ActivityStats | null>(null)
/** 系统健康状态 */
const healthStatus = ref<HealthStatus | null>(null)
/** 加载状态 */
const loading = ref(true)
/** 错误信息 */
const error = ref<string | null>(null)
/** 自动刷新定时器 */
let refreshTimer: number | null = null

/**
 * 获取所有统计数据
 */
async function fetchDashboardData() {
  try {
    loading.value = true
    error.value = null

    const [statsRes, activityRes, healthRes] = await Promise.all([
      getSystemStats(),
      getActivityStats(),
      getHealthStatus(),
    ])

    systemStats.value = statsRes.data ?? null
    activityStats.value = activityRes.data ?? null
    healthStatus.value = healthRes.data ?? null
  } catch (err) {
    if (err instanceof ApiError) {
      error.value = err.message
    } else {
      error.value = '获取数据失败'
    }
    console.error('Failed to fetch dashboard data:', err)
  } finally {
    loading.value = false
  }
}

/**
 * 格式化数字，超过1000显示为k
 */
function formatNumber(num: number | undefined): string {
  if (num === undefined || num === null) return '-'
  if (num >= 1000000) return (num / 1000000).toFixed(1) + 'M'
  if (num >= 1000) return (num / 1000).toFixed(1) + 'k'
  return num.toString()
}

/**
 * 获取健康状态样式
 */
function getHealthStatusClass(status: string | undefined): string {
  if (!status) return 'status-unknown'
  switch (status.toLowerCase()) {
    case 'healthy':
      return 'status-healthy'
    case 'degraded':
      return 'status-degraded'
    case 'unhealthy':
      return 'status-unhealthy'
    default:
      return 'status-unknown'
  }
}

/**
 * 获取健康状态文本
 */
function getHealthStatusText(status: string | undefined): string {
  if (!status) return '未知'
  switch (status.toLowerCase()) {
    case 'healthy':
      return '健康'
    case 'degraded':
      return '降级'
    case 'unhealthy':
      return '异常'
    default:
      return status
  }
}

onMounted(() => {
  fetchDashboardData()
  // 每30秒自动刷新
  refreshTimer = window.setInterval(fetchDashboardData, 30000)
})

onUnmounted(() => {
  if (refreshTimer) {
    clearInterval(refreshTimer)
  }
})
</script>

<template>
  <div class="dashboard">
    <div class="page-header">
      <h2 class="page-title">仪表盘</h2>
      <p class="page-desc">欢迎回来，这里是系统概览</p>
    </div>

    <!-- 错误提示 -->
    <div v-if="error" class="error-alert">
      <span>{{ error }}</span>
      <button class="retry-btn" @click="fetchDashboardData">重试</button>
    </div>

    <!-- 加载状态 -->
    <div v-if="loading && !systemStats" class="loading-state">
      <div class="loading-spinner"></div>
      <span>加载中...</span>
    </div>

    <template v-else>
      <!-- 核心统计卡片 -->
      <div class="card-grid">
        <!-- 总用户数 -->
        <div class="stat-card">
          <div class="stat-icon" style="background-color: var(--primary-alpha); color: var(--primary)">
            <User class="stat-svg-icon" />
          </div>
          <div class="stat-info">
            <span class="stat-value">{{ formatNumber(systemStats?.total_users) }}</span>
            <span class="stat-label">总用户数</span>
          </div>
        </div>

        <!-- 在线用户 -->
        <div class="stat-card">
          <div class="stat-icon" style="background-color: var(--success-alpha); color: var(--success)">
            <CircleCheck class="stat-svg-icon" />
          </div>
          <div class="stat-info">
            <span class="stat-value">{{ formatNumber(systemStats?.online_users) }}</span>
            <span class="stat-label">在线用户</span>
          </div>
        </div>

        <!-- 总房间数 -->
        <div class="stat-card">
          <div class="stat-icon" style="background-color: var(--warning-alpha); color: var(--warning)">
            <OfficeBuilding class="stat-svg-icon" />
          </div>
          <div class="stat-info">
            <span class="stat-value">{{ formatNumber(systemStats?.total_rooms) }}</span>
            <span class="stat-label">聊天室</span>
          </div>
        </div>

        <!-- 总消息数 -->
        <div class="stat-card">
          <div class="stat-icon" style="background-color: var(--info-alpha); color: var(--info)">
            <ChatDotRound class="stat-svg-icon" />
          </div>
          <div class="stat-info">
            <span class="stat-value">{{ formatNumber(systemStats?.total_messages) }}</span>
            <span class="stat-label">总消息数</span>
          </div>
        </div>
      </div>

      <!-- 第二行统计 -->
      <div class="card-grid secondary-grid">
        <!-- WebSocket 连接数 -->
        <div class="stat-card compact">
          <div class="stat-icon small" style="background-color: var(--primary-alpha); color: var(--primary)">
            <Connection class="stat-svg-icon small" />
          </div>
          <div class="stat-info">
            <span class="stat-value small">{{ formatNumber(systemStats?.active_connections) }}</span>
            <span class="stat-label">WebSocket 连接</span>
          </div>
        </div>

        <!-- 系统健康状态 -->
        <div class="stat-card compact">
          <div class="stat-icon small" :class="getHealthStatusClass(healthStatus?.status)">
            <CircleCheck v-if="healthStatus?.status === 'healthy'" class="stat-svg-icon small" />
            <WarnTriangleFilled v-else-if="healthStatus?.status === 'degraded'" class="stat-svg-icon small" />
            <InfoFilled v-else class="stat-svg-icon small" />
          </div>
          <div class="stat-info">
            <span class="stat-value small">{{ getHealthStatusText(healthStatus?.status) }}</span>
            <span class="stat-label">系统状态</span>
          </div>
        </div>
      </div>

      <!-- 活跃度统计 -->
      <div class="content-section">
        <div class="card">
          <div class="card-header">
            <h3>活跃度统计</h3>
            <span class="refresh-hint" :class="{ 'refreshing': loading }">每30秒自动刷新</span>
          </div>
          <div class="card-body">
            <div class="activity-grid">
              <div class="activity-item">
                <span class="activity-label">日活跃用户</span>
                <span class="activity-value">{{ formatNumber(activityStats?.daily_active_users) }}</span>
              </div>
              <div class="activity-item">
                <span class="activity-label">周活跃用户</span>
                <span class="activity-value">{{ formatNumber(activityStats?.weekly_active_users) }}</span>
              </div>
              <div class="activity-item">
                <span class="activity-label">月活跃用户</span>
                <span class="activity-value">{{ formatNumber(activityStats?.monthly_active_users) }}</span>
              </div>
              <div class="activity-item">
                <span class="activity-label">日消息</span>
                <span class="activity-value">{{ formatNumber(activityStats?.daily_messages) }}</span>
              </div>
              <div class="activity-item">
                <span class="activity-label">周消息</span>
                <span class="activity-value">{{ formatNumber(activityStats?.weekly_messages) }}</span>
              </div>
              <div class="activity-item">
                <span class="activity-label">月消息</span>
                <span class="activity-value">{{ formatNumber(activityStats?.monthly_messages) }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.dashboard {
  max-width: 1400px;
}

.page-header {
  margin-bottom: var(--spacing-6);
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

/* 错误提示 */
.error-alert {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--spacing-4);
  background-color: var(--error-alpha);
  border: 1px solid var(--error);
  border-radius: var(--radius-lg);
  margin-bottom: var(--spacing-6);
  color: var(--error);
}

.retry-btn {
  padding: var(--spacing-2) var(--spacing-4);
  background-color: var(--error);
  color: white;
  border: none;
  border-radius: var(--radius-md);
  cursor: pointer;
  font-size: var(--font-size-sm);
  transition: opacity var(--transition-fast);
}

.retry-btn:hover {
  opacity: 0.9;
}

/* 加载状态 */
.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--spacing-12);
  color: var(--text-secondary);
  gap: var(--spacing-4);
}

.loading-spinner {
  width: 40px;
  height: 40px;
  border: 3px solid var(--border-primary);
  border-top-color: var(--primary);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

/* 统计卡片 */
.card-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: var(--spacing-4);
  margin-bottom: var(--spacing-6);
}

.secondary-grid {
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
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
  transition: box-shadow var(--transition-fast);
}

.stat-card:hover {
  box-shadow: var(--shadow-md);
}

.stat-card.compact {
  padding: var(--spacing-4);
}

.stat-icon {
  flex-shrink: 0;
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-lg);
}

.stat-icon.small {
  width: 40px;
  height: 40px;
}

.stat-svg-icon {
  width: 24px;
  height: 24px;
  fill: currentColor;
}

.stat-svg-icon.small {
  width: 20px;
  height: 20px;
}

.stat-info {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-1);
}

.stat-value {
  font-size: var(--font-size-2xl);
  font-weight: var(--font-weight-bold);
  color: var(--text-primary);
}

.stat-value.small {
  font-size: var(--font-size-xl);
}

.stat-label {
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
}

/* 系统状态样式 */
.status-healthy {
  background-color: var(--success-alpha) !important;
  color: var(--success) !important;
}

.status-degraded {
  background-color: var(--warning-alpha) !important;
  color: var(--warning) !important;
}

.status-unhealthy {
  background-color: var(--error-alpha) !important;
  color: var(--error) !important;
}

.status-unknown {
  background-color: var(--border-primary) !important;
  color: var(--text-secondary) !important;
}

/* 内容区域 */
.content-section {
  margin-top: var(--spacing-6);
}

.card {
  background-color: var(--bg-card);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-sm);
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--spacing-4) var(--spacing-6);
  border-bottom: 1px solid var(--border-secondary);
}

.card-header h3 {
  font-size: var(--font-size-lg);
  font-weight: var(--font-weight-semibold);
  color: var(--text-primary);
}

.refresh-hint {
  font-size: var(--font-size-sm);
  color: var(--text-tertiary);
}

.refresh-hint.refreshing {
  color: var(--primary);
}

.card-body {
  padding: var(--spacing-6);
}

/* 活跃度统计 */
.activity-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: var(--spacing-6);
}

.activity-item {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-2);
  text-align: center;
}

.activity-label {
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
}

.activity-value {
  font-size: var(--font-size-2xl);
  font-weight: var(--font-weight-bold);
  color: var(--text-primary);
}
</style>
