<script setup lang="ts">
/**
 * 安全告警标签页
 * 提供安全告警查询、状态管理功能
 */

import { ref, onMounted, watch } from 'vue'
import { useAlerts } from '@/composables/useAudit'
import { type SecurityAlert } from '@/types'
import { Warning } from '@element-plus/icons-vue'

const {
  alerts,
  loading,
  error,
  pagination,
  filters,
  fetchAlerts,
  updateStatus,
  resetFilters,
  handlePageChange,
  handlePageSizeChange,
} = useAlerts()

const detailVisible = ref(false)
const selectedAlert = ref<SecurityAlert | null>(null)

onMounted(() => {
  fetchAlerts()
})

// 监听筛选器变化，自动查询
watch(
  () => [filters.severity, filters.status],
  () => {
    fetchAlerts()
  },
  { deep: true }
)

function showDetail(alert: SecurityAlert) {
  selectedAlert.value = alert
  detailVisible.value = true
}

async function handleUpdateStatus(alertId: string, status: 'acknowledged' | 'resolved' | 'ignored') {
  const success = await updateStatus(alertId, status)
  if (success) {
    // 刷新列表
    fetchAlerts()
  }
}

function getSeverityClass(severity: string): string {
  switch (severity) {
    case 'critical':
    case 'high':
      return 'severity-high'
    case 'medium':
      return 'severity-medium'
    case 'low':
      return 'severity-low'
    default:
      return 'severity-info'
  }
}

function getStatusClass(status: string): string {
  switch (status) {
    case 'new':
      return 'status-badge--error'
    case 'acknowledged':
      return 'status-badge--warning'
    case 'resolved':
      return 'status-badge--success'
    case 'ignored':
      return 'status-badge--default'
    default:
      return 'status-badge--info'
  }
}

function formatDateTime(datetime: string): string {
  return new Date(datetime).toLocaleString('zh-CN')
}
</script>

<template>
  <div class="audit-tab-content">
    <!-- 筛选工具栏 -->
    <div class="audit-toolbar">
      <div class="audit-toolbar-item">
        <span class="audit-toolbar-label">严重级别</span>
        <select v-model="filters.severity" class="form-select">
          <option :value="undefined">全部</option>
          <option value="critical">严重</option>
          <option value="high">高</option>
          <option value="medium">中</option>
          <option value="low">低</option>
        </select>
      </div>
      <div class="audit-toolbar-item">
        <span class="audit-toolbar-label">状态</span>
        <select v-model="filters.status" class="form-select">
          <option :value="undefined">全部</option>
          <option value="new">新建</option>
          <option value="acknowledged">已确认</option>
          <option value="resolved">已解决</option>
          <option value="ignored">已忽略</option>
        </select>
      </div>
      <div class="audit-toolbar-actions">
        <button class="btn btn-outline" @click="resetFilters">重置</button>
        <button class="btn btn-primary" @click="fetchAlerts">查询</button>
      </div>
    </div>

    <!-- 错误提示 -->
    <div v-if="error" class="audit-error">
      <span class="audit-error-text">{{ error }}</span>
      <div class="audit-error-actions">
        <button class="btn btn-primary" @click="fetchAlerts">重试</button>
      </div>
    </div>

    <!-- 加载状态 -->
    <div v-else-if="loading && alerts.length === 0" class="audit-loading">
      <div class="audit-loading-spinner"></div>
      <span>加载中...</span>
    </div>

    <!-- 空状态 -->
    <div v-else-if="alerts.length === 0" class="audit-empty">
      <el-icon class="audit-empty-icon" :size="48"><Warning /></el-icon>
      <span class="audit-empty-text">暂无安全告警</span>
      <span class="audit-empty-hint">系统运行正常</span>
    </div>

    <!-- 数据表格 -->
    <div v-else class="audit-table-wrapper">
      <table class="audit-table">
        <thead>
          <tr>
            <th class="audit-table-col--datetime">触发时间</th>
            <th>规则名称</th>
            <th class="audit-table-col--severity">严重级别</th>
            <th>告警信息</th>
            <th class="audit-table-col--status">状态</th>
            <th class="audit-table-col--actions">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="alert in alerts" :key="alert.id">
            <td class="audit-table-col--datetime">{{ formatDateTime(alert.triggered_at) }}</td>
            <td>{{ alert.rule_name }}</td>
            <td class="audit-table-col--severity">
              <span :class="getSeverityClass(alert.severity)">{{ alert.severity }}</span>
            </td>
            <td>{{ alert.message }}</td>
            <td class="audit-table-col--status">
              <span class="status-badge" :class="getStatusClass(alert.status)">{{ alert.status }}</span>
            </td>
            <td class="audit-table-col--actions">
              <button class="btn btn-sm btn-ghost" @click="showDetail(alert)">详情</button>
              <template v-if="alert.status === 'new'">
                <button class="btn btn-sm btn-primary" @click="handleUpdateStatus(alert.id, 'acknowledged')">确认</button>
              </template>
              <template v-if="alert.status === 'new' || alert.status === 'acknowledged'">
                <button class="btn btn-sm btn-outline" @click="handleUpdateStatus(alert.id, 'resolved')">解决</button>
              </template>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- 分页器 -->
    <div v-if="alerts.length > 0" class="audit-pagination">
      <span class="audit-pagination-info">
        共 {{ pagination.total }} 条，第 {{ pagination.page }} 页
      </span>
      <div class="audit-pagination-actions">
        <button
          class="btn btn-sm btn-outline"
          :disabled="pagination.page <= 1"
          @click="handlePageChange(pagination.page - 1)"
        >
          上一页
        </button>
        <button
          class="btn btn-sm btn-outline"
          :disabled="pagination.page * pagination.pageSize >= pagination.total"
          @click="handlePageChange(pagination.page + 1)"
        >
          下一页
        </button>
      </div>
    </div>

    <!-- 详情弹窗 -->
    <div v-if="detailVisible" class="modal" @click.self="detailVisible = false">
      <div class="modal-content">
        <div class="modal-header">
          <h3>告警详情</h3>
          <button class="btn btn-ghost" @click="detailVisible = false">✕</button>
        </div>
        <div class="modal-body" v-if="selectedAlert">
          <div class="audit-detail-section">
            <div class="audit-detail-title">基本信息</div>
            <div class="audit-detail-content">
              <div class="audit-detail-row">
                <span class="audit-detail-label">告警ID</span>
                <span class="audit-detail-value">{{ selectedAlert.id }}</span>
              </div>
              <div class="audit-detail-row">
                <span class="audit-detail-label">规则名称</span>
                <span class="audit-detail-value">{{ selectedAlert.rule_name }}</span>
              </div>
              <div class="audit-detail-row">
                <span class="audit-detail-label">严重级别</span>
                <span class="audit-detail-value" :class="getSeverityClass(selectedAlert.severity)">{{ selectedAlert.severity }}</span>
              </div>
              <div class="audit-detail-row">
                <span class="audit-detail-label">触发时间</span>
                <span class="audit-detail-value">{{ formatDateTime(selectedAlert.triggered_at) }}</span>
              </div>
            </div>
          </div>
          <div class="audit-detail-section">
            <div class="audit-detail-title">告警内容</div>
            <div class="audit-detail-content">
              <div class="audit-detail-row">
                <span class="audit-detail-label">告警信息</span>
                <span class="audit-detail-value">{{ selectedAlert.message }}</span>
              </div>
              <div class="audit-detail-row">
                <span class="audit-detail-label">当前状态</span>
                <span class="audit-detail-value">
                  <span class="status-badge" :class="getStatusClass(selectedAlert.status)">{{ selectedAlert.status }}</span>
                </span>
              </div>
            </div>
          </div>
          <div class="audit-detail-section" v-if="selectedAlert.context && Object.keys(selectedAlert.context).length > 0">
            <div class="audit-detail-title">上下文信息</div>
            <div class="audit-detail-content">
              <pre class="audit-detail-json">{{ JSON.stringify(selectedAlert.context, null, 2) }}</pre>
            </div>
          </div>
          <div class="audit-detail-section" v-if="selectedAlert.acknowledged_by">
            <div class="audit-detail-title">处理记录</div>
            <div class="audit-detail-content">
              <div class="audit-detail-row" v-if="selectedAlert.acknowledged_by">
                <span class="audit-detail-label">确认人</span>
                <span class="audit-detail-value">{{ selectedAlert.acknowledged_by }} ({{ formatDateTime(selectedAlert.acknowledged_at!) }})</span>
              </div>
              <div class="audit-detail-row" v-if="selectedAlert.resolved_by">
                <span class="audit-detail-label">解决人</span>
                <span class="audit-detail-value">{{ selectedAlert.resolved_by }} ({{ formatDateTime(selectedAlert.resolved_at!) }})</span>
              </div>
            </div>
          </div>
          <!-- 操作按钮 -->
          <div class="modal-actions" v-if="selectedAlert.status === 'new' || selectedAlert.status === 'acknowledged'">
            <button
              v-if="selectedAlert.status === 'new'"
              class="btn btn-primary"
              @click="handleUpdateStatus(selectedAlert.id, 'acknowledged'); detailVisible = false"
            >
              确认告警
            </button>
            <button
              class="btn btn-outline"
              @click="handleUpdateStatus(selectedAlert.id, 'resolved'); detailVisible = false"
            >
              标记为已解决
            </button>
            <button
              class="btn btn-ghost"
              @click="handleUpdateStatus(selectedAlert.id, 'ignored'); detailVisible = false"
            >
              忽略
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style>
/* 导入审计系统全局样式 */
@import '@/style/audit.css';
</style>

<style scoped>
/* 表单选择器 */
.form-select {
  padding: var(--spacing-2) var(--spacing-3);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-md);
  background-color: var(--bg-card);
  color: var(--text-primary);
  font-size: var(--font-size-sm);
  min-width: 120px;
}

.form-select:focus {
  outline: none;
  border-color: var(--primary);
}

/* 弹窗 */
.modal {
  position: fixed;
  inset: 0;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: var(--z-modal);
  padding: var(--spacing-4);
}

.modal-content {
  background-color: var(--bg-card);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-lg);
  max-width: 600px;
  width: 100%;
  max-height: 80vh;
  overflow-y: auto;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--spacing-4) var(--spacing-6);
  border-bottom: 1px solid var(--border-secondary);
}

.modal-header h3 {
  font-size: var(--font-size-lg);
  font-weight: var(--font-weight-semibold);
  color: var(--text-primary);
}

.modal-body {
  padding: var(--spacing-6);
}

.modal-actions {
  display: flex;
  gap: var(--spacing-2);
  margin-top: var(--spacing-6);
  padding-top: var(--spacing-4);
  border-top: 1px solid var(--border-secondary);
}
</style>
