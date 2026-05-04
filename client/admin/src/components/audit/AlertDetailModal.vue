<script setup lang="ts">
import {
  NDescriptions,
  NDescriptionsItem,
  NTag,
  NSpace,
  NCode,
  NDivider,
  NButton,
} from 'naive-ui'
import { CheckCircle, EyeOff } from 'lucide-vue-next'
import type { AuditAlert } from '@/api/audit'

/**
 * 组件属性定义
 */
interface Props {
  /** 告警数据 */
  alert: AuditAlert
}

const props = defineProps<Props>()

/**
 * 组件事件定义
 */
interface Emits {
  /** 确认告警 */
  (e: 'acknowledge', alert: AuditAlert): void
  /** 解决告警 */
  (e: 'resolve', alert: AuditAlert): void
  /** 忽略告警 */
  (e: 'ignore', alert: AuditAlert): void
}

const emit = defineEmits<Emits>()

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
    second: '2-digit',
  })
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
 * 获取状态配置（安全版本）
 */
const getStatusConfigSafe = (status: string): { text: string; type: 'default' | 'info' | 'warning' | 'error' | 'success' } => {
  const config = statusConfig[status]
  if (config) return config
  return { text: '未处理', type: 'error' as const }
}

/**
 * 是否可以确认
 */
const canAcknowledge = computed(() => {
  return props.alert.status === 'new'
})

/**
 * 是否可以解决
 */
const canResolve = computed(() => {
  return props.alert.status === 'new' || props.alert.status === 'acknowledged'
})

/**
 * 是否可以忽略
 */
const canIgnore = computed(() => {
  return props.alert.status === 'new'
})

import { computed } from 'vue'

/**
 * 关联日志ID格式化
 */
const relatedLogsText = computed(() => {
  if (!props.alert.related_logs || props.alert.related_logs.length === 0) {
    return '-'
  }
  return props.alert.related_logs.join('\n')
})
</script>

<template>
  <div class="alert-detail-modal">
    <NDescriptions :column="2" bordered size="small">
      <NDescriptionsItem label="告警ID">
        <span class="mono-text">{{ alert.id }}</span>
      </NDescriptionsItem>

      <NDescriptionsItem label="规则ID">
        <span v-if="alert.rule_id" class="mono-text">{{ alert.rule_id }}</span>
        <span v-else>-</span>
      </NDescriptionsItem>

      <NDescriptionsItem label="告警类型">
        <NTag type="default" size="small">{{ alert.alert_type }}</NTag>
      </NDescriptionsItem>

      <NDescriptionsItem label="严重级别">
        <NTag :type="getSeverityConfigSafe(alert.severity).type" size="small">
          {{ getSeverityConfigSafe(alert.severity).text }}
        </NTag>
      </NDescriptionsItem>

      <NDescriptionsItem label="状态">
        <NTag :type="getStatusConfigSafe(alert.status).type" size="small">
          {{ getStatusConfigSafe(alert.status).text }}
        </NTag>
      </NDescriptionsItem>

      <NDescriptionsItem label="来源IP">
        <span v-if="alert.source_ip" class="mono-text">{{ alert.source_ip }}</span>
        <span v-else>-</span>
      </NDescriptionsItem>

      <NDescriptionsItem label="标题" :span="2">
        <strong>{{ alert.title }}</strong>
      </NDescriptionsItem>

      <NDescriptionsItem label="描述" :span="2">
        {{ alert.description }}
      </NDescriptionsItem>

      <NDescriptionsItem label="受影响用户">
        <span v-if="alert.affected_user">
          {{ alert.affected_user.username }}
          <span class="mono-text text-secondary">({{ alert.affected_user.id }})</span>
        </span>
        <span v-else>-</span>
      </NDescriptionsItem>

      <NDescriptionsItem label="创建时间">
        {{ formatDateTime(alert.created_at) }}
      </NDescriptionsItem>

      <NDescriptionsItem label="确认人">
        <span v-if="alert.acknowledged_by">
          {{ alert.acknowledged_by.username }}
        </span>
        <span v-else>-</span>
      </NDescriptionsItem>

      <NDescriptionsItem label="确认时间">
        {{ formatDateTime(alert.acknowledged_at) }}
      </NDescriptionsItem>

      <NDescriptionsItem label="解决人">
        <span v-if="alert.resolved_by">
          {{ alert.resolved_by.username }}
        </span>
        <span v-else>-</span>
      </NDescriptionsItem>

      <NDescriptionsItem label="解决时间">
        {{ formatDateTime(alert.resolved_at) }}
      </NDescriptionsItem>
    </NDescriptions>

    <!-- 关联日志 -->
    <template v-if="alert.related_logs && alert.related_logs.length > 0">
      <NDivider title-placement="left">关联日志</NDivider>
      <div class="related-logs">
        <NCode :code="relatedLogsText" language="plaintext" />
      </div>
    </template>

    <!-- 操作按钮 -->
    <NDivider />
    <NSpace justify="end" size="small">
      <NButton
        v-if="canAcknowledge"
        type="warning"
        secondary
        @click="emit('acknowledge', alert)"
      >
        <template #icon>
          <CheckCircle :size="16" />
        </template>
        确认
      </NButton>

      <NButton
        v-if="canResolve"
        type="success"
        secondary
        @click="emit('resolve', alert)"
      >
        <template #icon>
          <CheckCircle :size="16" />
        </template>
        解决
      </NButton>

      <NButton
        v-if="canIgnore"
        tertiary
        @click="emit('ignore', alert)"
      >
        <template #icon>
          <EyeOff :size="16" />
        </template>
        忽略
      </NButton>
    </NSpace>
  </div>
</template>

<style scoped>
.alert-detail-modal {
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

.related-logs {
  background-color: var(--card-color);
  border-radius: 8px;
  padding: 12px;
}

.related-logs :deep(.n-code) {
  background-color: transparent !important;
}
</style>
