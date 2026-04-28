<template>
  <div class="user-profile-modal">
    <!-- 顶部：头像和基本信息 -->
    <div class="profile-header">
      <n-avatar :size="64" :src="userInfo?.avatar_url || undefined" :fallback-src="defaultAvatar" class="profile-avatar" />
      <div class="profile-info">
        <h3 class="profile-nickname">{{ displayName }}</h3>
        <p class="profile-username">@{{ userInfo?.username }}</p>
        <n-space v-if="userInfo?.role" class="profile-roles" :size="4">
          <n-tag size="tiny" type="primary" round>
            {{ userInfo.role }}
          </n-tag>
        </n-space>
      </div>
    </div>

    <!-- 底部：详细信息网格 -->
    <div class="profile-details">
      <div class="info-grid">
        <div class="info-item">
          <span class="info-label">用户ID</span>
          <span class="info-value">{{ userInfo?.id || '-' }}</span>
        </div>
        <div class="info-item">
          <span class="info-label">用户名</span>
          <span class="info-value">{{ userInfo?.username || '-' }}</span>
        </div>
        <div class="info-item">
          <span class="info-label">昵称</span>
          <span class="info-value">{{ userInfo?.nickname || '-' }}</span>
        </div>
        <div class="info-item">
          <span class="info-label">邮箱</span>
          <span class="info-value email-value">
            <n-ellipsis style="max-width: 120px">{{ userInfo?.email || '-' }}</n-ellipsis>
            <n-tag v-if="userInfo?.email" size="tiny" type="success">已验证</n-tag>
          </span>
        </div>
        <div class="info-item">
          <span class="info-label">注册时间</span>
          <span class="info-value">{{ registerTime }}</span>
        </div>
        <div class="info-item">
          <span class="info-label">账号状态</span>
          <span class="info-value">
            <n-tag type="success" size="tiny">正常</n-tag>
          </span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import {
  NAvatar,
  NTag,
  NSpace,
  NEllipsis,
} from 'naive-ui'
import type { UserInfo } from '@/types'

/**
 * 组件属性定义
 */
interface Props {
  /** 用户信息 */
  userInfo?: UserInfo | null
}

const props = withDefaults(defineProps<Props>(), {
  userInfo: null,
})

/**
 * 默认头像
 */
const defaultAvatar = 'https://api.dicebear.com/7.x/avataaars/svg?seed=Felix'

/**
 * 显示名称（优先显示昵称，否则显示用户名）
 */
const displayName = computed(() => {
  return props.userInfo?.nickname || props.userInfo?.username || '未知用户'
})

/**
 * 注册时间（模拟数据，实际应从用户信息中获取）
 */
const registerTime = computed(() => {
  return '2025-01-15'
})
</script>

<style scoped>
.user-profile-modal {
  width: 400px;
  height: 300px;
  padding: 20px;
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  gap: 16px;
  overflow: hidden;
}

/* 顶部区域 - 桌面端左右布局 */
.profile-header {
  display: flex;
  align-items: center;
  gap: 16px;
  flex-shrink: 0;
}

.profile-avatar {
  flex-shrink: 0;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
}

.profile-info {
  flex: 1;
  min-width: 0;
}

.profile-nickname {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-color);
  margin: 0 0 4px;
  line-height: 1.3;
}

.profile-username {
  font-size: 12px;
  color: var(--text-color-secondary);
  margin: 0 0 6px;
}

.profile-roles {
  flex-wrap: wrap;
}

/* 底部详情区域 */
.profile-details {
  flex: 1;
  overflow-y: auto;
}

.info-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
}

.info-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.info-label {
  font-size: 11px;
  color: var(--text-color-secondary);
}

.info-value {
  font-size: 13px;
  color: var(--text-color);
  display: flex;
  align-items: center;
  gap: 6px;
}

.email-value {
  flex-wrap: wrap;
}

/* 移动端适配 */
@media screen and (max-width: 480px) {
  .user-profile-modal {
    width: 100%;
    max-width: 320px;
    height: auto;
    max-height: 400px;
    padding: 16px;
    gap: 12px;
  }

  .profile-header {
    flex-direction: column;
    text-align: center;
    gap: 10px;
  }

  .profile-avatar {
    --n-size: 56px !important;
  }

  .profile-info {
    width: 100%;
  }

  .profile-nickname {
    font-size: 15px;
  }

  .profile-username {
    font-size: 11px;
  }

  .profile-roles {
    justify-content: center;
  }

  .info-grid {
    grid-template-columns: 1fr;
    gap: 8px;
  }

  .info-item {
    flex-direction: row;
    align-items: center;
    justify-content: space-between;
    padding: 4px 0;
    border-bottom: 1px solid var(--border-color-base);
  }

  .info-item:last-child {
    border-bottom: none;
  }

  .info-label {
    font-size: 12px;
  }

  .info-value {
    font-size: 12px;
    justify-content: flex-end;
  }
}
</style>
