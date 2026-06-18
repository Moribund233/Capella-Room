<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useNotificationStore } from '@/stores/notification'
import { Bell, Check, Delete, ArrowDown, Loading, InfoFilled, SuccessFilled, WarningFilled, CircleCloseFilled, Close } from '@element-plus/icons-vue'
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
  <ElDrawer
    :model-value="notificationStore.isPanelOpen"
    :title="t('quick.notifications')"
    size="420px"
    @close="notificationStore.closePanel()"
  >
    <template #header="{ close, titleId, titleClass }">
      <div class="notification-drawer__header">
        <div class="notification-drawer__title">
          <el-icon :size="18"><Bell /></el-icon>
          <span :id="titleId" :class="titleClass">{{ t('quick.notifications') }}</span>
          <span v-if="unreadCount > 0" class="notification-drawer__badge">
            {{ unreadCount }}
          </span>
        </div>
        <div class="notification-drawer__actions">
          <el-button
            v-if="unreadCount > 0"
            text
            :icon="Check"
            size="small"
            title="全部已读"
            @click="handleMarkAllAsRead"
          />
          <el-button
            v-if="hasNotifications"
            text
            :icon="Delete"
            size="small"
            title="清空全部"
            @click="handleClearAll"
          />
          <el-button text :icon="Close" size="small" @click="close" />
        </div>
      </div>
    </template>

    <div class="notification-drawer__body">
      <div v-if="loading && !hasNotifications" class="notification-drawer__loading">
        <el-icon class="is-loading" :size="24"><Loading /></el-icon>
        <span>{{ t('common.loading') }}</span>
      </div>

      <ElEmpty v-else-if="!hasNotifications" description="暂无通知" />

      <template v-else>
        <ElScrollbar class="notification-drawer__list">
          <div
            v-for="(items, date) in groupedNotifications"
            :key="date"
            class="notification-drawer__group"
          >
            <div class="notification-drawer__date">{{ date }}</div>
            <div
              v-for="notification in items"
              :key="notification.id"
              class="notification-drawer__item"
              :class="{ 'is-unread': !notification.isRead }"
            >
              <el-icon
                :size="14"
                class="notification-drawer__type-icon"
                :style="{ color: getTypeColor(notification.type) }"
              >
                <component :is="typeIconMap[notification.type] || InfoFilled" />
              </el-icon>

              <div class="notification-drawer__content">
                <div class="notification-drawer__item-header">
                  <span class="notification-drawer__item-title">{{ notification.title }}</span>
                  <span class="notification-drawer__time">{{ formatTime(notification.createdAt) }}</span>
                </div>
                <p class="notification-drawer__item-text">{{ notification.content }}</p>
              </div>

              <div class="notification-drawer__item-actions">
                <el-button
                  v-if="!notification.isRead"
                  text
                  :icon="Check"
                  size="small"
                  title="标记已读"
                  @click="handleMarkAsRead(notification.id)"
                />
                <el-button
                  text
                  :icon="Delete"
                  size="small"
                  title="删除"
                  @click="handleDelete(notification.id)"
                />
              </div>
            </div>
          </div>

          <div v-if="hasMore" class="notification-drawer__load-more">
            <el-button
              :loading="loading"
              :icon="ArrowDown"
              @click="handleLoadMore"
            >
              {{ loading ? '加载中...' : '加载更多' }}
            </el-button>
          </div>
        </ElScrollbar>
      </template>
    </div>
  </ElDrawer>
</template>

<style scoped lang="scss">
.notification-drawer__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
}

.notification-drawer__title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
}

.notification-drawer__badge {
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

.notification-drawer__actions {
  display: flex;
  align-items: center;
  gap: 4px;
}

.notification-drawer__body {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.notification-drawer__loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 48px 0;
  color: var(--muted);
}

.notification-drawer__list {
  flex: 1;
  padding: 8px 0;
}

.notification-drawer__group {
  margin-bottom: 16px;
}

.notification-drawer__date {
  font-size: 12px;
  font-weight: 500;
  color: var(--muted);
  padding: 4px 16px;
  margin-bottom: 4px;
}

.notification-drawer__item {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 10px 16px;
  margin: 0 8px;
  border-radius: 8px;
  transition: background 0.15s;
  margin-bottom: 2px;

  &:hover {
    background: var(--message-hover);

    .notification-drawer__item-actions {
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

.notification-drawer__type-icon {
  margin-top: 2px;
  flex-shrink: 0;
}

.notification-drawer__content {
  flex: 1;
  min-width: 0;
}

.notification-drawer__item-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.notification-drawer__item-title {
  font-size: 13px;
  font-weight: 500;
  color: var(--fg);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.notification-drawer__time {
  font-size: 11px;
  color: var(--muted);
  white-space: nowrap;
  flex-shrink: 0;
}

.notification-drawer__item-text {
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

.notification-drawer__item-actions {
  display: flex;
  align-items: center;
  gap: 2px;
  flex-shrink: 0;
  opacity: 0;
  transition: opacity 0.15s;
}

.notification-drawer__load-more {
  display: flex;
  justify-content: center;
  padding: 16px;
}
</style>
