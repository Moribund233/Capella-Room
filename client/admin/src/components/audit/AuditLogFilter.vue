<script setup lang="ts">
import { } from 'vue'
import {
  NSelect,
  NSpace,
  NButton,
  NDatePicker,
  NInput,
} from 'naive-ui'
import { Search, RefreshCw, Download } from 'lucide-vue-next'

/**
 * 搜索参数
 */
export interface AuditLogFilterParams {
  /** 事件类型 */
  eventType: string | null
  /** 严重级别 */
  severity: string | null
  /** 操作者ID */
  actorId: string | null
  /** 状态 */
  status: string | null
  /** 开始时间 */
  startTime: number | null
  /** 结束时间 */
  endTime: number | null
}

/**
 * 组件属性
 */
interface Props {
  eventType?: string | null
  severity?: string | null
  actorId?: string | null
  status?: string | null
  startTime?: number | null
  endTime?: number | null
  loading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  eventType: null,
  severity: null,
  actorId: null,
  status: null,
  startTime: null,
  endTime: null,
  loading: false,
})

const emit = defineEmits<{
  search: [params: AuditLogFilterParams]
  reset: []
  refresh: []
  export: [params: AuditLogFilterParams]
}>()

/** 事件类型选项 */
const eventTypeOptions = [
  { label: '全部事件', value: '' },
  { label: '用户登录', value: 'user_login' },
  { label: '用户登出', value: 'user_logout' },
  { label: '用户注册', value: 'user_register' },
  { label: '创建房间', value: 'room_create' },
  { label: '删除房间', value: 'room_delete' },
  { label: '发送消息', value: 'message_send' },
  { label: '编辑消息', value: 'message_edit' },
  { label: '删除消息', value: 'message_delete' },
  { label: '管理员操作', value: 'admin_user_disable' },
  { label: '系统登录失败', value: 'system_login_failure' },
  { label: '未授权访问', value: 'system_unauthorized_access' },
  { label: 'IP 安全事件', value: 'ip_blocked' },
]

/** 严重级别选项 */
const severityOptions = [
  { label: '全部级别', value: '' },
  { label: '信息', value: 'info' },
  { label: '警告', value: 'warning' },
  { label: '错误', value: 'error' },
  { label: '严重', value: 'critical' },
]

/** 状态选项 */
const statusOptions = [
  { label: '全部状态', value: '' },
  { label: '成功', value: 'success' },
  { label: '失败', value: 'failure' },
]

/**
 * 处理搜索
 */
const handleSearch = () => {
  emit('search', {
    eventType: props.eventType,
    severity: props.severity,
    actorId: props.actorId,
    status: props.status,
    startTime: props.startTime,
    endTime: props.endTime,
  })
}

/**
 * 处理导出
 */
const handleExport = () => {
  emit('export', {
    eventType: props.eventType,
    severity: props.severity,
    actorId: props.actorId,
    status: props.status,
    startTime: props.startTime,
    endTime: props.endTime,
  })
}

/**
 * 处理开始时间变化
 * @param value 时间戳
 */
const handleStartTimeChange = (value: number | null) => {
  if (value && props.endTime && value > props.endTime) {
    emit('search', {
      eventType: props.eventType,
      severity: props.severity,
      actorId: props.actorId,
      status: props.status,
      startTime: value,
      endTime: null,
    })
  } else {
    emit('search', {
      eventType: props.eventType,
      severity: props.severity,
      actorId: props.actorId,
      status: props.status,
      startTime: value,
      endTime: props.endTime,
    })
  }
}

/**
 * 处理结束时间变化
 * @param value 时间戳
 */
const handleEndTimeChange = (value: number | null) => {
  if (value && props.startTime && value < props.startTime) {
    emit('search', {
      eventType: props.eventType,
      severity: props.severity,
      actorId: props.actorId,
      status: props.status,
      startTime: null,
      endTime: value,
    })
  } else {
    emit('search', {
      eventType: props.eventType,
      severity: props.severity,
      actorId: props.actorId,
      status: props.status,
      startTime: props.startTime,
      endTime: value,
    })
  }
}
</script>

<template>
  <div class="audit-log-filter">
    <NSpace align="center" wrap>
      <NSelect
        :value="eventType"
        :options="eventTypeOptions"
        placeholder="事件类型"
        clearable
        style="width: 160px"
        @update:value="(v) => emit('search', { eventType: v || null, severity, actorId, status, startTime, endTime })"
      />

      <NSelect
        :value="severity"
        :options="severityOptions"
        placeholder="严重级别"
        clearable
        style="width: 140px"
        @update:value="(v) => emit('search', { eventType, severity: v || null, actorId, status, startTime, endTime })"
      />

      <NSelect
        :value="status"
        :options="statusOptions"
        placeholder="状态"
        clearable
        style="width: 120px"
        @update:value="(v) => emit('search', { eventType, severity, actorId, status: v || null, startTime, endTime })"
      />

      <NInput
        :value="actorId || ''"
        placeholder="操作者ID"
        clearable
        style="width: 180px"
        @update:value="(v) => emit('search', { eventType, severity, actorId: v || null, status, startTime, endTime })"
      />

      <!-- 开始时间和结束时间分开选择 -->
      <div class="date-range-wrapper">
        <NDatePicker
          type="datetime"
          :value="startTime"
          placeholder="开始时间"
          clearable
          class="date-picker-start"
          @update:value="handleStartTimeChange"
        />
        <span class="date-range-separator">至</span>
        <NDatePicker
          type="datetime"
          :value="endTime"
          placeholder="结束时间"
          clearable
          class="date-picker-end"
          @update:value="handleEndTimeChange"
        />
      </div>

      <NSpace>
        <NButton type="primary" :loading="loading" @click="handleSearch">
          <template #icon>
            <Search :size="16" />
          </template>
          搜索
        </NButton>

        <NButton @click="() => emit('reset')">
          <template #icon>
            <RefreshCw :size="16" />
          </template>
          重置
        </NButton>

        <NButton @click="handleExport">
          <template #icon>
            <Download :size="16" />
          </template>
          导出
        </NButton>
      </NSpace>
    </NSpace>
  </div>
</template>

<style scoped>
.audit-log-filter {
  padding: 16px 0;
}

.date-range-wrapper {
  display: flex;
  align-items: center;
  gap: 8px;
}

.date-picker-start,
.date-picker-end {
  width: 180px;
}

.date-range-separator {
  color: var(--text-secondary);
  font-size: 14px;
}

@media (max-width: 768px) {
  .date-range-wrapper {
    flex-direction: column;
    align-items: stretch;
    width: 100%;
  }

  .date-picker-start,
  .date-picker-end {
    width: 100%;
  }

  .date-range-separator {
    text-align: center;
  }
}
</style>
