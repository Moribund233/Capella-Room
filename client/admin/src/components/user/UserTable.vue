<script setup lang="ts">
/**
 * UserTable - 用户表格组件
 *
 * 封装 DataTable，提供用户列表的展示、选择、排序、操作功能
 *

 */
import { computed, h } from 'vue'
import { NTag, NAvatar, NSpace, NButton, NPopconfirm, NIcon, NText } from 'naive-ui'
import { Edit, Eye, Trash2 } from 'lucide-vue-next'
import { DataTable } from '@/components/common'
import type { TableColumn } from '@/composables'
import type { UserInfo, UserRole, UserStatus } from '@/types'

/**
 * 组件属性定义
 */
interface Props {
  /** 用户数据 */
  data: UserInfo[]
  /** 加载状态 */
  loading?: boolean
  /** 选中的用户 keys */
  selectedKeys?: (string | number)[]
  /** 总条数（用于分页） */
  total?: number
  /** 当前页码 */
  currentPage?: number
  /** 每页数量 */
  pageSize?: number
}

/**
 * 组件事件定义
 */
interface Emits {
  /** 选择变化事件 */
  (e: 'selection-change', keys: (string | number)[], rows: UserInfo[]): void
  /** 查看用户事件 */
  (e: 'view', user: UserInfo): void
  /** 编辑用户事件 */
  (e: 'edit', user: UserInfo): void
  /** 删除用户事件 */
  (e: 'delete', user: UserInfo): void
  /** 分页变化事件 */
  (e: 'page-change', page: number, pageSize: number): void
}

const props = withDefaults(defineProps<Props>(), {
  loading: false,
  selectedKeys: () => [],
  total: 0,
  currentPage: 1,
  pageSize: 10,
})

// 使用 props 避免未使用警告
// eslint-disable-next-line @typescript-eslint/no-unused-vars
const _props = props

const emit = defineEmits<Emits>()

/**
 * 处理分页变化
 * @param page - 页码
 * @param pageSize - 每页数量
 */
const handlePageChange = (page: number, pageSize: number) => {
  emit('page-change', page, pageSize)
}

/**
 * 获取在线状态标签类型
 * @param status - 用户在线状态
 */
const getOnlineStatusType = (status: UserStatus): 'success' | 'warning' | 'default' => {
  const statusMap: Record<UserStatus, 'success' | 'warning' | 'default'> = {
    online: 'success',
    offline: 'default',
    away: 'warning',
  }
  return statusMap[status] || 'default'
}

/**
 * 获取在线状态显示文本
 * @param status - 用户在线状态
 */
const getOnlineStatusText = (status: UserStatus): string => {
  const statusMap: Record<UserStatus, string> = {
    online: '在线',
    offline: '离线',
    away: '离开',
  }
  return statusMap[status] || status
}

/**
 * 获取账号状态标签类型
 * @param isActive - 账号是否启用
 */
const getAccountStatusType = (isActive: boolean): 'success' | 'error' => {
  return isActive ? 'success' : 'error'
}

/**
 * 获取账号状态显示文本
 * @param isActive - 账号是否启用
 */
const getAccountStatusText = (isActive: boolean): string => {
  return isActive ? '正常' : '已禁用'
}

/**
 * 获取角色标签类型
 * @param role - 用户角色
 */
const getRoleType = (role: UserRole): 'error' | 'warning' | 'default' => {
  const roleMap: Record<UserRole, 'error' | 'warning' | 'default'> = {
    super_admin: 'error',
    admin: 'warning',
    user: 'default',
  }
  return roleMap[role] || 'default'
}

/**
 * 获取角色显示文本
 * @param role - 用户角色
 */
const getRoleText = (role: UserRole): string => {
  const roleMap: Record<UserRole, string> = {
    super_admin: '超级管理员',
    admin: '管理员',
    user: '普通用户',
  }
  return roleMap[role] || role
}

/**
 * 处理查看用户
 * @param row - 用户数据
 */
const handleView = (row: UserInfo) => {
  emit('view', row)
}

/**
 * 处理编辑用户
 * @param row - 用户数据
 */
const handleEdit = (row: UserInfo) => {
  emit('edit', row)
}

/**
 * 处理删除用户
 * @param row - 用户数据
 */
const handleDelete = (row: UserInfo) => {
  emit('delete', row)
}

/**
 * 处理选择变化
 * @param keys - 选中的 keys
 * @param rows - 选中的行数据
 */
const handleSelectionChange = (keys: (string | number)[], rows: UserInfo[]) => {
  emit('selection-change', keys, rows)
}

/**
 * 表格列配置
 */
const columns = computed<TableColumn<UserInfo>[]>(() => [
  {
    key: 'username',
    title: '用户信息',
    minWidth: 200,
    render: (row: UserInfo) => {
      return h(
        NSpace,
        { align: 'center', size: 12 },
        {
          default: () => [
            h(NAvatar, {
              src: row.avatar_url || undefined,
              fallbackSrc: `https://api.dicebear.com/7.x/avataaars/svg?seed=${row.username}`,
              size: 40,
              round: true,
            }),
            h(
              'div',
              {},
              {
                default: () => [
                  h(
                    'div',
                    { style: { fontWeight: 500, marginBottom: '4px' } },
                    { default: () => row.nickname || row.username }
                  ),
                  h(
                    NText,
                    { depth: 3, style: { fontSize: '12px' } },
                    { default: () => row.email || '-' }
                  ),
                ],
              }
            ),
          ],
        }
      )
    },
  },
  {
    key: 'role',
    title: '角色',
    width: 120,
    align: 'center',
    sortable: true,
    render: (row: UserInfo) => {
      return h(
        NTag,
        { type: getRoleType(row.role || 'user'), size: 'small' },
        { default: () => getRoleText(row.role || 'user') }
      )
    },
  },
  {
    key: 'is_active',
    title: '账号状态',
    width: 100,
    align: 'center',
    sortable: true,
    render: (row: UserInfo) => {
      return h(
        NTag,
        { type: getAccountStatusType(row.is_active ?? true), size: 'small' },
        { default: () => getAccountStatusText(row.is_active ?? true) }
      )
    },
  },
  {
    key: 'status',
    title: '在线状态',
    width: 100,
    align: 'center',
    sortable: true,
    render: (row: UserInfo) => {
      return h(
        NTag,
        { type: getOnlineStatusType(row.status || 'offline'), size: 'small' },
        { default: () => getOnlineStatusText(row.status || 'offline') }
      )
    },
  },
  {
    key: 'created_at',
    title: '创建时间',
    width: 180,
    sortable: true,
    render: (row: UserInfo) => {
      if (!row.created_at) return '-'
      const date = new Date(row.created_at)
      return date.toLocaleString('zh-CN')
    },
  },
  {
    key: 'actions',
    title: '操作',
    width: 200,
    fixed: 'right',
    align: 'center',
    render: (row: UserInfo) => {
      return h(
        NSpace,
        { justify: 'center', size: 8 },
        {
          default: () => [
            h(
              NButton,
              {
                size: 'small',
                quaternary: true,
                onClick: () => handleView(row),
              },
              {
                icon: () => h(NIcon, null, { default: () => h(Eye) }),
                default: () => '查看',
              }
            ),
            h(
              NButton,
              {
                size: 'small',
                quaternary: true,
                type: 'primary',
                onClick: () => handleEdit(row),
              },
              {
                icon: () => h(NIcon, null, { default: () => h(Edit) }),
                default: () => '编辑',
              }
            ),
            h(
              NPopconfirm,
              {
                onPositiveClick: () => handleDelete(row),
              },
              {
                trigger: () =>
                  h(
                    NButton,
                    {
                      size: 'small',
                      quaternary: true,
                      type: 'error',
                    },
                    {
                      icon: () => h(NIcon, null, { default: () => h(Trash2) }),
                      default: () => '删除',
                    }
                  ),
                default: () => `确定要删除用户 "${row.nickname || row.username}" 吗？`,
              }
            ),
          ],
        }
      )
    },
  },
])
</script>

<template>
  <DataTable
    :data="data"
    :columns="columns"
    :loading="loading"
    :pagination="true"
    :selectable="true"
    :sortable="true"
    :remote="true"
    :total="total"
    row-key="id"
    :pagination-options="{
      defaultPage: currentPage,
      defaultPageSize: pageSize,
      pageSizes: [10, 20, 50, 100],
      showQuickJumper: true,
      showSizePicker: true,
    }"
    @selection-change="handleSelectionChange"
    @page-change="handlePageChange"
  />
</template>
