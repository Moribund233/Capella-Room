<script setup lang="ts">
import { computed } from 'vue'
import { User } from 'lucide-vue-next'
import type { UserSearchItem } from '@/types/search'

/**
 * 组件属性定义
 */
interface Props {
  /** 用户信息 */
  user: UserSearchItem
  /** 是否显示在线状态 */
  showStatus?: boolean
  /** 是否可点击 */
  clickable?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  showStatus: true,
  clickable: true,
})

/**
 * 组件事件定义
 */
interface Emits {
  /** 点击卡片 */
  (e: 'click', user: UserSearchItem): void
}

const emit = defineEmits<Emits>()

/**
 * 在线状态文本
 */
const statusText = computed(() => {
  const statusMap: Record<string, string> = {
    online: '在线',
    offline: '离线',
    away: '离开',
  }
  return statusMap[props.user.status] || '离线'
})

/**
 * 在线状态样式类
 */
const statusClass = computed(() => {
  return `status--${props.user.status}`
})

/**
 * 处理点击
 */
function handleClick() {
  if (props.clickable) {
    emit('click', props.user)
  }
}
</script>

<template>
  <div
    class="user-card"
    :class="{ 'user-card--clickable': clickable }"
    @click="handleClick"
  >
    <!-- 头像 -->
    <div class="user-card__avatar">
      <img
        v-if="user.avatar_url"
        :src="user.avatar_url"
        :alt="user.username"
        class="user-card__avatar-img"
      />
      <div v-else class="user-card__avatar-placeholder">
        <User :size="24" />
      </div>
      <!-- 在线状态指示器 -->
      <span
        v-if="showStatus"
        class="user-card__status-indicator"
        :class="statusClass"
      />
    </div>

    <!-- 用户信息 -->
    <div class="user-card__info">
      <div class="user-card__name">{{ user.username }}</div>
      <div v-if="showStatus" class="user-card__status" :class="statusClass">
        {{ statusText }}
      </div>
    </div>
  </div>
</template>

<style scoped>
.user-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  border-radius: 8px;
  transition: background-color 0.2s ease;
}

.user-card--clickable {
  cursor: pointer;
}

.user-card--clickable:hover {
  background-color: var(--color-background-soft, #f5f5f5);
}

.user-card__avatar {
  position: relative;
  flex-shrink: 0;
}

.user-card__avatar-img {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  object-fit: cover;
}

.user-card__avatar-placeholder {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  background-color: var(--color-background-soft, #f0f0f0);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-secondary, #666);
}

.user-card__status-indicator {
  position: absolute;
  bottom: 2px;
  right: 2px;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  border: 2px solid var(--color-white, #fff);
  background-color: var(--color-text-tertiary, #999);
}

.user-card__status-indicator.status--online {
  background-color: var(--color-success, #52c41a);
}

.user-card__status-indicator.status--away {
  background-color: var(--color-warning, #faad14);
}

.user-card__status-indicator.status--offline {
  background-color: var(--color-text-tertiary, #999);
}

.user-card__info {
  flex: 1;
  min-width: 0;
}

.user-card__name {
  font-size: 14px;
  font-weight: 500;
  color: var(--color-text, #333);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.user-card__status {
  font-size: 12px;
  margin-top: 2px;
}

.user-card__status.status--online {
  color: var(--color-success, #52c41a);
}

.user-card__status.status--away {
  color: var(--color-warning, #faad14);
}

.user-card__status.status--offline {
  color: var(--color-text-tertiary, #999);
}
</style>
