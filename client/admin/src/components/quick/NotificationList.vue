<template>
  <div class="notification-list">
    <n-empty v-if="!loading && notifications.length === 0" description="暂无通知" />

    <n-spin v-else-if="loading" />

    <div v-else class="list-container">
      <div v-for="item in notifications" :key="item.id" class="notification-item"
        :class="{ 'is-unread': !item.is_read }" @click="handleClick(item)">
        <!-- 图标 -->
        <div class="item-icon" :class="`type-${getDisplayType(item)}`">
          <component :is="getIcon(getDisplayType(item))" :size="18" />
        </div>

        <!-- 内容 -->
        <div class="item-content">
          <div class="item-header">
            <span class="item-title">{{ item.title || getDefaultTitle(item) }}</span>
            <span class="item-time">{{ formatTime(item.created_at) }}</span>
          </div>
          <p class="item-text">{{ item.content }}</p>
        </div>

        <!-- 操作 -->
        <div class="item-actions">
          <n-button v-if="!item.is_read" text size="tiny" @click.stop="$emit('read', item.id)">
            <Check :size="14" />
          </n-button>
          <n-button text size="tiny" type="error" @click.stop="$emit('delete', item.id)">
            <X :size="14" />
          </n-button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { NEmpty, NSpin, NButton } from 'naive-ui'
import { Bell, Shield, MessageSquare, Home, Check, X } from 'lucide-vue-next'
import type { FunctionalComponent } from 'vue'
import type { LucideProps } from 'lucide-vue-next'
import type { NotificationItem } from '@/api/notification'

/**
 * 组件属性
 */
interface Props {
  notifications: NotificationItem[]
  loading?: boolean
}

withDefaults(defineProps<Props>(), {
  loading: false,
})

/**
 * 组件事件
 */
defineEmits<{
  (e: 'read', id: string): void
  (e: 'delete', id: string): void
}>()

/**
 * 获取显示类型
 */
function getDisplayType(item: NotificationItem & { displayType?: string }): string {
  return item.displayType || mapType(item.notification_type)
}

/**
 * 映射通知类型为显示类型
 *
 * 支持新的 HTTP API 类型和旧类型（兼容）
 */
function mapType(type: string): 'system' | 'security' | 'message' | 'room' {
  switch (type) {
    // 系统通知类型
    case 'system':
    case 'system_notification':
    case 'config_reload_required':
      return 'system'
    // 安全/待办类型
    case 'pending_action':
      return 'security'
    // 消息相关类型
    case 'private_message':
    case 'mention':
    case 'mentioned':
      return 'message'
    // 房间相关类型
    case 'room_invitation':
      return 'room'
    // 文件上传类型归为系统通知
    case 'file_upload':
    case 'file_upload_complete':
      return 'system'
    default:
      return 'system'
  }
}

/**
 * 获取通知类型图标
 */
function getIcon(type: string): FunctionalComponent<LucideProps> {
  const iconMap: Record<string, FunctionalComponent<LucideProps>> = {
    system: Bell,
    security: Shield,
    message: MessageSquare,
    room: Home,
  }
  return iconMap[type] || Bell
}

/**
 * 获取默认标题
 *
 * 支持新的 HTTP API 类型和旧类型（兼容）
 */
function getDefaultTitle(item: NotificationItem): string {
  switch (item.notification_type) {
    case 'private_message':
      return '新私信'
    case 'mention':
    case 'mentioned':
      return '@提及'
    case 'room_invitation':
      return '房间邀请'
    case 'system':
    case 'system_notification':
      return '系统通知'
    case 'file_upload':
    case 'file_upload_complete':
      return '文件上传完成'
    case 'config_reload_required':
      return '配置更新'
    case 'pending_action':
      return '待办事项'
    default:
      return '通知'
  }
}

/**
 * 格式化时间
 */
function formatTime(time: string): string {
  const date = new Date(time)
  const now = new Date()
  const diff = now.getTime() - date.getTime()

  // 小于1分钟
  if (diff < 60000) {
    return '刚刚'
  }
  // 小于1小时
  if (diff < 3600000) {
    return `${Math.floor(diff / 60000)}分钟前`
  }
  // 小于24小时
  if (diff < 86400000) {
    return `${Math.floor(diff / 3600000)}小时前`
  }
  // 小于7天
  if (diff < 604800000) {
    return `${Math.floor(diff / 86400000)}天前`
  }

  return date.toLocaleDateString('zh-CN')
}

/**
 * 点击通知项
 */
function handleClick(item: NotificationItem): void {
  // TODO: 根据通知类型跳转或处理
  console.log('点击通知:', item)
}
</script>

<style scoped>
.notification-list {
  min-height: 200px;
  max-height: 400px;
  overflow-y: auto;
}

.list-container {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.notification-item {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 12px;
  border-radius: 8px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.notification-item:hover {
  background-color: var(--hover-color);
}

.notification-item.is-unread {
  background-color: var(--unread-bg-color, rgba(59, 130, 246, 0.05));
}

.notification-item.is-unread .item-title {
  font-weight: 600;
}

.item-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border-radius: 8px;
  flex-shrink: 0;
}

.type-system {
  background-color: rgba(59, 130, 246, 0.1);
  color: #3b82f6;
}

.type-security {
  background-color: rgba(239, 68, 68, 0.1);
  color: #ef4444;
}

.type-message {
  background-color: rgba(16, 185, 129, 0.1);
  color: #10b981;
}

.type-room {
  background-color: rgba(139, 92, 246, 0.1);
  color: #8b5cf6;
}

.item-content {
  flex: 1;
  min-width: 0;
}

.item-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 4px;
}

.item-title {
  font-size: 14px;
  color: var(--text-color);
}

.item-time {
  font-size: 12px;
  color: var(--text-color-secondary);
  flex-shrink: 0;
}

.item-text {
  font-size: 13px;
  color: var(--text-color-secondary);
  line-height: 1.5;
  margin: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

.item-actions {
  display: flex;
  gap: 4px;
  opacity: 0;
  transition: opacity 0.2s;
}

.notification-item:hover .item-actions {
  opacity: 1;
}
</style>
