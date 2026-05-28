<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { storeToRefs } from 'pinia'
import { useRoomStore } from '@/stores/room'
import { useAuthStore } from '@/stores/auth'
import { useMessageStore } from '@/stores/message'
import type { Room } from '@/types/room'
import {
  Search,
  ArrowDown,
} from '@element-plus/icons-vue'

const { t } = useI18n()
const router = useRouter()
const roomStore = useRoomStore()
const authStore = useAuthStore()
const messageStore = useMessageStore()

const { rooms, currentRoom } = storeToRefs(roomStore)
const user = computed(() => authStore.user)
const searchQuery = ref('')

const filteredRooms = computed(() => {
  if (!searchQuery.value.trim()) return rooms.value
  const q = searchQuery.value.toLowerCase()
  return rooms.value.filter((r) => r.name.toLowerCase().includes(q))
})

function selectRoom(room: Room) {
  if (currentRoom.value?.id === room.id) return
  roomStore.currentRoom = room
  messageStore.switchRoom(room.id)
  messageStore.fetchMessages(room.id)
  roomStore.fetchMembers(room.id)
  roomStore.clearUnreadCount(room.id)
  closeMobile()
}

function goToProfile() {
  router.push('/profile')
}

const emit = defineEmits<{
  closeMobile: []
}>()

function closeMobile() {
  emit('closeMobile')
}
</script>

<template>
  <aside class="room-sidebar">
    <!-- 侧边栏头部 -->
    <div class="sidebar-header" @click="goToProfile">
      <span>{{ t('common.appName') }}</span>
      <el-icon><ArrowDown /></el-icon>
    </div>

    <!-- 搜索 -->
    <div class="sidebar-search">
      <el-input
        v-model="searchQuery"
        :placeholder="t('chat.findRoom')"
        size="small"
        clearable
      >
        <template #prefix>
          <el-icon><Search /></el-icon>
        </template>
      </el-input>
    </div>

    <!-- 房间列表 -->
    <div class="channels">
      <template v-if="filteredRooms.length === 0 && !roomStore.loading">
        <div class="channel-empty">
          <el-empty :description="t('chat.noRooms')" :image-size="64" />
        </div>
      </template>

      <!-- 分组标题 -->
      <div class="channel-category">
        <span>{{ t('chat.channels') }}</span>
      </div>

      <!-- 房间项 -->
      <div
        v-for="room in filteredRooms"
        :key="room.id"
        class="channel"
        :class="{ active: currentRoom?.id === room.id }"
        @click="selectRoom(room)"
      >
        <span class="channel-hash">#</span>
        <span class="channel-name">{{ room.name }}</span>
        <span
          v-if="room.unread_count && room.unread_count > 0 && currentRoom?.id !== room.id"
          class="channel-badge"
        >
          {{ room.unread_count > 99 ? '99+' : room.unread_count }}
        </span>
      </div>

      <!-- 加载状态 -->
      <div v-if="roomStore.loading" class="channel-loading">
        <div v-for="i in 4" :key="i" class="channel-skeleton">
          <span class="skeleton-hash">#</span>
          <span class="skeleton-name" :style="{ width: 60 + i * 20 + 'px' }" />
        </div>
      </div>
    </div>

    <!-- 用户区域 -->
    <div class="user-section" @click="goToProfile">
      <div class="user-avatar">
        {{ user?.username?.charAt(0).toUpperCase() || '?' }}
        <span class="status-dot" />
      </div>
      <div class="user-info">
        <div class="name">{{ user?.username || 'User' }}</div>
        <div class="status">{{ t('chat.online') }}</div>
      </div>
      <div class="user-controls">
        <button title="Mute" @click.stop>
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/><path d="M19.07 4.93a10 10 0 010 14.14M15.54 8.46a5 5 0 010 7.07"/></svg>
        </button>
        <button title="Settings" @click.stop="goToProfile">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-2 2 2 2 0 01-2-2v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 01-2-2 2 2 0 012-2h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 010-2.83 2 2 0 012.83 0l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 012-2 2 2 0 012 2v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 0 2 2 0 010 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 012 2 2 2 0 01-2 2h-.09a1.65 1.65 0 00-1.51 1z"/></svg>
        </button>
      </div>
    </div>
  </aside>
</template>

<style scoped lang="scss">
.room-sidebar {
  width: var(--sidebar-w);
  min-width: var(--sidebar-w);
  height: 100vh;
  background: var(--sidebar-bg);
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--border);

  :deep(.el-input) {
    .el-input__wrapper {
      background: var(--bg);
      box-shadow: none;
      border-radius: var(--radius);
      padding: 4px 8px;

      &.is-focus {
        outline: 1px solid var(--accent);
        box-shadow: none;
      }
    }

    .el-input__inner {
      font-size: 13px;
      color: var(--fg);

      &::placeholder {
        color: var(--muted);
      }
    }

    .el-input__prefix {
      color: var(--muted);
    }
  }
}

.sidebar-header {
  height: var(--header-h);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  border-bottom: 1px solid var(--border);
  font-family: var(--font-display);
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;
  flex-shrink: 0;

  .el-icon {
    color: var(--muted);
    font-size: 16px;
  }

  &:hover {
    background: var(--message-hover);
  }
}

.sidebar-search {
  padding: 12px;
  flex-shrink: 0;
}

.channels {
  flex: 1;
  overflow-y: auto;
  padding: 4px 8px;
}

.channel-category {
  padding: 16px 16px 4px;
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--muted);
  display: flex;
  align-items: center;
  justify-content: space-between;
  cursor: pointer;

  &:hover {
    color: var(--fg);
  }
}

.channel {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 12px;
  border-radius: var(--radius);
  font-size: 15px;
  color: var(--muted);
  cursor: pointer;
  transition: background 0.1s;

  &:hover {
    background: var(--message-hover);
    color: var(--fg);
  }

  &.active {
    background: var(--accent-soft);
    color: var(--fg);

    .channel-hash {
      color: var(--accent);
    }
  }
}

.channel-hash {
  color: var(--muted);
  opacity: 0.6;
  font-weight: 300;
}

.channel-name {
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.channel-badge {
  background: var(--accent);
  color: #fff;
  font-size: 11px;
  font-weight: 600;
  padding: 1px 7px;
  border-radius: var(--radius-full);
  min-width: 20px;
  text-align: center;
}

.channel-empty {
  padding: 24px 8px;
}

.channel-loading {
  padding: 4px 8px;
}

.channel-skeleton {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 12px;

  .skeleton-hash {
    color: var(--muted);
    opacity: 0.3;
  }

  .skeleton-name {
    height: 12px;
    border-radius: 4px;
    background: var(--message-hover);
    animation: pulse 1.5s ease-in-out infinite;
  }
}

@keyframes pulse {
  0%, 100% { opacity: 0.4; }
  50% { opacity: 0.8; }
}

.user-section {
  border-top: 1px solid var(--border);
  padding: 10px 12px;
  display: flex;
  align-items: center;
  gap: 10px;
  cursor: pointer;
  flex-shrink: 0;

  &:hover {
    background: var(--message-hover);
  }
}

.user-avatar {
  width: 34px;
  height: 34px;
  border-radius: 50%;
  background: linear-gradient(135deg, var(--accent), var(--accent-pink));
  display: grid;
  place-items: center;
  font-size: 14px;
  font-weight: 600;
  color: #fff;
  position: relative;
  flex-shrink: 0;

  .status-dot {
    position: absolute;
    bottom: 0;
    right: 0;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--accent-green);
    border: 2px solid var(--sidebar-bg);
  }
}

.user-info {
  flex: 1;
  min-width: 0;

  .name {
    font-size: 14px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .status {
    font-size: 12px;
    color: var(--muted);
  }
}

.user-controls {
  display: flex;
  gap: 6px;

  button {
    background: none;
    border: none;
    color: var(--muted);
    cursor: pointer;
    padding: 4px;
    border-radius: 4px;
    display: grid;
    place-items: center;

    &:hover {
      color: var(--fg);
      background: var(--message-hover);
    }

    svg {
      width: 18px;
      height: 18px;
    }
  }
}
</style>
