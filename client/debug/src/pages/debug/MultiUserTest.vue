<template>
  <div class="multi-user-test">
    <!-- 页面标题 -->
    <div class="page-header">
      <h1 class="page-title">多用户测试</h1>
      <span class="page-subtitle">{{ stats.total }} 个用户</span>
    </div>

    <!-- 连接统计卡片 -->
    <ConnectionStatsCard :stats="stats" />

    <!-- 批量操作工具栏 -->
    <BatchOperationToolbar
      :loading="loading"
      :current-operation="currentOperation"
      :operation-progress="operationProgress"
      :user-stats="stats"
      @create-users="handleCreateUsers"
      @login-users="handleLoginUsers"
      @connect-web-socket="handleConnectWebSocket"
      @disconnect-web-socket="handleDisconnectWebSocket"
      @refresh-tokens="handleRefreshTokens"
      @logout-users="handleLogoutUsers"
      @clear-all="handleClearAll"
      @export-credentials="handleExportCredentials"
      @import-credentials="handleImportCredentials"
    />

    <!-- 用户卡片网格 -->
    <div v-if="!isEmpty" class="user-grid">
      <UserCard
        v-for="user in users"
        :key="user.id"
        :user="user"
        @delete="handleDeleteUser"
      />
    </div>

    <!-- 空状态 -->
    <div v-else class="empty-state">
      <n-empty description="暂无测试用户">
        <template #icon>
          <n-icon :component="Users" size="48" />
        </template>
        <template #extra>
          <n-text depth="3">
            点击上方"创建"按钮添加测试用户
          </n-text>
        </template>
      </n-empty>
    </div>

    <!-- 操作结果提示 -->
    <n-modal
      v-model:show="showResultModal"
      preset="dialog"
      :title="resultTitle"
      :content="resultContent"
      positive-text="确定"
      @positive-click="showResultModal = false"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import {
  NEmpty,
  NIcon,
  NText,
  NModal,
  useMessage,
} from 'naive-ui'
import { Users } from 'lucide-vue-next'
import {
  BatchOperationToolbar,
  ConnectionStatsCard,
  UserCard,
} from '@/components/test'
import { useMultiUser, type BatchOperationResult } from '@/composables/test/useMultiUser'
import { useTestUsersStore } from '@/store/testUsers'

const message = useMessage()
const store = useTestUsersStore()

// 使用多用户组合式函数
const {
  users,
  stats,
  loading,
  currentOperation,
  operationProgress,
  batchCreateUsers,
  batchLoginUsers,
  batchRefreshTokens,
  batchLogoutUsers,
  batchConnectWebSocket,
  batchDisconnectWebSocket,
  deleteUser,
  clearAllUsers,
} = useMultiUser()

// 空状态
const isEmpty = computed(() => store.isEmpty)

// 结果弹窗
const showResultModal = ref(false)
const resultTitle = ref('')
const resultContent = ref('')

// 显示操作结果
function showResult(result: BatchOperationResult, operation: string) {
  resultTitle.value = operation
  if (result.failed === 0) {
    resultContent.value = `${operation}完成！成功: ${result.success} 个`
    message.success(`${operation}成功`)
  } else {
    resultContent.value = `${operation}完成！\n成功: ${result.success} 个\n失败: ${result.failed} 个`
    if (result.errors.length > 0) {
      resultContent.value += `\n\n错误信息:\n${result.errors.slice(0, 5).join('\n')}`
      if (result.errors.length > 5) {
        resultContent.value += `\n...还有 ${result.errors.length - 5} 个错误`
      }
    }
    message.warning(`${operation}部分失败`)
  }
  showResultModal.value = true
}

// 事件处理
async function handleCreateUsers(count: number) {
  const result = await batchCreateUsers(count)
  showResult(result, '批量创建用户')
}

async function handleLoginUsers() {
  const result = await batchLoginUsers()
  showResult(result, '批量登录')
}

async function handleConnectWebSocket() {
  const result = await batchConnectWebSocket()
  showResult(result, '批量连接 WebSocket')
}

function handleDisconnectWebSocket() {
  const result = batchDisconnectWebSocket()
  showResult(result, '批量断开 WebSocket')
}

async function handleRefreshTokens() {
  const result = await batchRefreshTokens()
  showResult(result, '批量刷新 Token')
}

async function handleLogoutUsers() {
  const result = await batchLogoutUsers()
  showResult(result, '批量登出')
}

async function handleClearAll() {
  await clearAllUsers()
  message.success('已清空所有测试用户')
}

async function handleDeleteUser(userId: string) {
  await deleteUser(userId)
  message.success('用户已删除')
}

/**
 * 导出用户凭据
 */
function handleExportCredentials() {
  const credentials = store.exportUserCredentials()
  if (credentials.length === 0) {
    message.warning('没有可导出的用户')
    return
  }

  const dataStr = JSON.stringify(credentials, null, 2)
  const blob = new Blob([dataStr], { type: 'application/json' })
  const url = URL.createObjectURL(blob)

  const link = document.createElement('a')
  link.href = url
  link.download = `test-users-${new Date().toISOString().slice(0, 10)}.json`
  document.body.appendChild(link)
  link.click()
  document.body.removeChild(link)

  URL.revokeObjectURL(url)
  message.success(`已导出 ${credentials.length} 个用户凭据`)
}

/**
 * 导入用户凭据
 */
function handleImportCredentials(credentials: { username: string; password: string; createdAt: number }[]) {
  const importedCount = store.importUserCredentials(credentials)
  if (importedCount > 0) {
    message.success(`成功导入 ${importedCount} 个用户凭据`)
  } else {
    message.info('没有新用户被导入（可能已存在）')
  }
}
</script>

<style scoped>
.multi-user-test {
  padding: 12px;
  max-width: 1200px;
  margin: 0 auto;
  /* 设置高度限制和滚动 */
  height: 100%;
  overflow-y: auto;
}

/* 页面标题 */
.page-header {
  display: flex;
  align-items: baseline;
  gap: 12px;
  margin-bottom: 16px;
}

.page-title {
  font-size: 20px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
}

.page-subtitle {
  font-size: 14px;
  color: var(--text-secondary);
}

/* 统计卡片和操作栏间距 */
.multi-user-test > * + * {
  margin-top: 12px;
}

/* 用户网格 */
.user-grid {
  display: grid;
  grid-template-columns: 1fr;
  gap: 12px;
  margin-top: 16px;
}

/* 空状态 */
.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 200px;
  margin-top: 24px;
  background: var(--bg-container);
  border-radius: 12px;
}

/* 平板端 */
@media (min-width: 640px) {
  .user-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}

/* 桌面端 */
@media (min-width: 768px) {
  .multi-user-test {
    padding: 20px;
  }

  .page-title {
    font-size: 24px;
  }

  .multi-user-test > * + * {
    margin-top: 16px;
  }

  .user-grid {
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: 16px;
  }

  .empty-state {
    min-height: 300px;
  }
}
</style>
