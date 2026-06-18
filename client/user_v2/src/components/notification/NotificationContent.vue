<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useNotificationStore } from '@/stores/notification'
import {
  Bell,
  Check,
  Delete,
  ArrowDown,
  Loading,
  InfoFilled,
  SuccessFilled,
  WarningFilled,
  CircleCloseFilled,
} from '@element-plus/icons-vue'
import type { NotificationItem } from '@/types/notification'

const { t } = useI18n()
const notificationStore = useNotificationStore()

onMounted(() => {
  if (!notificationStore.initialized) {
    notificationStore.initialize()
  }
  if (notificationStore.notifications.length === 0) {
    notificationStore.fetchOfflineNotifications()
  }
})

const notifications = computed(() => notificationStore.notifications)
const loading = computed(() => notificationStore.loading)
const hasMore = computed(() => notificationStore.hasMoreOffline)
const hasNotifications = computed(() => notifications.value.length > 0)

const unreadCount = computed(() =>
  notifications.value.filter((n: NotificationItem) => !n.isRead).length
)

const groupedNotifications = computed(() => {
  const groups: Record<string, NotificationItem[]> = {}
  notifications.value.forEach((n: NotificationItem) => {
    const date = new Date(n.createdAt).toLocaleDateString()
    if (!groups[date]) groups[date] = []
    groups[date].push(n)
  })
  return groups
})

function getTypeColor(type: NotificationItem['type']): string {
  const map: Record<string, string> = {
    info: 'var(--accent-blue)',
    success: 'var(--accent-green)',
    warning: 'var(--accent-orange)',
    error: 'var(--accent-pink)',
  }
  return map[type] || 'var(--accent-blue)'
}

const typeIconMap: Record<string, unknown> = {
  info: InfoFilled,
  success: SuccessFilled,
  warning: WarningFilled,
  error: CircleCloseFilled,
}

function formatTime(dateStr: string): string {
  const date = new Date(dateStr)
  const now = new Date()
  const diff = now.getTime() - date.getTime()

  if (diff < 60 * 60 * 1000) {
    const minutes = Math.floor(diff / (60 * 1000))
    if (minutes < 1) return '刚刚'
    return `${minutes}分钟前`
  }

  if (date.toDateString() === now.toDateString()) {
    return date.toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' })
  }

  return date.toLocaleString(undefined, {
    month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit',
  })
}

function handleMarkAsRead(id: string) {
  notificationStore.markAsRead(id)
}

function handleMarkAllAsRead() {
  notificationStore.markAllAsRead()
}

function handleDelete(id: string) {
  notificationStore.deleteNotification(id)
}

function handleClearAll() {
  notificationStore.clearAll()
}

function handleLoadMore() {
  notificationStore.loadMoreOfflineNotifications()
}
</script>

<template>
  <div class="notification-content">
    <!-- 头部操作栏 -->
    <div class="notification-content__toolbar">
      <div class="notification-content__toolbar-left">
        <el-icon :size="16"><Bell /></el-icon>
        <span>{{ t('quick.notifications') }}</span>
        <span v-if="unreadCount > 0" class="notification-content__badge">
          {{ unreadCount }}
        </span>
      </div>
      <div class="notification-content__toolbar-right">
        <el-button
          v-if="unreadCount > 0"
          text
          :icon="Check"
          size="small"
          @click="handleMarkAllAsRead"
        >
          {{ t('notification.markAllRead') }}
        </el-button>
        <el-button
          v-if="hasNotifications"
          text
          :icon="Delete"
          size="small"
          @click="handleClearAll"
        >
          {{ t('notification.clearAll') }}
        </el-button>
      </div>
    </div>

    <!-- 加载中 -->
    <div v-if="loading && !hasNotifications" class="notification-content__loading">
      <el-icon class="is-loading" :size="24"><Loading /></el-icon>
      <span>{{ t('common.loading') }}</span>
    </div>

    <!-- 空状态 -->
    <ElEmpty v-else-if="!hasNotifications">
      <template #description>
        <span>{{ t('notification.noNotifications') }}</span>
      </template>
    </ElEmpty>

    <!-- 通知列表 -->
    <ElScrollbar v-else class="notification-content__list">
      <div
        v-for="(items, date) in groupedNotifications"
        :key="date"
        class="notification-content__group"
      >
        <div class="notification-content__date">{{ date }}</div>
        <div
          v-for="notification in items"
          :key="notification.id"
          class="notification-content__item"
          :class="{ 'is-unread': !notification.isRead }"
        >
          <el-icon
            :size="14"
            class="notification-content__type-icon"
            :style="{ color: getTypeColor(notification.type) }"
          >
            <component :is="typeIconMap[notification.type] || InfoFilled" />
          </el-icon>

          <div class="notification-content__body">
            <div class="notification-content__item-header">
              <span class="notification-content__item-title">{{ notification.title }}</span>
              <span class="notification-content__time">{{ formatTime(notification.createdAt) }}</span>
            </div>
            <p class="notification-content__item-text">{{ notification.content }}</p>
          </div>

          <div class="notification-content__item-actions">
            <el-button
              v-if="!notification.isRead"
              text
              :icon="Check"
              size="small"
              @click="handleMarkAsRead(notification.id)"
            />
            <el-button
              text
              :icon="Delete"
              size="small"
              @click="handleDelete(notification.id)"
            />
          </div>
        </div>
      </div>

      <div v-if="hasMore" class="notification-content__load-more">
        <el-button
          :loading="loading"
          :icon="ArrowDown"
          @click="handleLoadMore"
        >
          {{ loading ? t('common.loading') : t('common.loadMore') }}
        </el-button>
      </div>
    </ElScrollbar>
  </div>
</template>

<style scoped lang="scss">
.notification-content {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 300px;
}

.notification-content__toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 0 12px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.notification-content__toolbar-left {
  display: flex;
  align-items: center;
  gap: 6px;
  font-weight: 600;
  font-size: 14px;
}

.notification-content__badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 18px;
  height: 18px;
  padding: 0 5px;
  background: var(--accent-pink);
  color: #fff;
  font-size: 11px;
  font-weight: 600;
  border-radius: 9px;
}

.notification-content__toolbar-right {
  display: flex;
  align-items: center;
  gap: 4px;
}

.notification-content__loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 48px 0;
  color: var(--muted);
  flex: 1;
}

.notification-content__list {
  flex: 1;
  margin: 0 -20px;
  padding: 0 20px;
}

.notification-content__group {
  margin-bottom: 16px;
}

.notification-content__date {
  font-size: 12px;
  font-weight: 500;
  color: var(--muted);
  padding: 8px 0 4px;
}

.notification-content__item {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 10px 12px;
  border-radius: 8px;
  transition: background 0.15s;
  margin-bottom: 2px;

  &:hover {
    background: var(--message-hover);

    .notification-content__item-actions {
      opacity: 1;
    }
  }

  &.is-unread {
    background: var(--accent-soft);

    &:hover {
      background: color-mix(in oklch, var(--accent) 24%, transparent);
    }
  }
}

.notification-content__type-icon {
  margin-top: 2px;
  flex-shrink: 0;
}

.notification-content__body {
  flex: 1;
  min-width: 0;
}

.notification-content__item-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.notification-content__item-title {
  font-size: 13px;
  font-weight: 500;
  color: var(--fg);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.notification-content__time {
  font-size: 11px;
  color: var(--muted);
  white-space: nowrap;
  flex-shrink: 0;
}

.notification-content__item-text {
  font-size: 12px;
  color: var(--muted);
  margin: 4px 0 0;
  line-height: 1.5;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

.notification-content__item-actions {
  display: flex;
  align-items: center;
  gap: 2px;
  flex-shrink: 0;
  opacity: 0;
  transition: opacity 0.15s;
}

.notification-content__load-more {
  display: flex;
  justify-content: center;
  padding: 16px 0;
}
</style>
