<script setup lang="ts">
import { ref } from 'vue'
import {
  Wifi,
  Users,
  MessageSquare,
  Server,
  Activity,
  Clock,
  CheckCircle,
  AlertCircle
} from 'lucide-vue-next'

const serverStatus = ref('running')
const wsStatus = ref('connected')
const activeUsers = ref(128)
const totalRooms = ref(16)

const stats = [
  { label: '服务器状态', value: '运行中', icon: Server, status: 'success' },
  { label: 'WebSocket', value: '已连接', icon: Wifi, status: 'success' },
  { label: '在线用户', value: '128', icon: Users, status: 'normal' },
  { label: '活跃房间', value: '16', icon: MessageSquare, status: 'normal' }
]

const recentLogs = [
  { time: '10:42:15', level: 'info', message: 'WebSocket 连接已建立' },
  { time: '10:40:32', level: 'success', message: '用户登录成功: user_123' },
  { time: '10:38:10', level: 'info', message: '获取房间列表: 16个房间' },
  { time: '10:35:45', level: 'warning', message: 'API 响应较慢: 2.3s' },
  { time: '10:30:00', level: 'info', message: '系统初始化完成' }
]

const quickActions = [
  { label: '测试 WebSocket', icon: Wifi, desc: '连接并测试实时通信' },
  { label: 'API 调试', icon: Activity, desc: '测试 REST API 接口' },
  { label: '查看日志', icon: Clock, desc: '查看系统运行日志' },
  { label: '性能监控', icon: Server, desc: '查看服务器性能指标' }
]
</script>

<template>
  <div class="page-container">
    <div class="page-header">
      <h1 class="page-title">
        <Activity class="icon-lg" style="display: inline; vertical-align: middle; margin-right: 8px" />
        调试仪表盘
      </h1>
      <p class="page-subtitle">Seredeli Room 后端服务调试控制台</p>
    </div>

    <!-- 统计卡片 -->
    <div class="card-grid" style="margin-bottom: var(--space-xl)">
      <n-card v-for="(stat, index) in stats" :key="index" class="stat-card">
        <div class="stat-icon">
          <component :is="stat.icon" class="icon-lg" />
        </div>
        <div class="stat-content">
          <div class="stat-value">{{ stat.value }}</div>
          <div class="stat-label">{{ stat.label }}</div>
        </div>
      </n-card>
    </div>

    <div style="display: grid; grid-template-columns: 2fr 1fr; gap: var(--space-lg)">
      <!-- 左侧：日志面板 -->
      <n-card title="系统日志">
        <template #header-extra>
          <n-button size="small" text>
            <template #icon>
              <Clock class="icon-sm" />
            </template>
            刷新
          </n-button>
        </template>
        <div class="log-panel">
          <div v-for="(log, index) in recentLogs" :key="index" class="log-entry">
            <span class="log-time">[{{ log.time }}]</span>
            <span :class="`log-${log.level}`">[{{ log.level.toUpperCase() }}]</span>
            <span style="color: var(--text-white); margin-left: 8px">{{ log.message }}</span>
          </div>
        </div>
      </n-card>

      <!-- 右侧：快捷操作 -->
      <n-card title="快捷操作">
        <div style="display: flex; flex-direction: column; gap: var(--space-md)">
          <n-button
            v-for="(action, index) in quickActions"
            :key="index"
            size="large"
            style="justify-content: flex-start; height: 60px"
          >
            <template #icon>
              <component :is="action.icon" class="icon-md" />
            </template>
            <div style="text-align: left; margin-left: var(--space-sm)">
              <div style="font-weight: 500">{{ action.label }}</div>
              <div style="font-size: 12px; color: var(--text-secondary)">{{ action.desc }}</div>
            </div>
          </n-button>
        </div>
      </n-card>
    </div>

    <!-- 连接信息 -->
    <n-card title="连接信息" style="margin-top: var(--space-lg)">
      <n-descriptions :columns="3" bordered>
        <n-descriptions-item label="API 地址">
          http://localhost:8080
        </n-descriptions-item>
        <n-descriptions-item label="WebSocket">
          ws://localhost:8080/ws
        </n-descriptions-item>
        <n-descriptions-item label="数据库">
          PostgreSQL (localhost:5432)
        </n-descriptions-item>
        <n-descriptions-item label="Redis">
          localhost:6379
        </n-descriptions-item>
        <n-descriptions-item label="版本">
          v0.9.0 (Phase 9)
        </n-descriptions-item>
        <n-descriptions-item label="环境">
          Development
        </n-descriptions-item>
      </n-descriptions>
    </n-card>
  </div>
</template>

<style scoped>
.stat-card {
  display: flex;
  align-items: center;
  gap: var(--space-md);
}

.stat-icon {
  width: 56px;
  height: 56px;
  border-radius: var(--radius-md);
  background: var(--gradient-primary);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-white);
}

.stat-content {
  flex: 1;
}

.stat-value {
  font-size: 24px;
  font-weight: 600;
  color: var(--text-primary);
}

.stat-label {
  font-size: 14px;
  color: var(--text-secondary);
}

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
</style>
