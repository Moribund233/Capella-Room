<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { storeToRefs } from 'pinia'
import { useRoomStore } from '@/stores/room'
import { useDirectRoomStore } from '@/stores/directRoom'
import { useAuthStore } from '@/stores/auth'
import { useMessageStore } from '@/stores/message'
import { useWebSocketStore } from '@/stores/websocket'
import { useResponsive } from '@/composables/useResponsive'
import { SidebarHeader, SidebarCategory, SidebarUserSection } from '@/components/sidebar'
import type { ChannelItemData } from '@/components/sidebar/SidebarChannelItem.vue'
import type { Room } from '@/types/room'
import type { DirectRoom } from '@/types/room'

const { t } = useI18n()
const router = useRouter()
const roomStore = useRoomStore()
const directRoomStore = useDirectRoomStore()
const authStore = useAuthStore()
const messageStore = useMessageStore()
const wsStore = useWebSocketStore()
const { isMobile, sidebarCollapsed } = useResponsive()

const { rooms, currentRoom } = storeToRefs(roomStore)
const { directRooms } = storeToRefs(directRoomStore)
const searchQuery = ref('')

const filteredRooms = computed(() => {
  if (!searchQuery.value.trim()) return rooms.value
  const q = searchQuery.value.toLowerCase()
  return rooms.value.filter((r) => r.name.toLowerCase().includes(q))
})

const sortedRooms = computed(() => {
  const order = roomStore.roomOrder
  if (order.length === 0) return filteredRooms.value
  const map = new Map(filteredRooms.value.map((r) => [r.id, r]))
  const sorted: Room[] = []
  for (const id of order) {
    const room = map.get(id)
    if (room) {
      sorted.push(room)
      map.delete(id)
    }
  }
  for (const room of map.values()) sorted.push(room)
  return sorted
})

const filteredDirectRooms = computed(() => {
  if (!searchQuery.value.trim()) return directRooms.value
  const q = searchQuery.value.toLowerCase()
  return directRooms.value.filter((r) =>
    r.target_user.username.toLowerCase().includes(q),
  )
})

const channelItems = computed<ChannelItemData[]>(() =>
  sortedRooms.value.map((r) => ({
    id: r.id,
    name: r.name,
    type: 'channel' as const,
    unreadCount: currentRoom.value?.id === r.id ? 0 : (r.unread_count || 0),
    isActive: currentRoom.value?.id === r.id,
    isPrivate: r.is_private,
  })),
)

const dmItems = computed<ChannelItemData[]>(() =>
  filteredDirectRooms.value.map((r) => ({
    id: r.id,
    name: r.target_user.username,
    type: 'dm' as const,
    unreadCount: currentRoom.value?.id === r.id ? 0 : (r.unread_count || 0),
    isActive: currentRoom.value?.id === r.id,
    userStatus: 'offline',
  })),
)

function selectRoom(room: Room) {
  if (currentRoom.value?.id === room.id) return
  roomStore.currentRoom = room
  directRoomStore.setCurrentDirectRoom(null)
  messageStore.switchRoom(room.id)
  messageStore.fetchMessages(room.id).then(() => {
    messageStore.sendReadReceiptForRoom()
  })
  roomStore.fetchMembers(room.id)
  roomStore.clearUnreadCount(room.id)
  if (wsStore.isConnected) {
    wsStore.send('JoinRoom', { room_id: room.id })
  }
  if (isMobile.value) sidebarCollapsed.value = true
}

function selectDirectRoom(dm: DirectRoom) {
  if (currentRoom.value?.id === dm.id) return
  const roomObj: Room = {
    id: dm.id,
    name: dm.target_user.username,
    description: null,
    owner: { id: '', username: '', avatar_url: null },
    is_private: true,
    max_members: 2,
    member_count: 2,
    created_at: dm.created_at,
    updated_at: dm.created_at,
    unread_count: dm.unread_count,
    last_message: dm.last_message,
  }
  roomStore.currentRoom = roomObj
  directRoomStore.setCurrentDirectRoom(dm)
  messageStore.switchRoom(dm.id)
  messageStore.fetchMessages(dm.id).then(() => {
    messageStore.sendReadReceiptForRoom()
  })
  roomStore.clearUnreadCount(dm.id)
  if (wsStore.isConnected) {
    wsStore.send('JoinRoom', { room_id: dm.id })
  }
  if (isMobile.value) sidebarCollapsed.value = true
}

function handleChannelReorder(items: ChannelItemData[]) {
  roomStore.setRoomOrder(items.map((i) => i.id))
}

function handleChannelSelect(item: ChannelItemData) {
  const room = rooms.value.find((r) => r.id === item.id)
  if (room) selectRoom(room)
}

function handleDMSelect(item: ChannelItemData) {
  const dm = directRooms.value.find((r) => r.id === item.id)
  if (dm) selectDirectRoom(dm)
}

function closeRoom() {
  roomStore.currentRoom = null
  directRoomStore.setCurrentDirectRoom(null)
  messageStore.switchRoom('')
}

function goToProfile() {
  router.push('/profile')
}

function handleLogout() {
  authStore.logout()
  router.push('/login')
}

onMounted(() => {
  directRoomStore.fetchDirectRooms()
})
</script>

<template>
  <aside class="room-sidebar">
    <SidebarHeader
      v-model="searchQuery"
    />

    <SidebarUserSection
      :is-mobile="isMobile"
      @open-profile="goToProfile"
      @open-settings="goToProfile"
      @logout="handleLogout"
    />

    <div class="room-sidebar__list">
      <SidebarCategory
        v-if="channelItems.length > 0 || roomStore.loading"
        :name="t('chat.channels')"
        :items="channelItems"
        draggable
        @select="handleChannelSelect"
        @close="closeRoom"
        @update:items="handleChannelReorder"
      />

      <SidebarCategory
        v-if="dmItems.length > 0"
        :name="t('chat.directMessages') || 'Direct Messages'"
        :items="dmItems"
        draggable
        @select="handleDMSelect"
        @close="closeRoom"
      />

      <div v-if="roomStore.loading" class="room-sidebar__loading">
        <div v-for="i in 4" :key="i" class="room-sidebar__skeleton">
          <el-skeleton :rows="1" animated />
        </div>
      </div>

      <div
        v-if="!roomStore.loading && channelItems.length === 0 && dmItems.length === 0"
        class="room-sidebar__empty"
      >
        <el-empty :description="t('chat.noRooms')" :image-size="48" />
      </div>
    </div>
  </aside>
</template>

<style scoped lang="scss">
.room-sidebar {
  width: var(--sidebar-w);
  min-width: var(--sidebar-w);
  height: 100%;
  background: var(--sidebar-bg);
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--border);

  &__list {
    flex: 1;
    overflow-y: auto;
    padding: 4px 4px 8px;
  }

  &__loading {
    padding: 8px;
  }

  &__skeleton {
    padding: 8px;
  }

  &__empty {
    padding: 24px 8px;
  }
}

// 桌面端：用户区放底部
@media (min-width: 769px) {
  .room-sidebar {
    .room-sidebar__list {
      order: 2;
    }
    :deep(.user-section) {
      order: 3;
    }
  }
}
</style>
