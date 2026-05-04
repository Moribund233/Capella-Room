<template>
  <div class="stats-card">
    <!-- 主要统计数字 -->
    <div class="stats-grid">
      <div class="stat-item" :class="{ active: stats.total > 0 }">
        <div class="stat-value">{{ stats.total }}</div>
        <div class="stat-label">总用户</div>
      </div>
      <div class="stat-item online" :class="{ active: stats.online > 0 }">
        <div class="stat-value">{{ stats.online }}</div>
        <div class="stat-label">在线</div>
      </div>
      <div class="stat-item connected" :class="{ active: stats.connected > 0 }">
        <div class="stat-value">{{ stats.connected }}</div>
        <div class="stat-label">WS连接</div>
      </div>
      <div class="stat-item offline">
        <div class="stat-value">{{ stats.offline }}</div>
        <div class="stat-label">离线</div>
      </div>
    </div>

    <!-- 进度条 -->
    <div class="progress-section">
      <div class="progress-item">
        <div class="progress-header">
          <span class="progress-label">在线率</span>
          <span class="progress-value">{{ onlineRate }}%</span>
        </div>
        <div class="progress-bar">
          <div class="progress-fill online" :style="{ width: onlineRate + '%' }"></div>
        </div>
      </div>
      <div class="progress-item">
        <div class="progress-header">
          <span class="progress-label">WS连接率</span>
          <span class="progress-value">{{ wsRate }}%</span>
        </div>
        <div class="progress-bar">
          <div class="progress-fill connected" :style="{ width: wsRate + '%' }"></div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  stats: {
    total: number
    online: number
    connected: number
    offline: number
  }
}>()

const onlineRate = computed(() => {
  if (props.stats.total === 0) return 0
  return Math.round((props.stats.online / props.stats.total) * 100)
})

const wsRate = computed(() => {
  if (props.stats.total === 0) return 0
  return Math.round((props.stats.connected / props.stats.total) * 100)
})
</script>

<style scoped>
.stats-card {
  background: var(--bg-container);
  border-radius: 12px;
  padding: 16px;
}

/* 统计网格 */
.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
  margin-bottom: 16px;
}

.stat-item {
  text-align: center;
  padding: 8px 4px;
  border-radius: 8px;
  background: var(--bg-default);
  transition: all 0.3s ease;
}

.stat-item.active {
  background: rgba(32, 128, 240, 0.1);
}

.stat-item.online.active {
  background: rgba(32, 128, 240, 0.15);
}

.stat-item.connected.active {
  background: rgba(24, 160, 88, 0.15);
}

.stat-value {
  font-size: 24px;
  font-weight: 700;
  color: var(--text-primary);
  line-height: 1.2;
}

.stat-item.online .stat-value {
  color: #2080f0;
}

.stat-item.connected .stat-value {
  color: #18a058;
}

.stat-item.offline .stat-value {
  color: var(--text-secondary);
}

.stat-label {
  font-size: 11px;
  color: var(--text-secondary);
  margin-top: 2px;
}

/* 进度条区域 */
.progress-section {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding-top: 12px;
  border-top: 1px solid var(--border-color);
}

.progress-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.progress-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 12px;
}

.progress-label {
  color: var(--text-secondary);
}

.progress-value {
  color: var(--text-primary);
  font-weight: 500;
}

.progress-bar {
  height: 4px;
  background: var(--bg-default);
  border-radius: 2px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  border-radius: 2px;
  transition: width 0.3s ease;
}

.progress-fill.online {
  background: linear-gradient(90deg, #2080f0, #4098f7);
}

.progress-fill.connected {
  background: linear-gradient(90deg, #18a058, #36ad6a);
}

/* 桌面端适配 */
@media (min-width: 768px) {
  .stats-card {
    padding: 20px;
  }

  .stats-grid {
    gap: 16px;
    margin-bottom: 20px;
  }

  .stat-item {
    padding: 12px 8px;
  }

  .stat-value {
    font-size: 28px;
  }

  .stat-label {
    font-size: 12px;
    margin-top: 4px;
  }

  .progress-section {
    flex-direction: row;
    gap: 24px;
    padding-top: 16px;
  }

  .progress-item {
    flex: 1;
  }

  .progress-bar {
    height: 6px;
    border-radius: 3px;
  }

  .progress-fill {
    border-radius: 3px;
  }
}
</style>
