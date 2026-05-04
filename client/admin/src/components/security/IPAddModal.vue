<script setup lang="ts">
import { ref, computed } from 'vue'
import {
  NForm,
  NFormItem,
  NInput,
  NSelect,
  NRadioGroup,
  NRadio,
  NSpace,
  NDatePicker,
  NInputNumber,
  type FormInst,
  type FormRules,
} from 'naive-ui'
import type { IPListType } from '@/api/security'

/**
 * 添加IP表单数据
 */
export interface IPAddFormData {
  /** IP地址 */
  ipAddress: string
  /** 列表类型 */
  listType: IPListType
  /** 备注 */
  remark: string
  /** 过期类型 */
  expireType: 'never' | 'custom' | 'days'
  /** 过期时间 (时间戳) */
  expireTime: number | null
  /** 过期天数 */
  expireDays: number | null
}

const emit = defineEmits<{
  /** 确认添加 */
  confirm: [data: IPAddFormData]
  /** 取消 */
  cancel: []
}>()

/** 表单引用 */
const formRef = ref<FormInst | null>(null)

/** 表单数据 */
const formData = ref<IPAddFormData>({
  ipAddress: '',
  listType: 'blacklist',
  remark: '',
  expireType: 'never',
  expireTime: null,
  expireDays: null,
})

/** 列表类型选项 */
const listTypeOptions = [
  { label: '黑名单', value: 'blacklist' },
  { label: '白名单', value: 'whitelist' },
]

/**
 * 验证IP地址格式
 * @param ip IP地址
 * @returns 是否有效
 */
const isValidIP = (ip: string): boolean => {
  // IPv4 验证
  const ipv4Regex = /^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$/
  // IPv6 验证（简化版）
  const ipv6Regex = /^(?:[0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}$|^(?:[0-9a-fA-F]{1,4}:){1,7}:$|^(?:[0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}$|^(?:[0-9a-fA-F]{1,4}:){1,5}(?::[0-9a-fA-F]{1,4}){1,2}$|^(?:[0-9a-fA-F]{1,4}:){1,4}(?::[0-9a-fA-F]{1,4}){1,3}$|^(?:[0-9a-fA-F]{1,4}:){1,3}(?::[0-9a-fA-F]{1,4}){1,4}$|^(?:[0-9a-fA-F]{1,4}:){1,2}(?::[0-9a-fA-F]{1,4}){1,5}$|^[0-9a-fA-F]{1,4}:(?::[0-9a-fA-F]{1,4}){1,6}$|^:(?::[0-9a-fA-F]{1,4}){1,7}$|^::$/
  // CIDR 验证
  const cidrRegex = /^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\/(?:[0-9]|[1-2][0-9]|3[0-2])$/

  return ipv4Regex.test(ip) || ipv6Regex.test(ip) || cidrRegex.test(ip)
}

/**
 * 表单验证规则
 */
const rules: FormRules = {
  ipAddress: [
    { required: true, message: '请输入IP地址', trigger: 'blur' },
    {
      validator: (_rule, value: string) => {
        if (!value) return true
        if (!isValidIP(value)) {
          return new Error('请输入有效的IP地址（支持IPv4、IPv6、CIDR）')
        }
        return true
      },
      trigger: 'blur',
    },
  ],
  listType: [
    { required: true, message: '请选择列表类型', trigger: 'change' },
  ],
  expireTime: [
    {
      validator: () => {
        if (formData.value.expireType !== 'custom') return true
        if (formData.value.expireTime === null) {
          return new Error('请选择过期时间')
        }
        return true
      },
      trigger: 'change',
    },
  ],
  expireDays: [
    {
      validator: () => {
        if (formData.value.expireType !== 'days') return true
        if (formData.value.expireDays === null || formData.value.expireDays <= 0) {
          return new Error('请输入有效的天数')
        }
        return true
      },
      trigger: 'change',
    },
  ],
}

/**
 * 计算过期时间
 */
const calculatedExpireTime = computed(() => {
  if (formData.value.expireType === 'never') return null
  if (formData.value.expireType === 'custom') return formData.value.expireTime
  if (formData.value.expireType === 'days' && formData.value.expireDays) {
    const date = new Date()
    date.setDate(date.getDate() + formData.value.expireDays)
    return date.getTime()
  }
  return null
})

/**
 * 处理确认
 */
const handleConfirm = async () => {
  if (!formRef.value) return

  try {
    await formRef.value.validate()
    emit('confirm', {
      ...formData.value,
      expireTime: calculatedExpireTime.value,
    })
    // 重置表单
    formData.value = {
      ipAddress: '',
      listType: 'blacklist',
      remark: '',
      expireType: 'never',
      expireTime: null,
      expireDays: null,
    }
  } catch {
    // 验证失败
  }
}

/**
 * 处理取消
 */
const handleCancel = () => {
  emit('cancel')
  // 重置表单
  formData.value = {
    ipAddress: '',
    listType: 'blacklist',
    remark: '',
    expireType: 'never',
    expireTime: null,
    expireDays: null,
  }
}

/**
 * 禁用过去的日期
 */
const disabledDate = (ts: number) => {
  return ts < Date.now() - 86400000 // 禁用昨天及之前的日期
}

defineExpose({
  handleConfirm,
  handleCancel,
})
</script>

<template>
  <NForm
    ref="formRef"
    :model="formData"
    :rules="rules"
    label-placement="left"
    label-width="100px"
    class="ip-add-form"
  >
    <NFormItem label="IP地址" path="ipAddress">
      <NInput
        v-model:value="formData.ipAddress"
        placeholder="支持 IPv4、IPv6、CIDR 格式"
        clearable
      />
    </NFormItem>

    <NFormItem label="列表类型" path="listType">
      <NSelect
        v-model:value="formData.listType"
        :options="listTypeOptions"
        placeholder="选择列表类型"
      />
    </NFormItem>

    <NFormItem label="过期时间" path="expireType">
      <NRadioGroup v-model:value="formData.expireType">
        <NSpace vertical>
          <NRadio value="never">永不过期</NRadio>
          <NRadio value="days">
            <NSpace align="center">
              <span>指定天数</span>
              <NInputNumber
                v-model:value="formData.expireDays"
                :disabled="formData.expireType !== 'days'"
                :min="1"
                :max="365"
                style="width: 100px"
                placeholder="天数"
              />
              <span>天后过期</span>
            </NSpace>
          </NRadio>
          <NRadio value="custom">
            <NSpace align="center">
              <span>指定日期</span>
              <NDatePicker
                v-model:value="formData.expireTime"
                :disabled="formData.expireType !== 'custom'"
                type="datetime"
                :disabled-date="disabledDate"
                placeholder="选择过期时间"
                style="width: 200px"
              />
            </NSpace>
          </NRadio>
        </NSpace>
      </NRadioGroup>
    </NFormItem>

    <NFormItem label="备注" path="remark">
      <NInput
        v-model:value="formData.remark"
        type="textarea"
        :rows="3"
        placeholder="可选：添加备注说明"
        maxlength="200"
        show-count
      />
    </NFormItem>
  </NForm>
</template>

<style scoped>
.ip-add-form {
  padding: 8px 0;
}
</style>
