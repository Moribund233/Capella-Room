<script setup lang="ts">
import { ref, watch } from 'vue'
import {
  NForm,
  NFormItem,
  NInput,
  NSelect,
  NSwitch,
  NInputNumber,
  NButton,
  NSpace,
  NCard,
  NCode,
} from 'naive-ui'
import type { AlertRule, UpdateAlertRuleRequest } from '@/api/audit'

/**
 * 组件属性定义
 */
interface Props {
  /** 规则数据 */
  rule: AlertRule | null
  /** 加载状态 */
  loading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  loading: false,
})

/**
 * 组件事件定义
 */
interface Emits {
  /** 保存规则 */
  (e: 'save', id: string, data: UpdateAlertRuleRequest): void
  /** 取消编辑 */
  (e: 'cancel'): void
}

const emit = defineEmits<Emits>()

/** 表单数据 */
const formData = ref<UpdateAlertRuleRequest>({
  name: '',
  description: null,
  condition: {},
  severity: 'info',
  enabled: true,
  cooldown_minutes: 60,
  notify_admins: true,
})

/** 严重级别选项 */
const severityOptions = [
  { label: '信息', value: 'info' },
  { label: '警告', value: 'warning' },
  { label: '错误', value: 'error' },
  { label: '严重', value: 'critical' },
]

/** 表单校验规则 */
const rules = {
  name: {
    required: true,
    message: '请输入规则名称',
    trigger: 'blur',
  },
  severity: {
    required: true,
    message: '请选择严重级别',
    trigger: 'change',
  },
  cooldown_minutes: {
    required: true,
    type: 'number' as const,
    min: 1,
    message: '冷却时间至少为1分钟',
    trigger: 'change',
  },
}

/** 表单引用 */
const formRef = ref<InstanceType<typeof NForm>>()

/**
 * 监听规则变化，更新表单数据
 */
watch(() => props.rule, (newRule) => {
  if (newRule) {
    formData.value = {
      name: newRule.name,
      description: newRule.description,
      condition: newRule.condition,
      severity: newRule.severity,
      enabled: newRule.enabled,
      cooldown_minutes: newRule.cooldown_minutes,
      notify_admins: newRule.notify_admins,
    }
  }
}, { immediate: true })

/**
 * 处理保存
 */
const handleSave = async () => {
  if (!props.rule) return

  try {
    await formRef.value?.validate()
    emit('save', props.rule.id, { ...formData.value })
  } catch {
    // 表单校验失败
  }
}

/**
 * 格式化条件配置为 JSON 字符串
 */
const conditionJson = computed(() => {
  return JSON.stringify(formData.value.condition || {}, null, 2)
})

import { computed } from 'vue'
</script>

<template>
  <div class="rule-editor">
    <NForm
      v-if="rule"
      ref="formRef"
      :model="formData"
      :rules="rules"
      label-placement="left"
      label-width="120px"
      require-mark-placement="right-hanging"
    >
      <NFormItem label="规则ID">
        <span class="mono-text">{{ rule.id }}</span>
      </NFormItem>

      <NFormItem label="规则名称" path="name">
        <NInput
          v-model:value="formData.name"
          placeholder="请输入规则名称"
          maxlength="100"
          show-count
        />
      </NFormItem>

      <NFormItem label="描述">
        <NInput
          v-model:value="formData.description"
          type="textarea"
          placeholder="请输入规则描述"
          :rows="3"
        />
      </NFormItem>

      <NFormItem label="事件类型">
        <NTag type="default" size="small">
          {{ rule.event_type || '全部事件' }}
        </NTag>
      </NFormItem>

      <NFormItem label="严重级别" path="severity">
        <NSelect
          v-model:value="formData.severity"
          :options="severityOptions"
          placeholder="请选择严重级别"
        />
      </NFormItem>

      <NFormItem label="启用状态">
        <NSwitch v-model:value="formData.enabled">
          <template #checked>启用</template>
          <template #unchecked>禁用</template>
        </NSwitch>
      </NFormItem>

      <NFormItem label="冷却时间(分钟)" path="cooldown_minutes">
        <NInputNumber
          v-model:value="formData.cooldown_minutes"
          :min="1"
          :max="1440"
          placeholder="请输入冷却时间"
        />
      </NFormItem>

      <NFormItem label="通知管理员">
        <NSwitch v-model:value="formData.notify_admins">
          <template #checked>是</template>
          <template #unchecked>否</template>
        </NSwitch>
      </NFormItem>

      <NFormItem label="条件配置">
        <NCard size="small" class="condition-card">
          <NCode :code="conditionJson" language="json" />
        </NCard>
      </NFormItem>

      <NFormItem>
        <NSpace>
          <NButton type="primary" :loading="loading" @click="handleSave">
            保存
          </NButton>
          <NButton @click="emit('cancel')">
            取消
          </NButton>
        </NSpace>
      </NFormItem>
    </NForm>

    <div v-else class="empty-state">
      <NEmpty description="请选择一条规则进行编辑" />
    </div>
  </div>
</template>

<style scoped>
.rule-editor {
  padding: 16px;
}

.mono-text {
  font-family: monospace;
  font-size: 13px;
  color: var(--text-secondary);
}

.condition-card {
  background-color: var(--card-color);
}

.condition-card :deep(.n-code) {
  background-color: transparent !important;
}

.empty-state {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 200px;
}
</style>
