<template>
  <n-grid :cols="4" :x-gap="16" :y-gap="16" responsive="screen">
    <n-grid-item v-for="stat in stats" :key="stat.key" span="4 s:2 l:1">
      <n-card class="stat-card" :bordered="false">
        <div class="stat-content">
          <div class="stat-icon" :style="{ background: stat.iconBg }">
            <n-icon :size="24" :component="stat.icon" />
          </div>
          <div class="stat-info">
            <n-text class="stat-label" depth="3">{{ stat.label }}</n-text>
            <n-text class="stat-value" :style="{ color: stat.valueColor }">
              {{ stat.value }}
            </n-text>
            <n-text v-if="stat.trend" class="stat-trend" :type="stat.trend > 0 ? 'success' : 'error'">
              {{ stat.trend > 0 ? '↑' : '↓' }} {{ Math.abs(stat.trend) }}%
            </n-text>
          </div>
        </div>
      </n-card>
    </n-grid-item>
  </n-grid>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { NCard, NGrid, NGridItem, NText, NIcon } from 'naive-ui'
import { Users, Home, Zap, MessageCircle } from 'lucide-vue-next'
import { useSystemStatus } from '@/composables'
import type { Component } from 'vue'

interface StatItem {
  key: string
  label: string
  value: string | number
  icon: Component
  iconBg: string
  valueColor: string
  trend?: number
}

const { status: systemStatus } = useSystemStatus({
  interval: 5000,
  autoStart: true,
})

const stats = computed<StatItem[]>(() => [
  {
    key: 'online',
    label: '在线用户',
    value: systemStatus.value.onlineUsers || 0,
    icon: Users,
    iconBg: 'linear-gradient(135deg, #18a058 0%, #36ad6a 100%)',
    valueColor: '#18a058',
    trend: 12,
  },
  {
    key: 'rooms',
    label: '活跃房间',
    value: systemStatus.value.activeRooms || 0,
    icon: Home,
    iconBg: 'linear-gradient(135deg, #2080f0 0%, #4098fc 100%)',
    valueColor: '#2080f0',
    trend: 5,
  },
  {
    key: 'latency',
    label: '网络延迟',
    value: `${systemStatus.value.latency || 0}ms`,
    icon: Zap,
    iconBg: 'linear-gradient(135deg, #f0a020 0%, #fcb040 100%)',
    valueColor: systemStatus.value.latency < 50 ? '#18a058' : systemStatus.value.latency < 100 ? '#f0a020' : '#d03050',
  },
  {
    key: 'messages',
    label: '今日消息',
    value: '1.2K',
    icon: MessageCircle,
    iconBg: 'linear-gradient(135deg, #8b5cf6 0%, #a78bfa 100%)',
    valueColor: '#8b5cf6',
    trend: 23,
  },
])
</script>

<style scoped>
.stat-card {
  transition: all 0.3s ease;
}

.stat-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.1);
}

.stat-content {
  display: flex;
  align-items: center;
  gap: 16px;
}

.stat-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 48px;
  height: 48px;
  border-radius: 12px;
  color: white;
  flex-shrink: 0;
}

.stat-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.stat-label {
  font-size: 12px;
}

.stat-value {
  font-size: 24px;
  font-weight: 700;
  line-height: 1.2;
}

.stat-trend {
  font-size: 12px;
  font-weight: 500;
}
</style>
