<script setup lang="ts">
import { NInput, NSelect, NSpace, NButton } from 'naive-ui'
import { Search, RefreshCw } from 'lucide-vue-next'
import type { UserRole, UserStatus } from '@/types'

/**
 * 搜索参数
 */
interface SearchParams {
  keyword: string
  status: string
  role: string
}

/**
 * 组件属性
 */
interface Props {
  keyword?: string
  status?: string
  role?: string
  loading?: boolean
}

withDefaults(defineProps<Props>(), {
  keyword: '',
  status: '',
  role: '',
  loading: false,
})

const emit = defineEmits<{
  search: [params: SearchParams]
  reset: []
  refresh: []
}>()

const statusOptions = [
  { label: '全部状态', value: '' },
  { label: '在线', value: 'online' as UserStatus },
  { label: '离线', value: 'offline' as UserStatus },
  { label: '离开', value: 'away' as UserStatus },
]

const roleOptions = [
  { label: '全部角色', value: '' },
  { label: '超级管理员', value: 'super_admin' as UserRole },
  { label: '管理员', value: 'admin' as UserRole },
  { label: '普通用户', value: 'user' as UserRole },
]
</script>

<template>
  <NSpace align="center" wrap>
    <NInput
      :value="keyword"
      placeholder="搜索用户名/昵称/邮箱"
      clearable
      style="width: 240px"
      @update:value="(v) => emit('search', { keyword: v, status, role })"
      @keyup.enter="emit('search', { keyword, status, role })"
    >
      <template #prefix>
        <Search :size="16" />
      </template>
    </NInput>

    <NSelect
      :value="status"
      :options="statusOptions"
      placeholder="选择状态"
      clearable
      style="width: 140px"
      @update:value="(v) => emit('search', { keyword, status: v, role })"
    />

    <NSelect
      :value="role"
      :options="roleOptions"
      placeholder="选择角色"
      clearable
      style="width: 140px"
      @update:value="(v) => emit('search', { keyword, status, role: v })"
    />

    <NButton type="primary" :loading="loading" @click="emit('search', { keyword, status, role })">
      <template #icon>
        <Search :size="16" />
      </template>
      搜索
    </NButton>

    <NButton :disabled="loading" @click="emit('reset')">
      重置
    </NButton>

    <NButton quaternary :loading="loading" @click="emit('refresh')">
      <template #icon>
        <RefreshCw :size="16" />
      </template>
    </NButton>
  </NSpace>
</template>
