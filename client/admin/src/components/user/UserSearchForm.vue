<script setup lang="ts">
/**
 * UserSearchForm - 用户搜索筛选表单组件
 *
 * 提供用户列表的搜索、状态筛选、角色筛选功能
 *
 */
import { ref, watch } from 'vue'
import { NInput, NSelect, NSpace, NButton } from 'naive-ui'
import { Search, RefreshCw } from 'lucide-vue-next'
import type { UserRole, UserStatus } from '@/types'

/**
 * 组件属性定义
 */
interface Props {
  /** 搜索关键词 */
  keyword?: string
  /** 状态筛选值 */
  status?: string
  /** 角色筛选值 */
  role?: string
  /** 加载状态 */
  loading?: boolean
}

/**
 * 组件事件定义
 */
interface Emits {
  /** 搜索事件 */
  (e: 'search', params: { keyword: string; status: string; role: string }): void
  /** 重置事件 */
  (e: 'reset'): void
  /** 刷新事件 */
  (e: 'refresh'): void
  /** 关键词变化事件 */
  (e: 'update:keyword', value: string): void
  /** 状态变化事件 */
  (e: 'update:status', value: string): void
  /** 角色变化事件 */
  (e: 'update:role', value: string): void
}

const props = withDefaults(defineProps<Props>(), {
  keyword: '',
  status: '',
  role: '',
  loading: false,
})

const emit = defineEmits<Emits>()

/**
 * 本地状态（用于内部编辑）
 */
const localKeyword = ref(props.keyword)
const localStatus = ref(props.status)
const localRole = ref(props.role)

/**
 * 同步外部属性变化
 */
watch(() => props.keyword, (val) => { localKeyword.value = val })
watch(() => props.status, (val) => { localStatus.value = val })
watch(() => props.role, (val) => { localRole.value = val })

/**
 * 状态选项
 */
const statusOptions = [
  { label: '全部状态', value: '' },
  { label: '在线', value: 'online' as UserStatus },
  { label: '离线', value: 'offline' as UserStatus },
  { label: '离开', value: 'away' as UserStatus },
  { label: '禁用', value: 'disabled' as UserStatus },
]

/**
 * 角色选项
 */
const roleOptions = [
  { label: '全部角色', value: '' },
  { label: '超级管理员', value: 'super_admin' as UserRole },
  { label: '管理员', value: 'admin' as UserRole },
  { label: '普通用户', value: 'user' as UserRole },
]

/**
 * 处理搜索
 */
const handleSearch = () => {
  emit('update:keyword', localKeyword.value)
  emit('update:status', localStatus.value)
  emit('update:role', localRole.value)
  emit('search', {
    keyword: localKeyword.value,
    status: localStatus.value,
    role: localRole.value,
  })
}

/**
 * 处理重置
 */
const handleReset = () => {
  localKeyword.value = ''
  localStatus.value = ''
  localRole.value = ''
  emit('update:keyword', '')
  emit('update:status', '')
  emit('update:role', '')
  emit('reset')
}

/**
 * 处理刷新
 */
const handleRefresh = () => {
  emit('refresh')
}

/**
 * 处理回车键搜索
 */
const handleKeyup = (e: KeyboardEvent) => {
  if (e.key === 'Enter') {
    handleSearch()
  }
}
</script>

<template>
  <NSpace align="center" wrap>
    <NInput
      v-model:value="localKeyword"
      placeholder="搜索用户名/昵称/邮箱"
      clearable
      style="width: 240px"
      @keyup="handleKeyup"
    >
      <template #prefix>
        <Search :size="16" />
      </template>
    </NInput>

    <NSelect
      v-model:value="localStatus"
      :options="statusOptions"
      placeholder="选择状态"
      clearable
      style="width: 140px"
      @update:value="handleSearch"
    />

    <NSelect
      v-model:value="localRole"
      :options="roleOptions"
      placeholder="选择角色"
      clearable
      style="width: 140px"
      @update:value="handleSearch"
    />

    <NButton type="primary" :loading="loading" @click="handleSearch">
      <template #icon>
        <Search :size="16" />
      </template>
      搜索
    </NButton>

    <NButton :disabled="loading" @click="handleReset">
      重置
    </NButton>

    <NButton quaternary :loading="loading" @click="handleRefresh">
      <template #icon>
        <RefreshCw :size="16" />
      </template>
    </NButton>
  </NSpace>
</template>
