<template>
  <div class="home-view">
    <!-- 1. 欢迎区域 -->
    <HomeWelcome />

    <!-- 2. 快速操作 -->
    <QuickActions />

    <!-- 3. 统计数据卡片 -->
    <StatCards />

    <!-- 4. 图表区域 -->
    <n-grid :cols="2" :x-gap="16" :y-gap="16" responsive="screen">
      <n-grid-item span="2 m:1">
        <OnlineTrendChart />
      </n-grid-item>
      <n-grid-item span="2 m:1">
        <RoomActivityChart />
      </n-grid-item>
    </n-grid>

    <!-- 5. 列表区域 -->
    <n-grid :cols="2" :x-gap="16" :y-gap="16" responsive="screen">
      <n-grid-item span="2 m:1">
        <HotRoomsList />
      </n-grid-item>
      <n-grid-item span="2 m:1">
        <SystemTimeline />
      </n-grid-item>
    </n-grid>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import { NGrid, NGridItem } from 'naive-ui'
import {
  HomeWelcome,
  QuickActions,
  StatCards,
  OnlineTrendChart,
  RoomActivityChart,
  HotRoomsList,
  SystemTimeline,
} from '@/components/home'
import { useWebSocketStore } from '@/store'
import { useStatusBar } from '@/composables'

const wsStore = useWebSocketStore()
const statusBar = useStatusBar()

onMounted(() => {
  // 自动连接 WebSocket
  if (!wsStore.isConnected) {
    wsStore.connect()
  }

  // 设置状态栏为简洁模式（主页已有完整信息展示）
  statusBar.setItems([])
})

onUnmounted(() => {
  // 清除状态栏内容
  statusBar.clear()
})
</script>

<style scoped>
.home-view {
  display: flex;
  flex-direction: column;
  gap: 24px;
  padding: 24px;
  max-width: 1400px;
  margin: 0 auto;
}

@media (max-width: 768px) {
  .home-view {
    padding: 16px;
    gap: 16px;
  }
}
</style>
