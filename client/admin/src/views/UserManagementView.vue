<script setup lang="ts">
/**
 * UserManagementView - 用户管理页面
 *
 * 用户管理主视图，负责数据流管理和组件协调
 *
 */
import { ref, computed, h, onMounted } from 'vue'
import { NCard, useMessage } from 'naive-ui'
import { Users } from 'lucide-vue-next'
import { UserSearchForm, UserTableToolbar, UserTable } from '@/components/user'
import { useStatusBar } from '@/composables'
import { adminApi } from '@/api/admin'
import type { UserInfo, UserRole, UserStatus } from '@/types'

/**
 * 消息提示
 */
const message = useMessage()

/**
 * 状态栏
 */
const { setContent } = useStatusBar()

/**
 * 加载状态
 */
const loading = ref(false)

/**
 * 搜索关键词
 */
const searchKeyword = ref('')

/**
 * 状态筛选
 */
const statusFilter = ref('')

/**
 * 角色筛选
 */
const roleFilter = ref('')

/**
 * 选中的用户 keys
 */
const selectedKeys = ref<(string | number)[]>([])

/**
 * 用户数据
 */
const userData = ref<UserInfo[]>([])

/**
 * 总用户数（用于分页）
 */
const totalUsers = ref(0)

/**
 * 当前页码
 */
const currentPage = ref(1)

/**
 * 每页数量
 */
const pageSize = ref(10)

/**
 * 筛选后的用户数据（前端筛选）
 */
const filteredData = computed(() => {
  let result = userData.value

  // 状态筛选（前端筛选）
  if (statusFilter.value) {
    result = result.filter((user) => user.status === (statusFilter.value as UserStatus))
  }

  // 角色筛选（前端筛选）
  if (roleFilter.value) {
    result = result.filter((user) => user.role === (roleFilter.value as UserRole))
  }

  return result
})

/**
 * 获取用户列表
 */
const fetchUserList = async () => {
  loading.value = true
  try {
    const response = await adminApi.getUserList({
      page: currentPage.value,
      page_size: pageSize.value,
      search: searchKeyword.value || undefined,
    })

    if (response.success && response.data) {
      userData.value = response.data.users
      totalUsers.value = response.data.total
      // 更新状态栏
      setContent([
        h(Users, { size: 14, style: { marginRight: '6px' } }),
        ` 共 ${totalUsers.value} 位用户`,
      ])
    } else {
      message.error(response.message || '获取用户列表失败')
    }
  } catch (error) {
    message.error('获取用户列表失败，请检查网络连接')
    console.error('获取用户列表失败:', error)
  } finally {
    loading.value = false
  }
}

/**
 * 处理搜索
 */
const handleSearch = () => {
  currentPage.value = 1
  fetchUserList()
}

/**
 * 重置筛选
 */
const handleReset = () => {
  searchKeyword.value = ''
  statusFilter.value = ''
  roleFilter.value = ''
  currentPage.value = 1
  fetchUserList()
  message.success('已重置筛选条件')
}

/**
 * 刷新数据
 */
const handleRefresh = () => {
  fetchUserList()
}

/**
 * 新增用户
 */
const handleAdd = () => {
  message.info('打开新增用户弹窗')
  // TODO: 打开新增用户弹窗
}

/**
 * 查看用户详情
 * @param user - 用户数据
 */
const handleView = (user: UserInfo) => {
  message.info(`查看用户: ${user.nickname || user.username}`)
  // TODO: 打开用户详情弹窗
}

/**
 * 编辑用户
 * @param user - 用户数据
 */
const handleEdit = (user: UserInfo) => {
  message.info(`编辑用户: ${user.nickname || user.username}`)
  // TODO: 打开编辑用户弹窗
}

/**
 * 删除用户
 * @param user - 用户数据
 */
const handleDelete = async (user: UserInfo) => {
  try {
    const response = await adminApi.deleteUser(user.id)
    if (response.success) {
      message.success(`用户 "${user.nickname || user.username}" 已删除`)
      // 刷新列表
      await fetchUserList()
    } else {
      message.error(response.message || '删除失败')
    }
  } catch (error) {
    message.error('删除用户失败，请检查网络连接')
    console.error('删除用户失败:', error)
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
    // 逐个删除选中的用户
    const deletePromises = selectedKeys.value.map((userId) =>
      adminApi.deleteUser(String(userId))
    )
    const results = await Promise.all(deletePromises)

    // 检查是否有失败的删除
    const failedCount = results.filter((r) => !r.success).length
    const successCount = selectedKeys.value.length - failedCount

    if (failedCount === 0) {
      message.success(`已成功删除 ${successCount} 个用户`)
    } else {
      message.warning(`删除完成：成功 ${successCount} 个，失败 ${failedCount} 个`)
    }

    // 清空选中并刷新列表
    selectedKeys.value = []
    await fetchUserList()
  } catch (error) {
    message.error('批量删除失败，请检查网络连接')
    console.error('批量删除失败:', error)
  } finally {
    loading.value = false
  }
}

/**
 * 处理选择变化
 * @param keys - 选中的 keys
 */
const handleSelectionChange = (keys: (string | number)[]) => {
  selectedKeys.value = keys
}

/**
 * 处理分页变化
 * @param page - 页码
 * @param size - 每页数量
 */
const handlePageChange = (page: number, size: number) => {
  currentPage.value = page
  pageSize.value = size
  fetchUserList()
}

/**
 * 页面挂载时获取数据
 */
onMounted(() => {
  fetchUserList()
})
</script>

<template>
  <div class="user-management-view">
    <!-- 页面标题 -->
    <div class="page-header">
      <h1 class="page-title">用户管理</h1>
      <p class="page-description">管理系统用户，包括查看、编辑、删除用户等操作</p>
    </div>

    <!-- 搜索筛选区域 -->
    <NCard class="search-card" :bordered="false">
      <UserSearchForm
        v-model:keyword="searchKeyword"
        v-model:status="statusFilter"
        v-model:role="roleFilter"
        :loading="loading"
        @search="handleSearch"
        @reset="handleReset"
        @refresh="handleRefresh"
      />
    </NCard>

    <!-- 操作栏 -->
    <NCard class="toolbar-card" :bordered="false">
      <UserTableToolbar
        :selected-count="selectedKeys.length"
        :total="filteredData.length"
        :loading="loading"
        @add="handleAdd"
        @batch-delete="handleBatchDelete"
      />
    </NCard>

    <!-- 数据表格 -->
    <NCard class="table-card" :bordered="false">
      <UserTable
        :data="filteredData"
        :loading="loading"
        :selected-keys="selectedKeys"
        :total="totalUsers"
        :current-page="currentPage"
        :page-size="pageSize"
        @selection-change="handleSelectionChange"
        @view="handleView"
        @edit="handleEdit"
        @delete="handleDelete"
        @page-change="handlePageChange"
      />
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

.search-card {
  margin-bottom: 16px;
}

.toolbar-card {
  margin-bottom: 16px;
}

.table-card {
  min-height: 400px;
}

/* 响应式调整 */
@media (max-width: 768px) {
  .user-management-view {
    padding: 16px;
  }

  .page-title {
    font-size: 24px;
  }
}
</style>
