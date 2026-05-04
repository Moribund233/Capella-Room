<script setup lang="ts">
import { computed } from 'vue'
import {
  NDescriptions,
  NDescriptionsItem,
  NTag,
  NSpace,
  NCode,
  NDivider,
} from 'naive-ui'
import type { AuditLog } from '@/api/audit'

/**
 * 组件属性定义
 */
interface Props {
  /** 审计日志数据 */
  log: AuditLog
}

const props = defineProps<Props>()

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
 * 获取事件类型中文名称
 * @param eventType 事件类型
 * @returns 中文名称
 */
const getEventTypeText = (eventType: string): string => {
  return eventTypeMap[eventType] || eventType
}

/**
 * 获取严重级别配置（安全版本）
 */
const getSeverityConfigSafe = (severity: string): { text: string; type: 'default' | 'info' | 'warning' | 'error' | 'success' } => {
  const config = severityConfig[severity]
  if (config) return config
  return { text: '信息', type: 'info' as const }
}

/**
 * 格式化元数据为 JSON 字符串
 */
const formattedMetadata = computed(() => {
  if (!props.log.metadata) return '{}'
  return JSON.stringify(props.log.metadata, null, 2)
})

/**
 * 是否有元数据
 */
const hasMetadata = computed(() => {
  return props.log.metadata && Object.keys(props.log.metadata).length > 0
})

/**
 * 是否有错误信息
 */
const hasError = computed(() => {
  return !!props.log.error_message
})
</script>

<template>
  <div class="audit-log-detail">
    <NDescriptions :column="2" bordered size="small">
      <NDescriptionsItem label="日志ID">
        <span class="mono-text">{{ log.id }}</span>
      </NDescriptionsItem>

      <NDescriptionsItem label="创建时间">
        {{ formatDateTime(log.created_at) }}
      </NDescriptionsItem>

      <NDescriptionsItem label="事件类型">
        <NTag type="default" size="small">
          {{ getEventTypeText(log.event_type) }}
        </NTag>
      </NDescriptionsItem>

      <NDescriptionsItem label="严重级别">
        <NTag :type="getSeverityConfigSafe(log.severity).type" size="small">
          {{ getSeverityConfigSafe(log.severity).text }}
        </NTag>
      </NDescriptionsItem>

      <NDescriptionsItem label="操作者">
        <NSpace align="center" size="small">
          <span>{{ log.actor_name || '-' }}</span>
          <span v-if="log.actor_id" class="mono-text text-secondary">({{ log.actor_id }})</span>
        </NSpace>
      </NDescriptionsItem>

      <NDescriptionsItem label="角色">
        <NTag v-if="log.actor_role" type="info" size="small">
          {{ log.actor_role }}
        </NTag>
        <span v-else>-</span>
      </NDescriptionsItem>

      <NDescriptionsItem label="目标类型">
        <NTag v-if="log.target_type" type="default" size="small" bordered>
          {{ log.target_type }}
        </NTag>
        <span v-else>-</span>
      </NDescriptionsItem>

      <NDescriptionsItem label="目标ID">
        <span v-if="log.target_id" class="mono-text">{{ log.target_id }}</span>
        <span v-else>-</span>
      </NDescriptionsItem>

      <NDescriptionsItem label="动作" :span="2">
        <NTag type="default" size="small" bordered>{{ log.action }}</NTag>
      </NDescriptionsItem>

      <NDescriptionsItem label="描述" :span="2">
        {{ log.description }}
      </NDescriptionsItem>

      <NDescriptionsItem label="状态">
        <NTag :type="log.status === 'success' ? 'success' : 'error'" size="small">
          {{ log.status === 'success' ? '成功' : '失败' }}
        </NTag>
      </NDescriptionsItem>
    </NDescriptions>

    <!-- 错误信息 -->
    <template v-if="hasError">
      <NDivider title-placement="left">错误信息</NDivider>
      <div class="error-message">
        <NCode :code="log.error_message || ''" language="plaintext" />
      </div>
    </template>

    <!-- 元数据 -->
    <template v-if="hasMetadata">
      <NDivider title-placement="left">元数据</NDivider>
      <div class="metadata-code">
        <NCode :code="formattedMetadata" language="json" />
      </div>
    </template>
  </div>
</template>

<style scoped>
.audit-log-detail {
  max-height: 70vh;
  overflow-y: auto;
}

.mono-text {
  font-family: monospace;
  font-size: 13px;
}

.text-secondary {
  color: var(--text-secondary);
}

.error-message {
  background-color: var(--error-color-faded, rgba(208, 48, 80, 0.1));
  border-radius: 8px;
  padding: 12px;
}

.metadata-code {
  background-color: var(--card-color);
  border-radius: 8px;
  padding: 12px;
}

.metadata-code :deep(.n-code) {
  background-color: transparent !important;
}
</style>
