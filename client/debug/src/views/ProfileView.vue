<template>
  <div class="profile-view">
    <n-card title="个人中心">
      <n-descriptions bordered>
        <n-descriptions-item label="用户名">
          {{ authStore.user?.username }}
        </n-descriptions-item>
        <n-descriptions-item label="邮箱">
          {{ authStore.user?.email }}
        </n-descriptions-item>
        <n-descriptions-item label="角色">
          {{ roleText }}
        </n-descriptions-item>
        <n-descriptions-item label="状态">
          <n-tag :type="statusType">
            {{ statusText }}
          </n-tag>
        </n-descriptions-item>
        <n-descriptions-item label="注册时间">
          {{ formatDate(authStore.user?.created_at) }}
        </n-descriptions-item>
        <n-descriptions-item label="更新时间">
          {{ formatDate(authStore.user?.updated_at) }}
        </n-descriptions-item>
      </n-descriptions>

      <n-divider />

      <n-space>
        <n-button type="error" @click="handleLogout">
          退出登录
        </n-button>
      </n-space>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { NCard, NDescriptions, NDescriptionsItem, NTag, NDivider, NSpace, NButton } from 'naive-ui'
import { useAuthStore, useWebSocketStore } from '@/store'
import type { UserOnlineStatus, UserRole } from '@/types/api'

const router = useRouter()
const authStore = useAuthStore()
const wsStore = useWebSocketStore()

const roleText = computed(() => {
  const roleMap: Record<UserRole, string> = {
    user: '普通用户',
    admin: '管理员',
    super_admin: '超级管理员'
  }
  return roleMap[authStore.user?.role || 'user']
})

const statusType = computed(() => {
  const status = authStore.user?.status
  if (status === 'disabled') return 'error'
  if (status === 'online') return 'success'
  if (status === 'away') return 'warning'
  return 'default'
})

const statusText = computed(() => {
  const status = authStore.user?.status
  const statusMap: Record<UserOnlineStatus, string> = {
    online: '在线',
    offline: '离线',
    away: '离开',
    disabled: '禁用'
  }
  return statusMap[status || 'offline']
})

function formatDate(date: string | undefined) {
  if (!date) return '-'
  return new Date(date).toLocaleString()
}

async function handleLogout() {
  // 断开 WebSocket
  wsStore.disconnect()
  // 执行登出
  await authStore.logout()
  // 跳转到登录页
  router.push('/login')
}
</script>

<style scoped>
.profile-view {
  padding: 24px;
  max-width: 800px;
  margin: 0 auto;
}
</style>
