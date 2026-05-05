<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { storeToRefs } from 'pinia'
import { useMessage } from 'naive-ui'
import { useAuthStore } from '@/stores/auth'
import { useResponsive } from '@/composables/useResponsive'
import { userApi, type UserStats } from '@/api/user'
import { uploadApi } from '@/api/upload'
import AvatarUpload from '@/components/profile/AvatarUpload.vue'
import ProfileEditModal from '@/components/profile/ProfileEditModal.vue'
import PasswordChangeModal from '@/components/profile/PasswordChangeModal.vue'
import StatCard from '@/components/profile/StatCard.vue'
import { formatDate } from '@/utils/date'
import {
  User,
  Mail,
  Calendar,
  Shield,
  Edit3,
  Key,
  LogOut,
  Award
} from 'lucide-vue-next'
import type { User as UserType } from '@/types/user'

const message = useMessage()
const authStore = useAuthStore()
const { user } = storeToRefs(authStore)
const { isMobile } = useResponsive()
const showEditModal = ref(false)
const showPasswordModal = ref(false)
const stats = ref<UserStats>({
  joined_rooms: 0,
  total_messages: 0,
  online_hours: 0
})

const userInfo = computed(() => {
  return user.value
})

const statusConfig = computed(() => {
  const configs = {
    online: { label: '在线', color: '#07c160', bgColor: '#e6f7ed' },
    away: { label: '离开', color: '#faad14', bgColor: '#fff3e0' },
    busy: { label: '忙碌', color: '#ff4d4f', bgColor: '#fff2f0' },
    offline: { label: '离线', color: '#8c8c8c', bgColor: '#f5f5f5' }
  }
  return configs[user.value?.status || 'offline']
})

const roleConfig = computed(() => {
  const configs = {
    user: { label: '普通用户', icon: User },
    admin: { label: '管理员', icon: Shield },
    super_admin: { label: '超级管理员', icon: Award }
  }
  return configs[user.value?.role || 'user']
})

async function fetchUserStats() {
  try {
    const res = await userApi.getStats()
    if (res.data) {
      stats.value = res.data
    }
  } catch {
    // 静默失败，使用默认值
  }
}

async function handleAvatarUpload(file: File) {
  try {
    const res = await uploadApi.uploadAvatar(file)
    if (res.data) {
      await userApi.updateProfile({ avatar_url: res.data.url })
      await authStore.fetchUser()
      message.success('头像更新成功')
    }
  } catch {
    message.error('头像上传失败')
  }
}

async function handleProfileUpdate(data: Partial<UserType>) {
  try {
    await userApi.updateProfile(data)
    await authStore.fetchUser()
    showEditModal.value = false
    message.success('资料更新成功')
  } catch {
    message.error('资料更新失败')
  }
}

async function handlePasswordChange(data: { oldPassword: string; newPassword: string }) {
  try {
    await userApi.changePassword(data)
    showPasswordModal.value = false
    message.success('密码修改成功')
  } catch {
    message.error('密码修改失败')
  }
}

async function handleLogout() {
  await authStore.logout()
  message.success('已退出登录')
}

onMounted(() => {
  fetchUserStats()
})
</script>

<template>
  <div class="profile-view">
    <!-- 页面标题 -->
    <div class="profile-header">
      <h1 class="profile-title">
        <User class="profile-title-icon" />
        个人中心
      </h1>
      <p class="profile-subtitle">管理您的个人信息和账号设置</p>
    </div>

    <!-- 主要内容区 -->
    <div class="profile-content" :class="{ 'profile-content--mobile': isMobile }">
      <!-- 左侧：用户信息卡片 -->
      <div class="profile-main">
        <div class="profile-card profile-card--user">
          <div class="profile-card__header">
            <div class="profile-avatar-section">
              <AvatarUpload
                :avatar-url="userInfo?.avatar_url"
                :username="userInfo?.username"
                @upload="handleAvatarUpload"
              />
              <div class="profile-status-badge" :style="{ backgroundColor: statusConfig.bgColor, color: statusConfig.color }">
                {{ statusConfig.label }}
              </div>
            </div>

            <div class="profile-info">
              <h2 class="profile-name">{{ userInfo?.username }}</h2>
              <p class="profile-email">
                <Mail class="profile-icon" />
                {{ userInfo?.email }}
              </p>
              <div class="profile-role" :class="`profile-role--${userInfo?.role}`">
                <component :is="roleConfig.icon" class="profile-icon" />
                {{ roleConfig.label }}
              </div>
            </div>
          </div>

          <div class="profile-card__divider" />

          <div class="profile-meta">
            <div class="profile-meta-item">
              <Calendar class="profile-meta-icon" />
              <span class="profile-meta-label">注册时间</span>
              <span class="profile-meta-value">{{ formatDate(userInfo?.created_at) }}</span>
            </div>
          </div>

          <div class="profile-actions">
            <button class="profile-btn profile-btn--primary" @click="showEditModal = true">
              <Edit3 class="profile-btn-icon" />
              编辑资料
            </button>
            <button class="profile-btn profile-btn--secondary" @click="showPasswordModal = true">
              <Key class="profile-btn-icon" />
              修改密码
            </button>
            <button class="profile-btn profile-btn--danger" @click="handleLogout">
              <LogOut class="profile-btn-icon" />
              退出登录
            </button>
          </div>
        </div>

        <!-- 统计卡片 -->
        <div class="profile-stats">
          <StatCard
            title="加入聊天室"
            :value="stats.joined_rooms"
            icon="MessageSquare"
            color="#07c160"
          />
          <StatCard
            title="发送消息"
            :value="stats.total_messages"
            icon="MessageSquare"
            color="#1890ff"
          />
          <StatCard
            title="在线时长"
            :value="`${stats.online_hours}h`"
            icon="Clock"
            color="#faad14"
          />
        </div>
      </div>

      <!-- 右侧：账号安全与设置 -->
      <div class="profile-sidebar">
        <div class="profile-card profile-card--security">
          <h3 class="profile-card__title">
            <Shield class="profile-card__title-icon" />
            账号安全
          </h3>
          <div class="security-list">
            <div class="security-item">
              <div class="security-item__info">
                <span class="security-item__label">登录密码</span>
                <span class="security-item__status security-item__status--safe">已设置</span>
              </div>
              <button class="security-item__action" @click="showPasswordModal = true">
                修改
              </button>
            </div>
            <div class="security-item">
              <div class="security-item__info">
                <span class="security-item__label">账号状态</span>
                <span class="security-item__status" :class="userInfo?.is_active ? 'security-item__status--safe' : 'security-item__status--danger'">
                  {{ userInfo?.is_active ? '正常' : '已禁用' }}
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 编辑资料弹窗 -->
    <ProfileEditModal
      v-model:show="showEditModal"
      :user="userInfo"
      @submit="handleProfileUpdate"
    />

    <!-- 修改密码弹窗 -->
    <PasswordChangeModal
      v-model:show="showPasswordModal"
      @submit="handlePasswordChange"
    />
  </div>
</template>

<style scoped>
.profile-view {
  padding: var(--space-2xl);
  max-width: 1200px;
  margin: 0 auto;
  animation: fade-in 0.3s ease-out;
  overflow-y: auto;
  height: 100%;
}

@keyframes fade-in {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* 页面标题 */
.profile-header {
  margin-bottom: var(--space-2xl);
}

.profile-title {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  font-size: var(--font-size-h1);
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0 0 var(--space-sm) 0;
}

.profile-title-icon {
  width: 28px;
  height: 28px;
  color: var(--color-primary);
}

.profile-subtitle {
  font-size: var(--font-size-body);
  color: var(--color-text-secondary);
  margin: 0;
}

/* 内容布局 */
.profile-content {
  display: grid;
  grid-template-columns: 1fr 320px;
  gap: var(--space-2xl);
}

.profile-content--mobile {
  grid-template-columns: 1fr;
}

/* 卡片基础样式 */
.profile-card {
  background: var(--color-white);
  border-radius: var(--radius-xl);
  box-shadow: var(--shadow-sm);
  border: 1px solid var(--color-border-light);
  overflow: hidden;
  transition: box-shadow 0.2s ease;
}

.profile-card:hover {
  box-shadow: var(--shadow-md);
}

.profile-card__header {
  padding: var(--space-2xl);
  display: flex;
  gap: var(--space-xl);
  align-items: flex-start;
}

.profile-card__divider {
  height: 1px;
  background: var(--color-border-light);
  margin: 0 var(--space-2xl);
}

.profile-card__title {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  font-size: var(--font-size-h3);
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0 0 var(--space-lg) 0;
}

.profile-card__title-icon {
  width: 20px;
  height: 20px;
  color: var(--color-primary);
}

/* 头像区域 */
.profile-avatar-section {
  position: relative;
  flex-shrink: 0;
}

.profile-status-badge {
  position: absolute;
  bottom: 4px;
  right: 4px;
  padding: 2px 8px;
  border-radius: var(--radius-full);
  font-size: var(--font-size-tiny);
  font-weight: 500;
  border: 2px solid var(--color-white);
}

/* 用户信息 */
.profile-info {
  flex: 1;
  min-width: 0;
}

.profile-name {
  font-size: var(--font-size-h2);
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0 0 var(--space-sm) 0;
}

.profile-email {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  font-size: var(--font-size-body);
  color: var(--color-text-secondary);
  margin: 0 0 var(--space-sm) 0;
}

.profile-icon {
  width: 16px;
  height: 16px;
}

.profile-role {
  display: inline-flex;
  align-items: center;
  gap: var(--space-xs);
  padding: 4px 12px;
  border-radius: var(--radius-full);
  font-size: var(--font-size-small);
  font-weight: 500;
}

.profile-role--user {
  background: var(--color-info-light);
  color: var(--color-info);
}

.profile-role--admin {
  background: var(--color-warning-light);
  color: var(--color-warning);
}

.profile-role--super_admin {
  background: var(--color-error-light);
  color: var(--color-error);
}

/* 元信息 */
.profile-meta {
  padding: var(--space-lg) var(--space-2xl);
}

.profile-meta-item {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  color: var(--color-text-secondary);
  font-size: var(--font-size-body);
}

.profile-meta-icon {
  width: 16px;
  height: 16px;
  color: var(--color-text-tertiary);
}

.profile-meta-label {
  color: var(--color-text-tertiary);
}

.profile-meta-value {
  color: var(--color-text-primary);
  font-weight: 500;
  margin-left: auto;
}

/* 操作按钮 */
.profile-actions {
  padding: var(--space-lg) var(--space-2xl) var(--space-2xl);
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-md);
}

.profile-btn {
  display: inline-flex;
  align-items: center;
  gap: var(--space-xs);
  padding: 10px 20px;
  border-radius: var(--radius-md);
  font-size: var(--font-size-body);
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  border: none;
  outline: none;
}

.profile-btn-icon {
  width: 16px;
  height: 16px;
}

.profile-btn--primary {
  background: var(--color-primary);
  color: var(--color-white);
}

.profile-btn--primary:hover {
  background: var(--color-primary-hover);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(7, 193, 96, 0.3);
}

.profile-btn--secondary {
  background: var(--color-background);
  color: var(--color-text-primary);
  border: 1px solid var(--color-border);
}

.profile-btn--secondary:hover {
  background: var(--color-background-light);
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.profile-btn--danger {
  background: transparent;
  color: var(--color-error);
  border: 1px solid var(--color-error);
}

.profile-btn--danger:hover {
  background: var(--color-error-light);
}

/* 统计卡片 */
.profile-stats {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: var(--space-lg);
  margin-top: var(--space-xl);
}

/* 侧边栏 */
.profile-sidebar {
  display: flex;
  flex-direction: column;
  gap: var(--space-lg);
}

.profile-card--security {
  padding: var(--space-xl);
}

.security-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-lg);
}

.security-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-md) 0;
  border-bottom: 1px solid var(--color-border-lighter);
}

.security-item:last-child {
  border-bottom: none;
}

.security-item__info {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
}

.security-item__label {
  font-size: var(--font-size-body);
  color: var(--color-text-primary);
  font-weight: 500;
}

.security-item__status {
  font-size: var(--font-size-small);
  padding: 2px 8px;
  border-radius: var(--radius-sm);
  width: fit-content;
}

.security-item__status--safe {
  background: var(--color-success-light);
  color: var(--color-success);
}

.security-item__status--danger {
  background: var(--color-error-light);
  color: var(--color-error);
}

.security-item__action {
  padding: 6px 16px;
  border-radius: var(--radius-md);
  font-size: var(--font-size-small);
  color: var(--color-primary);
  background: var(--color-primary-soft);
  border: none;
  cursor: pointer;
  transition: all 0.2s ease;
}

.security-item__action:hover {
  background: var(--color-primary-light);
}

/* 响应式适配 */
@media (max-width: 1024px) {
  .profile-content {
    grid-template-columns: 1fr;
  }

  .profile-sidebar {
    flex-direction: row;
  }

  .profile-card--security {
    flex: 1;
  }
}

@media (max-width: 768px) {
  .profile-view {
    padding: var(--space-lg);
  }

  .profile-card__header {
    flex-direction: column;
    align-items: center;
    text-align: center;
  }

  .profile-stats {
    grid-template-columns: 1fr;
  }

  .profile-actions {
    justify-content: center;
  }

  .profile-sidebar {
    flex-direction: column;
  }
}

@media (max-width: 480px) {
  .profile-btn {
    width: 100%;
    justify-content: center;
  }
}
</style>
