<script setup lang="ts">
import { computed } from 'vue'
import type { Friend } from '@/types/friend'

const props = defineProps<{
  friend: Friend
}>()

const emit = defineEmits<{
  click: [friend: Friend]
}>()

const statusText = computed(() => {
  const map: Record<string, string> = {
    online: '在线',
    away: '离开',
    busy: '忙碌',
    offline: '离线',
  }
  return map[props.friend.friend.status] || '离线'
})

const initial = computed(() =>
  props.friend.friend.username.charAt(0).toUpperCase(),
)
</script>

<template>
  <div class="friend-card" @click="emit('click', friend)">
    <div class="friend-card__avatar">
      <img
        v-if="friend.friend.avatar_url"
        :src="friend.friend.avatar_url"
        :alt="friend.friend.username"
        class="friend-card__avatar-img"
      />
      <div v-else class="friend-card__avatar-placeholder">
        <span class="friend-card__avatar-text">{{ initial }}</span>
      </div>
      <span
        class="friend-card__status-dot"
        :class="`friend-card__status-dot--${friend.friend.status}`"
      />
    </div>
    <div class="friend-card__body">
      <span class="friend-card__name">{{ friend.friend.username }}</span>
      <span class="friend-card__status" :class="`friend-card__status--${friend.friend.status}`">
        {{ statusText }}
      </span>
    </div>
  </div>
</template>

<style scoped>
.friend-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  cursor: pointer;
  transition: background var(--duration-fast, 0.15s);
  border-bottom: 1px solid var(--color-border, #eee);
}

.friend-card:hover {
  background: var(--color-background, #f5f5f5);
}

.friend-card__avatar {
  position: relative;
  flex-shrink: 0;
}

.friend-card__avatar-img {
  width: 44px;
  height: 44px;
  border-radius: 50%;
  object-fit: cover;
}

.friend-card__avatar-placeholder {
  width: 44px;
  height: 44px;
  border-radius: 50%;
  background: var(--color-primary, #2080f0);
  display: flex;
  align-items: center;
  justify-content: center;
}

.friend-card__avatar-text {
  color: white;
  font-weight: 600;
  font-size: 16px;
}

.friend-card__status-dot {
  position: absolute;
  bottom: 2px;
  right: 2px;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  border: 2px solid var(--color-white, #fff);
  background: var(--color-text-tertiary, #999);
}

.friend-card__status-dot--online { background: var(--color-success, #52c41a); }
.friend-card__status-dot--away { background: var(--color-warning, #faad14); }
.friend-card__status-dot--busy { background: var(--color-error, #f5222d); }
.friend-card__status-dot--offline { background: var(--color-text-tertiary, #999); }

.friend-card__body {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.friend-card__name {
  font-size: 14px;
  font-weight: 500;
  color: var(--color-text, #333);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.friend-card__status {
  font-size: 12px;
}

.friend-card__status--online { color: var(--color-success, #52c41a); }
.friend-card__status--away { color: var(--color-warning, #faad14); }
.friend-card__status--busy { color: var(--color-error, #f5222d); }
.friend-card__status--offline { color: var(--color-text-tertiary, #999); }
</style>
