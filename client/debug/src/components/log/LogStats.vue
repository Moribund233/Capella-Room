<script setup lang="ts">
import { computed } from 'vue'
import { useWebSocketStore, type LogEntry } from '@/stores/websocket'
import { BarChart3, AlertCircle, AlertTriangle, Info, FileText } from 'lucide-vue-next'

/**
 * 日志统计组件
 * 显示日志数量统计和级别分布
 */

const wsStore = useWebSocketStore()

/**
 * 日志统计数据
 */
const stats = computed(() => {
  const logs = wsStore.logs
  const total = logs.length

  const errorCount = logs.filter(l => l.level.toLowerCase() === 'error').length
  const warnCount = logs.filter(l => l.level.toLowerCase() === 'warn').length
  const infoCount = logs.filter(l => l.level.toLowerCase() === 'info').length
  const debugCount = logs.filter(l => l.level.toLowerCase() === 'debug').length

  return {
    total,
    error: errorCount,
    warn: warnCount,
    info: infoCount,
    debug: debugCount
  }
})

/**
 * 模块分布统计
 */
const moduleStats = computed(() => {
  const logs = wsStore.logs
  const moduleCount = new Map<string, number>()

  logs.forEach(log => {
    const count = moduleCount.get(log.target) || 0
    moduleCount.set(log.target, count + 1)
  })

  // 按数量排序，取前5个
  return Array.from(moduleCount.entries())
    .sort((a, b) => b[1] - a[1])
    .slice(0, 5)
})

/**
 * 计算百分比
 */
function getPercentage(count: number): number {
  if (stats.value.total === 0) return 0
  return Math.round((count / stats.value.total) * 100)
}

/**
 * 获取级别颜色
 */
function getLevelColor(level: string): string {
  switch (level.toLowerCase()) {
    case 'error':
      return '#f5222d'
    case 'warn':
      return '#faad14'
    case 'info':
      return '#1890ff'
    case 'debug':
      return '#52c41a'
    default:
      return '#8c8c8c'
  }
}

/**
 * 获取级别图标
 */
function getLevelIcon(level: string) {
  switch (level.toLowerCase()) {
    case 'error':
      return AlertCircle
    case 'warn':
      return AlertTriangle
    case 'info':
      return Info
    default:
      return FileText
  }
}
</script>

<template>
  <div class="log-stats">
    <div class="stats-header">
      <n-icon size="18"><BarChart3 /></n-icon>
      <span class="stats-title">日志统计</span>
    </div>

    <n-divider style="margin: 12px 0" />

    <!-- 总览卡片 -->
    <div class="stats-overview">
      <div class="stat-card total">
        <div class="stat-value">{{ stats.total }}</div>
        <div class="stat-label">总日志数</div>
      </div>
    </div>

    <!-- 级别分布 -->
    <div class="stats-section">
      <div class="section-title">级别分布</div>
      <div class="level-stats">
        <div class="level-stat-item">
          <div class="level-info">
            <n-icon size="14" :color="getLevelColor('error')">
              <AlertCircle />
            </n-icon>
            <span class="level-name">错误</span>
          </div>
          <n-progress
            type="line"
            :percentage="getPercentage(stats.error)"
            :color="getLevelColor('error')"
            :show-indicator="false"
            :height="4"
          />
          <span class="level-count">{{ stats.error }}</span>
        </div>

        <div class="level-stat-item">
          <div class="level-info">
            <n-icon size="14" :color="getLevelColor('warn')">
              <AlertTriangle />
            </n-icon>
            <span class="level-name">警告</span>
          </div>
          <n-progress
            type="line"
            :percentage="getPercentage(stats.warn)"
            :color="getLevelColor('warn')"
            :show-indicator="false"
            :height="4"
          />
          <span class="level-count">{{ stats.warn }}</span>
        </div>

        <div class="level-stat-item">
          <div class="level-info">
            <n-icon size="14" :color="getLevelColor('info')">
              <Info />
            </n-icon>
            <span class="level-name">信息</span>
          </div>
          <n-progress
            type="line"
            :percentage="getPercentage(stats.info)"
            :color="getLevelColor('info')"
            :show-indicator="false"
            :height="4"
          />
          <span class="level-count">{{ stats.info }}</span>
        </div>

        <div class="level-stat-item">
          <div class="level-info">
            <n-icon size="14" :color="getLevelColor('debug')">
              <FileText />
            </n-icon>
            <span class="level-name">调试</span>
          </div>
          <n-progress
            type="line"
            :percentage="getPercentage(stats.debug)"
            :color="getLevelColor('debug')"
            :show-indicator="false"
            :height="4"
          />
          <span class="level-count">{{ stats.debug }}</span>
        </div>
      </div>
    </div>

    <!-- 模块分布 -->
    <div v-if="moduleStats.length > 0" class="stats-section">
      <div class="section-title">活跃模块 (Top 5)</div>
      <div class="module-stats">
        <div
          v-for="[module, count] in moduleStats"
          :key="module"
          class="module-stat-item"
        >
          <n-tag size="tiny" class="module-tag">{{ module }}</n-tag>
          <n-progress
            type="line"
            :percentage="getPercentage(count)"
            :show-indicator="false"
            :height="4"
          />
          <span class="module-count">{{ count }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.log-stats {
  padding: 16px;
  background: #fff;
  border-radius: 8px;
}

.stats-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 500;
  color: #262626;
}

.stats-title {
  font-size: 14px;
}

.stats-overview {
  margin-bottom: 16px;
}

.stat-card {
  text-align: center;
  padding: 16px;
  border-radius: 8px;
  background: linear-gradient(135deg, #1890ff 0%, #36cfc9 100%);
  color: #fff;
}

.stat-card.total {
  background: linear-gradient(135deg, #722ed1 0%, #eb2f96 100%);
}

.stat-value {
  font-size: 28px;
  font-weight: 600;
  line-height: 1;
}

.stat-label {
  font-size: 12px;
  margin-top: 4px;
  opacity: 0.9;
}

.stats-section {
  margin-bottom: 16px;
}

.stats-section:last-child {
  margin-bottom: 0;
}

.section-title {
  font-size: 12px;
  color: #8c8c8c;
  margin-bottom: 12px;
}

.level-stats {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.level-stat-item {
  display: grid;
  grid-template-columns: 60px 1fr 40px;
  align-items: center;
  gap: 8px;
}

.level-info {
  display: flex;
  align-items: center;
  gap: 4px;
}

.level-name {
  font-size: 12px;
  color: #595959;
}

.level-count {
  font-size: 12px;
  color: #8c8c8c;
  text-align: right;
}

.module-stats {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.module-stat-item {
  display: grid;
  grid-template-columns: 80px 1fr 40px;
  align-items: center;
  gap: 8px;
}

.module-tag {
  max-width: 80px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.module-count {
  font-size: 12px;
  color: #8c8c8c;
  text-align: right;
}
</style>
