<script setup lang="ts">
import { h } from 'vue'
import {
  NDataTable,
  NTag,
  NSpace,
  NButton,
  NPopconfirm,
  NIcon,
  type DataTableColumns,
} from 'naive-ui'
import { Eye, Trash2, Shield, ShieldCheck } from 'lucide-vue-next'
import type { IPEntry, IPListType } from '@/api/security'

/**
 * 组件属性定义
 */
interface Props {
  /** IP列表数据 */
  data: IPEntry[]
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
  /** 查看IP详情 */
  (e: 'view', ip: IPEntry): void
  /** 删除IP */
  (e: 'delete', ip: IPEntry): void
}

withDefaults(defineProps<Props>(), {
  data: () => [],
  loading: false,
  selectedKeys: () => [],
})

const emit = defineEmits<Emits>()

/**
 * 列表类型映射配置
 */
const listTypeConfig: Record<IPListType, { text: string; type: 'error' | 'success'; icon: typeof Shield }> = {
  blacklist: { text: '黑名单', type: 'error', icon: Shield },
  whitelist: { text: '白名单', type: 'success', icon: ShieldCheck },
}

/**
 * 格式化日期时间
 * @param dateString 日期字符串
 * @returns 格式化后的日期时间
 */
const formatDateTime = (dateString: string | null): string => {
  if (!dateString) return '永不过期'
  const date = new Date(dateString)
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
}

/**
 * 检查IP是否已过期
 * @param expiresAt 过期时间
 * @returns 是否已过期
 */
const isExpired = (expiresAt: string | null): boolean => {
  if (!expiresAt) return false
  return new Date(expiresAt) < new Date()
}

/**
 * 表格列配置
 */
const columns: DataTableColumns<IPEntry> = [
  {
    type: 'selection',
  },
  {
    title: 'IP地址',
    key: 'ip_address',
    minWidth: 150,
    render(row) {
      return h('span', {
        style: {
          fontFamily: 'monospace',
          fontSize: '14px',
          fontWeight: 500,
        },
      }, row.ip_address)
    },
  },
  {
    title: '类型',
    key: 'list_type',
    width: 120,
    align: 'center',
    render(row) {
      const config = listTypeConfig[row.list_type]
      return h(NTag, {
        type: config.type,
        size: 'small',
        bordered: false,
      }, {
        default: () => h(NSpace, { align: 'center', size: 4 }, {
          default: () => [
            h(NIcon, { component: config.icon, size: 12 }),
            config.text,
          ],
        }),
      })
    },
  },
  {
    title: '备注',
    key: 'remark',
    minWidth: 200,
    ellipsis: {
      tooltip: true,
    },
    render(row) {
      return h('span', {}, row.remark || '-')
    },
  },
  {
    title: '过期时间',
    key: 'expires_at',
    width: 160,
    render(row) {
      const expired = isExpired(row.expires_at)
      return h('span', {
        style: {
          color: expired ? 'var(--error-color)' : 'inherit',
          textDecoration: expired ? 'line-through' : 'none',
        },
      }, formatDateTime(row.expires_at))
    },
  },
  {
    title: '创建时间',
    key: 'created_at',
    width: 160,
    render(row) {
      return h('span', {}, formatDateTime(row.created_at))
    },
  },
  {
    title: '操作',
    key: 'actions',
    width: 120,
    fixed: 'right',
    render(row) {
      return h(NSpace, { size: 8 }, {
        default: () => [
          h(
            NButton,
            {
              size: 'small',
              type: 'primary',
              tertiary: true,
              onClick: () => emit('view', row),
            },
            {
              icon: () => h(NIcon, null, { default: () => h(Eye) }),
            },
          ),
          h(
            NPopconfirm,
            {
              onPositiveClick: () => emit('delete', row),
            },
            {
              trigger: () =>
                h(
                  NButton,
                  {
                    size: 'small',
                    type: 'error',
                    tertiary: true,
                  },
                  {
                    icon: () => h(NIcon, null, { default: () => h(Trash2) }),
                  },
                ),
              default: () => `确定删除IP "${row.ip_address}" 吗？`,
            },
          ),
        ],
      })
    },
  },
]
</script>

<template>
  <NDataTable
    :columns="columns"
    :data="data"
    :loading="loading"
    :pagination="false"
    :row-key="(row: IPEntry) => row.id"
    :checked-row-keys="selectedKeys"
    @update:checked-row-keys="$emit('update:selectedKeys', $event as string[])"
    striped
    size="small"
    class="ip-table"
  />
</template>

<style scoped>
.ip-table :deep(.n-data-table-td) {
  font-size: 13px;
}
</style>
