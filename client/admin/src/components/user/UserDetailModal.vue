<script setup lang="ts">
import { NAvatar, NTag, NSpace, NDescriptions, NDescriptionsItem, NDivider } from 'naive-ui'
import type { UserInfo, UserRole, UserStatus } from '@/types'

/**
 * 组件属性定义
 */
interface Props {
  /** 用户信息 */
  user: UserInfo | null
}

defineProps<Props>()

/**
 * 状态映射配置
 */
const statusConfig: Record<UserStatus, { text: string; type: 'success' | 'warning' | 'default' }> = {
  online: { text: '在线', type: 'success' },
  offline: { text: '离线', type: 'default' },
  away: { text: '离开', type: 'warning' },
}

/**
 * 角色映射配置
 */
const roleConfig: Record<UserRole, { text: string; type: 'error' | 'warning' | 'default' }> = {
  super_admin: { text: '超级管理员', type: 'error' },
  admin: { text: '管理员', type: 'warning' },
  user: { text: '普通用户', type: 'default' },
}

/**
 * 格式化日期时间
 * @param dateStr ISO 8601 格式日期字符串
 * @returns 格式化后的日期时间字符串
 */
const formatDateTime = (dateStr: string | undefined): string => {
  if (!dateStr) return '-'
  try {
    return new Date(dateStr).toLocaleString('zh-CN')
  } catch {
    return dateStr
  }
}
</script>

<template>
  <div v-if="user" class="user-detail-modal">
    <!-- 用户基本信息头部 -->
    <div class="user-header">
      <NAvatar
        :src="user.avatar_url || undefined"
        :fallback-src="`https://api.dicebear.com/7.x/avataaars/svg?seed=${user.username}`"
        :size="80"
        round
        class="user-avatar"
      />
      <div class="user-basic-info">
        <h3 class="user-name">{{ user.nickname || user.username }}</h3>
        <p class="user-username">@{{ user.username }}</p>
        <NSpace size="small">
          <NTag :type="roleConfig[(user.role || 'user') as UserRole].type" size="small">
            {{ roleConfig[(user.role || 'user') as UserRole].text }}
          </NTag>
          <NTag :type="user.is_active !== false ? 'success' : 'error'" size="small">
            {{ user.is_active !== false ? '正常' : '已禁用' }}
          </NTag>
          <NTag :type="statusConfig[(user.status || 'offline') as UserStatus].type" size="small">
            {{ statusConfig[(user.status || 'offline') as UserStatus].text }}
          </NTag>
        </NSpace>
      </div>
    </div>

    <NDivider />

    <!-- 用户详细信息 -->
    <NDescriptions :columns="1" label-placement="left" label-align="right" label-style="width: 100px">
      <NDescriptionsItem label="用户ID">
        <span class="copyable-text">{{ user.id }}</span>
      </NDescriptionsItem>
      <NDescriptionsItem label="用户名">
        {{ user.username }}
      </NDescriptionsItem>
      <NDescriptionsItem label="昵称">
        {{ user.nickname || '-' }}
      </NDescriptionsItem>
      <NDescriptionsItem label="邮箱">
        {{ user.email || '-' }}
      </NDescriptionsItem>
      <NDescriptionsItem label="角色">
        {{ roleConfig[(user.role || 'user') as UserRole].text }}
      </NDescriptionsItem>
      <NDescriptionsItem label="账号状态">
        {{ user.is_active !== false ? '正常' : '已禁用' }}
      </NDescriptionsItem>
      <NDescriptionsItem label="在线状态">
        {{ statusConfig[(user.status || 'offline') as UserStatus].text }}
      </NDescriptionsItem>
      <NDescriptionsItem label="创建时间">
        {{ formatDateTime(user.created_at) }}
      </NDescriptionsItem>
    </NDescriptions>
  </div>

  <div v-else class="user-detail-empty">
    <p>未找到用户信息</p>
  </div>
</template>

<style scoped>
.user-detail-modal {
  padding: 8px 0;
}

.user-header {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 0 8px;
}

.user-avatar {
  flex-shrink: 0;
}

.user-basic-info {
  flex: 1;
}

.user-name {
  margin: 0 0 4px 0;
  font-size: 20px;
  font-weight: 600;
  color: var(--text-primary);
}

.user-username {
  margin: 0 0 8px 0;
  font-size: 14px;
  color: var(--text-secondary);
}

.copyable-text {
  font-family: monospace;
  font-size: 13px;
  color: var(--text-secondary);
  word-break: break-all;
}

.user-detail-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 200px;
  color: var(--text-secondary);
}
</style>
