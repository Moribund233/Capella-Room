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
import { Refresh } from '@element-plus/icons-vue'
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
const filterType = ref<'all' | 'groups' | 'dms'>('all')

// Pull-to-refresh state
const isRefreshing = ref(false)
const pullDistance = ref(0)
const isPulling = ref(false)
const touchStartY = ref(0)
const listRef = ref<HTMLElement | null>(null)
const pullThreshold = 80

function handleTouchStart(e: TouchEvent) {
  if (!isMobile.value || isRefreshing.value) return
  const list = listRef.value
  if (!list || list.scrollTop > 0) return
  
  const touch = e.touches[0]
  if (!touch) return
  isPulling.value = true
  touchStartY.value = touch.clientY
  pullDistance.value = 0
}

function handleTouchMove(e: TouchEvent) {
  if (!isPulling.value || !isMobile.value || isRefreshing.value) return
  
  const touch = e.touches[0]
  if (!touch) return
  
  const list = listRef.value
  if (!list || list.scrollTop > 0) {
    isPulling.value = false
    pullDistance.value = 0
    return
  }
  
  const deltaY = touch.clientY - touchStartY.value
  
  // Only allow pulling down
  if (deltaY > 0) {
    pullDistance.value = Math.min(deltaY * 0.5, pullThreshold * 1.5)
    e.preventDefault()
  }
}

function handleTouchEnd() {
  if (!isPulling.value || !isMobile.value) return
  
  if (pullDistance.value >= pullThreshold) {
    refreshRooms()
  }
  
  isPulling.value = false
  pullDistance.value = 0
}

async function refreshRooms() {
  if (isRefreshing.value) return
  
  isRefreshing.value = true
  try {
    await Promise.all([
      roomStore.fetchMyRooms(),
      directRoomStore.fetchDirectRooms(),
    ])
  } finally {
    isRefreshing.value = false
  }
}

const filteredRooms = computed(() => {
  let result = rooms.value
  
  // Apply search filter
  if (searchQuery.value.trim()) {
    const q = searchQuery.value.toLowerCase()
    result = result.filter((r) => r.name.toLowerCase().includes(q))
  }
  
  // Apply type filter
  if (filterType.value === 'dms') {
    return [] // Hide groups when filtering for DMs
  }
  
  return result
})

const sortedRooms = computed(() => {
  const order = roomStore.roomOrder
  let filtered = filteredRooms.value

  // 如果有自定义排序，使用自定义排序
  if (order.length > 0) {
    const map = new Map(filtered.map((r) => [r.id, r]))
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
  }

  // 否则使用默认排序：未读优先 → 私有房间排后 → 最后更新时间
  return [...filtered].sort((a, b) => {
    // 未读消息优先
    const aUnread = a.unread_count || 0
    const bUnread = b.unread_count || 0
    if (aUnread > 0 && bUnread === 0) return -1
    if (aUnread === 0 && bUnread > 0) return 1

    // 私有房间排后面
    if (a.is_private !== b.is_private) {
      return a.is_private ? 1 : -1
    }

    // 最后更新时间
    return new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime()
  })
})

const filteredDirectRooms = computed(() => {
  let result = directRooms.value
  
  // Apply search filter
  if (searchQuery.value.trim()) {
    const q = searchQuery.value.toLowerCase()
    result = result.filter((r) =>
      r.target_user.username.toLowerCase().includes(q),
    )
  }
  
  // Apply type filter
  if (filterType.value === 'groups') {
    return [] // Hide DMs when filtering for groups
  }
  
  return result
})

const channelItems = computed<ChannelItemData[]>(() =>
  sortedRooms.value.map((r) => ({
    id: r.id,
    name: r.name,
    type: 'channel' as const,
    unreadCount: currentRoom.value?.id === r.id ? 0 : (r.unread_count || 0),
    isActive: currentRoom.value?.id === r.id,
    isPrivate: r.is_private,
    lastMessage: r.last_message?.content,
    memberCount: r.member_count,
  })),
)

const dmItems = computed<ChannelItemData[]>(() => {
  let filtered = filteredDirectRooms.value

  // 排序：未读优先 → 最后更新时间
  return [...filtered]
    .sort((a, b) => {
      // 未读消息优先
      const aUnread = a.unread_count || 0
      const bUnread = b.unread_count || 0
      if (aUnread > 0 && bUnread === 0) return -1
      if (aUnread === 0 && bUnread > 0) return 1

      // 最后更新时间
      return new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
    })
    .map((r) => ({
      id: r.id,
      name: r.target_user.username,
      type: 'dm' as const,
      unreadCount: currentRoom.value?.id === r.id ? 0 : (r.unread_count || 0),
      isActive: currentRoom.value?.id === r.id,
      userStatus: 'offline',
      lastMessage: r.last_message?.content,
    }))
})

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
    room_type: 'direct',
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
      :filter="filterType"
      @update:filter="filterType = $event"
    />

    <SidebarUserSection
      :is-mobile="isMobile"
      @open-profile="goToProfile"
      @open-settings="goToProfile"
      @logout="handleLogout"
    />

    <div 
      ref="listRef"
      class="room-sidebar__list"
      @touchstart="handleTouchStart"
      @touchmove="handleTouchMove"
      @touchend="handleTouchEnd"
    >
      <!-- Pull-to-refresh indicator -->
      <div 
        v-if="isMobile && (isRefreshing || pullDistance > 0)"
        class="room-sidebar__refresh"
        :style="{ height: `${isRefreshing ? 50 : pullDistance}px` }"
      >
        <div 
          class="room-sidebar__refresh-spinner"
          :class="{ 'room-sidebar__refresh-spinner--active': isRefreshing }"
        >
          <el-icon :size="20"><Refresh /></el-icon>
        </div>
      </div>

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

  &__refresh {
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
    transition: height 0.2s ease;
  }

  &__refresh-spinner {
    color: var(--accent);
    transition: transform 0.2s ease;

    &--active {
      animation: spin 1s linear infinite;
    }
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

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
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
