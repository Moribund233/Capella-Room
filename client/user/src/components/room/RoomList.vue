<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { storeToRefs } from 'pinia'
import { useRoomStore } from '@/stores/room'
import RoomCard from './RoomCard.vue'

const router = useRouter()
const route = useRoute()
const roomStore = useRoomStore()
const { rooms, loading } = storeToRefs(roomStore)

const searchQuery = ref('')

const filteredRooms = computed(() => {
  if (!searchQuery.value.trim()) return rooms.value
  const q = searchQuery.value.toLowerCase()
  return rooms.value.filter(
    (r) =>
      r.name.toLowerCase().includes(q) ||
      (r.description && r.description.toLowerCase().includes(q)),
  )
})

const currentRoomId = computed(() => route.params.roomId as string | undefined)

function handleRoomClick(roomId: string) {
  router.push(`/room/${roomId}`)
}
</script>

<template>
  <div class="room-list">
    <div class="room-list__search">
      <input
        v-model="searchQuery"
        type="text"
        class="room-list__search-input"
        placeholder="搜索聊天室..."
      />
    </div>

    <div class="room-list__items">
      <div v-if="loading" class="room-list__loading">
        <span>加载中...</span>
      </div>

      <template v-else-if="filteredRooms.length > 0">
        <RoomCard
          v-for="room in filteredRooms"
          :key="room.id"
          :room="room"
          :active="room.id === currentRoomId"
          @click="handleRoomClick"
        />
      </template>

      <div v-else class="room-list__empty">
        <p v-if="searchQuery">未找到匹配的聊天室</p>
        <p v-else>暂无聊天室</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.room-list {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.room-list__search {
  padding: 8px 12px;
  flex-shrink: 0;
}

.room-list__search-input {
  width: 100%;
  padding: 6px 12px;
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: var(--radius-sm, 6px);
  font-size: var(--font-size-small, 13px);
  outline: none;
  box-sizing: border-box;
  transition: border-color var(--duration-fast, 0.15s);
}

.room-list__search-input:focus {
  border-color: var(--color-primary, #2080f0);
}

.room-list__search-input::placeholder {
  color: var(--color-text-tertiary, #999);
}

.room-list__items {
  flex: 1;
  overflow-y: auto;
}

.room-list__loading {
  display: flex;
  justify-content: center;
  padding: 24px;
  color: var(--color-text-tertiary, #999);
  font-size: var(--font-size-small, 13px);
}

.room-list__empty {
  display: flex;
  justify-content: center;
  padding: 32px 16px;
  color: var(--color-text-tertiary, #999);
  font-size: var(--font-size-small, 13px);
}
</style>
