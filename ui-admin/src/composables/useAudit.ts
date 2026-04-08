/**
 * 审计系统组合式函数
 * 提供审计日志、安全告警、告警规则的通用逻辑
 */

import { ref, reactive, computed } from 'vue'
import {
  listAuditLogs,
  getAuditStats,
  exportAuditLogs,
  listAlerts,
  updateAlertStatus,
  listAlertRules,
  updateAlertRule,
  cleanupAuditLogs,
} from '@/api/audit'
import type {
  AuditLog,
  AuditLogQuery,
  AuditStats,
  SecurityAlert,
  AlertQuery,
  AlertRule,
  UpdateAlertStatusRequest,
  UpdateAlertRuleRequest,
  ExportFormat,
} from '@/types'
import { ApiError } from '@/api/client'

// ==================== 公共逻辑 ====================

/** 分页参数 */
export interface PaginationParams {
  page: number
  pageSize: number
  total: number
}

/** 公共加载状态 */
export function useAuditCommon() {
  const loading = ref(false)
  const error = ref<string | null>(null)

  const pagination = reactive<PaginationParams>({
    page: 1,
    pageSize: 20,
    total: 0,
  })

  const offset = computed(() => (pagination.page - 1) * pagination.pageSize)

  /**
   * 处理错误
   */
  function handleError(err: unknown) {
    if (err instanceof ApiError) {
      error.value = err.message
    } else if (err instanceof Error) {
      error.value = err.message
    } else {
      error.value = '操作失败'
    }
  }

  /**
   * 重置错误
   */
  function resetError() {
    error.value = null
  }

  /**
   * 重置分页
   */
  function resetPagination() {
    pagination.page = 1
    pagination.total = 0
  }

  return {
    loading,
    error,
    pagination,
    offset,
    handleError,
    resetError,
    resetPagination,
  }
}

// ==================== 审计日志 ====================

/**
 * 审计日志组合式函数
 */
export function useAuditLogs() {
  const { loading, error, pagination, offset, handleError, resetError, resetPagination } = useAuditCommon()

  const logs = ref<AuditLog[]>([])
  const stats = ref<AuditStats | null>(null)

  const filters = reactive<AuditLogQuery>({
    event_type: undefined,
    severity: undefined,
    actor_id: undefined,
    target_type: undefined,
    status: undefined,
    start_time: undefined,
    end_time: undefined,
  })

  /**
   * 获取审计日志列表
   */
  async function fetchLogs() {
    loading.value = true
    resetError()

    try {
      const response = await listAuditLogs({
        ...filters,
        limit: pagination.pageSize,
        offset: offset.value,
      })

      if (response.success && response.data) {
        logs.value = response.data.logs
        pagination.total = response.data.total
      }
    } catch (err) {
      handleError(err)
    } finally {
      loading.value = false
    }
  }

  /**
   * 获取审计统计
   */
  async function fetchStats(startTime?: string, endTime?: string) {
    try {
      const response = await getAuditStats(startTime, endTime)
      if (response.success && response.data) {
        stats.value = response.data
      }
    } catch (err) {
      console.error('Failed to fetch audit stats:', err)
    }
  }

  /**
   * 导出日志
   */
  async function exportLogs(format: ExportFormat) {
    try {
      const blob = await exportAuditLogs({
        format,
        event_type: filters.event_type,
        severity: filters.severity,
        start_time: filters.start_time,
        end_time: filters.end_time,
      })

      // 创建下载链接
      const url = window.URL.createObjectURL(blob)
      const link = document.createElement('a')
      link.href = url
      link.download = `audit_logs_${new Date().toISOString().split('T')[0]}.${format}`
      document.body.appendChild(link)
      link.click()
      document.body.removeChild(link)
      window.URL.revokeObjectURL(url)
    } catch (err) {
      handleError(err)
    }
  }

  /**
   * 重置筛选条件
   */
  function resetFilters() {
    filters.event_type = undefined
    filters.severity = undefined
    filters.actor_id = undefined
    filters.target_type = undefined
    filters.status = undefined
    filters.start_time = undefined
    filters.end_time = undefined
    resetPagination()
  }

  /**
   * 处理页码变化
   */
  function handlePageChange(page: number) {
    pagination.page = page
    fetchLogs()
  }

  /**
   * 处理每页条数变化
   */
  function handlePageSizeChange(pageSize: number) {
    pagination.pageSize = pageSize
    pagination.page = 1
    fetchLogs()
  }

  return {
    logs,
    stats,
    loading,
    error,
    pagination,
    filters,
    fetchLogs,
    fetchStats,
    exportLogs,
    resetFilters,
    handlePageChange,
    handlePageSizeChange,
  }
}

// ==================== 安全告警 ====================

/**
 * 安全告警组合式函数
 */
export function useAlerts() {
  const { loading, error, pagination, offset, handleError, resetError, resetPagination } = useAuditCommon()

  const alerts = ref<SecurityAlert[]>([])

  const filters = reactive<AlertQuery>({
    severity: undefined,
    status: undefined,
    start_time: undefined,
    end_time: undefined,
  })

  /**
   * 获取告警列表
   */
  async function fetchAlerts() {
    loading.value = true
    resetError()

    try {
      const response = await listAlerts({
        ...filters,
        limit: pagination.pageSize,
        offset: offset.value,
      })

      if (response.success && response.data) {
        alerts.value = response.data.alerts
        pagination.total = response.data.total
      }
    } catch (err) {
      handleError(err)
    } finally {
      loading.value = false
    }
  }

  /**
   * 更新告警状态
   */
  async function updateStatus(alertId: string, status: UpdateAlertStatusRequest['status']) {
    try {
      const response = await updateAlertStatus(alertId, { status })
      if (response.success && response.data) {
        // 更新本地数据
        const index = alerts.value.findIndex((a) => a.id === alertId)
        if (index !== -1) {
          alerts.value[index] = response.data
        }
      }
      return response.success
    } catch (err) {
      handleError(err)
      return false
    }
  }

  /**
   * 重置筛选条件
   */
  function resetFilters() {
    filters.severity = undefined
    filters.status = undefined
    filters.start_time = undefined
    filters.end_time = undefined
    resetPagination()
  }

  /**
   * 处理页码变化
   */
  function handlePageChange(page: number) {
    pagination.page = page
    fetchAlerts()
  }

  /**
   * 处理每页条数变化
   */
  function handlePageSizeChange(pageSize: number) {
    pagination.pageSize = pageSize
    pagination.page = 1
    fetchAlerts()
  }

  return {
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
  }
}

// ==================== 告警规则 ====================

/**
 * 告警规则组合式函数
 */
export function useAlertRules() {
  const { loading, error, handleError, resetError } = useAuditCommon()

  const rules = ref<AlertRule[]>([])

  /**
   * 获取规则列表
   */
  async function fetchRules() {
    loading.value = true
    resetError()

    try {
      const response = await listAlertRules()
      if (response.success && response.data) {
        rules.value = response.data.rules
      }
    } catch (err) {
      handleError(err)
    } finally {
      loading.value = false
    }
  }

  /**
   * 更新规则
   */
  async function updateRule(ruleId: string, request: UpdateAlertRuleRequest) {
    try {
      const response = await updateAlertRule(ruleId, request)
      if (response.success && response.data) {
        // 更新本地数据
        const index = rules.value.findIndex((r) => r.id === ruleId)
        if (index !== -1) {
          rules.value[index] = response.data
        }
      }
      return response.success
    } catch (err) {
      handleError(err)
      return false
    }
  }

  /**
   * 切换规则启用状态
   */
  async function toggleRule(ruleId: string, enabled: boolean) {
    return updateRule(ruleId, { enabled })
  }

  return {
    rules,
    loading,
    error,
    fetchRules,
    updateRule,
    toggleRule,
  }
}

// ==================== 日志清理 ====================

/**
 * 日志清理组合式函数
 */
export function useAuditCleanup() {
  const loading = ref(false)
  const error = ref<string | null>(null)
  const result = ref<{ deleted: number } | null>(null)

  /**
   * 清理过期日志
   */
  async function cleanup(days: number, archive?: boolean, archiveDir?: string) {
    loading.value = true
    error.value = null
    result.value = null

    try {
      const response = await cleanupAuditLogs(days, archive, archiveDir)
      if (response.success && response.data) {
        result.value = response.data
        return true
      }
      return false
    } catch (err) {
      if (err instanceof ApiError) {
        error.value = err.message
      } else {
        error.value = '清理失败'
      }
      return false
    } finally {
      loading.value = false
    }
  }

  return {
    loading,
    error,
    result,
    cleanup,
  }
}
