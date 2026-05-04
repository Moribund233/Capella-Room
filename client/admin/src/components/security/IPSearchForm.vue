<script setup lang="ts">
import { NInput, NSelect, NSpace, NButton } from 'naive-ui'
import { Search, RefreshCw } from 'lucide-vue-next'
import type { IPListType } from '@/api/security'

/**
 * IP搜索参数
 */
export interface IPSearchParams {
  /** IP地址搜索关键词 */
  ipAddress: string
  /** 列表类型过滤 */
  listType: IPListType | null
}

/**
 * 组件属性
 */
interface Props {
  /** IP地址 */
  ipAddress?: string
  /** 列表类型 */
  listType?: IPListType | null
  /** 加载状态 */
  loading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  ipAddress: '',
  listType: null,
  loading: false,
})

const emit = defineEmits<{
  /** 搜索事件 */
  search: [params: IPSearchParams]
  /** 重置事件 */
  reset: []
  /** 刷新事件 */
  refresh: []
}>()

/** 列表类型选项 */
const listTypeOptions = [
  { label: '全部类型', value: '' },
  { label: '黑名单', value: 'blacklist' },
  { label: '白名单', value: 'whitelist' },
]

/**
 * 处理搜索
 */
const handleSearch = () => {
  emit('search', {
    ipAddress: props.ipAddress,
    listType: props.listType,
  })
}

/**
 * 处理重置
 */
const handleReset = () => {
  emit('reset')
}

/**
 * 处理刷新
 */
const handleRefresh = () => {
  emit('refresh')
}
</script>

<template>
  <NSpace align="center" wrap>
    <NInput
      :value="ipAddress"
      @update:value="$emit('search', { ipAddress: $event, listType })"
      placeholder="搜索IP地址"
      clearable
      style="width: 200px"
    >
      <template #prefix>
        <Search :size="16" />
      </template>
    </NInput>

    <NSelect
      :value="listType"
      @update:value="$emit('search', { ipAddress, listType: $event || null })"
      :options="listTypeOptions"
      placeholder="选择类型"
      clearable
      style="width: 140px"
    />

    <NSpace>
      <NButton type="primary" @click="handleSearch" :loading="loading">
        <template #icon>
          <Search :size="16" />
        </template>
        搜索
      </NButton>

      <NButton @click="handleReset">
        <template #icon>
          <RefreshCw :size="16" />
        </template>
        重置
      </NButton>

      <NButton secondary @click="handleRefresh" :loading="loading">
        <template #icon>
          <RefreshCw :size="16" />
        </template>
        刷新
      </NButton>
    </NSpace>
  </NSpace>
</template>
