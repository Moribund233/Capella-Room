<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { storeToRefs } from 'pinia'
import { useRoomStore } from '@/stores/room'
import { useDirectRoomStore } from '@/stores/directRoom'
import { RoomType, type Room, type DirectRoom } from '@/types/room'
import RoomTypeTabs from './RoomTypeTabs.vue'
import RoomCard from './RoomCard.vue'
import DirectRoomCard from './DirectRoomCard.vue'

const router = useRouter()
const route = useRoute()
const roomStore = useRoomStore()
const directRoomStore = useDirectRoomStore()
const { rooms, loading: roomsLoading } = storeToRefs(roomStore)
const { directRooms, loading: directRoomsLoading } = storeToRefs(directRoomStore)

const searchQuery = ref('')
const currentRoomType = ref(RoomType.Group)

const loading = computed(() => {
  return currentRoomType.value === RoomType.Group
    ? roomsLoading.value
    : directRoomsLoading.value
})

const filteredGroupRooms = computed<Room[]>(() => {
  if (!searchQuery.value.trim()) return rooms.value
  const q = searchQuery.value.toLowerCase()
  return rooms.value.filter(
    (r) =>
      r.name.toLowerCase().includes(q) ||
      (r.description && r.description.toLowerCase().includes(q)),
  )
})

const filteredDirectRooms = computed<DirectRoom[]>(() => {
  if (!searchQuery.value.trim()) return directRooms.value
  const q = searchQuery.value.toLowerCase()
  return directRooms.value.filter(
    (r) => r.target_user.username.toLowerCase().includes(q),
  )
})

const currentRoomId = computed(() => route.params.roomId as string | undefined)

function handleRoomClick(roomId: string) {
  router.push(`/room/${roomId}`)
}

// RoomTypeTabs 组件通过 v-model 自动处理类型切换
// 不需要额外的事件处理函数

onMounted(() => {
  // 加载群聊和私聊列表
  roomStore.fetchMyRooms()
  directRoomStore.fetchDirectRooms()
})
</script>

<template>
  <div class="room-list">
    <RoomTypeTabs v-model="currentRoomType" />

    <div class="room-list__search">
      <input
        v-model="searchQuery"
        type="text"
        class="room-list__search-input"
        :placeholder="currentRoomType === RoomType.Group ? '搜索聊天室...' : '搜索用户...'"
      />
    </div>

    <div class="room-list__items">
      <div v-if="loading" class="room-list__loading">
        <span>加载中...</span>
      </div>

      <template v-else-if="currentRoomType === RoomType.Group">
        <template v-if="filteredGroupRooms.length > 0">
          <RoomCard
            v-for="room in filteredGroupRooms"
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
      </template>

      <template v-else>
        <template v-if="filteredDirectRooms.length > 0">
          <DirectRoomCard
            v-for="room in filteredDirectRooms"
            :key="room.id"
            :room="room"
            :active="room.id === currentRoomId"
            @click="handleRoomClick"
          />
        </template>
        <div v-else class="room-list__empty">
          <p v-if="searchQuery">未找到匹配的用户</p>
          <p v-else>暂无私聊</p>
        </div>
      </template>
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
  border-bottom: 1px solid var(--color-border, #eee);
}

.room-list__search-input {
  width: 100%;
  padding: 6px 12px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm, 6px);
  font-size: var(--font-size-small, 13px);
  outline: none;
  box-sizing: border-box;
  transition: border-color var(--duration-fast, 0.15s);
}

.room-list__search-input:focus {
  border-color: var(--color-primary);
}

.room-list__search-input::placeholder {
  color: var(--color-text-tertiary);
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
