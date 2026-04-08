<script setup lang="ts">
/**
 * 审计日志标签页
 * 提供审计日志查询、筛选、导出功能
 */

import { ref, onMounted, onUnmounted, watch } from 'vue'
import { useAuditLogs } from '@/composables/useAudit'
import { type AuditLog } from '@/types'
import { Document, ArrowDown, ArrowRight } from '@element-plus/icons-vue'

const {
  logs,
  loading,
  error,
  pagination,
  filters,
  fetchLogs,
  exportLogs,
  resetFilters,
  handlePageChange,
  handlePageSizeChange,
} = useAuditLogs()

const detailVisible = ref(false)
const selectedLog = ref<AuditLog | null>(null)

// 级联选择器状态
const cascadeSelectRef = ref<HTMLElement | null>(null)
const categoryDropdownVisible = ref(false)
const selectedCategory = ref<string | null>(null)

// 点击外部关闭级联选择器
function handleClickOutside(event: MouseEvent) {
  if (cascadeSelectRef.value && !cascadeSelectRef.value.contains(event.target as Node)) {
    categoryDropdownVisible.value = false
    selectedCategory.value = null
  }
}

onMounted(() => {
  fetchLogs()
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})

// 事件类型分类数据
const eventCategories = [
  {
    key: 'user',
    label: '用户事件',
    events: [
      { value: 'user_login', label: '用户登录' },
      { value: 'user_logout', label: '用户登出' },
      { value: 'user_register', label: '用户注册' },
      { value: 'user_password_change', label: '密码修改' },
      { value: 'user_profile_update', label: '资料更新' },
    ],
  },
  {
    key: 'room',
    label: '房间事件',
    events: [
      { value: 'room_create', label: '创建房间' },
      { value: 'room_delete', label: '删除房间' },
      { value: 'room_member_add', label: '添加成员' },
      { value: 'room_member_remove', label: '移除成员' },
      { value: 'room_member_role_change', label: '角色变更' },
    ],
  },
  {
    key: 'message',
    label: '消息事件',
    events: [
      { value: 'message_send', label: '发送消息' },
      { value: 'message_edit', label: '编辑消息' },
      { value: 'message_delete', label: '删除消息' },
      { value: 'message_report', label: '举报消息' },
    ],
  },
  {
    key: 'admin',
    label: '管理员事件',
    events: [
      { value: 'admin_user_disable', label: '禁用用户' },
      { value: 'admin_user_role_change', label: '用户角色变更' },
      { value: 'admin_user_delete', label: '删除用户' },
      { value: 'admin_room_delete', label: '删除房间' },
      { value: 'admin_message_delete', label: '删除消息' },
      { value: 'admin_config_update', label: '配置更新' },
    ],
  },
  {
    key: 'system',
    label: '系统事件',
    events: [
      { value: 'system_login_failure', label: '登录失败' },
      { value: 'system_unauthorized_access', label: '未授权访问' },
      { value: 'system_rate_limit_triggered', label: '触发限流' },
    ],
  },
  {
    key: 'audit',
    label: '审计系统事件',
    events: [
      { value: 'audit_query', label: '审计查询' },
      { value: 'audit_export', label: '审计导出' },
      { value: 'audit_stats_query', label: '审计统计查询' },
      { value: 'alert_query', label: '告警查询' },
      { value: 'alert_rule_update', label: '告警规则更新' },
      { value: 'audit_cleanup', label: '审计清理' },
    ],
  },
]

// 获取当前选中事件的显示文本
const selectedEventLabel = ref('全部')

function updateSelectedEventLabel() {
  if (!filters.event_type) {
    selectedEventLabel.value = '全部'
    return
  }
  for (const category of eventCategories) {
    const event = category.events.find((e) => e.value === filters.event_type)
    if (event) {
      selectedEventLabel.value = `${category.label} / ${event.label}`
      return
    }
  }
  selectedEventLabel.value = filters.event_type
}

function selectEvent(category: string, eventValue?: string, eventLabel?: string) {
  if (eventValue) {
    filters.event_type = eventValue
    const categoryData = eventCategories.find((c) => c.key === category)
    selectedEventLabel.value = `${categoryData?.label} / ${eventLabel}`
  } else {
    // 选择整个分类
    filters.event_type = undefined
    selectedEventLabel.value = '全部'
  }
  categoryDropdownVisible.value = false
  selectedCategory.value = null
}

function clearEventFilter() {
  filters.event_type = undefined
  selectedEventLabel.value = '全部'
  categoryDropdownVisible.value = false
  selectedCategory.value = null
}

// 监听筛选器变化，自动查询
watch(
  () => [filters.event_type, filters.severity, filters.status],
  () => {
    fetchLogs()
  },
  { deep: true }
)

// 监听事件类型变化，更新显示标签
watch(() => filters.event_type, updateSelectedEventLabel)

function showDetail(log: AuditLog) {
  selectedLog.value = log
  detailVisible.value = true
}

function getSeverityClass(severity: string): string {
  switch (severity) {
    case 'critical':
      return 'severity-critical'
    case 'error':
      return 'severity-error'
    case 'warning':
      return 'severity-warning'
    case 'info':
      return 'severity-info'
    default:
      return 'severity-info'
  }
}

function getStatusClass(status: string): string {
  switch (status) {
    case 'success':
      return 'status-badge--success'
    case 'failure':
      return 'status-badge--error'
    default:
      return 'status-badge--default'
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
        <span class="audit-toolbar-label">事件类型</span>
        <div class="cascade-select" ref="cascadeSelectRef">
          <div
            class="cascade-select-trigger"
            :class="{ 'cascade-select-trigger--active': categoryDropdownVisible }"
            @click="categoryDropdownVisible = !categoryDropdownVisible"
          >
            <span class="cascade-select-value">{{ selectedEventLabel }}</span>
            <el-icon class="cascade-select-arrow" :size="16"><ArrowDown /></el-icon>
          </div>
          <div v-if="categoryDropdownVisible" class="cascade-select-dropdown">
            <!-- 分类列表 -->
            <div class="cascade-select-menu">
              <div
                v-for="category in eventCategories"
                :key="category.key"
                class="cascade-select-item"
                :class="{ 'cascade-select-item--active': selectedCategory === category.key }"
                @mouseenter="selectedCategory = category.key"
                @click="selectEvent(category.key)"
              >
                <span class="cascade-select-item-label">{{ category.label }}</span>
                <el-icon class="cascade-select-item-arrow" :size="14"><ArrowRight /></el-icon>
              </div>
            </div>
            <!-- 子菜单 -->
            <div v-if="selectedCategory" class="cascade-select-submenu">
              <div
                v-for="event in eventCategories.find((c) => c.key === selectedCategory)?.events"
                :key="event.value"
                class="cascade-select-subitem"
                :class="{ 'cascade-select-subitem--active': filters.event_type === event.value }"
                @click="selectEvent(selectedCategory!, event.value, event.label)"
              >
                {{ event.label }}
              </div>
            </div>
          </div>
        </div>
      </div>
      <div class="audit-toolbar-item">
        <span class="audit-toolbar-label">严重级别</span>
        <select v-model="filters.severity" class="form-select">
          <option :value="undefined">全部</option>
          <option value="critical">严重</option>
          <option value="error">错误</option>
          <option value="warning">警告</option>
          <option value="info">信息</option>
        </select>
      </div>
      <div class="audit-toolbar-item">
        <span class="audit-toolbar-label">状态</span>
        <select v-model="filters.status" class="form-select">
          <option :value="undefined">全部</option>
          <option value="success">成功</option>
          <option value="failure">失败</option>
        </select>
      </div>
      <div class="audit-toolbar-actions">
        <button class="btn btn-outline" @click="resetFilters">重置</button>
        <button class="btn btn-primary" @click="fetchLogs">查询</button>
        <button class="btn btn-secondary" @click="exportLogs('json')">导出JSON</button>
        <button class="btn btn-secondary" @click="exportLogs('csv')">导出CSV</button>
      </div>
    </div>

    <!-- 错误提示 -->
    <div v-if="error" class="audit-error">
      <span class="audit-error-text">{{ error }}</span>
      <div class="audit-error-actions">
        <button class="btn btn-primary" @click="fetchLogs">重试</button>
      </div>
    </div>

    <!-- 加载状态 -->
    <div v-else-if="loading && logs.length === 0" class="audit-loading">
      <div class="audit-loading-spinner"></div>
      <span>加载中...</span>
    </div>

    <!-- 空状态 -->
    <div v-else-if="logs.length === 0" class="audit-empty">
      <el-icon class="audit-empty-icon" :size="48"><Document /></el-icon>
      <span class="audit-empty-text">暂无审计日志</span>
      <span class="audit-empty-hint">请调整筛选条件后重试</span>
    </div>

    <!-- 数据表格 -->
    <div v-else class="audit-table-wrapper">
      <table class="audit-table">
        <thead>
          <tr>
            <th class="audit-table-col--datetime">时间</th>
            <th>事件类型</th>
            <th class="audit-table-col--severity">严重级别</th>
            <th>操作者</th>
            <th>操作</th>
            <th>目标</th>
            <th class="audit-table-col--status">状态</th>
            <th class="audit-table-col--actions">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="log in logs" :key="log.id">
            <td class="audit-table-col--datetime">{{ formatDateTime(log.created_at) }}</td>
            <td>{{ log.event_type }}</td>
            <td class="audit-table-col--severity">
              <span :class="getSeverityClass(log.severity)">{{ log.severity }}</span>
            </td>
            <td>{{ log.actor_name || log.actor_id }}</td>
            <td>{{ log.action }}</td>
            <td>{{ log.target_type ? `${log.target_type}${log.target_id ? ' (' + log.target_id.slice(0, 8) + '...)' : ''}` : '-' }}</td>
            <td class="audit-table-col--status">
              <span class="status-badge" :class="getStatusClass(log.status)">{{ log.status }}</span>
            </td>
            <td class="audit-table-col--actions">
              <button class="btn btn-sm btn-ghost" @click="showDetail(log)">详情</button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- 分页器 -->
    <div v-if="logs.length > 0" class="audit-pagination">
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
          <h3>日志详情</h3>
          <button class="btn btn-ghost" @click="detailVisible = false">✕</button>
        </div>
        <div class="modal-body" v-if="selectedLog">
          <div class="audit-detail-section">
            <div class="audit-detail-title">基本信息</div>
            <div class="audit-detail-content">
              <div class="audit-detail-row">
                <span class="audit-detail-label">日志ID</span>
                <span class="audit-detail-value">{{ selectedLog.id }}</span>
              </div>
              <div class="audit-detail-row">
                <span class="audit-detail-label">时间</span>
                <span class="audit-detail-value">{{ formatDateTime(selectedLog.created_at) }}</span>
              </div>
              <div class="audit-detail-row">
                <span class="audit-detail-label">事件类型</span>
                <span class="audit-detail-value">{{ selectedLog.event_type }}</span>
              </div>
              <div class="audit-detail-row">
                <span class="audit-detail-label">严重级别</span>
                <span class="audit-detail-value" :class="getSeverityClass(selectedLog.severity)">{{ selectedLog.severity }}</span>
              </div>
            </div>
          </div>
          <div class="audit-detail-section">
            <div class="audit-detail-title">操作信息</div>
            <div class="audit-detail-content">
              <div class="audit-detail-row">
                <span class="audit-detail-label">操作者</span>
                <span class="audit-detail-value">{{ selectedLog.actor_name || selectedLog.actor_id }} ({{ selectedLog.actor_type }})</span>
              </div>
              <div class="audit-detail-row">
                <span class="audit-detail-label">操作</span>
                <span class="audit-detail-value">{{ selectedLog.action }}</span>
              </div>
              <div class="audit-detail-row">
                <span class="audit-detail-label">目标</span>
                <span class="audit-detail-value">{{ selectedLog.target_type ? `${selectedLog.target_type}${selectedLog.target_id ? ' (' + selectedLog.target_id + ')' : ''}` : '-' }}</span>
              </div>
              <div class="audit-detail-row">
                <span class="audit-detail-label">状态</span>
                <span class="audit-detail-value">
                  <span class="status-badge" :class="getStatusClass(selectedLog.status)">{{ selectedLog.status }}</span>
                </span>
              </div>
            </div>
          </div>
          <div class="audit-detail-section" v-if="selectedLog.ip_address || selectedLog.user_agent">
            <div class="audit-detail-title">客户端信息</div>
            <div class="audit-detail-content">
              <div class="audit-detail-row" v-if="selectedLog.ip_address">
                <span class="audit-detail-label">IP地址</span>
                <span class="audit-detail-value">{{ selectedLog.ip_address }}</span>
              </div>
              <div class="audit-detail-row" v-if="selectedLog.user_agent">
                <span class="audit-detail-label">User Agent</span>
                <span class="audit-detail-value">{{ selectedLog.user_agent }}</span>
              </div>
            </div>
          </div>
          <div class="audit-detail-section" v-if="selectedLog.details">
            <div class="audit-detail-title">详细信息</div>
            <div class="audit-detail-content">
              <pre class="audit-detail-json">{{ JSON.stringify(selectedLog.details, null, 2) }}</pre>
            </div>
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

/* 级联选择器 */
.cascade-select {
  position: relative;
  display: inline-block;
}

.cascade-select-trigger {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--spacing-2);
  padding: var(--spacing-2) var(--spacing-3);
  min-width: 180px;
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-md);
  background-color: var(--bg-card);
  color: var(--text-primary);
  font-size: var(--font-size-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.cascade-select-trigger:hover {
  border-color: var(--primary);
}

.cascade-select-trigger--active {
  border-color: var(--primary);
  box-shadow: 0 0 0 2px var(--primary-alpha);
}

.cascade-select-value {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cascade-select-arrow {
  flex-shrink: 0;
  color: var(--text-secondary);
  transition: transform var(--transition-fast);
}

.cascade-select-trigger--active .cascade-select-arrow {
  transform: rotate(180deg);
}

.cascade-select-dropdown {
  position: absolute;
  top: 100%;
  left: 0;
  margin-top: var(--spacing-1);
  display: flex;
  background-color: var(--bg-card);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-lg);
  z-index: var(--z-dropdown);
  min-width: 320px;
}

.cascade-select-menu {
  min-width: 120px;
  padding: var(--spacing-1);
  border-right: 1px solid var(--border-secondary);
}

.cascade-select-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--spacing-2) var(--spacing-3);
  border-radius: var(--radius-sm);
  cursor: pointer;
  font-size: var(--font-size-sm);
  color: var(--text-primary);
  transition: background-color var(--transition-fast);
}

.cascade-select-item:hover,
.cascade-select-item--active {
  background-color: var(--bg-secondary);
}

.cascade-select-item-label {
  flex: 1;
}

.cascade-select-item-arrow {
  flex-shrink: 0;
  color: var(--text-secondary);
}

.cascade-select-submenu {
  flex: 1;
  padding: var(--spacing-1);
  min-width: 160px;
}

.cascade-select-subitem {
  padding: var(--spacing-2) var(--spacing-3);
  border-radius: var(--radius-sm);
  cursor: pointer;
  font-size: var(--font-size-sm);
  color: var(--text-primary);
  transition: background-color var(--transition-fast);
}

.cascade-select-subitem:hover {
  background-color: var(--bg-secondary);
}

.cascade-select-subitem--active {
  background-color: var(--primary-alpha);
  color: var(--primary);
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
</style>
