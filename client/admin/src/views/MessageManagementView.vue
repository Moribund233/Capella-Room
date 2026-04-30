<script setup lang="ts">
import { ref, h, onMounted } from 'vue'
import { NCard, NPagination, NButton, useMessage, useDialog } from 'naive-ui'
import { Shield, Eye, Trash2 } from 'lucide-vue-next'
import type { MobileAction } from '@/components/common/MobileTableCard.vue'
import { MessageSearchForm, MessageTable, MessageDetailModal } from '@/components/messages'
import type { MessageSearchParams } from '@/components/messages/MessageSearchForm.vue'
import { MobileTableCard } from '@/components/common'
import { useStatusBar } from '@/composables'
import { adminApi, type AdminMessageInfo } from '@/api/admin'

const message = useMessage()
const dialog = useDialog()
const { setContent } = useStatusBar()

// ==================== 数据状态 ====================

/** 消息列表数据 */
const data = ref<AdminMessageInfo[]>([])
/** 加载状态 */
const loading = ref(false)
/** 总消息数 */
const total = ref(0)
/** 当前页码 */
const page = ref(1)
/** 每页数量 */
const pageSize = ref(10)
/** 选中的消息 keys */
const selectedKeys = ref<(string | number)[]>([])

/** 搜索参数 */
const searchParams = ref<MessageSearchParams>({
  keyword: '',
  roomId: null,
  messageType: null,
  startTime: null,
  endTime: null,
})

/** 当前搜索参数缓存（用于刷新） */
const currentSearchParams = ref<MessageSearchParams>({
  keyword: '',
  roomId: null,
  messageType: null,
  startTime: null,
  endTime: null,
})

// ==================== 数据获取 ====================

/**
 * 获取消息列表
 * @param params 搜索参数
 */
const fetchMessageList = async (params: {
  keyword?: string
  roomId?: string | null
  messageType?: string | null
  startTime?: number | null
  endTime?: number | null
  page?: number
  pageSize?: number
} = {}) => {
  loading.value = true

  try {
    const response = await adminApi.getMessageList({
      page: params.page ?? page.value,
      page_size: params.pageSize ?? pageSize.value,
      search: params.keyword || undefined,
      room_id: params.roomId || undefined,
    })

    if (response.success && response.data) {
      data.value = response.data.messages
      total.value = response.data.total
      page.value = params.page ?? page.value
      pageSize.value = params.pageSize ?? pageSize.value
      return true
    }
    return false
  } catch (error) {
    console.error('获取消息列表失败:', error)
    message.error('获取消息列表失败')
    return false
  } finally {
    loading.value = false
  }
}

/**
 * 刷新当前列表
 */
const refresh = async () => {
  return fetchMessageList({
    keyword: currentSearchParams.value.keyword,
    roomId: currentSearchParams.value.roomId,
    messageType: currentSearchParams.value.messageType,
    startTime: currentSearchParams.value.startTime,
    endTime: currentSearchParams.value.endTime,
    page: page.value,
    pageSize: pageSize.value,
  })
}

// ==================== 事件处理 ====================

/**
 * 更新状态栏
 */
const updateStatusBar = () => {
  setContent([
    h(Shield, { size: 14, style: { marginRight: '6px' } }),
    ` 共 ${total.value} 条消息`,
  ])
}

/**
 * 处理搜索
 */
const handleSearch = async (params: MessageSearchParams) => {
  searchParams.value = params
  currentSearchParams.value = { ...params }
  selectedKeys.value = []

  const success = await fetchMessageList({
    keyword: params.keyword,
    roomId: params.roomId,
    messageType: params.messageType,
    startTime: params.startTime,
    endTime: params.endTime,
    page: 1,
    pageSize: pageSize.value,
  })

  if (success) updateStatusBar()
}

/**
 * 重置搜索
 */
const handleReset = async () => {
  searchParams.value = {
    keyword: '',
    roomId: null,
    messageType: null,
    startTime: null,
    endTime: null,
  }
  currentSearchParams.value = { ...searchParams.value }
  selectedKeys.value = []
  page.value = 1

  const success = await fetchMessageList({ page: 1, pageSize: pageSize.value })

  if (success) {
    updateStatusBar()
    message.success('已重置筛选条件')
  }
}

/**
 * 刷新
 */
const handleRefresh = async () => {
  const success = await refresh()

  if (success) {
    updateStatusBar()
    message.success('刷新成功')
  }
}

/**
 * 分页变化
 */
const handlePageChange = async (newPage: number, newPageSize: number) => {
  selectedKeys.value = []

  const success = await fetchMessageList({
    keyword: currentSearchParams.value.keyword,
    roomId: currentSearchParams.value.roomId,
    messageType: currentSearchParams.value.messageType,
    startTime: currentSearchParams.value.startTime,
    endTime: currentSearchParams.value.endTime,
    page: newPage,
    pageSize: newPageSize,
  })

  if (success) updateStatusBar()
}

/**
 * 查看消息
 */
const handleView = (msg: AdminMessageInfo) => {
  dialog.info({
    title: '消息详情',
    content: () => h(MessageDetailModal, { message: msg }),
    showIcon: false,
    closable: true,
    maskClosable: true,
    positiveText: '',
    style: {
      width: 'auto',
      maxWidth: 'calc(100vw - 32px)',
    },
  })
}

/**
 * 删除消息
 */
const handleDelete = async (msg: AdminMessageInfo) => {
  try {
    const response = await adminApi.deleteMessage(msg.id)
    if (response.success) {
      message.success('消息已删除')
      await refresh()
      updateStatusBar()
    } else {
      message.error(response.message || '删除失败')
    }
  } catch {
    message.error('删除消息失败')
  }
}

/**
 * 批量删除
 */
const handleBatchDelete = async () => {
  if (selectedKeys.value.length === 0) {
    message.warning('请先选择要删除的消息')
    return
  }

  loading.value = true
  try {
    const results = await Promise.all(
      selectedKeys.value.map((id) => adminApi.deleteMessage(String(id)))
    )
    const failedCount = results.filter((r) => !r.success).length
    const successCount = selectedKeys.value.length - failedCount

    if (failedCount === 0) {
      message.success(`已成功删除 ${successCount} 条消息`)
    } else {
      message.warning(`删除完成：成功 ${successCount} 条，失败 ${failedCount} 条`)
    }

    selectedKeys.value = []
    await refresh()
    updateStatusBar()
  } catch {
    message.error('批量删除失败')
  } finally {
    loading.value = false
  }
}

/**
 * 处理移动端行点击
 */
const handleMobileRowClick = (row: unknown) => {
  handleView(row as AdminMessageInfo)
}

// ==================== 移动端配置 ====================

/** 移动端列配置 */
const mobileColumns = [
  { key: 'sender', title: '发送者' },
  { key: 'room_id', title: '房间ID' },
  { key: 'message_type', title: '类型' },
  { key: 'is_deleted', title: '状态' },
  { key: 'created_at', title: '发送时间' },
]

/** 移动端操作按钮配置 */
const mobileActions: MobileAction<AdminMessageInfo>[] = [
  {
    label: '查看',
    icon: Eye,
    type: 'default',
    onClick: (msg: AdminMessageInfo) => handleView(msg),
  },
  {
    label: '删除',
    icon: Trash2,
    type: 'error',
    onClick: (msg: AdminMessageInfo) => {
      dialog.warning({
        title: '确认删除',
        content: '确定要删除这条消息吗？',
        positiveText: '删除',
        negativeText: '取消',
        onPositiveClick: () => handleDelete(msg),
      })
    },
  },
]

// ==================== 生命周期 ====================

onMounted(async () => {
  const success = await fetchMessageList({ page: 1, pageSize: 10 })
  if (success) updateStatusBar()
})
</script>

<template>
  <div class="message-management-view">
    <div class="page-header">
      <h1 class="page-title">消息审核</h1>
      <p class="page-description">审核和管理系统消息，支持搜索、查看、删除违规消息</p>
    </div>

    <NCard class="search-card" :bordered="false">
      <MessageSearchForm
        v-bind="searchParams"
        :loading="loading"
        @search="handleSearch"
        @reset="handleReset"
        @refresh="handleRefresh"
      />
    </NCard>

    <NCard class="toolbar-card" :bordered="false">
      <div class="toolbar-content">
        <div class="toolbar-left">
          <span v-if="selectedKeys.length > 0" class="selected-info">
            已选择 {{ selectedKeys.length }} 条消息
          </span>
        </div>
        <div class="toolbar-right">
          <NButton
            v-if="selectedKeys.length > 0"
            type="error"
            size="small"
            :loading="loading"
            @click="handleBatchDelete"
          >
            <template #icon>
              <Trash2 :size="16" />
            </template>
            批量删除
          </NButton>
        </div>
      </div>
    </NCard>

    <NCard class="table-card" :bordered="false">
      <!-- 桌面端：MessageTable 组件 -->
      <div class="desktop-view">
        <MessageTable
          v-model:selected-keys="selectedKeys"
          :data="data"
          :loading="loading"
          @view="handleView"
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
            @update:page="handlePageChange($event, pageSize)"
            @update:page-size="handlePageChange(1, $event)"
          />
        </div>
      </div>

      <!-- 移动端：卡片视图 -->
      <div class="mobile-view">
        <MobileTableCard
          :data="data"
          :columns="mobileColumns"
          title-column="content"
          :actions="mobileActions as MobileAction[]"
          @row-click="handleMobileRowClick"
        />
        <div v-if="total > 0" class="mobile-pagination">
          <NPagination
            :page="page"
            :page-count="Math.ceil(total / pageSize)"
            :simple="true"
            @update:page="handlePageChange($event, pageSize)"
          />
        </div>
      </div>
    </NCard>
  </div>
</template>

<style scoped>
.message-management-view {
  padding: 24px;
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

.search-card {
  margin-bottom: 16px;
}

.toolbar-card {
  margin-bottom: 16px;
}

.toolbar-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.toolbar-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.selected-info {
  font-size: 14px;
  color: var(--text-secondary);
}

.table-card {
  margin-bottom: 24px;
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

/* 响应式布局 */
.desktop-view {
  display: block;
}

.mobile-view {
  display: none;
}

@media (max-width: 768px) {
  .message-management-view {
    padding: 16px;
  }

  .page-title {
    font-size: 24px;
  }

  .desktop-view {
    display: none;
  }

  .mobile-view {
    display: block;
  }

  .toolbar-content {
    flex-direction: column;
    gap: 12px;
    align-items: flex-start;
  }
}
</style>
