<template>
  <div class="websocket-test">
    <!-- 页面标题 -->
    <div class="page-header">
      <h1 class="page-title">WebSocket 测试</h1>
      <span class="page-subtitle">{{ connectionStats.authenticated }} 个连接</span>
    </div>

    <!-- 提示信息 -->
    <n-alert v-if="connectedCount === 0" type="warning" class="warning-alert">
      <template #icon>
        <n-icon :component="AlertCircle" />
      </template>
      当前没有WebSocket连接，请先前往
      <n-button text type="primary" @click="goToMultiUser">多用户测试页面</n-button>
      创建用户并连接WebSocket
    </n-alert>

    <!-- 连接状态概览 -->
    <n-card v-else class="connection-overview">
      <div class="overview-content">
        <div class="overview-item">
          <n-icon :component="Users" size="24" />
          <span class="label">已连接用户</span>
          <span class="value">{{ connectedCount }}</span>
        </div>
        <div class="overview-item">
          <n-icon :component="DoorOpen" size="24" />
          <span class="label">当前房间</span>
          <span class="value">{{ currentRoomId || '未加入' }}</span>
        </div>
      </div>
    </n-card>

    <!-- 测试模块标签页 -->
    <n-card class="test-tabs-card">
      <n-tabs v-model:value="activeTab" type="line" animated size="large">
        <n-tab-pane name="latency" tab="延迟测试">
          <LatencyTestCard />
        </n-tab-pane>
        <n-tab-pane name="stability" tab="稳定性测试">
          <StabilityTestCard />
        </n-tab-pane>
        <n-tab-pane name="stress" tab="压力测试">
          <StressTestCard />
        </n-tab-pane>
      </n-tabs>
    </n-card>

    <!-- WebSocket日志面板 -->
    <WsLogPanel />
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { NCard, NTabs, NTabPane, NAlert, NButton, NIcon } from 'naive-ui'
import { AlertCircle, Users, DoorOpen } from 'lucide-vue-next'
import { useWsTestStore } from '@/store/wsTest'
import {
  LatencyTestCard,
  StabilityTestCard,
  StressTestCard,
  WsLogPanel,
} from '@/components/test'

const router = useRouter()
const wsStore = useWsTestStore()

// 当前激活的标签页
const activeTab = ref('latency')

// 连接统计
const connectionStats = computed(() => wsStore.connectionStats)
const connectedCount = computed(() => wsStore.connectedCount)
const currentRoomId = computed(() => wsStore.currentRoomId)

// 跳转到多用户页面
function goToMultiUser() {
  router.push('/debug/multi-user')
}
</script>

<style scoped>
.websocket-test {
  padding: 16px;
  max-width: 1200px;
  margin: 0 auto;
  /* 设置高度限制和滚动 */
  height: 100%;
  overflow-y: auto;
}

.page-header {
  display: flex;
  align-items: baseline;
  gap: 12px;
  margin-bottom: 20px;
}

.page-title {
  font-size: 24px;
  font-weight: 600;
  margin: 0;
  color: var(--text-color-1);
}

.page-subtitle {
  font-size: 14px;
  color: var(--text-color-3);
}

.warning-alert {
  margin-bottom: 16px;
}

.connection-overview {
  margin-bottom: 16px;
}

.overview-content {
  display: flex;
  gap: 32px;
}

.overview-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.overview-item .label {
  color: var(--text-color-3);
  font-size: 14px;
}

.overview-item .value {
  font-weight: 600;
  font-size: 16px;
  color: var(--text-color-1);
}

.test-tabs-card {
  margin-bottom: 16px;
}

.test-tabs-card :deep(.n-card__content) {
  padding-top: 0;
}

/* 响应式适配 */
@media (max-width: 768px) {
  .websocket-test {
    padding: 12px;
  }

  .page-header {
    flex-direction: column;
    gap: 4px;
  }

  .page-title {
    font-size: 20px;
  }

  .overview-content {
    flex-direction: column;
    gap: 16px;
  }
}
</style>
