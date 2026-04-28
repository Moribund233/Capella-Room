<script setup lang="ts">
import { ref, h, onMounted } from 'vue'
import { NCard, NPagination, useMessage, useDialog } from 'naive-ui'
import { Users, Eye, Edit, Trash2 } from 'lucide-vue-next'
import type { MobileAction } from '@/components/common/MobileTableCard.vue'
import { UserSearchForm, UserTableToolbar, UserTable, UserDetailModal } from '@/components/user'
import { MobileTableCard } from '@/components/common'
import { useStatusBar } from '@/composables'
import { adminApi } from '@/api/admin'
import type { UserInfo } from '@/types'

const message = useMessage()
const dialog = useDialog()
const { setContent } = useStatusBar()

// ==================== 数据状态 ====================

/** 用户列表数据 */
const data = ref<UserInfo[]>([])
/** 加载状态 */
const loading = ref(false)
/** 总用户数 */
const total = ref(0)
/** 当前页码 */
const page = ref(1)
/** 每页数量 */
const pageSize = ref(10)
/** 选中的用户 keys */
const selectedKeys = ref<(string | number)[]>([])

/** 搜索参数 */
const searchParams = ref({
  keyword: '',
  status: '',
  role: '',
})

/** 当前搜索参数缓存（用于刷新） */
const currentSearchParams = ref({
  keyword: '',
  status: '',
  role: '',
})

// ==================== 数据获取 ====================

/**
 * 获取用户列表
 * @param params 搜索参数
 */
const fetchUserList = async (params: {
  keyword?: string
  status?: string
  role?: string
  page?: number
  pageSize?: number
} = {}) => {
  loading.value = true

  try {
    const response = await adminApi.getUserList({
      page: params.page ?? page.value,
      page_size: params.pageSize ?? pageSize.value,
      search: params.keyword || undefined,
    })

    if (response.success && response.data) {
      data.value = response.data.users
      total.value = response.data.total
      page.value = params.page ?? page.value
      pageSize.value = params.pageSize ?? pageSize.value
      return true
    }
    return false
  } catch (error) {
    console.error('获取用户列表失败:', error)
    message.error('获取用户列表失败')
    return false
  } finally {
    loading.value = false
  }
}

/**
 * 刷新当前列表
 */
const refresh = async () => {
  return fetchUserList({
    keyword: currentSearchParams.value.keyword,
    status: currentSearchParams.value.status,
    role: currentSearchParams.value.role,
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
    h(Users, { size: 14, style: { marginRight: '6px' } }),
    ` 共 ${total.value} 位用户`,
  ])
}

/**
 * 处理搜索
 */
const handleSearch = async (params: { keyword: string; status: string; role: string }) => {
  searchParams.value = params
  currentSearchParams.value = { ...params }
  selectedKeys.value = []

  const success = await fetchUserList({
    keyword: params.keyword,
    status: params.status,
    role: params.role,
    page: 1,
    pageSize: pageSize.value,
  })

  if (success) updateStatusBar()
}

/**
 * 重置搜索
 */
const handleReset = async () => {
  searchParams.value = { keyword: '', status: '', role: '' }
  currentSearchParams.value = { keyword: '', status: '', role: '' }
  selectedKeys.value = []
  page.value = 1

  const success = await fetchUserList({ page: 1, pageSize: pageSize.value })

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

  const success = await fetchUserList({
    keyword: currentSearchParams.value.keyword,
    status: currentSearchParams.value.status,
    role: currentSearchParams.value.role,
    page: newPage,
    pageSize: newPageSize,
  })

  if (success) updateStatusBar()
}

/**
 * 新增用户
 */
const handleAdd = () => {
  message.info('打开新增用户弹窗')
}

/**
 * 查看用户
 */
const handleView = (user: UserInfo) => {
  dialog.info({
    title: '用户详情',
    content: () => h(UserDetailModal, { user }),
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
 * 编辑用户
 */
const handleEdit = (user: UserInfo) => {
  message.info(`编辑用户: ${user.nickname || user.username}`)
}

/**
 * 删除用户
 */
const handleDelete = async (user: UserInfo) => {
  try {
    const response = await adminApi.deleteUser(user.id)
    if (response.success) {
      message.success(`用户 "${user.nickname || user.username}" 已删除`)
      await refresh()
      updateStatusBar()
    } else {
      message.error(response.message || '删除失败')
    }
  } catch {
    message.error('删除用户失败')
  }
}

/**
 * 批量删除
 */
const handleBatchDelete = async () => {
  if (selectedKeys.value.length === 0) {
    message.warning('请先选择要删除的用户')
    return
  }

  loading.value = true
  try {
    const results = await Promise.all(
      selectedKeys.value.map((id) => adminApi.deleteUser(String(id)))
    )
    const failedCount = results.filter((r) => !r.success).length
    const successCount = selectedKeys.value.length - failedCount

    if (failedCount === 0) {
      message.success(`已成功删除 ${successCount} 个用户`)
    } else {
      message.warning(`删除完成：成功 ${successCount} 个，失败 ${failedCount} 个`)
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
  handleView(row as UserInfo)
}

// ==================== 移动端配置 ====================

/** 移动端列配置 */
const mobileColumns = [
  { key: 'role', title: '角色' },
  { key: 'is_active', title: '账号状态' },
  { key: 'status', title: '在线状态' },
  { key: 'created_at', title: '创建时间' },
]

/** 移动端操作按钮配置 */
const mobileActions: MobileAction<UserInfo>[] = [
  {
    label: '查看',
    icon: Eye,
    type: 'default',
    onClick: (user: UserInfo) => handleView(user),
  },
  {
    label: '编辑',
    icon: Edit,
    type: 'primary',
    onClick: (user: UserInfo) => handleEdit(user),
  },
  {
    label: '删除',
    icon: Trash2,
    type: 'error',
    onClick: (user: UserInfo) => {
      dialog.warning({
        title: '确认删除',
        content: `确定要删除用户 "${user.nickname || user.username}" 吗？`,
        positiveText: '删除',
        negativeText: '取消',
        onPositiveClick: () => handleDelete(user),
      })
    },
  },
]

// ==================== 生命周期 ====================

onMounted(async () => {
  const success = await fetchUserList({ page: 1, pageSize: 10 })
  if (success) updateStatusBar()
})
</script>

<template>
  <div class="user-management-view">
    <div class="page-header">
      <h1 class="page-title">用户管理</h1>
      <p class="page-description">管理系统用户，包括查看、编辑、删除用户等操作</p>
    </div>

    <NCard class="search-card" :bordered="false">
      <UserSearchForm
        v-bind="searchParams"
        :loading="loading"
        @search="handleSearch"
        @reset="handleReset"
        @refresh="handleRefresh"
      />
    </NCard>

    <NCard class="toolbar-card" :bordered="false">
      <UserTableToolbar
        :selected-count="selectedKeys.length"
        :total="total"
        :loading="loading"
        @add="handleAdd"
        @batch-delete="handleBatchDelete"
      />
    </NCard>

    <NCard class="table-card" :bordered="false">
      <!-- 桌面端：UserTable 组件 -->
      <div class="desktop-view">
        <UserTable
          v-model:selected-keys="selectedKeys"
          :data="data"
          :loading="loading"
          @view="handleView"
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
          title-column="username"
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
.user-management-view {
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
  .user-management-view {
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
}
</style>
