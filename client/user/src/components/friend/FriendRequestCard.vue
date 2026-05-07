<script setup lang="ts">
import { computed } from 'vue'
import { User, X } from 'lucide-vue-next'
import type { FriendRequest } from '@/types/friend'

const props = defineProps<{
  request: FriendRequest
  /** 是否为收到的请求（显示接受/拒绝按钮） */
  isReceived: boolean
}>()

const emit = defineEmits<{
  accept: [requestId: string]
  reject: [requestId: string]
  cancel: [requestId: string]
}>()

const user = computed(() =>
  props.isReceived ? props.request.sender : props.request.receiver,
)

const statusLabel = computed(() => {
  if (props.request.status === 'accepted') return '已接受'
  if (props.request.status === 'rejected') return '已拒绝'
  return props.isReceived ? '等待你处理' : '等待对方接受'
})
</script>

<template>
  <div class="friend-request-card">
    <div class="friend-request-card__avatar">
      <img
        v-if="user.avatar_url"
        :src="user.avatar_url"
        :alt="user.username"
        class="friend-request-card__avatar-img"
      />
      <div v-else class="friend-request-card__avatar-placeholder">
        <User :size="20" />
      </div>
    </div>
    <div class="friend-request-card__body">
      <span class="friend-request-card__name">{{ user.username }}</span>
      <span v-if="request.message" class="friend-request-card__message">
        {{ request.message }}
      </span>
      <span class="friend-request-card__status">{{ statusLabel }}</span>
    </div>
    <div class="friend-request-card__actions">
      <template v-if="isReceived && request.status === 'pending'">
        <button
          class="friend-request-card__btn friend-request-card__btn--accept"
          @click="emit('accept', request.id)"
          title="接受"
        >
          接受
        </button>
        <button
          class="friend-request-card__btn friend-request-card__btn--reject"
          @click="emit('reject', request.id)"
          title="拒绝"
        >
          拒绝
        </button>
      </template>
      <button
        v-else-if="!isReceived && request.status === 'pending'"
        class="friend-request-card__btn friend-request-card__btn--cancel"
        @click="emit('cancel', request.id)"
        title="取消请求"
      >
        <X :size="16" />
      </button>
    </div>
  </div>
</template>

<style scoped>
.friend-request-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--color-border, #eee);
}

.friend-request-card__avatar {
  flex-shrink: 0;
}

.friend-request-card__avatar-img {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  object-fit: cover;
}

.friend-request-card__avatar-placeholder {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background: var(--color-background-soft, #f0f0f0);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-secondary, #666);
}

.friend-request-card__body {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.friend-request-card__name {
  font-size: 14px;
  font-weight: 500;
  color: var(--color-text, #333);
}

.friend-request-card__message {
  font-size: 12px;
  color: var(--color-text-secondary, #666);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.friend-request-card__status {
  font-size: 12px;
  color: var(--color-text-tertiary, #999);
}

.friend-request-card__actions {
  display: flex;
  gap: 6px;
  flex-shrink: 0;
}

.friend-request-card__btn {
  padding: 4px 12px;
  border: 1px solid var(--color-border, #d9d9d9);
  border-radius: 6px;
  font-size: 12px;
  cursor: pointer;
  transition: all var(--duration-fast, 0.15s);
  background: var(--color-white, #fff);
}

.friend-request-card__btn--accept {
  color: var(--color-primary, #2080f0);
  border-color: var(--color-primary, #2080f0);
}

.friend-request-card__btn--accept:hover {
  background: var(--color-primary, #2080f0);
  color: white;
}

.friend-request-card__btn--reject {
  color: var(--color-error, #f5222d);
  border-color: var(--color-error, #f5222d);
}

.friend-request-card__btn--reject:hover {
  background: var(--color-error, #f5222d);
  color: white;
}

.friend-request-card__btn--cancel {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  padding: 0;
  color: var(--color-text-tertiary, #999);
}

.friend-request-card__btn--cancel:hover {
  color: var(--color-error, #f5222d);
  border-color: var(--color-error, #f5222d);
}
</style>
