<script setup lang="ts">
import { h } from 'vue'
import { NDataTable, NTag, NAvatar, NSpace, NButton, NPopconfirm, NIcon, type DataTableColumns } from 'naive-ui'
import { Edit, Eye, Trash2 } from 'lucide-vue-next'
import type { UserInfo, UserRole, UserStatus } from '@/types'

/**
 * 组件属性定义
 */
interface Props {
  /** 用户列表数据 */
  data: UserInfo[]
  /** 加载状态 */
  loading?: boolean
  /** 选中的行keys */
  selectedKeys?: (string | number)[]
}

/**
 * 组件事件定义
 */
interface Emits {
  /** 选中行变化 */
  (e: 'update:selectedKeys', keys: (string | number)[]): void
  /** 查看用户 */
  (e: 'view', user: UserInfo): void
  /** 编辑用户 */
  (e: 'edit', user: UserInfo): void
  /** 删除用户 */
  (e: 'delete', user: UserInfo): void
}

withDefaults(defineProps<Props>(), {
  loading: false,
  selectedKeys: () => [],
})

const emit = defineEmits<Emits>()

/**
 * 状态映射配置
 */
const statusConfig: Record<UserStatus, { text: string; type: 'success' | 'warning' | 'default' }> = {
  online: { text: '在线', type: 'success' },
  offline: { text: '离线', type: 'default' },
  away: { text: '离开', type: 'warning' },
}

/**
 * 角色映射配置
 */
const roleConfig: Record<UserRole, { text: string; type: 'error' | 'warning' | 'default' }> = {
  super_admin: { text: '超级管理员', type: 'error' },
  admin: { text: '管理员', type: 'warning' },
  user: { text: '普通用户', type: 'default' },
}

/**
 * 表格列配置
 */
const columns: DataTableColumns<UserInfo> = [
  {
    type: 'selection',
  },
  {
    title: '用户信息',
    key: 'username',
    minWidth: 200,
    render(row) {
      return h(NSpace, { align: 'center', size: 12 }, {
        default: () => [
          h(NAvatar, {
            src: row.avatar_url || undefined,
            fallbackSrc: `https://api.dicebear.com/7.x/avataaars/svg?seed=${row.username}`,
            size: 40,
            round: true,
          }),
          h('div', {}, [
            h('div', { style: { fontWeight: 500, marginBottom: '4px' } }, row.nickname || row.username),
            h('div', { style: { fontSize: '12px', color: 'var(--text-secondary)' } }, row.email || '-'),
          ]),
        ],
      })
    },
  },
  {
    title: '角色',
    key: 'role',
    width: 120,
    align: 'center',
    sorter: true,
    render(row) {
      const config = roleConfig[(row.role || 'user') as UserRole]
      return h(NTag, { type: config.type, size: 'small' }, { default: () => config.text })
    },
  },
  {
    title: '账号状态',
    key: 'is_active',
    width: 100,
    align: 'center',
    sorter: true,
    render(row) {
      const isActive = row.is_active !== false
      return h(NTag, { type: isActive ? 'success' : 'error', size: 'small' }, {
        default: () => isActive ? '正常' : '已禁用',
      })
    },
  },
  {
    title: '在线状态',
    key: 'status',
    width: 100,
    align: 'center',
    sorter: true,
    render(row) {
      const config = statusConfig[(row.status || 'offline') as UserStatus]
      return h(NTag, { type: config.type, size: 'small' }, { default: () => config.text })
    },
  },
  {
    title: '创建时间',
    key: 'created_at',
    width: 180,
    sorter: true,
    render(row) {
      return row.created_at ? new Date(row.created_at).toLocaleString('zh-CN') : '-'
    },
  },
  {
    title: '操作',
    key: 'actions',
    width: 200,
    fixed: 'right',
    align: 'center',
    render(row) {
      return h(NSpace, { justify: 'center', size: 8 }, {
        default: () => [
          h(NButton, { size: 'small', quaternary: true, onClick: () => emit('view', row) }, {
            icon: () => h(NIcon, null, { default: () => h(Eye) }),
            default: () => '查看',
          }),
          h(NButton, { size: 'small', quaternary: true, type: 'primary', onClick: () => emit('edit', row) }, {
            icon: () => h(NIcon, null, { default: () => h(Edit) }),
            default: () => '编辑',
          }),
          h(NPopconfirm, { onPositiveClick: () => emit('delete', row) }, {
            trigger: () => h(NButton, { size: 'small', quaternary: true, type: 'error' }, {
              icon: () => h(NIcon, null, { default: () => h(Trash2) }),
              default: () => '删除',
            }),
            default: () => `确定要删除用户 "${row.nickname || row.username}" 吗？`,
          }),
        ],
      })
    },
  },
]

/**
 * 处理选中行变化
 */
const handleCheckedRowKeysChange = (keys: (string | number)[]) => {
  emit('update:selectedKeys', keys)
}
</script>

<template>
  <NDataTable
    :columns="columns"
    :data="data"
    :row-key="(row: UserInfo) => row.id"
    :loading="loading"
    :checked-row-keys="selectedKeys"
    @update:checked-row-keys="handleCheckedRowKeysChange"
  />
</template>
