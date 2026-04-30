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
import { Eye, Trash2 } from 'lucide-vue-next'
import type { AdminMessageInfo } from '@/api/admin'

/**
 * 组件属性定义
 */
interface Props {
  /** 消息列表数据 */
  data: AdminMessageInfo[]
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
  /** 查看消息 */
  (e: 'view', message: AdminMessageInfo): void
  /** 删除消息 */
  (e: 'delete', message: AdminMessageInfo): void
}

withDefaults(defineProps<Props>(), {
  loading: false,
  selectedKeys: () => [],
})

const emit = defineEmits<Emits>()

/**
 * 消息类型映射配置
 */
const messageTypeConfig: Record<string, { text: string; type: 'default' | 'info' | 'success' | 'warning' }> = {
  text: { text: '文本', type: 'default' },
  image: { text: '图片', type: 'info' },
  file: { text: '文件', type: 'success' },
  system: { text: '系统', type: 'warning' },
}

/**
 * 格式化日期时间
 * @param dateString 日期字符串
 * @returns 格式化后的日期时间
 */
const formatDateTime = (dateString: string): string => {
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
 * 截断文本
 * @param text 原始文本
 * @param maxLength 最大长度
 * @returns 截断后的文本
 */
const truncateText = (text: string, maxLength: number = 50): string => {
  if (text.length <= maxLength) return text
  return text.substring(0, maxLength) + '...'
}

/**
 * 表格列配置
 */
const columns: DataTableColumns<AdminMessageInfo> = [
  {
    type: 'selection',
  },
  {
    title: '消息内容',
    key: 'content',
    minWidth: 250,
    ellipsis: {
      tooltip: true,
    },
    render(row) {
      const typeConfig = messageTypeConfig[row.message_type] ?? messageTypeConfig.text
      return h(NSpace, { align: 'center', size: 8 }, {
        default: () => [
          h(NTag, { type: typeConfig!.type, size: 'small' }, { default: () => typeConfig!.text }),
          h('span', {
            style: {
              textDecoration: row.is_deleted ? 'line-through' : 'none',
              color: row.is_deleted ? 'var(--text-secondary)' : 'inherit',
            },
          }, truncateText(row.content, 60)),
        ],
      })
    },
  },
  {
    title: '发送者',
    key: 'sender',
    width: 150,
    render(row) {
      return h('span', {}, row.sender?.username || '-')
    },
  },
  {
    title: '房间ID',
    key: 'room_id',
    width: 120,
    ellipsis: {
      tooltip: true,
    },
  },
  {
    title: '消息类型',
    key: 'message_type',
    width: 100,
    align: 'center',
    render(row) {
      const config = messageTypeConfig[row.message_type] ?? messageTypeConfig.text
      return h(NTag, { type: config!.type, size: 'small' }, { default: () => config!.text })
    },
  },
  {
    title: '状态',
    key: 'is_deleted',
    width: 80,
    align: 'center',
    render(row) {
      if (row.is_deleted) {
        return h(NTag, { type: 'error', size: 'small' }, { default: () => '已删除' })
      }
      return h(NTag, { type: 'success', size: 'small' }, { default: () => '正常' })
    },
  },
  {
    title: '发送时间',
    key: 'created_at',
    width: 150,
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
          !row.is_deleted
            ? h(
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
                  default: () => '确定删除这条消息吗？',
                },
              )
            : null,
        ],
      })
    },
  },
]

/**
 * 处理选中行变化
 * @param keys 选中的行keys
 */
const handleCheckedRowKeysChange = (keys: (string | number)[]) => {
  emit('update:selectedKeys', keys)
}
</script>

<template>
  <NDataTable
    :columns="columns"
    :data="data"
    :loading="loading"
    :pagination="false"
    :row-key="(row: AdminMessageInfo) => row.id"
    :checked-row-keys="selectedKeys"
    @update:checked-row-keys="handleCheckedRowKeysChange"
    size="small"
    striped
  />
</template>

<style scoped>
:deep(.n-data-table-td) {
  font-size: 13px;
}
</style>
