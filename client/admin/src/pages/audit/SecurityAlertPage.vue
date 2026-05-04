<script setup lang="ts">
import { ref, h, onMounted } from 'vue'
import { NCard, NSelect, NSpace, NButton, NEmpty, NPagination, useMessage, useDialog } from 'naive-ui'
import { AlertTriangle, RefreshCw } from 'lucide-vue-next'
import { AlertCard, AlertDetailModal } from '@/components/audit'
import { MobileTableCard } from '@/components/common'
import { useStatusBar } from '@/composables'
import { useLayoutStore } from '@/store/layout'
import { auditApi, type AuditAlert, type AlertQueryParams } from '@/api/audit'
import type { MobileColumn, MobileAction } from '@/components/common'

const message = useMessage()
const dialog = useDialog()
const { setContent } = useStatusBar()
const layoutStore = useLayoutStore()
const { isMobile } = layoutStore

// ==================== 数据状态 ====================

/** 告警列表数据 */
const data = ref<AuditAlert[]>([])
/** 加载状态 */
const loading = ref(false)
/** 总告警数 */
const total = ref(0)
/** 当前页码 */
const page = ref(1)
/** 每页数量 */
const pageSize = ref(20)

/** 筛选状态 */
const filterStatus = ref<string | null>(null)
/** 筛选严重级别 */
const filterSeverity = ref<string | null>(null)

// ==================== 选项配置 ====================

/** 状态选项 */
const statusOptions = [
  { label: '全部状态', value: '' },
  { label: '未处理', value: 'new' },
  { label: '已确认', value: 'acknowledged' },
  { label: '已解决', value: 'resolved' },
  { label: '已忽略', value: 'ignored' },
]

/** 严重级别选项 */
const severityOptions = [
  { label: '全部级别', value: '' },
  { label: '信息', value: 'info' },
  { label: '警告', value: 'warning' },
  { label: '错误', value: 'error' },
  { label: '严重', value: 'critical' },
]

/**
 * 严重级别映射配置
 */
const severityConfig: Record<string, { text: string; type: 'default' | 'info' | 'warning' | 'error' | 'success' }> = {
  info: { text: '信息', type: 'info' },
  warning: { text: '警告', type: 'warning' },
  error: { text: '错误', type: 'error' },
  critical: { text: '严重', type: 'error' },
}

/**
 * 状态映射配置
 */
const statusConfig: Record<string, { text: string; type: 'default' | 'info' | 'warning' | 'error' | 'success' }> = {
  new: { text: '未处理', type: 'error' },
  acknowledged: { text: '已确认', type: 'warning' },
  resolved: { text: '已解决', type: 'success' },
  ignored: { text: '已忽略', type: 'default' },
}

/**
 * 格式化日期时间
 * @param dateString 日期字符串
 * @returns 格式化后的日期时间
 */
const formatDateTime = (dateString: string | null): string => {
  if (!dateString) return '-'
  const date = new Date(dateString)
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
}

// ==================== 移动端表格配置 ====================

/**
 * 移动端表格列配置
 */
const mobileColumns: MobileColumn<AuditAlert>[] = [
  {
    key: 'severity',
    title: '严重级别',
    render: (row: AuditAlert) => severityConfig[row.severity]?.text || row.severity,
  },
  {
    key: 'alert_type',
    title: '告警类型',
    render: (row: AuditAlert) => row.alert_type,
  },
  {
    key: 'status',
    title: '状态',
    render: (row: AuditAlert) => statusConfig[row.status]?.text || row.status,
  },
  {
    key: 'source_ip',
    title: '来源IP',
    render: (row: AuditAlert) => row.source_ip || '-',
  },
  {
    key: 'created_at',
    title: '创建时间',
    render: (row: AuditAlert) => formatDateTime(row.created_at),
  },
]

/**
 * 移动端表格操作配置
 */
const mobileActions: MobileAction<AuditAlert>[] = [
  {
    label: '查看',
    type: 'primary',
    onClick: (row: AuditAlert) => handleView(row),
  },
  {
    label: '确认',
    type: 'warning',
    show: (row: AuditAlert) => row.status === 'new',
    onClick: (row: AuditAlert) => handleAcknowledge(row),
  },
  {
    label: '解决',
    type: 'success',
    show: (row: AuditAlert) => row.status === 'new' || row.status === 'acknowledged',
    onClick: (row: AuditAlert) => handleResolve(row),
  },
  {
    label: '忽略',
    type: 'default',
    show: (row: AuditAlert) => row.status === 'new',
    onClick: (row: AuditAlert) => handleIgnore(row),
  },
]

// ==================== 数据获取 ====================

/**
 * 获取告警列表
 */
const fetchAlerts = async (params: AlertQueryParams = {}) => {
  loading.value = true

  try {
    const queryParams: AlertQueryParams = {
      limit: pageSize.value,
      offset: (page.value - 1) * pageSize.value,
      ...params,
    }

    if (filterStatus.value) queryParams.status = filterStatus.value
    if (filterSeverity.value) queryParams.severity = filterSeverity.value

    const response = await auditApi.getAlerts(queryParams)

    if (response.success && response.data) {
      data.value = response.data.alerts
      total.value = response.data.total
      return true
    }
    return false
  } catch (error) {
    console.error('获取安全告警失败:', error)
    message.error('获取安全告警失败')
    return false
  } finally {
    loading.value = false
  }
}

/**
 * 刷新列表
 */
const refresh = async () => {
  return fetchAlerts({
    limit: pageSize.value,
    offset: (page.value - 1) * pageSize.value,
  })
}

// ==================== 事件处理 ====================

/**
 * 更新状态栏
 */
const updateStatusBar = () => {
  setContent([
    h(AlertTriangle, { size: 14, style: { marginRight: '6px' } }),
    ` 共 ${total.value} 条告警`,
  ])
}

/**
 * 处理筛选变化
 */
const handleFilterChange = async () => {
  page.value = 1
  const success = await fetchAlerts()
  if (success) updateStatusBar()
}

/**
 * 处理分页变化
 */
const handlePageChange = async (newPage: number) => {
  page.value = newPage
  const success = await refresh()
  if (success) updateStatusBar()
}

/**
 * 处理刷新
 */
const handleRefresh = async () => {
  const success = await refresh()
  if (success) {
    updateStatusBar()
    message.success('刷新成功')
  }
}

/**
 * 查看告警详情
 */
const handleView = (alert: AuditAlert) => {
  dialog.info({
    title: '告警详情',
    content: () => h(AlertDetailModal, {
      alert,
      onAcknowledge: handleAcknowledge,
      onResolve: handleResolve,
      onIgnore: handleIgnore,
    }),
    showIcon: false,
    closable: true,
    maskClosable: true,
    style: {
      width: 'auto',
      maxWidth: 'calc(100vw - 32px)',
    },
  })
}

/**
 * 确认告警
 */
const handleAcknowledge = async (alert: AuditAlert) => {
  try {
    const response = await auditApi.updateAlertStatus(alert.id, { status: 'acknowledged' })
    if (response.success) {
      message.success('已确认告警')
      await refresh()
      updateStatusBar()
    } else {
      message.error(response.message || '确认失败')
    }
  } catch {
    message.error('确认告警失败')
  }
}

/**
 * 解决告警
 */
const handleResolve = async (alert: AuditAlert) => {
  try {
    const response = await auditApi.updateAlertStatus(alert.id, { status: 'resolved' })
    if (response.success) {
      message.success('已解决告警')
      await refresh()
      updateStatusBar()
    } else {
      message.error(response.message || '解决失败')
    }
  } catch {
    message.error('解决告警失败')
  }
}

/**
 * 忽略告警
 */
const handleIgnore = async (alert: AuditAlert) => {
  dialog.warning({
    title: '确认忽略',
    content: '确定要忽略这条告警吗？忽略后将不再显示在待处理列表中。',
    positiveText: '忽略',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        const response = await auditApi.updateAlertStatus(alert.id, { status: 'ignored' })
        if (response.success) {
          message.success('已忽略告警')
          await refresh()
          updateStatusBar()
        } else {
          message.error(response.message || '忽略失败')
        }
      } catch {
        message.error('忽略告警失败')
      }
    },
  })
}

// ==================== 生命周期 ====================

onMounted(async () => {
  const success = await fetchAlerts()
  if (success) updateStatusBar()
})
</script>

<template>
  <div class="security-alert-page">
    <div class="page-header">
      <h1 class="page-title">安全告警</h1>
      <p class="page-description">管理系统安全告警，支持确认、解决、忽略操作</p>
    </div>

    <!-- 筛选栏 -->
    <NCard class="filter-card" :bordered="false">
      <NSpace align="center" justify="space-between">
        <NSpace align="center" wrap>
          <NSelect
            v-model:value="filterStatus"
            :options="statusOptions"
            placeholder="告警状态"
            clearable
            style="width: 140px"
            @update:value="handleFilterChange"
          />
          <NSelect
            v-model:value="filterSeverity"
            :options="severityOptions"
            placeholder="严重级别"
            clearable
            style="width: 140px"
            @update:value="handleFilterChange"
          />
        </NSpace>
        <NButton @click="handleRefresh" :loading="loading">
          <template #icon>
            <RefreshCw :size="16" />
          </template>
          刷新
        </NButton>
      </NSpace>
    </NCard>

    <!-- 告警列表 - 桌面端使用 AlertCard -->
    <div v-if="!isMobile" class="alerts-container">
      <div v-if="loading && data.length === 0" class="loading-state">
        <NCard v-for="i in 3" :key="i" class="skeleton-card" :bordered="false">
          <div class="skeleton-header">
            <div class="skeleton-title"></div>
            <div class="skeleton-tag"></div>
          </div>
          <div class="skeleton-content">
            <div class="skeleton-line"></div>
            <div class="skeleton-line short"></div>
          </div>
        </NCard>
      </div>

      <NEmpty
        v-else-if="data.length === 0"
        description="暂无安全告警"
        class="empty-state"
      />

      <template v-else>
        <AlertCard
          v-for="alert in data"
          :key="alert.id"
          :alert="alert"
          @view="handleView"
          @acknowledge="handleAcknowledge"
          @resolve="handleResolve"
          @ignore="handleIgnore"
        />
      </template>
    </div>

    <!-- 告警列表 - 移动端使用 MobileTableCard -->
    <NCard v-else class="mobile-table-card" :bordered="false">
      <MobileTableCard
        :data="data"
        :columns="mobileColumns"
        :actions="mobileActions"
        title-column="title"
        empty-text="暂无安全告警"
      />
    </NCard>

    <!-- 分页 -->
    <div v-if="total > pageSize" class="pagination-wrapper">
      <NPagination
        :page="page"
        :page-size="pageSize"
        :item-count="total"
        show-quick-jumper
        @update:page="handlePageChange"
      />
    </div>
  </div>
</template>

<style scoped>
.security-alert-page {
  padding: 24px;
  min-height: 100%;
}

.page-header {
  margin-bottom: 24px;
}

.page-title {
  font-size: 28px;
  font-weight: 600;
  margin: 0 0 8px 0;
  color: var(--text-primary);
}

.page-description {
  font-size: 14px;
  color: var(--text-secondary);
  margin: 0;
}

.filter-card {
  margin-bottom: 16px;
}

.alerts-container {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-bottom: 24px;
}

.empty-state {
  padding: 48px 0;
}

.pagination-wrapper {
  display: flex;
  justify-content: flex-end;
}

/* 骨架屏样式 */
.loading-state {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.skeleton-card {
  opacity: 0.6;
}

.skeleton-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.skeleton-title {
  width: 200px;
  height: 20px;
  background-color: var(--border-color);
  border-radius: 4px;
}

.skeleton-tag {
  width: 60px;
  height: 24px;
  background-color: var(--border-color);
  border-radius: 4px;
}

.skeleton-content {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.skeleton-line {
  width: 100%;
  height: 16px;
  background-color: var(--border-color);
  border-radius: 4px;
}

.skeleton-line.short {
  width: 60%;
}

.mobile-table-card {
  margin-bottom: 24px;
}

@media (max-width: 768px) {
  .security-alert-page {
    padding: 16px;
  }

  .page-title {
    font-size: 24px;
  }
}
</style>
