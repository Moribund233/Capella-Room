<script setup lang="ts">
import { computed } from 'vue'
import type { DirectRoom } from '@/types/room'
import { formatTime, truncate } from '@/utils'

const props = defineProps<{
  room: DirectRoom
  active?: boolean
  compact?: boolean
}>()

const emit = defineEmits<{
  click: [roomId: string]
}>()

const previewText = computed(() => {
  if (!props.room.last_message) return '暂无消息'
  return truncate(props.room.last_message.content, 50)
})

const timeText = computed(() => {
  if (!props.room.last_message) return ''
  return formatTime(props.room.last_message.created_at)
})

const initial = computed(() =>
  props.room.target_user.username.charAt(0).toUpperCase(),
)
</script>

<template>
  <div
    class="direct-room-card"
    :class="{
      'direct-room-card--active': active,
      'direct-room-card--compact': compact
    }"
    @click="emit('click', room.id)"
    :title="compact ? room.target_user.username : ''"
  >
    <div class="direct-room-card__avatar" :class="{ 'direct-room-card__avatar--compact': compact }">
      <img
        v-if="room.target_user.avatar_url"
        :src="room.target_user.avatar_url"
        :alt="room.target_user.username"
        class="direct-room-card__avatar-img"
      />
      <span v-else class="direct-room-card__avatar-text">{{ initial }}</span>
      <span v-if="compact && room.unread_count" class="direct-room-card__unread-dot" />
    </div>

    <div v-if="!compact" class="direct-room-card__body">
      <div class="direct-room-card__top">
        <span class="direct-room-card__name">{{ room.target_user.username }}</span>
        <span class="direct-room-card__time">{{ timeText }}</span>
      </div>
      <div class="direct-room-card__bottom">
        <span class="direct-room-card__preview">{{ previewText }}</span>
        <span v-if="room.unread_count" class="direct-room-card__unread">
          {{ room.unread_count > 99 ? '99+' : room.unread_count }}
        </span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.direct-room-card {
  display: flex;
  gap: 12px;
  padding: 12px 16px;
  cursor: pointer;
  transition: background var(--duration-fast, 0.15s);
  border-bottom: 1px solid var(--color-border, #eee);
}

.direct-room-card:hover {
  background: var(--color-background, #f5f5f5);
}

.direct-room-card--active {
  background: var(--color-primary-soft, #e8f5fe);
}

.direct-room-card__avatar {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background: var(--color-primary, #2080f0);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  overflow: hidden;
}

.direct-room-card__avatar-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.direct-room-card__avatar-text {
  color: var(--color-white);
  font-weight: 600;
  font-size: 16px;
}

.direct-room-card__body {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  justify-content: center;
  gap: 4px;
}

.direct-room-card__top {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.direct-room-card__name {
  font-weight: 600;
  font-size: var(--font-size-body, 14px);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.direct-room-card__time {
  font-size: var(--font-size-small, 12px);
  color: var(--color-text-tertiary, #999);
  flex-shrink: 0;
  margin-left: 8px;
}

.direct-room-card__bottom {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.direct-room-card__preview {
  font-size: var(--font-size-small, 12px);
  color: var(--color-text-tertiary, #999);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
}

.direct-room-card__unread {
  font-size: 11px;
  min-width: 18px;
  height: 18px;
  padding: 0 5px;
  border-radius: 9px;
  background: var(--color-error);
  color: var(--color-white);
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  flex-shrink: 0;
  margin-left: 8px;
}

/* Compact mode */
.direct-room-card--compact {
  padding: 8px;
  justify-content: center;
  border-bottom: none;
}

.direct-room-card--compact:hover {
  background: var(--color-background, #f5f5f5);
  border-radius: var(--radius-md, 8px);
}

.direct-room-card__avatar--compact {
  width: 44px;
  height: 44px;
  position: relative;
}

.direct-room-card__unread-dot {
  position: absolute;
  top: 0;
  right: 0;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: var(--color-error);
  border: 2px solid var(--color-white);
}
</style>
