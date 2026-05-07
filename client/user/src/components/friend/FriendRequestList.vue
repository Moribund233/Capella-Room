<script setup lang="ts">
import type { FriendRequest } from '@/types/friend'
import FriendRequestCard from './FriendRequestCard.vue'

defineProps<{
  requests: FriendRequest[]
  loading: boolean
  isReceived: boolean
}>()

const emit = defineEmits<{
  accept: [requestId: string]
  reject: [requestId: string]
  cancel: [requestId: string]
}>()
</script>

<template>
  <div class="friend-request-list">
    <div v-if="loading" class="friend-request-list__loading">加载中...</div>
    <template v-else-if="requests.length > 0">
      <FriendRequestCard
        v-for="request in requests"
        :key="request.id"
        :request="request"
        :is-received="isReceived"
        @accept="emit('accept', $event)"
        @reject="emit('reject', $event)"
        @cancel="emit('cancel', $event)"
      />
    </template>
    <div v-else class="friend-request-list__empty">
      {{ isReceived ? '暂无好友请求' : '暂无已发送的请求' }}
    </div>
  </div>
</template>

<style scoped>
.friend-request-list__loading,
.friend-request-list__empty {
  text-align: center;
  padding: 32px 16px;
  color: var(--color-text-tertiary, #999);
  font-size: 13px;
}
</style>
