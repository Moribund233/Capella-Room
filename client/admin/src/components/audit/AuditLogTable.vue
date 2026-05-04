<script setup lang="ts">
import { h } from 'vue'
import {
  NDataTable,
  NTag,
  NButton,
  NIcon,
  type DataTableColumns,
} from 'naive-ui'
import { Eye } from 'lucide-vue-next'
import { MobileTableCard } from '@/components/common'
import { useLayoutStore } from '@/store/layout'
import type { AuditLog } from '@/api/audit'
import type { MobileColumn, MobileAction } from '@/components/common'

/**
 * 组件属性定义
 */
interface Props {
  /** 审计日志列表数据 */
  data: AuditLog[]
  /** 加载状态 */
  loading?: boolean
}

withDefaults(defineProps<Props>(), {
  loading: false,
})

/**
 * 组件事件定义
 */
interface Emits {
  /** 查看日志详情 */
  (e: 'view', log: AuditLog): void
}

const emit = defineEmits<Emits>()

/**
 * 响应式断点
 */
const layoutStore = useLayoutStore()
const { isMobile } = layoutStore

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
 * 事件类型中文映射
 */
const eventTypeMap: Record<string, string> = {
  user_login: '用户登录',
  user_logout: '用户登出',
  user_register: '用户注册',
  user_password_change: '修改密码',
  user_profile_update: '更新资料',
  room_create: '创建房间',
  room_delete: '删除房间',
  room_member_add: '添加成员',
  room_member_remove: '移除成员',
  room_member_role_change: '变更角色',
  message_send: '发送消息',
  message_edit: '编辑消息',
  message_delete: '删除消息',
  message_report: '举报消息',
  admin_user_disable: '禁用用户',
  admin_user_role_change: '变更用户角色',
  admin_user_delete: '删除用户',
  admin_room_delete: '删除房间',
  admin_message_delete: '删除消息',
  admin_config_update: '更新配置',
  system_login_failure: '登录失败',
  system_unauthorized_access: '未授权访问',
  system_rate_limit_triggered: '触发限流',
  audit_query: '查询审计',
  audit_export: '导出审计',
  audit_stats_query: '查询统计',
  alert_query: '查询告警',
  alert_rule_update: '更新规则',
  audit_cleanup: '清理日志',
  ip_blocked: 'IP 被封禁',
  ip_whitelist_denied: 'IP 白名单拒绝',
  ip_rate_limited: 'IP 被限流',
  ip_list_added: '添加 IP 列表',
  ip_list_removed: '移除 IP 列表',
  ip_list_updated: '更新 IP 列表',
}

/**
 * 格式化日期时间
 * @param dateString 日期字符串
 * @returns 格式化后的日期时间
 */
const formatDateTime = (dateString: string): string => {
  const date = new Date(dateString)
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  })
}

/**
 * 截断文本
 * @param text 原始文本
 * @param maxLength 最大长度
 * @returns 截断后的文本
 */
const truncateText = (text: string, maxLength: number = 60): string => {
  if (text.length <= maxLength) return text
  return text.substring(0, maxLength) + '...'
}

/**
 * 获取事件类型中文名称
 * @param eventType 事件类型
 * @returns 中文名称
 */
const getEventTypeText = (eventType: string): string => {
  return eventTypeMap[eventType] || eventType
}

/**
 * 移动端表格列配置
 */
const mobileColumns: MobileColumn<AuditLog>[] = [
  {
    key: 'event_type',
    title: '事件类型',
    render: (row: AuditLog) => getEventTypeText(row.event_type),
  },
  {
    key: 'severity',
    title: '严重级别',
    render: (row: AuditLog) => {
      const config = severityConfig[row.severity]
      return config?.text || row.severity
    },
  },
  {
    key: 'actor_name',
    title: '操作者',
    render: (row: AuditLog) => row.actor_name || row.actor_id?.slice(0, 8) || '-',
  },
  {
    key: 'action',
    title: '动作',
    render: (row: AuditLog) => row.action,
  },
  {
    key: 'status',
    title: '状态',
    render: (row: AuditLog) => (row.status === 'success' ? '成功' : '失败'),
  },
  {
    key: 'created_at',
    title: '时间',
    render: (row: AuditLog) => formatDateTime(row.created_at),
  },
]

/**
 * 移动端表格操作配置
 */
const mobileActions: MobileAction<AuditLog>[] = [
  {
    label: '查看',
    type: 'primary',
    onClick: (row: AuditLog) => emit('view', row),
  },
]

/**
 * 表格列配置
 */
const columns: DataTableColumns<AuditLog> = [
  {
    title: '时间',
    key: 'created_at',
    width: 170,
    render(row) {
      return h('span', { class: 'audit-time' }, formatDateTime(row.created_at))
    },
  },
  {
    title: '事件类型',
    key: 'event_type',
    width: 140,
    render(row) {
      return h(NTag, { type: 'default', size: 'small' }, { default: () => getEventTypeText(row.event_type) })
    },
  },
  {
    title: '严重级别',
    key: 'severity',
    width: 100,
    align: 'center',
    render(row) {
      const config = severityConfig[row.severity] || severityConfig.info
      return h(NTag, { type: config?.type || 'default', size: 'small' }, { default: () => config?.text || row.severity })
    },
  },
  {
    title: '操作者',
    key: 'actor_name',
    width: 120,
    render(row) {
      return h('span', {}, row.actor_name || row.actor_id?.slice(0, 8) || '-')
    },
  },
  {
    title: '动作',
    key: 'action',
    width: 100,
    render(row) {
      return h(NTag, { type: 'default', size: 'small', bordered: true }, { default: () => row.action })
    },
  },
  {
    title: '描述',
    key: 'description',
    minWidth: 250,
    ellipsis: {
      tooltip: true,
    },
    render(row) {
      return h('span', {}, truncateText(row.description, 80))
    },
  },
  {
    title: '状态',
    key: 'status',
    width: 90,
    align: 'center',
    render(row) {
      const isSuccess = row.status === 'success'
      return h(NTag,
        { type: isSuccess ? 'success' : 'error', size: 'small' },
        { default: () => isSuccess ? '成功' : '失败' }
      )
    },
  },
  {
    title: '操作',
    key: 'actions',
    width: 80,
    fixed: 'right',
    align: 'center',
    render(row) {
      return h(
        NButton,
        {
          size: 'small',
          type: 'primary',
          tertiary: true,
          onClick: () => emit('view', row),
        },
        {
          icon: () => h(NIcon, null, { default: () => h(Eye) }),
        },
      )
    },
  },
]
</script>

<template>
  <div class="audit-log-table">
    <!-- 桌面端表格 -->
    <NDataTable
      v-if="!isMobile"
      :columns="columns"
      :data="data"
      :loading="loading"
      :pagination="false"
      :bordered="false"
      :striped="true"
      :scroll-x="1100"
      size="small"
      class="audit-table"
    />
    <!-- 移动端表格 -->
    <MobileTableCard
      v-else
      :data="data"
      :columns="mobileColumns"
      :actions="mobileActions"
      title-column="description"
      empty-text="暂无审计日志"
    />
  </div>
</template>

<style scoped>
.audit-log-table {
  width: 100%;
  overflow-x: auto;
}

.audit-table {
  width: 100%;
}

.audit-table :deep(.n-data-table-th) {
  font-weight: 600;
  background-color: var(--card-color);
}

.audit-time {
  font-family: monospace;
  font-size: 13px;
}
</style>
