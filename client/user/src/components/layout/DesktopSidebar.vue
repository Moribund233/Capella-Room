<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { Plus, Search, MessageSquare, ChevronLeft, ChevronRight, Compass, Users, LogIn } from 'lucide-vue-next'
import { useRoomStore } from '@/stores/room'
import { useFriendStore } from '@/stores/friend'
import { useAuthStore } from '@/stores/auth'
import { ROUTE_PATHS } from '@/constants'
import ConnectionStatus from '@/components/chat/ConnectionStatus.vue'
import RoomCard from '@/components/room/RoomCard.vue'
import JoinByInviteModal from '@/components/room/JoinByInviteModal.vue'

const friendStore = useFriendStore()
const showJoinModal = ref(false)

const emit = defineEmits<{
  createRoom: []
}>()

const router = useRouter()
const route = useRoute()
const roomStore = useRoomStore()
const authStore = useAuthStore()

/** 是否在发现页面 */
const isDiscoverPage = computed(() => {
  return route.path === ROUTE_PATHS.DISCOVER
})

const searchQuery = ref('')
const collapsed = ref(false)

const filteredRooms = computed(() => {
  if (!searchQuery.value.trim()) return roomStore.rooms
  const q = searchQuery.value.toLowerCase()
  return roomStore.rooms.filter(
    (r) =>
      r.name.toLowerCase().includes(q) ||
      (r.description && r.description.toLowerCase().includes(q)),
  )
})

function handleRoomClick(roomId: string) {
  router.push(`/room/${roomId}`)
}

function toggleCollapse() {
  collapsed.value = !collapsed.value
}
</script>

<template>
  <aside
    class="desktop-sidebar"
    :class="{ 'desktop-sidebar--collapsed': collapsed }"
  >
    <!-- 头部 -->
    <header class="desktop-sidebar__header">
      <div class="desktop-sidebar__brand">
        <div class="desktop-sidebar__logo">
          <MessageSquare :size="collapsed ? 20 : 24" />
        </div>
        <h1 v-if="!collapsed" class="desktop-sidebar__title">Seredeli</h1>
      </div>
      <div class="desktop-sidebar__actions">
        <ConnectionStatus v-if="!collapsed" />
        <button
          class="desktop-sidebar__collapse-btn"
          @click="toggleCollapse"
          :title="collapsed ? '展开侧边栏' : '收起侧边栏'"
        >
          <ChevronLeft v-if="!collapsed" :size="18" />
          <ChevronRight v-else :size="18" />
        </button>
      </div>
    </header>

    <!-- 搜索栏 -->
    <div v-if="!collapsed" class="desktop-sidebar__search">
      <div class="desktop-sidebar__search-input-wrapper">
        <Search :size="16" class="desktop-sidebar__search-icon" />
        <input
          v-model="searchQuery"
          type="text"
          class="desktop-sidebar__search-input"
          placeholder="搜索聊天室..."
        />
      </div>
    </div>

    <!-- 发现入口 -->
    <div class="desktop-sidebar__discover">
      <button
        class="desktop-sidebar__discover-btn"
        :class="{
          'desktop-sidebar__discover-btn--collapsed': collapsed,
          'desktop-sidebar__discover-btn--active': isDiscoverPage
        }"
        @click="router.push(ROUTE_PATHS.DISCOVER)"
        :title="collapsed ? '发现' : ''"
      >
        <Compass :size="collapsed ? 20 : 18" />
        <span v-if="!collapsed">发现</span>
      </button>
    </div>

    <!-- 好友入口 -->
    <div class="desktop-sidebar__friends">
      <button
        class="desktop-sidebar__friends-btn"
        :class="{ 'desktop-sidebar__friends-btn--collapsed': collapsed }"
        @click="router.push(ROUTE_PATHS.FRIENDS)"
        :title="collapsed ? '好友' : ''"
      >
        <Users :size="collapsed ? 20 : 18" />
        <span v-if="!collapsed">好友</span>
        <span
          v-if="!collapsed && friendStore.unreadRequestCount > 0"
          class="desktop-sidebar__friends-badge"
        >
          {{ friendStore.unreadRequestCount > 99 ? '99+' : friendStore.unreadRequestCount }}
        </span>
        <span
          v-if="collapsed && friendStore.unreadRequestCount > 0"
          class="desktop-sidebar__friends-dot"
        />
      </button>
    </div>

    <!-- 创建按钮 -->
    <div class="desktop-sidebar__create">
      <button
        class="desktop-sidebar__create-btn"
        :class="{ 'desktop-sidebar__create-btn--collapsed': collapsed }"
        @click="emit('createRoom')"
        :title="collapsed ? '创建聊天室' : ''"
      >
        <Plus :size="collapsed ? 20 : 18" />
        <span v-if="!collapsed">创建聊天室</span>
      </button>
    </div>

    <!-- 聊天室列表 -->
    <nav class="desktop-sidebar__nav">
      <div v-if="roomStore.loading" class="desktop-sidebar__loading">
        <div class="desktop-sidebar__spinner" />
        <span v-if="!collapsed">加载中...</span>
      </div>

      <template v-else-if="filteredRooms.length > 0">
        <div v-if="!collapsed" class="desktop-sidebar__section-title">
          我的聊天室 ({{ filteredRooms.length }})
        </div>
        <div class="desktop-sidebar__room-list">
          <RoomCard
            v-for="room in filteredRooms"
            :key="room.id"
            :room="room"
            :active="false"
            :compact="collapsed"
            @click="handleRoomClick"
          />
        </div>
      </template>

      <div v-else class="desktop-sidebar__empty">
        <MessageSquare :size="collapsed ? 24 : 32" class="desktop-sidebar__empty-icon" />
        <span v-if="!collapsed">
          <p v-if="searchQuery">未找到匹配的聊天室</p>
          <p v-else>暂无聊天室</p>
        </span>
      </div>
    </nav>

    <!-- 加入房间按钮 -->
    <div class="desktop-sidebar__join">
      <button
        class="desktop-sidebar__join-btn"
        :class="{ 'desktop-sidebar__join-btn--collapsed': collapsed }"
        @click="showJoinModal = true"
        :title="collapsed ? '通过邀请码加入' : ''"
      >
        <LogIn :size="collapsed ? 18 : 16" />
        <span v-if="!collapsed">通过邀请码加入</span>
      </button>
    </div>

    <!-- 底部用户信息 -->
    <footer v-if="authStore.user" class="desktop-sidebar__footer">
      <div class="desktop-sidebar__user">
        <div class="desktop-sidebar__avatar">
          {{ authStore.user.username.charAt(0).toUpperCase() }}
        </div>
        <div v-if="!collapsed" class="desktop-sidebar__user-info">
          <span class="desktop-sidebar__username">{{ authStore.user.username }}</span>
          <span class="desktop-sidebar__user-status">在线</span>
        </div>
      </div>
    </footer>

    <!-- 通过邀请码加入弹窗 -->
    <JoinByInviteModal
      :show="showJoinModal"
      @close="showJoinModal = false"
      @joined="(roomId: string) => { showJoinModal = false; router.push(`/room/${roomId}`) }"
    />
  </aside>
</template>

<style scoped>
.desktop-sidebar {
  display: flex;
  flex-direction: column;
  background: var(--color-white);
  border-right: 1px solid var(--color-border);
  height: 100%;
  transition: width var(--duration-normal) var(--ease-default);
  overflow: hidden;
}

.desktop-sidebar:not(.desktop-sidebar--collapsed) {
  width: 280px;
}

.desktop-sidebar--collapsed {
  width: 64px;
}

/* 头部 */
.desktop-sidebar__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-md) var(--space-lg);
  border-bottom: 1px solid var(--color-divider);
  flex-shrink: 0;
  height: 56px;
}

.desktop-sidebar--collapsed .desktop-sidebar__header {
  padding: var(--space-md);
  justify-content: space-between;
  flex-direction: column;
  height: auto;
  gap: var(--space-sm);
}

.desktop-sidebar--collapsed .desktop-sidebar__brand {
  justify-content: center;
}

.desktop-sidebar--collapsed .desktop-sidebar__actions {
  justify-content: center;
}

.desktop-sidebar__brand {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  overflow: hidden;
}

.desktop-sidebar__logo {
  width: 32px;
  height: 32px;
  background: var(--color-primary);
  color: white;
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.desktop-sidebar__title {
  font-size: var(--font-size-h3);
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0;
  white-space: nowrap;
}

.desktop-sidebar__actions {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
}

.desktop-sidebar__collapse-btn {
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-tertiary);
  cursor: pointer;
  transition: all var(--duration-fast);
}

.desktop-sidebar__collapse-btn:hover {
  background: var(--color-background);
  color: var(--color-text-secondary);
}

/* 搜索栏 */
.desktop-sidebar__search {
  padding: var(--space-md) var(--space-lg);
  flex-shrink: 0;
}

.desktop-sidebar__search-input-wrapper {
  position: relative;
  display: flex;
  align-items: center;
}

.desktop-sidebar__search-icon {
  position: absolute;
  left: var(--space-md);
  color: var(--color-text-tertiary);
}

.desktop-sidebar__search-input {
  width: 100%;
  padding: var(--space-sm) var(--space-sm) var(--space-sm) 36px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  font-size: var(--font-size-small);
  background: var(--color-background-light);
  outline: none;
  transition: all var(--duration-fast);
}

.desktop-sidebar__search-input:focus {
  border-color: var(--color-primary);
  background: var(--color-white);
}

/* 发现按钮 */
.desktop-sidebar__discover {
  padding: 0 var(--space-lg) var(--space-sm);
  flex-shrink: 0;
}

.desktop-sidebar--collapsed .desktop-sidebar__discover {
  padding: 0 var(--space-sm) var(--space-sm);
}

.desktop-sidebar__discover-btn {
  width: 100%;
  padding: var(--space-sm) var(--space-md);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-white);
  color: var(--color-text-secondary);
  font-size: var(--font-size-small);
  font-weight: 500;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-sm);
  cursor: pointer;
  transition: all var(--duration-fast);
  white-space: nowrap;
}

.desktop-sidebar__discover-btn:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
  background: var(--color-primary-light);
}

.desktop-sidebar__discover-btn--active {
  background: var(--color-primary);
  border-color: var(--color-primary);
  color: white;
}

.desktop-sidebar__discover-btn--active:hover {
  background: var(--color-primary-hover);
  border-color: var(--color-primary-hover);
  color: white;
}

/* 好友入口 */
.desktop-sidebar__friends {
  padding: 0 var(--space-lg) var(--space-sm);
  flex-shrink: 0;
}

.desktop-sidebar--collapsed .desktop-sidebar__friends {
  padding: 0 var(--space-sm) var(--space-sm);
}

.desktop-sidebar__friends-btn {
  width: 100%;
  padding: var(--space-sm) var(--space-md);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-white);
  color: var(--color-text-secondary);
  font-size: var(--font-size-small);
  font-weight: 500;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-sm);
  cursor: pointer;
  transition: all var(--duration-fast);
  white-space: nowrap;
  position: relative;
}

.desktop-sidebar__friends-btn:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
  background: var(--color-primary-light);
}

.desktop-sidebar__friends-btn--collapsed {
  padding: var(--space-md);
  aspect-ratio: 1;
}

.desktop-sidebar__friends-badge {
  min-width: 18px;
  height: 18px;
  padding: 0 5px;
  border-radius: 9px;
  background: var(--color-error);
  color: white;
  font-size: 11px;
  font-weight: 600;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-left: auto;
}

.desktop-sidebar__friends-dot {
  position: absolute;
  top: 4px;
  right: 4px;
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: var(--color-error);
  border: 2px solid var(--color-white);
}

.desktop-sidebar__discover-btn--collapsed {
  padding: var(--space-md);
  aspect-ratio: 1;
}

/* 创建按钮 */
.desktop-sidebar__create {
  padding: 0 var(--space-lg) var(--space-md);
  flex-shrink: 0;
}

.desktop-sidebar--collapsed .desktop-sidebar__create {
  padding: 0 var(--space-sm) var(--space-md);
}

.desktop-sidebar__create-btn {
  width: 100%;
  padding: var(--space-sm) var(--space-md);
  border: 1px dashed var(--color-primary);
  border-radius: var(--radius-md);
  background: var(--color-primary-light);
  color: var(--color-primary);
  font-size: var(--font-size-small);
  font-weight: 500;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-sm);
  cursor: pointer;
  transition: all var(--duration-fast);
  white-space: nowrap;
}

.desktop-sidebar__create-btn:hover {
  background: var(--color-primary);
  color: white;
}

.desktop-sidebar__create-btn--collapsed {
  padding: var(--space-md);
  aspect-ratio: 1;
}

/* 聊天室列表 */
.desktop-sidebar__nav {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 0 var(--space-lg);
}

.desktop-sidebar--collapsed .desktop-sidebar__nav {
  padding: 0 var(--space-sm);
}

.desktop-sidebar__section-title {
  font-size: var(--font-size-small);
  color: var(--color-text-tertiary);
  font-weight: 500;
  padding: var(--space-md) 0 var(--space-sm);
  margin-bottom: var(--space-xs);
}

.desktop-sidebar__loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--space-3xl);
  gap: var(--space-md);
  color: var(--color-text-secondary);
}

.desktop-sidebar__spinner {
  width: 24px;
  height: 24px;
  border: 2px solid var(--color-divider);
  border-top-color: var(--color-primary);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.desktop-sidebar__empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--space-3xl);
  text-align: center;
  color: var(--color-text-tertiary);
}

.desktop-sidebar__empty-icon {
  margin-bottom: var(--space-md);
  opacity: 0.5;
}

.desktop-sidebar--collapsed .desktop-sidebar__empty-icon {
  margin-bottom: 0;
}

/* 通过邀请码加入按钮 */
.desktop-sidebar__join {
  padding: 0 var(--space-lg) var(--space-sm);
  flex-shrink: 0;
}

.desktop-sidebar--collapsed .desktop-sidebar__join {
  padding: 0 var(--space-sm) var(--space-sm);
}

.desktop-sidebar__join-btn {
  width: 100%;
  padding: var(--space-xs) var(--space-sm);
  border: none;
  border-radius: var(--radius-md);
  background: transparent;
  color: var(--color-text-tertiary);
  font-size: var(--font-size-small);
  font-weight: 400;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-xs);
  cursor: pointer;
  transition: all var(--duration-fast);
}

.desktop-sidebar__join-btn:hover {
  color: var(--color-primary);
  background: var(--color-primary-light);
}

.desktop-sidebar__join-btn--collapsed {
  padding: var(--space-sm);
  aspect-ratio: 1;
}

/* 底部用户信息 */
.desktop-sidebar__footer {
  padding: var(--space-md) var(--space-lg);
  border-top: 1px solid var(--color-divider);
  flex-shrink: 0;
}

.desktop-sidebar--collapsed .desktop-sidebar__footer {
  padding: var(--space-md);
}

.desktop-sidebar__user {
  display: flex;
  align-items: center;
  gap: var(--space-md);
  overflow: hidden;
}

.desktop-sidebar--collapsed .desktop-sidebar__user {
  justify-content: center;
}

.desktop-sidebar__avatar {
  width: 36px;
  height: 36px;
  background: var(--color-primary);
  color: white;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  font-size: var(--font-size-body);
  flex-shrink: 0;
}

.desktop-sidebar__user-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  overflow: hidden;
}

.desktop-sidebar__username {
  font-weight: 500;
  color: var(--color-text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.desktop-sidebar__user-status {
  font-size: var(--font-size-small);
  color: var(--color-success);
}
</style>
