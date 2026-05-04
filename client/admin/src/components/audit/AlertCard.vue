<script setup lang="ts">
import { computed } from 'vue'
import {
  NCard,
  NTag,
  NSpace,
  NButton,
  NText,
} from 'naive-ui'
import { CheckCircle, EyeOff, AlertTriangle, Info, AlertOctagon } from 'lucide-vue-next'
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
  /** 查看详情 */
  (e: 'view', alert: AuditAlert): void
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
const severityConfig: Record<string, { text: string; type: 'default' | 'info' | 'warning' | 'error' | 'success'; icon: typeof Info }> = {
  info: { text: '信息', type: 'info', icon: Info },
  warning: { text: '警告', type: 'warning', icon: AlertTriangle },
  error: { text: '错误', type: 'error', icon: AlertOctagon },
  critical: { text: '严重', type: 'error', icon: AlertOctagon },
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
 * 获取严重级别配置
 */
const severityConfigValue = computed(() => {
  return severityConfig[props.alert.severity] || severityConfig.info
})

/**
 * 获取状态配置
 */
const statusConfigValue = computed(() => {
  return statusConfig[props.alert.status] || statusConfig.new
})

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
</script>

<template>
  <NCard
    class="alert-card"
    :class="`severity-${alert.severity} status-${alert.status}`"
    size="small"
    :bordered="true"
  >
    <template #header>
      <NSpace align="center" justify="space-between">
        <NSpace align="center" size="small">
          <component :is="severityConfigValue?.icon || Info" :size="18" :class="`icon-${alert.severity}`" />
          <span class="alert-title">{{ alert.title }}</span>
        </NSpace>
        <NTag :type="statusConfigValue?.type || 'default'" size="small">
          {{ statusConfigValue?.text || alert.status }}
        </NTag>
      </NSpace>
    </template>

    <div class="alert-content">
      <NText class="alert-description">{{ alert.description }}</NText>

      <NSpace class="alert-meta" align="center" size="small">
        <NTag :type="severityConfigValue?.type || 'default'" size="small">
          {{ severityConfigValue?.text || alert.severity }}
        </NTag>
        <span class="alert-type">{{ alert.alert_type }}</span>
        <span v-if="alert.source_ip" class="source-ip">
          IP: {{ alert.source_ip }}
        </span>
      </NSpace>

      <div class="alert-time">
        <NText depth="3" class="time-text">
          创建: {{ formatDateTime(alert.created_at) }}
        </NText>
        <NText v-if="alert.acknowledged_by" depth="3" class="time-text">
          确认人: {{ alert.acknowledged_by.username }}
        </NText>
        <NText v-if="alert.resolved_by" depth="3" class="time-text">
          解决人: {{ alert.resolved_by.username }}
        </NText>
      </div>
    </div>

    <template #footer>
      <NSpace justify="end" size="small">
        <NButton
          v-if="canAcknowledge"
          size="small"
          type="warning"
          secondary
          @click="emit('acknowledge', alert)"
        >
          <template #icon>
            <CheckCircle :size="14" />
          </template>
          确认
        </NButton>

        <NButton
          v-if="canResolve"
          size="small"
          type="success"
          secondary
          @click="emit('resolve', alert)"
        >
          <template #icon>
            <CheckCircle :size="14" />
          </template>
          解决
        </NButton>

        <NButton
          v-if="canIgnore"
          size="small"
          tertiary
          @click="emit('ignore', alert)"
        >
          <template #icon>
            <EyeOff :size="14" />
          </template>
          忽略
        </NButton>

        <NButton
          size="small"
          type="primary"
          tertiary
          @click="emit('view', alert)"
        >
          查看详情
        </NButton>
      </NSpace>
    </template>
  </NCard>
</template>

<style scoped>
.alert-card {
  transition: all 0.2s ease;
}

.alert-card:hover {
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.1);
}

.alert-title {
  font-weight: 600;
  font-size: 14px;
}

.alert-content {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.alert-description {
  font-size: 13px;
  line-height: 1.5;
  color: var(--text-color);
}

.alert-meta {
  margin-top: 4px;
}

.alert-type {
  font-size: 12px;
  color: var(--text-secondary);
}

.source-ip {
  font-size: 12px;
  color: var(--text-secondary);
  font-family: monospace;
}

.alert-time {
  display: flex;
  flex-direction: column;
  gap: 2px;
  margin-top: 4px;
}

.time-text {
  font-size: 12px;
}

.icon-info {
  color: var(--info-color);
}

.icon-warning {
  color: var(--warning-color);
}

.icon-error,
.icon-critical {
  color: var(--error-color);
}

.severity-critical {
  border-left: 3px solid var(--error-color);
}

.severity-error {
  border-left: 3px solid var(--error-color);
}

.severity-warning {
  border-left: 3px solid var(--warning-color);
}

.severity-info {
  border-left: 3px solid var(--info-color);
}
</style>
