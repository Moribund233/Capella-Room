<script setup lang="ts">
import { ref, h, onMounted } from 'vue'
import { NCard, NPagination, useMessage, useDialog } from 'naive-ui'
import { ClipboardList } from 'lucide-vue-next'
import { AuditLogTable, AuditLogFilter } from '@/components/audit'
import type { AuditLogFilterParams } from '@/components/audit/AuditLogFilter.vue'
import { AuditLogDetailModal } from '@/components/audit'
import { useStatusBar } from '@/composables'
import { auditApi, type AuditLog } from '@/api/audit'

const message = useMessage()
const dialog = useDialog()
const { setContent } = useStatusBar()

// ==================== 数据状态 ====================

/** 审计日志列表数据 */
const data = ref<AuditLog[]>([])
/** 加载状态 */
const loading = ref(false)
/** 总日志数 */
const total = ref(0)
/** 当前页码 */
const page = ref(1)
/** 每页数量 */
const pageSize = ref(20)
/** 当前查看的日志 */
const currentLog = ref<AuditLog | null>(null)

/** 搜索参数 */
const searchParams = ref<AuditLogFilterParams>({
  eventType: null,
  severity: null,
  actorId: null,
  status: null,
  startTime: null,
  endTime: null,
})

/** 当前搜索参数缓存（用于刷新） */
const currentSearchParams = ref<AuditLogFilterParams>({
  eventType: null,
  severity: null,
  actorId: null,
  status: null,
  startTime: null,
  endTime: null,
})

// ==================== 数据获取 ====================

/**
 * 获取审计日志列表
 * @param params 搜索参数
 */
const fetchAuditLogs = async (params: {
  eventType?: string | null
  severity?: string | null
  actorId?: string | null
  status?: string | null
  startTime?: number | null
  endTime?: number | null
  page?: number
  pageSize?: number
} = {}) => {
  loading.value = true

  try {
    const queryParams: Record<string, unknown> = {
      limit: params.pageSize ?? pageSize.value,
      offset: ((params.page ?? page.value) - 1) * (params.pageSize ?? pageSize.value),
    }

    if (params.eventType) queryParams.event_type = params.eventType
    if (params.severity) queryParams.severity = params.severity
    if (params.actorId) queryParams.actor_id = params.actorId
    if (params.status) queryParams.status = params.status
    if (params.startTime) queryParams.start_time = new Date(params.startTime).toISOString()
    if (params.endTime) queryParams.end_time = new Date(params.endTime).toISOString()

    const response = await auditApi.getAuditLogs(queryParams)

    if (response.success && response.data) {
      data.value = response.data.logs
      total.value = response.data.total
      page.value = params.page ?? page.value
      pageSize.value = params.pageSize ?? pageSize.value
      return true
    }
    return false
  } catch (error) {
    console.error('获取审计日志失败:', error)
    message.error('获取审计日志失败')
    return false
  } finally {
    loading.value = false
  }
}

/**
 * 刷新当前列表
 */
const refresh = async () => {
  return fetchAuditLogs({
    eventType: currentSearchParams.value.eventType,
    severity: currentSearchParams.value.severity,
    actorId: currentSearchParams.value.actorId,
    status: currentSearchParams.value.status,
    startTime: currentSearchParams.value.startTime,
    endTime: currentSearchParams.value.endTime,
    page: page.value,
    pageSize: pageSize.value,
  })
}

// ==================== 事件处理 ====================

/**
 * 更新状态栏
 */
const updateStatusBar = () => {
  setContent([
    h(ClipboardList, { size: 14, style: { marginRight: '6px' } }),
    ` 共 ${total.value} 条日志`,
  ])
}

/**
 * 处理搜索
 */
const handleSearch = async (params: AuditLogFilterParams) => {
  searchParams.value = params
  currentSearchParams.value = { ...params }
  page.value = 1

  const success = await fetchAuditLogs({
    eventType: params.eventType,
    severity: params.severity,
    actorId: params.actorId,
    status: params.status,
    startTime: params.startTime,
    endTime: params.endTime,
    page: 1,
    pageSize: pageSize.value,
  })

  if (success) updateStatusBar()
}

/**
 * 重置搜索
 */
const handleReset = async () => {
  searchParams.value = {
    eventType: null,
    severity: null,
    actorId: null,
    status: null,
    startTime: null,
    endTime: null,
  }
  currentSearchParams.value = { ...searchParams.value }
  page.value = 1

  const success = await fetchAuditLogs({ page: 1, pageSize: pageSize.value })

  if (success) {
    updateStatusBar()
    message.success('已重置筛选条件')
  }
}

/**
 * 刷新
 */
const handleRefresh = async () => {
  const success = await refresh()

  if (success) {
    updateStatusBar()
    message.success('刷新成功')
  }
}

/**
 * 分页变化
 */
const handlePageChange = async (newPage: number, newPageSize: number) => {
  const success = await fetchAuditLogs({
    eventType: currentSearchParams.value.eventType,
    severity: currentSearchParams.value.severity,
    actorId: currentSearchParams.value.actorId,
    status: currentSearchParams.value.status,
    startTime: currentSearchParams.value.startTime,
    endTime: currentSearchParams.value.endTime,
    page: newPage,
    pageSize: newPageSize,
  })

  if (success) updateStatusBar()
}

/**
 * 查看日志详情
 */
const handleView = (log: AuditLog) => {
  currentLog.value = log
  dialog.info({
    title: '审计日志详情',
    content: () => h(AuditLogDetailModal, { log }),
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
 * 导出日志
 */
const handleExport = async (params: AuditLogFilterParams) => {
  try {
    const exportParams = {
      format: 'json' as const,
      event_type: params.eventType || undefined,
      severity: params.severity || undefined,
      actor_id: params.actorId || undefined,
      start_time: params.startTime ? new Date(params.startTime).toISOString() : undefined,
      end_time: params.endTime ? new Date(params.endTime).toISOString() : undefined,
    }

    const blob = await auditApi.exportAuditLogs(exportParams)
    const url = window.URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `audit_logs_${new Date().toISOString().slice(0, 10)}.json`
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    window.URL.revokeObjectURL(url)

    message.success('导出成功')
  } catch (error) {
    console.error('导出失败:', error)
    message.error('导出失败')
  }
}

// ==================== 生命周期 ====================

onMounted(async () => {
  const success = await fetchAuditLogs({ page: 1, pageSize: 20 })
  if (success) updateStatusBar()
})
</script>

<template>
  <div class="audit-log-page">
    <div class="page-header">
      <h1 class="page-title">审计日志</h1>
      <p class="page-description">查看系统审计日志，支持按事件类型、严重级别、时间范围筛选</p>
    </div>

    <NCard class="search-card" :bordered="false">
      <AuditLogFilter
        v-bind="searchParams"
        :loading="loading"
        @search="handleSearch"
        @reset="handleReset"
        @refresh="handleRefresh"
        @export="handleExport"
      />
    </NCard>

    <NCard class="table-card" :bordered="false">
      <AuditLogTable
        :data="data"
        :loading="loading"
        @view="handleView"
      />
      <div v-if="total > 0" class="pagination-wrapper">
        <NPagination
          :page="page"
          :page-size="pageSize"
          :item-count="total"
          :page-sizes="[10, 20, 50, 100]"
          show-size-picker
          show-quick-jumper
          @update:page="handlePageChange($event, pageSize)"
          @update:page-size="handlePageChange(1, $event)"
        />
      </div>
    </NCard>
  </div>
</template>

<style scoped>
.audit-log-page {
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

.search-card {
  margin-bottom: 16px;
}

.table-card {
  margin-bottom: 24px;
}

.pagination-wrapper {
  margin-top: 16px;
  display: flex;
  justify-content: flex-end;
}

@media (max-width: 768px) {
  .audit-log-page {
    padding: 16px;
  }

  .page-title {
    font-size: 24px;
  }
}
</style>
