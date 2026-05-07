<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { User, MessageCircle, UserPlus } from 'lucide-vue-next'
import GlobalModal from '@/components/common/GlobalModal.vue'
import { userApi } from '@/api/user'
import type { User as UserType } from '@/types/user'
import type { UserSearchItem } from '@/types/search'

/**
 * 组件属性定义
 */
interface Props {
  /** 是否显示 */
  visible: boolean
  /** 用户ID */
  userId: string
  /** 用户基本信息（可选，用于快速展示） */
  userInfo?: UserSearchItem
}

const props = defineProps<Props>()

/**
 * 组件事件定义
 */
interface Emits {
  /** 更新显示状态 */
  (e: 'update:visible', value: boolean): void
  /** 发送私信 */
  (e: 'sendMessage'): void
  /** 添加好友 */
  (e: 'addFriend', userId: string): void
}

const emit = defineEmits<Emits>()

/** 用户详细信息 */
const userDetail = ref<UserType | null>(null)
/** 加载状态 */
const loading = ref(false)
/** 错误信息 */
const error = ref<string | null>(null)

/**
 * 是否是自己
 */
const isSelf = computed(() => {
  // TODO: 与当前登录用户比较
  return false
})

/**
 * 显示的用户名
 */
const displayName = computed(() => {
  return userDetail.value?.username || props.userInfo?.username || '未知用户'
})

/**
 * 显示的头像
 */
const displayAvatar = computed(() => {
  return userDetail.value?.avatar_url || props.userInfo?.avatar_url
})

/**
 * 在线状态
 */
const onlineStatus = computed(() => {
  return userDetail.value?.status || props.userInfo?.status || 'offline'
})

/**
 * 获取用户详情
 */
async function fetchUserDetail() {
  if (!props.userId) return

  loading.value = true
  error.value = null

  try {
    const res = await userApi.getUser(props.userId)
    if (res.success && res.data) {
      userDetail.value = res.data
    } else {
      error.value = res.message || '获取用户信息失败'
    }
  } catch (err) {
    error.value = '获取用户信息出错'
    console.error('[UserProfileModal] fetchUserDetail error:', err)
  } finally {
    loading.value = false
  }
}

/**
 * 处理关闭
 */
function handleClose() {
  emit('update:visible', false)
}

/**
 * 处理发送私信
 */
function handleSendMessage() {
  emit('sendMessage')
  handleClose()
}

/**
 * 处理添加好友
 */
function handleAddFriend() {
  emit('addFriend', props.userId)
  handleClose()
}

/**
 * 监听显示状态变化
 */
watch(
  () => props.visible,
  (newVal) => {
    if (newVal && props.userId) {
      fetchUserDetail()
    }
  }
)
</script>

<template>
  <GlobalModal
    :visible="visible"
    title="用户资料"
    preset="card"
    @update:visible="$emit('update:visible', $event)"
  >
    <div class="user-profile-modal">
      <!-- 加载中 -->
      <div v-if="loading" class="profile-loading">
        <div class="skeleton skeleton-avatar" />
        <div class="skeleton skeleton-name" />
        <div class="skeleton skeleton-info" />
      </div>

      <!-- 错误状态 -->
      <div v-else-if="error" class="profile-error">
        <p>{{ error }}</p>
      </div>

      <!-- 用户资料内容 -->
      <template v-else>
        <!-- 头部信息 -->
        <div class="profile-header">
          <div class="profile-avatar">
            <img
              v-if="displayAvatar"
              :src="displayAvatar"
              :alt="displayName"
              class="avatar-img"
            />
            <div v-else class="avatar-placeholder">
              <User :size="48" />
            </div>
            <span
              class="status-indicator"
              :class="`status--${onlineStatus}`"
            />
          </div>

          <div class="profile-info">
            <h3 class="profile-name">{{ displayName }}</h3>
            <span class="profile-status" :class="`status--${onlineStatus}`">
              {{ onlineStatus === 'online' ? '在线' : onlineStatus === 'away' ? '离开' : '离线' }}
            </span>
          </div>
        </div>

        <!-- 详细信息 -->
        <div v-if="userDetail" class="profile-details">
          <div class="detail-item">
            <span class="detail-label">邮箱</span>
            <span class="detail-value">{{ userDetail.email }}</span>
          </div>
          <div class="detail-item">
            <span class="detail-label">角色</span>
            <span class="detail-value">{{ userDetail.role === 'user' ? '普通用户' : '管理员' }}</span>
          </div>
          <div class="detail-item">
            <span class="detail-label">注册时间</span>
            <span class="detail-value">{{ new Date(userDetail.created_at).toLocaleDateString() }}</span>
          </div>
        </div>

        <!-- 操作按钮 -->
        <div v-if="!isSelf" class="profile-actions">
          <button class="action-btn action-btn--primary" @click="handleSendMessage">
            <MessageCircle :size="18" />
            <span>发送私信</span>
          </button>
          <button class="action-btn" @click="handleAddFriend">
            <UserPlus :size="18" />
            <span>添加好友</span>
          </button>
        </div>
      </template>
    </div>
  </GlobalModal>
</template>

<style scoped>
.user-profile-modal {
  padding: 8px;
}

.profile-loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  padding: 24px;
}

.skeleton {
  background: linear-gradient(90deg, #f0f0f0 25%, #e0e0e0 50%, #f0f0f0 75%);
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
  border-radius: 8px;
}

@keyframes shimmer {
  0% {
    background-position: 200% 0;
  }
  100% {
    background-position: -200% 0;
  }
}

.skeleton-avatar {
  width: 80px;
  height: 80px;
  border-radius: 50%;
}

.skeleton-name {
  width: 120px;
  height: 20px;
}

.skeleton-info {
  width: 200px;
  height: 16px;
}

.profile-error {
  text-align: center;
  padding: 24px;
  color: var(--color-error, #f5222d);
}

.profile-header {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  padding: 16px 0;
}

.profile-avatar {
  position: relative;
}

.avatar-img {
  width: 80px;
  height: 80px;
  border-radius: 50%;
  object-fit: cover;
}

.avatar-placeholder {
  width: 80px;
  height: 80px;
  border-radius: 50%;
  background: var(--color-background-soft, #f0f0f0);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-secondary, #666);
}

.status-indicator {
  position: absolute;
  bottom: 4px;
  right: 4px;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  border: 3px solid var(--color-white, #fff);
  background: var(--color-text-tertiary, #999);
}

.status-indicator.status--online {
  background: var(--color-success, #52c41a);
}

.status-indicator.status--away {
  background: var(--color-warning, #faad14);
}

.profile-info {
  text-align: center;
}

.profile-name {
  font-size: 20px;
  font-weight: 600;
  color: var(--color-text, #333);
  margin: 0 0 8px;
}

.profile-status {
  font-size: 13px;
  padding: 4px 12px;
  border-radius: 12px;
  background: var(--color-background-soft, #f0f0f0);
}

.profile-status.status--online {
  background: rgba(82, 196, 26, 0.1);
  color: var(--color-success, #52c41a);
}

.profile-status.status--away {
  background: rgba(250, 173, 20, 0.1);
  color: var(--color-warning, #faad14);
}

.profile-status.status--offline {
  color: var(--color-text-tertiary, #999);
}

.profile-details {
  padding: 16px 0;
  border-top: 1px solid var(--color-border, #eee);
  border-bottom: 1px solid var(--color-border, #eee);
}

.detail-item {
  display: flex;
  justify-content: space-between;
  padding: 8px 0;
}

.detail-label {
  font-size: 14px;
  color: var(--color-text-secondary, #666);
}

.detail-value {
  font-size: 14px;
  color: var(--color-text, #333);
}

.profile-actions {
  display: flex;
  gap: 12px;
  padding-top: 16px;
}

.action-btn {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 10px 16px;
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 8px;
  background: var(--color-white, #fff);
  color: var(--color-text, #333);
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.action-btn:hover {
  border-color: var(--color-primary, #2080f0);
  color: var(--color-primary, #2080f0);
}

.action-btn--primary {
  background: var(--color-primary, #2080f0);
  border-color: var(--color-primary, #2080f0);
  color: var(--color-white, #fff);
}

.action-btn--primary:hover {
  background: var(--color-primary-hover, #4096ff);
  border-color: var(--color-primary-hover, #4096ff);
}
</style>
