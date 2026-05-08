<template>
  <div class="notification-panel">
    <!-- 通知类型标签页 -->
    <n-tabs v-model:value="activeTab" type="line" animated>
      <n-tab-pane name="all" tab="全部">
        <NotificationList :notifications="allNotifications" :loading="notificationStore.loading" @read="handleRead"
          @delete="handleDelete" />
      </n-tab-pane>
      <n-tab-pane name="unread" tab="未读">
        <NotificationList :notifications="unreadNotifications" :loading="notificationStore.loading" @read="handleRead"
          @delete="handleDelete" />
      </n-tab-pane>
      <n-tab-pane name="system" tab="系统">
        <NotificationList :notifications="systemNotifications" :loading="notificationStore.loading" @read="handleRead"
          @delete="handleDelete" />
      </n-tab-pane>
      <n-tab-pane name="security" tab="安全">
        <NotificationList :notifications="securityNotifications" :loading="notificationStore.loading" @read="handleRead"
          @delete="handleDelete" />
      </n-tab-pane>
    </n-tabs>

    <!-- 底部操作栏 -->
    <div class="panel-footer">
      <n-button text size="small" @click="handleMarkAllRead">
        <template #icon>
          <CheckCheck :size="14" />
        </template>
        全部标记为已读
      </n-button>
      <n-button text size="small" type="error" @click="handleClearAll">
        <template #icon>
          <Trash2 :size="14" />
        </template>
        清空所有
      </n-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { NTabs, NTabPane, NButton } from 'naive-ui'
import { CheckCheck, Trash2 } from 'lucide-vue-next'
import { useNotificationStore } from '@/store'
import NotificationList from './NotificationList.vue'
import type { NotificationItem } from '@/api/notification'

const notificationStore = useNotificationStore()
const activeTab = ref('all')

/**
 * 将后端通知类型映射为前端显示类型
 *
 * 支持新的 HTTP API 类型和旧类型（兼容）
 */
function mapNotificationType(type: string): 'system' | 'security' | 'message' | 'room' {
  switch (type) {
    // 系统通知类型
    case 'system':
    case 'system_notification':
    case 'config_reload_required':
    case 'file_upload':
    case 'file_upload_complete':
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
    default:
      return 'system'
  }
}

/**
 * 转换通知数据为组件所需格式
 */
const allNotifications = computed(() =>
  notificationStore.notifications.map((n: NotificationItem): NotificationItem & { displayType: string } => ({
    ...n,
    displayType: mapNotificationType(n.notification_type),
  })),
)

const unreadNotifications = computed(() =>
  allNotifications.value.filter((n: NotificationItem & { displayType: string }) => !n.is_read),
)

const systemNotifications = computed(() =>
  allNotifications.value.filter((n: NotificationItem & { displayType: string }) => n.displayType === 'system'),
)

const securityNotifications = computed(() =>
  allNotifications.value.filter((n: NotificationItem & { displayType: string }) => n.displayType === 'security'),
)

/**
 * 标记通知为已读
 */
async function handleRead(id: string): Promise<void> {
  try {
    await notificationStore.markAsRead(id)
  } catch (err) {
    console.error('标记已读失败:', err)
  }
}

/**
 * 删除通知
 */
async function handleDelete(id: string): Promise<void> {
  try {
    await notificationStore.removeNotification(id)
  } catch (err) {
    console.error('删除通知失败:', err)
  }
}

/**
 * 标记所有为已读
 */
async function handleMarkAllRead(): Promise<void> {
  try {
    await notificationStore.markAllAsRead()
  } catch (err) {
    console.error('标记全部已读失败:', err)
  }
}

/**
 * 清空所有通知
 */
function handleClearAll(): void {
  notificationStore.clearAllNotifications()
}

onMounted(() => {
  // 从 API 获取通知列表
  notificationStore.fetchNotifications(50, 0)
})
</script>

<style scoped>
.notification-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.panel-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-top: 1px solid var(--border-color);
  margin-top: auto;
}
</style>
