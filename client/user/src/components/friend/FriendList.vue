<script setup lang="ts">
import type { Friend } from '@/types/friend'
import FriendCard from './FriendCard.vue'

defineProps<{
  friends: Friend[]
  loading: boolean
}>()

const emit = defineEmits<{
  click: [friend: Friend]
}>()
</script>

<template>
  <div class="friend-list">
    <div v-if="loading" class="friend-list__loading">加载中...</div>
    <template v-else-if="friends.length > 0">
      <FriendCard
        v-for="friend in friends"
        :key="friend.id"
        :friend="friend"
        @click="emit('click', friend)"
      />
    </template>
    <div v-else class="friend-list__empty">暂无好友</div>
  </div>
</template>

<style scoped>
.friend-list__loading,
.friend-list__empty {
  text-align: center;
  padding: 32px 16px;
  color: var(--color-text-tertiary, #999);
  font-size: 13px;
}
</style>
