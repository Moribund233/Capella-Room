<script setup lang="ts">
import { computed } from 'vue'
import type { Room } from '@/types/room'
import { formatTime, truncate } from '@/utils'

const props = defineProps<{
  room: Room
  active?: boolean
  compact?: boolean
}>()

const emit = defineEmits<{
  click: [roomId: string]
}>()

const previewText = computed(() => {
  if (!props.room.last_message) return '暂无消息'
  const prefix = props.room.last_message.sender_name
    ? `${props.room.last_message.sender_name}: `
    : ''
  return truncate(`${prefix}${props.room.last_message.content}`, 50)
})

const timeText = computed(() => {
  if (!props.room.last_message) return ''
  return formatTime(props.room.last_message.created_at)
})

const initial = computed(() =>
  props.room.name.charAt(0).toUpperCase(),
)
</script>

<template>
  <div
    class="room-card"
    :class="{
      'room-card--active': active,
      'room-card--compact': compact
    }"
    @click="emit('click', room.id)"
    :title="compact ? room.name : ''"
  >
    <div class="room-card__avatar" :class="{ 'room-card__avatar--compact': compact }">
      <span class="room-card__avatar-text">{{ initial }}</span>
      <span v-if="compact && room.unread_count" class="room-card__unread-dot" />
    </div>

    <div v-if="!compact" class="room-card__body">
      <div class="room-card__top">
        <span class="room-card__name">{{ room.name }}</span>
        <span class="room-card__time">{{ timeText }}</span>
      </div>
      <div class="room-card__bottom">
        <span class="room-card__preview">{{ previewText }}</span>
        <div class="room-card__meta">
          <span v-if="room.is_private" class="room-card__badge">私密</span>
          <span v-if="room.unread_count" class="room-card__unread">
            {{ room.unread_count > 99 ? '99+' : room.unread_count }}
          </span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.room-card {
  display: flex;
  gap: 12px;
  padding: 12px 16px;
  cursor: pointer;
  transition: background var(--duration-fast, 0.15s);
  border-bottom: 1px solid var(--color-border, #eee);
}

.room-card:hover {
  background: var(--color-background, #f5f5f5);
}

.room-card--active {
  background: var(--color-primary-soft, #e8f5fe);
}

.room-card__avatar {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background: var(--color-primary, #2080f0);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.room-card__avatar-text {
  color: var(--color-white);
  font-weight: 600;
  font-size: 16px;
}

.room-card__body {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  justify-content: center;
  gap: 4px;
}

.room-card__top {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.room-card__name {
  font-weight: 600;
  font-size: var(--font-size-body, 14px);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.room-card__time {
  font-size: var(--font-size-small, 12px);
  color: var(--color-text-tertiary, #999);
  flex-shrink: 0;
  margin-left: 8px;
}

.room-card__bottom {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.room-card__preview {
  font-size: var(--font-size-small, 12px);
  color: var(--color-text-tertiary, #999);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
}

.room-card__meta {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
  margin-left: 8px;
}

.room-card__badge {
  font-size: 10px;
  padding: 1px 5px;
  border-radius: 4px;
  background: var(--color-warning-soft, #fff3e0);
  color: var(--color-warning, #f0a020);
}

.room-card__unread {
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
}

/* Compact mode */
.room-card--compact {
  padding: 8px;
  justify-content: center;
  border-bottom: none;
}

.room-card--compact:hover {
  background: var(--color-background, #f5f5f5);
  border-radius: var(--radius-md, 8px);
}

.room-card__avatar--compact {
  width: 44px;
  height: 44px;
  position: relative;
}

.room-card__unread-dot {
  position: absolute;
  top: 0;
  right: 0;
  width: 12px;
  height: 12px;
  background: var(--color-error, #d03050);
  border-radius: 50%;
  border: 2px solid var(--color-white, #fff);
}
</style>
