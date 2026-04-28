<script setup lang="ts">
import { h } from 'vue'
import { NDataTable, NButton, NSpace, NTag, NTooltip } from 'naive-ui'
import { Edit, Trash2, MessageSquare, BarChart3 } from 'lucide-vue-next'
import type { DataTableColumns } from 'naive-ui'
import type { RoomInfo } from '@/api/rooms'

/**
 * 组件属性
 */
interface Props {
  /** 表格数据 */
  data: RoomInfo[]
  /** 加载状态 */
  loading?: boolean
  /** 选中行的 keys */
  selectedKeys?: (string | number)[]
}

withDefaults(defineProps<Props>(), {
  loading: false,
  selectedKeys: () => [],
})

/**
 * 组件事件
 */
interface Emits {
  /** 更新选中 keys */
  (e: 'update:selectedKeys', keys: (string | number)[]): void
  /** 选择房间 */
  (e: 'select', room: RoomInfo): void
  /** 查看消息 */
  (e: 'viewMessages', room: RoomInfo): void
  /** 查看分析 */
  (e: 'viewAnalytics', room: RoomInfo): void
  /** 编辑 */
  (e: 'edit', room: RoomInfo): void
  /** 删除 */
  (e: 'delete', room: RoomInfo): void
}

const emit = defineEmits<Emits>()

/**
 * 处理选择变化
 */
const handleCheck = (keys: (string | number)[]) => {
  emit('update:selectedKeys', keys)
}

/**
 * 处理行点击
 */
const handleRowClick = (row: RoomInfo) => {
  emit('select', row)
}

/**
 * 处理查看消息
 */
const handleViewMessages = (row: RoomInfo, event?: Event) => {
  event?.stopPropagation()
  emit('viewMessages', row)
}

/**
 * 处理查看分析
 */
const handleViewAnalytics = (row: RoomInfo, event?: Event) => {
  event?.stopPropagation()
  emit('viewAnalytics', row)
}

/**
 * 处理编辑
 */
const handleEdit = (row: RoomInfo, event?: Event) => {
  event?.stopPropagation()
  emit('edit', row)
}

/**
 * 处理删除
 */
const handleDelete = (row: RoomInfo, event?: Event) => {
  event?.stopPropagation()
  emit('delete', row)
}

/**
 * 格式化时间
 */
const formatTime = (time: string) => {
  return new Date(time).toLocaleString('zh-CN')
}

/**
 * 表格列定义
 */
const columns: DataTableColumns<RoomInfo> = [
  {
    type: 'selection',
    fixed: 'left',
  },
  {
    title: '房间名称',
    key: 'name',
    fixed: 'left',
    width: 180,
    render(row) {
      return h(
        'div',
        {
          style: {
            display: 'flex',
            alignItems: 'center',
            gap: '8px',
          },
        },
        [
          h(
            NTag,
            {
              size: 'small',
              type: row.is_private ? 'error' : 'success',
            },
            { default: () => (row.is_private ? '私有' : '公开') }
          ),
          h('span', row.name),
        ]
      )
    },
  },
  {
    title: '描述',
    key: 'description',
    ellipsis: {
      tooltip: true,
    },
    width: 200,
    render(row) {
      return row.description || '-'
    },
  },
  {
    title: '房主',
    key: 'owner',
    width: 120,
    render(row) {
      return row.owner?.username || '-'
    },
  },
  {
    title: '成员',
    key: 'member_count',
    width: 100,
    render(row) {
      return `${row.member_count} / ${row.max_members}`
    },
  },
  {
    title: '创建时间',
    key: 'created_at',
    width: 170,
    render(row) {
      return formatTime(row.created_at)
    },
  },
  {
    title: '操作',
    key: 'actions',
    fixed: 'right',
    width: 220,
    render(row) {
      return h(
        NSpace,
        { size: 'small' },
        {
          default: () => [
            h(
              NTooltip,
              {},
              {
                trigger: () =>
                  h(
                    NButton,
                    {
                      size: 'small',
                      quaternary: true,
                      circle: true,
                      onClick: (e: Event) => handleViewMessages(row, e),
                    },
                    {
                      icon: () => h(MessageSquare, { size: 16 }),
                    }
                  ),
                default: () => '消息管理',
              }
            ),
            h(
              NTooltip,
              {},
              {
                trigger: () =>
                  h(
                    NButton,
                    {
                      size: 'small',
                      quaternary: true,
                      circle: true,
                      onClick: (e: Event) => handleViewAnalytics(row, e),
                    },
                    {
                      icon: () => h(BarChart3, { size: 16 }),
                    }
                  ),
                default: () => '数据分析',
              }
            ),
            h(
              NTooltip,
              {},
              {
                trigger: () =>
                  h(
                    NButton,
                    {
                      size: 'small',
                      quaternary: true,
                      circle: true,
                      onClick: (e: Event) => handleEdit(row, e),
                    },
                    {
                      icon: () => h(Edit, { size: 16 }),
                    }
                  ),
                default: () => '编辑',
              }
            ),
            h(
              NTooltip,
              {},
              {
                trigger: () =>
                  h(
                    NButton,
                    {
                      size: 'small',
                      type: 'error',
                      quaternary: true,
                      circle: true,
                      onClick: (e: Event) => handleDelete(row, e),
                    },
                    {
                      icon: () => h(Trash2, { size: 16 }),
                    }
                  ),
                default: () => '删除',
              }
            ),
          ],
        }
      )
    },
  },
]

/**
 * 行属性
 */
const rowProps = (row: RoomInfo) => {
  return {
    style: {
      cursor: 'pointer',
    },
    onClick: () => handleRowClick(row),
  }
}
</script>

<template>
  <div class="room-table">
    <NDataTable
      :columns="columns"
      :data="data"
      :loading="loading"
      :row-key="(row) => row.id"
      :checked-row-keys="selectedKeys"
      :row-props="rowProps"
      @update:checked-row-keys="handleCheck"
      striped
      size="medium"
      :scroll-x="1000"
    />
  </div>
</template>

<style scoped>
.room-table {
  width: 100%;
}

.room-table :deep(.n-data-table-td) {
  cursor: pointer;
}

.room-table :deep(.n-data-table-td:last-child) {
  cursor: default;
}
</style>
