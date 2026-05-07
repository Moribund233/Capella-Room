<script setup lang="ts">
import { computed } from 'vue'
import type { DirectRoom } from '@/types/room'

const props = defineProps<{
  room: DirectRoom | null
}>()

const emit = defineEmits<{
  back: []
  viewProfile: [userId: string]
}>()

const targetUser = computed(() => props.room?.target_user)

const initial = computed(() =>
  targetUser.value?.username.charAt(0).toUpperCase() ?? '?',
)

function handleViewProfile() {
  if (targetUser.value) {
    emit('viewProfile', targetUser.value.id)
  }
}
</script>

<template>
  <div class="direct-chat-header">
    <div class="direct-chat-header__left">
      <button class="direct-chat-header__back-btn" @click="emit('back')">
        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="m15 18-6-6 6-6"/>
        </svg>
      </button>
      <div class="direct-chat-header__avatar" @click="handleViewProfile">
        <img
          v-if="targetUser?.avatar_url"
          :src="targetUser.avatar_url"
          :alt="targetUser.username"
          class="direct-chat-header__avatar-img"
        />
        <span v-else class="direct-chat-header__avatar-text">{{ initial }}</span>
      </div>
      <div class="direct-chat-header__info" @click="handleViewProfile">
        <h3 class="direct-chat-header__name">{{ targetUser?.username || '加载中...' }}</h3>
        <span class="direct-chat-header__status">私聊</span>
      </div>
    </div>
    <div class="direct-chat-header__right">
      <button class="direct-chat-header__action" @click="handleViewProfile" title="查看资料">
        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="8" r="5"/>
          <path d="M20 21a8 8 0 1 0-16 0"/>
        </svg>
      </button>
    </div>
  </div>
</template>

<style scoped>
.direct-chat-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  background: var(--color-background, #fff);
  border-bottom: 1px solid var(--color-border, #eee);
  flex-shrink: 0;
}

.direct-chat-header__left {
  display: flex;
  align-items: center;
  gap: 12px;
  flex: 1;
  min-width: 0;
}

.direct-chat-header__back-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: none;
  border-radius: var(--radius-md, 8px);
  background: transparent;
  color: var(--color-text-secondary, #666);
  cursor: pointer;
  transition: all var(--duration-fast, 0.15s);
  flex-shrink: 0;
}

.direct-chat-header__back-btn:hover {
  background: var(--color-background-soft, #f5f5f5);
  color: var(--color-text-primary, #333);
}

.direct-chat-header__avatar {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background: var(--color-primary, #2080f0);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  overflow: hidden;
  cursor: pointer;
  transition: opacity var(--duration-fast, 0.15s);
}

.direct-chat-header__avatar:hover {
  opacity: 0.8;
}

.direct-chat-header__avatar-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.direct-chat-header__avatar-text {
  color: var(--color-white);
  font-weight: 600;
  font-size: 16px;
}

.direct-chat-header__info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
  cursor: pointer;
}

.direct-chat-header__name {
  font-size: var(--font-size-body, 14px);
  font-weight: 600;
  color: var(--color-text-primary, #333);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.direct-chat-header__status {
  font-size: var(--font-size-small, 12px);
  color: var(--color-text-tertiary, #999);
}

.direct-chat-header__right {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.direct-chat-header__action {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border: none;
  border-radius: var(--radius-md, 8px);
  background: transparent;
  color: var(--color-text-secondary, #666);
  cursor: pointer;
  transition: all var(--duration-fast, 0.15s);
}

.direct-chat-header__action:hover {
  background: var(--color-background-soft, #f5f5f5);
  color: var(--color-primary, #2080f0);
}
</style>
