<script setup lang="ts">
import { h } from 'vue'
import { storeToRefs } from 'pinia'
import { useRouter } from 'vue-router'
import { NCard, NPagination, NTag, useDialog } from 'naive-ui'
import { MessageSquare, BarChart3, Edit, Trash2 } from 'lucide-vue-next'
import { RoomSearchForm, RoomTable, RoomTableToolbar } from '@/components/rooms'
import { MobileTableCard } from '@/components/common'
import { useRoomStore } from '@/stores'
import type { RoomInfo } from '@/api/rooms'
import type { MobileAction, MobileColumn } from '@/components/common/MobileTableCard.vue'

/**
 * 使用房间管理 Store
 */
const roomStore = useRoomStore()
const router = useRouter()
const dialog = useDialog()

// 使用 storeToRefs 保持响应性
const {
  rooms,
  loading,
  total,
  page,
  pageSize,
  selectedKeys,
  searchParams,
} = storeToRefs(roomStore)

/**
 * 处理搜索
 */
const handleSearch = (values: { keyword: string }) => {
  roomStore.searchRooms(values.keyword)
}

/**
 * 处理重置
 */
const handleReset = () => {
  roomStore.resetSearch()
}

/**
 * 处理刷新
 */
const handleRefresh = async () => {
  await roomStore.refreshRooms()
}

/**
 * 处理分页变化
 */
const handlePaginationChange = (newPage: number, newPageSize: number) => {
  roomStore.handlePageChange(newPage, newPageSize)
}

/**
 * 处理选择房间
 */
const handleSelectRoom = (room: RoomInfo) => {
  roomStore.selectRoom(room)
}

/**
 * 处理查看消息
 */
const handleViewMessages = (room: RoomInfo) => {
  roomStore.selectRoom(room)
  router.push(`/rooms/${room.id}/messages`)
}

/**
 * 处理数据分析
 */
const handleViewAnalytics = (room: RoomInfo) => {
  roomStore.selectRoom(room)
  router.push(`/rooms/${room.id}/analytics`)
}

/**
 * 处理编辑房间
 */
const handleEdit = (room: RoomInfo) => {
  // TODO: 实现编辑功能
  console.log('编辑房间:', room)
}

/**
 * 处理删除房间
 */
const handleDelete = async (room: RoomInfo) => {
  dialog.warning({
    title: '确认删除',
    content: `确定要删除房间 "${room.name}" 吗？此操作不可恢复。`,
    positiveText: '删除',
    negativeText: '取消',
    onPositiveClick: async () => {
      await roomStore.deleteRoom(room.id)
    },
  })
}

/**
 * 处理移动端行点击
 */
const handleMobileRowClick = (row: unknown) => {
  handleSelectRoom(row as RoomInfo)
}

// ==================== 移动端配置 ====================

/** 移动端列配置 */
const mobileColumns: MobileColumn<RoomInfo>[] = [
  {
    key: 'is_private',
    title: '类型',
    render: (row: RoomInfo) => {
      return h(
        NTag,
        { size: 'small', type: row.is_private ? 'error' : 'success' },
        { default: () => (row.is_private ? '私有' : '公开') }
      )
    },
  },
  { key: 'description', title: '描述' },
  { key: 'owner', title: '房主', render: (row: RoomInfo) => row.owner?.username || '-' },
  { key: 'member_count', title: '成员', render: (row: RoomInfo) => `${row.member_count} / ${row.max_members}` },
  { key: 'created_at', title: '创建时间', render: (row: RoomInfo) => new Date(row.created_at).toLocaleString('zh-CN') },
]

/** 移动端操作按钮配置 */
const mobileActions: MobileAction<RoomInfo>[] = [
  {
    label: '消息',
    icon: MessageSquare,
    type: 'default',
    onClick: (room: RoomInfo) => handleViewMessages(room),
  },
  {
    label: '分析',
    icon: BarChart3,
    type: 'info',
    onClick: (room: RoomInfo) => handleViewAnalytics(room),
  },
  {
    label: '编辑',
    icon: Edit,
    type: 'primary',
    onClick: (room: RoomInfo) => handleEdit(room),
  },
  {
    label: '删除',
    icon: Trash2,
    type: 'error',
    onClick: (room: RoomInfo) => handleDelete(room),
  },
]
</script>

<template>
  <div class="room-list-page">
    <div class="page-header">
      <h1 class="page-title">房间管理</h1>
      <p class="page-description">管理聊天室，包括查看、编辑、删除房间等操作</p>
    </div>

    <NCard class="search-card" :bordered="false">
      <RoomSearchForm
        :keyword="searchParams.keyword"
        :loading="loading"
        @search="handleSearch"
        @reset="handleReset"
        @refresh="handleRefresh"
      />
    </NCard>

    <NCard class="toolbar-card" :bordered="false">
      <RoomTableToolbar
        :selected-count="selectedKeys.length"
        :total="total"
        :loading="loading"
      />
    </NCard>

    <NCard class="table-card" :bordered="false">
      <!-- 桌面端：表格视图 -->
      <div class="desktop-view">
        <RoomTable
          v-model:selected-keys="selectedKeys"
          :data="rooms"
          :loading="loading"
          @select="handleSelectRoom"
          @view-messages="handleViewMessages"
          @view-analytics="handleViewAnalytics"
          @edit="handleEdit"
          @delete="handleDelete"
        />
        <div v-if="total > 0" class="pagination-wrapper">
          <NPagination
            :page="page"
            :page-size="pageSize"
            :item-count="total"
            :page-sizes="[10, 20, 50, 100]"
            show-size-picker
            show-quick-jumper
            @update:page="handlePaginationChange($event, pageSize)"
            @update:page-size="handlePaginationChange(1, $event)"
          />
        </div>
      </div>

      <!-- 移动端：卡片视图 -->
      <div class="mobile-view">
        <MobileTableCard
          :data="rooms"
          :columns="mobileColumns as MobileColumn[]"
          title-column="name"
          :actions="mobileActions as MobileAction[]"
          @row-click="handleMobileRowClick"
        />
        <div v-if="total > 0" class="mobile-pagination">
          <NPagination
            :page="page"
            :page-count="Math.ceil(total / pageSize)"
            :simple="true"
            @update:page="handlePaginationChange($event, pageSize)"
          />
        </div>
      </div>
    </NCard>
  </div>
</template>

<style scoped>
.room-list-page {
  min-height: 100%;
}

.page-header {
  margin-bottom: 24px;
}

.page-title {
  font-size: 28px;
  font-weight: 600;
  margin: 0 0 8px 0;
  color: var(--text-primary);
}

.page-description {
  font-size: 14px;
  color: var(--text-secondary);
  margin: 0;
}

.search-card,
.toolbar-card {
  margin-bottom: 16px;
}

.table-card {
  min-height: 400px;
}

/* 桌面端视图 */
.desktop-view {
  display: block;
}

/* 移动端视图 */
.mobile-view {
  display: none;
}

.pagination-wrapper {
  margin-top: 16px;
  display: flex;
  justify-content: flex-end;
}

.mobile-pagination {
  margin-top: 16px;
  display: flex;
  justify-content: center;
}

/* 移动端断点：768px */
@media screen and (max-width: 768px) {
  .page-title {
    font-size: 24px;
  }

  .desktop-view {
    display: none;
  }

  .mobile-view {
    display: block;
  }
}
</style>
