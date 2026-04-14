<script setup lang="ts">
/**
 * WebSocket 测试主页面
 * 采用主视图 + 组件扩展的结构
 * 包含：稳定性测试、延迟测试、压力测试等多个测试卡片
 */
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useWebSocketStore } from '@/stores/websocket'
import { useAuthStore } from '@/stores/auth'
import { storeToRefs } from 'pinia'
import { useMessage } from 'naive-ui'
import {
  Wifi,
  WifiOff,
  Activity,
  Shield,
  Zap,
  BarChart3,
  Settings,
  RefreshCw,
  Play,
  Square,
} from 'lucide-vue-next'

// 导入测试组件
import StabilityTestCard from '@/components/websocket/StabilityTestCard.vue'
import LatencyTestCard from '@/components/websocket/LatencyTestCard.vue'
import StressTestCard from '@/components/websocket/StressTestCard.vue'

const message = useMessage()
const authStore = useAuthStore()
const wsStore = useWebSocketStore()
const { isConnected, status, reconnectAttempts, latency } = storeToRefs(wsStore)

// ========== 状态 ==========
const activeTab = ref('overview')
const autoRefresh = ref(false)
let refreshTimer: ReturnType<typeof setInterval> | null = null

// ========== 计算属性 ==========
const statusText = computed(() => {
  switch (status.value) {
    case 'connected':
      return '已连接'
    case 'connecting':
      return '连接中...'
    case 'reconnecting':
      return `重连中(${reconnectAttempts.value})`
    case 'disconnected':
      return '未连接'
    default:
      return '未知'
  }
})

const statusType = computed(() => {
  switch (status.value) {
    case 'connected':
      return 'success'
    case 'connecting':
    case 'reconnecting':
      return 'warning'
    case 'disconnected':
      return 'error'
    default:
      return 'default'
  }
})

const isReconnecting = computed(() => status.value === 'reconnecting')

const latencyDisplay = computed(() => {
  return latency.value !== null ? `${latency.value}ms` : '-'
})

// ========== 方法 ==========
const connectWebSocket = () => {
  wsStore.connect()
  message.success('正在连接 WebSocket...')
}

const disconnectWebSocket = () => {
  wsStore.disconnect()
  message.info('WebSocket 连接已断开')
}

const toggleAutoRefresh = () => {
  autoRefresh.value = !autoRefresh.value
  if (autoRefresh.value) {
    refreshTimer = setInterval(() => {
      wsStore.ping()
    }, 5000)
    message.success('已开启自动刷新 (5秒)')
  } else {
    if (refreshTimer) {
      clearInterval(refreshTimer)
      refreshTimer = null
    }
    message.info('已关闭自动刷新')
  }
}

// ========== 生命周期 ==========
onMounted(() => {
  // 如果未连接且不在连接中，自动连接
  if (!isConnected.value && status.value !== 'connecting') {
    wsStore.connect()
  }
})

onUnmounted(() => {
  if (refreshTimer) {
    clearInterval(refreshTimer)
  }
})
</script>

<template>
  <div class="page-container">
    <!-- 页面头部 -->
    <div class="page-header">
      <h1 class="page-title">
        <Activity class="icon-lg" style="display: inline; vertical-align: middle; margin-right: 8px" />
        WebSocket 测试中心
      </h1>
      <p class="page-subtitle">全面测试 WebSocket 连接稳定性、延迟和性能</p>
    </div>

    <!-- 连接状态卡片 -->
    <n-card style="margin-bottom: var(--space-lg);">
      <n-space align="center" justify="space-between" wrap>
        <!-- 左侧：状态信息 -->
        <n-space align="center">
          <n-tag :type="statusType" size="large">
            <template #icon>
              <component :is="isConnected ? Wifi : WifiOff" class="icon-sm" />
            </template>
            {{ statusText }}
            <n-spin v-if="status === 'connecting' || status === 'reconnecting'" size="small" style="margin-left: 8px" />
          </n-tag>

          <n-divider vertical />

          <n-space>
            <n-statistic label="当前用户" :value="authStore.username || '未登录'" />
            <n-statistic label="延迟" :value="latency !== null ? `${latency}ms` : '-'" />
            <n-statistic label="重连次数" :value="reconnectAttempts" />
          </n-space>
        </n-space>

        <!-- 右侧：控制按钮 -->
        <n-space>
          <n-button
            type="primary"
            :disabled="isConnected || status === 'connecting'"
            @click="connectWebSocket"
          >
            <template #icon>
              <Play class="icon-sm" />
            </template>
            连接
          </n-button>
          <n-button
            type="error"
            :disabled="!isConnected"
            @click="disconnectWebSocket"
          >
            <template #icon>
              <Square class="icon-sm" />
            </template>
            断开
          </n-button>
          <n-button
            :disabled="!isConnected"
            @click="wsStore.ping()"
          >
            <template #icon>
              <RefreshCw class="icon-sm" />
            </template>
            Ping
          </n-button>
          <n-button
            :type="autoRefresh ? 'success' : 'default'"
            @click="toggleAutoRefresh"
          >
            <template #icon>
              <Zap class="icon-sm" />
            </template>
            {{ autoRefresh ? '自动刷新中' : '自动刷新' }}
          </n-button>
        </n-space>
      </n-space>
    </n-card>

    <!-- 测试标签页 -->
    <n-tabs v-model:value="activeTab" type="line" animated>
      <!-- 概览 -->
      <n-tab-pane name="overview" tab="概览">
        <n-grid :cols="3" :x-gap="16" :y-gap="16">
          <n-grid-item>
            <n-card title="连接状态" class="overview-card">
              <n-space vertical>
                <n-statistic label="连接状态">
                  <n-tag :type="statusType">{{ statusText }}</n-tag>
                </n-statistic>
                <n-statistic label="认证状态">
                  <n-tag :type="authStore.isAuthenticated ? 'success' : 'error'">
                    {{ authStore.isAuthenticated ? '已认证' : '未认证' }}
                  </n-tag>
                </n-statistic>
                <n-statistic label="用户角色" :value="authStore.roleText" />
              </n-space>
            </n-card>
          </n-grid-item>

          <n-grid-item>
            <n-card title="性能指标" class="overview-card">
              <n-space vertical>
                <n-statistic label="当前延迟" :value="latencyDisplay" />
                <n-statistic label="重连次数" :value="reconnectAttempts" />
                <n-statistic label="连接时长" value="--" />
              </n-space>
            </n-card>
          </n-grid-item>

          <n-grid-item>
            <n-card title="快速操作" class="overview-card">
              <n-space vertical>
                <n-button @click="activeTab = 'stability'">
                  <Shield class="icon-sm" style="margin-right: 8px" />
                  稳定性测试
                </n-button>
                <n-button @click="activeTab = 'latency'">
                  <BarChart3 class="icon-sm" style="margin-right: 8px" />
                  延迟测试
                </n-button>
                <n-button @click="activeTab = 'stress'">
                  <Zap class="icon-sm" style="margin-right: 8px" />
                  压力测试
                </n-button>
              </n-space>
            </n-card>
          </n-grid-item>
        </n-grid>
      </n-tab-pane>

      <!-- 稳定性测试 -->
      <n-tab-pane name="stability" tab="稳定性测试">
        <StabilityTestCard />
      </n-tab-pane>

      <!-- 延迟测试 -->
      <n-tab-pane name="latency" tab="延迟测试">
        <LatencyTestCard />
      </n-tab-pane>

      <!-- 压力测试 -->
      <n-tab-pane name="stress" tab="压力测试">
        <StressTestCard />
      </n-tab-pane>
    </n-tabs>
  </div>
</template>

<style scoped>
.page-container {
  padding: var(--space-lg);
}

.page-header {
  margin-bottom: var(--space-lg);
}

.page-title {
  font-size: 24px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: var(--space-sm);
}

.page-subtitle {
  color: var(--text-secondary);
  font-size: 14px;
}

.overview-card {
  height: 100%;
}

.icon-lg {
  width: 24px;
  height: 24px;
}

.icon-sm {
  width: 16px;
  height: 16px;
}

:deep(.n-tabs-nav) {
  margin-bottom: var(--space-lg);
}
</style>
