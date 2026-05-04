<script setup lang="ts">
import { computed } from 'vue'
import { Bell, CheckCheck, Trash2, Settings } from 'lucide-vue-next'

/**
 * 通知项接口
 */
interface NotificationItem {
  id: string
  title: string
  content: string
  type: 'info' | 'success' | 'warning' | 'error'
  isRead: boolean
  createdAt: string
}

/**
 * 组件属性定义
 */
interface Props {
  /** 通知列表 */
  notifications?: NotificationItem[]
  /** 加载状态 */
  loading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  notifications: () => [],
  loading: false,
})

/**
 * 组件事件定义
 */
const emit = defineEmits<{
  /** 标记单个已读 */
  (e: 'markAsRead', id: string): void
  /** 标记全部已读 */
  (e: 'markAllAsRead'): void
  /** 删除通知 */
  (e: 'delete', id: string): void
  /** 清空所有 */
  (e: 'clearAll'): void
  /** 打开设置 */
  (e: 'openSettings'): void
}>()

/**
 * 未读数量
 */
const unreadCount = computed(() => props.notifications.filter(n => !n.isRead).length)

/**
 * 是否有通知
 */
const hasNotifications = computed(() => props.notifications.length > 0)

/**
 * 按日期分组的通知
 */
const groupedNotifications = computed(() => {
  const groups: Record<string, NotificationItem[]> = {}
  
  props.notifications.forEach(notification => {
    const date = new Date(notification.createdAt).toLocaleDateString('zh-CN')
    if (!groups[date]) {
      groups[date] = []
    }
    groups[date].push(notification)
  })
  
  return groups
})

/**
 * 获取类型图标颜色
 */
function getTypeColor(type: NotificationItem['type']): string {
  const colors = {
    info: 'var(--color-info)',
    success: 'var(--color-success)',
    warning: 'var(--color-warning)',
    error: 'var(--color-danger)',
  }
  return colors[type]
}

/**
 * 获取类型标签
 */
function getTypeLabel(type: NotificationItem['type']): string {
  const labels = {
    info: '信息',
    success: '成功',
    warning: '警告',
    error: '错误',
  }
  return labels[type]
}

/**
 * 格式化时间
 */
function formatTime(dateStr: string): string {
  const date = new Date(dateStr)
  const now = new Date()
  const diff = now.getTime() - date.getTime()
  
  // 小于1小时显示相对时间
  if (diff < 60 * 60 * 1000) {
    const minutes = Math.floor(diff / (60 * 1000))
    return minutes < 1 ? '刚刚' : `${minutes}分钟前`
  }
  
  // 今天显示时间
  if (date.toDateString() === now.toDateString()) {
    return date.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' })
  }
  
  // 其他显示日期时间
  return date.toLocaleString('zh-CN', { 
    month: 'short', 
    day: 'numeric', 
    hour: '2-digit', 
    minute: '2-digit' 
  })
}

/**
 * 处理标记已读
 */
function handleMarkAsRead(id: string) {
  emit('markAsRead', id)
}

/**
 * 处理标记全部已读
 */
function handleMarkAllAsRead() {
  emit('markAllAsRead')
}

/**
 * 处理删除
 */
function handleDelete(id: string) {
  emit('delete', id)
}

/**
 * 处理清空
 */
function handleClearAll() {
  emit('clearAll')
}

/**
 * 处理设置
 */
function handleSettings() {
  emit('openSettings')
}
</script>

<template>
  <div class="notification-panel">
    <!-- 头部工具栏 -->
    <div class="notification-panel__header">
      <div class="notification-panel__title">
        <Bell :size="18" />
        <span>通知中心</span>
        <span v-if="unreadCount > 0" class="notification-panel__badge">
          {{ unreadCount }}
        </span>
      </div>
      <div class="notification-panel__actions">
        <button
          v-if="unreadCount > 0"
          class="notification-panel__action-btn"
          title="全部已读"
          @click="handleMarkAllAsRead"
        >
          <CheckCheck :size="16" />
        </button>
        <button
          v-if="hasNotifications"
          class="notification-panel__action-btn"
          title="清空全部"
          @click="handleClearAll"
        >
          <Trash2 :size="16" />
        </button>
        <button
          class="notification-panel__action-btn"
          title="设置"
          @click="handleSettings"
        >
          <Settings :size="16" />
        </button>
      </div>
    </div>

    <!-- 加载状态 -->
    <div v-if="loading" class="notification-panel__loading">
      <div class="notification-panel__spinner" />
      <span>加载中...</span>
    </div>

    <!-- 空状态 -->
    <div v-else-if="!hasNotifications" class="notification-panel__empty">
      <Bell :size="48" class="notification-panel__empty-icon" />
      <p class="notification-panel__empty-text">暂无通知</p>
      <p class="notification-panel__empty-hint">当有新消息时会显示在这里</p>
    </div>

    <!-- 通知列表 -->
    <div v-else class="notification-panel__list">
      <div
        v-for="(items, date) in groupedNotifications"
        :key="date"
        class="notification-panel__group"
      >
        <div class="notification-panel__date">{{ date }}</div>
        <div
          v-for="notification in items"
          :key="notification.id"
          class="notification-panel__item"
          :class="{ 'is-unread': !notification.isRead }"
        >
          <!-- 类型指示器 -->
          <div
            class="notification-panel__type-indicator"
            :style="{ backgroundColor: getTypeColor(notification.type) }"
          />
          
          <!-- 内容 -->
          <div class="notification-panel__content">
            <div class="notification-panel__item-header">
              <span
                class="notification-panel__type-label"
                :style="{ color: getTypeColor(notification.type) }"
              >
                {{ getTypeLabel(notification.type) }}
              </span>
              <span class="notification-panel__time">
                {{ formatTime(notification.createdAt) }}
              </span>
            </div>
            <h4 class="notification-panel__item-title">{{ notification.title }}</h4>
            <p class="notification-panel__item-text">{{ notification.content }}</p>
          </div>

          <!-- 操作 -->
          <div class="notification-panel__item-actions">
            <button
              v-if="!notification.isRead"
              class="notification-panel__item-btn"
              title="标记已读"
              @click="handleMarkAsRead(notification.id)"
            >
              <CheckCheck :size="14" />
            </button>
            <button
              class="notification-panel__item-btn"
              title="删除"
              @click="handleDelete(notification.id)"
            >
              <Trash2 :size="14" />
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.notification-panel {
  display: flex;
  flex-direction: column;
  max-height: 60vh;
  min-height: 300px;
}

/* 头部 */
.notification-panel__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-md) var(--space-lg);
  border-bottom: 1px solid var(--color-border);
  flex-shrink: 0;
}

.notification-panel__title {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  font-weight: 600;
  color: var(--color-text-primary);
}

.notification-panel__badge {
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: 18px;
  height: 18px;
  padding: 0 5px;
  background: var(--color-danger);
  color: white;
  font-size: 11px;
  font-weight: 600;
  border-radius: 9px;
}

.notification-panel__actions {
  display: flex;
  gap: var(--space-xs);
}

.notification-panel__action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: none;
  background: transparent;
  color: var(--color-text-secondary);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--duration-fast);
}

.notification-panel__action-btn:hover {
  background: var(--color-background);
  color: var(--color-text-primary);
}

/* 加载状态 */
.notification-panel__loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--space-md);
  padding: var(--space-xl);
  color: var(--color-text-secondary);
}

.notification-panel__spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--color-border);
  border-top-color: var(--color-primary);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

/* 空状态 */
.notification-panel__empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--space-2xl);
  text-align: center;
}

.notification-panel__empty-icon {
  color: var(--color-text-tertiary);
  margin-bottom: var(--space-md);
}

.notification-panel__empty-text {
  font-size: var(--font-size-h6);
  font-weight: 500;
  color: var(--color-text-secondary);
  margin: 0 0 var(--space-xs);
}

.notification-panel__empty-hint {
  font-size: var(--font-size-small);
  color: var(--color-text-tertiary);
  margin: 0;
}

/* 列表 */
.notification-panel__list {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-sm);
}

.notification-panel__group {
  margin-bottom: var(--space-md);
}

.notification-panel__date {
  padding: var(--space-xs) var(--space-md);
  font-size: var(--font-size-small);
  font-weight: 500;
  color: var(--color-text-secondary);
  background: var(--color-background);
  border-radius: var(--radius-sm);
  margin-bottom: var(--space-sm);
}

/* 通知项 */
.notification-panel__item {
  display: flex;
  gap: var(--space-sm);
  padding: var(--space-md);
  border-radius: var(--radius-md);
  background: var(--color-white);
  border: 1px solid var(--color-border);
  margin-bottom: var(--space-sm);
  transition: all var(--duration-fast);
}

.notification-panel__item:hover {
  border-color: var(--color-primary-light);
  box-shadow: var(--shadow-sm);
}

.notification-panel__item.is-unread {
  background: var(--color-primary-light, rgba(99, 102, 241, 0.05));
  border-color: var(--color-primary-light);
}

.notification-panel__type-indicator {
  width: 4px;
  height: 100%;
  min-height: 40px;
  border-radius: 2px;
  flex-shrink: 0;
}

.notification-panel__content {
  flex: 1;
  min-width: 0;
}

.notification-panel__item-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--space-xs);
}

.notification-panel__type-label {
  font-size: var(--font-size-small);
  font-weight: 500;
}

.notification-panel__time {
  font-size: var(--font-size-small);
  color: var(--color-text-tertiary);
}

.notification-panel__item-title {
  font-size: var(--font-size-body);
  font-weight: 500;
  color: var(--color-text-primary);
  margin: 0 0 var(--space-xs);
  line-height: 1.4;
}

.notification-panel__item-text {
  font-size: var(--font-size-small);
  color: var(--color-text-secondary);
  margin: 0;
  line-height: 1.5;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.notification-panel__item-actions {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
  opacity: 0;
  transition: opacity var(--duration-fast);
}

.notification-panel__item:hover .notification-panel__item-actions {
  opacity: 1;
}

.notification-panel__item-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  color: var(--color-text-tertiary);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--duration-fast);
}

.notification-panel__item-btn:hover {
  background: var(--color-background);
  color: var(--color-text-primary);
}

/* 移动端适配 */
@media (max-width: 640px) {
  .notification-panel {
    max-height: 70vh;
    min-height: 250px;
  }

  .notification-panel__item-actions {
    opacity: 1;
  }
}
</style>
