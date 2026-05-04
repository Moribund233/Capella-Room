<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useResponsive } from '@/composables/useResponsive'
import { useRoomStore } from '@/stores/room'
import ConnectionStatus from '@/components/chat/ConnectionStatus.vue'
import RoomList from '@/components/room/RoomList.vue'
import CreateRoomModal from '@/components/room/CreateRoomModal.vue'
import NavBar from '@/components/nav/NavBar.vue'

const { isMobile, isTablet, isDesktop, sidebarCollapsed, toggleSidebar } = useResponsive()
const roomStore = useRoomStore()

const showCreateModal = ref(false)

function handleRoomCreated(roomId: string) {
  showCreateModal.value = false
}

onMounted(() => {
  roomStore.fetchMyRooms()
})
</script>

<template>
  <div class="main-layout">
    <NavBar />
    <aside
      class="main-layout__sidebar"
      :class="{
        'main-layout__sidebar--collapsed': !isDesktop && sidebarCollapsed,
        'main-layout__sidebar--mobile': isMobile,
        'main-layout__sidebar--tablet': isTablet,
        'main-layout__sidebar--desktop': isDesktop,
      }"
    >
      <div class="main-layout__sidebar-header">
        <h2 class="main-layout__logo">Seredeli</h2>
        <div class="main-layout__sidebar-actions">
          <ConnectionStatus />
          <button
            v-if="isTablet"
            class="main-layout__toggle-btn"
            @click="toggleSidebar"
            aria-label="切换侧边栏"
          >
            <span v-if="sidebarCollapsed">☰</span>
            <span v-else>✕</span>
          </button>
        </div>
      </div>

      <div class="main-layout__toolbar">
        <button class="main-layout__create-btn" @click="showCreateModal = true">
          ＋ 创建聊天室
        </button>
      </div>

      <nav class="main-layout__nav">
        <RoomList />
      </nav>
    </aside>

    <main class="main-layout__content" :class="{ 'main-layout__content--mobile': isMobile }">
      <router-view />
    </main>

    <CreateRoomModal
      :show="showCreateModal"
      @close="showCreateModal = false"
      @created="handleRoomCreated"
    />
  </div>
</template>

<style scoped>
.main-layout {
  display: flex;
  width: 100vw;
  height: 100vh;
  overflow: hidden;
}

.main-layout__sidebar {
  display: flex;
  flex-direction: column;
  background: var(--color-white);
  border-right: 1px solid var(--color-border);
  transition: width var(--duration-normal) var(--ease-default);
  overflow: hidden;
}

.main-layout__sidebar--desktop {
  width: 300px;
  flex-shrink: 0;
}

.main-layout__sidebar--tablet {
  width: 260px;
  flex-shrink: 0;
}

.main-layout__sidebar--tablet.main-layout__sidebar--collapsed {
  width: 64px;
}

.main-layout__sidebar--mobile {
  position: fixed;
  top: 0;
  left: 0;
  z-index: 100;
  width: 100vw;
  height: 100vh;
  transform: translateX(-100%);
}

.main-layout__sidebar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-md) var(--space-lg);
  border-bottom: 1px solid var(--color-divider);
  height: 52px;
  flex-shrink: 0;
}

.main-layout__sidebar-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.main-layout__logo {
  font-size: var(--font-size-h3);
  font-weight: 600;
  color: var(--color-primary);
  white-space: nowrap;
  margin: 0;
}

.main-layout__toggle-btn {
  background: none;
  border: none;
  cursor: pointer;
  font-size: var(--font-size-h3);
  color: var(--color-text-secondary);
  padding: var(--space-xs);
  border-radius: var(--radius-sm);
}

.main-layout__toggle-btn:hover {
  background: var(--color-background);
}

.main-layout__toolbar {
  padding: 8px 12px;
  flex-shrink: 0;
}

.main-layout__create-btn {
  width: 100%;
  padding: 8px 16px;
  border: 1px dashed var(--color-primary, #2080f0);
  border-radius: var(--radius-sm, 6px);
  background: transparent;
  color: var(--color-primary, #2080f0);
  font-size: var(--font-size-small, 13px);
  cursor: pointer;
  transition: all var(--duration-fast, 0.15s);
}

.main-layout__create-btn:hover {
  background: var(--color-primary, #2080f0);
  color: #fff;
}

.main-layout__nav {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
}

.main-layout__content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--color-background);
}

.main-layout__content--mobile {
  padding-bottom: 56px;
}
</style>
