<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { useRouter } from 'vue-router'
import { X, Plus, Search, MessageSquare } from 'lucide-vue-next'
import { useRoomStore } from '@/stores/room'
import { useAuthStore } from '@/stores/auth'
import ConnectionStatus from '@/components/chat/ConnectionStatus.vue'
import RoomCard from '@/components/room/RoomCard.vue'

const props = defineProps<{
  show: boolean
}>()

const emit = defineEmits<{
  close: []
  createRoom: []
}>()

const router = useRouter()
const roomStore = useRoomStore()
const authStore = useAuthStore()

const searchQuery = ref('')
const sidebarRef = ref<HTMLElement>()
const touchStartX = ref(0)
const touchCurrentX = ref(0)
const isSwiping = ref(false)

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
  emit('close')
}

function handleBackdropClick(e: MouseEvent) {
  if (e.target === e.currentTarget) {
    emit('close')
  }
}

// 触摸滑动关闭
function onTouchStart(e: TouchEvent) {
  const touch = e.touches[0]
  if (!touch) return
  touchStartX.value = touch.clientX
  isSwiping.value = true
}

function onTouchMove(e: TouchEvent) {
  if (!isSwiping.value) return
  const touch = e.touches[0]
  if (!touch) return
  touchCurrentX.value = touch.clientX
  const diff = touchCurrentX.value - touchStartX.value
  if (diff < 0 && sidebarRef.value) {
    sidebarRef.value.style.transform = `translateX(${diff}px)`
  }
}

function onTouchEnd() {
  if (!isSwiping.value) return
  isSwiping.value = false
  const diff = touchCurrentX.value - touchStartX.value
  if (diff < -80) {
    emit('close')
  } else if (sidebarRef.value) {
    sidebarRef.value.style.transform = ''
  }
}

// 防止背景滚动
onMounted(() => {
  if (props.show) {
    document.body.style.overflow = 'hidden'
  }
})

onUnmounted(() => {
  document.body.style.overflow = ''
})

watch(() => props.show, (show) => {
  if (show) {
    document.body.style.overflow = 'hidden'
    nextTick(() => {
      if (sidebarRef.value) {
        sidebarRef.value.style.transform = ''
      }
    })
  } else {
    document.body.style.overflow = ''
  }
})
</script>

<template>
  <Transition name="mobile-sidebar">
    <div
      v-if="show"
      class="mobile-sidebar"
      @click="handleBackdropClick"
    >
      <!-- 遮罩层 -->
      <div class="mobile-sidebar__backdrop" />

      <!-- 侧边栏内容 -->
      <aside
        ref="sidebarRef"
        class="mobile-sidebar__content"
        @touchstart="onTouchStart"
        @touchmove="onTouchMove"
        @touchend="onTouchEnd"
      >
        <!-- 头部 -->
        <header class="mobile-sidebar__header">
          <div class="mobile-sidebar__brand">
            <div class="mobile-sidebar__logo">
              <MessageSquare :size="24" />
            </div>
            <h1 class="mobile-sidebar__title">Seredeli</h1>
          </div>
          <div class="mobile-sidebar__actions">
            <ConnectionStatus />
            <button
              class="mobile-sidebar__close"
              @click="emit('close')"
              aria-label="关闭侧边栏"
            >
              <X :size="24" />
            </button>
          </div>
        </header>

        <!-- 搜索栏 -->
        <div class="mobile-sidebar__search">
          <div class="mobile-sidebar__search-input-wrapper">
            <Search :size="18" class="mobile-sidebar__search-icon" />
            <input
              v-model="searchQuery"
              type="text"
              class="mobile-sidebar__search-input"
              placeholder="搜索聊天室..."
            />
          </div>
        </div>

        <!-- 创建按钮 -->
        <div class="mobile-sidebar__create">
          <button
            class="mobile-sidebar__create-btn"
            @click="emit('createRoom')"
          >
            <Plus :size="20" />
            <span>创建聊天室</span>
          </button>
        </div>

        <!-- 聊天室列表 -->
        <nav class="mobile-sidebar__nav">
          <div v-if="roomStore.loading" class="mobile-sidebar__loading">
            <div class="mobile-sidebar__spinner" />
            <span>加载中...</span>
          </div>

          <template v-else-if="filteredRooms.length > 0">
            <div class="mobile-sidebar__section-title">
              我的聊天室 ({{ filteredRooms.length }})
            </div>
            <RoomCard
              v-for="room in filteredRooms"
              :key="room.id"
              :room="room"
              :active="false"
              @click="handleRoomClick"
            />
          </template>

          <div v-else class="mobile-sidebar__empty">
            <MessageSquare :size="48" class="mobile-sidebar__empty-icon" />
            <p v-if="searchQuery">未找到匹配的聊天室</p>
            <p v-else>暂无聊天室，创建一个吧！</p>
          </div>
        </nav>

        <!-- 底部用户信息 -->
        <footer v-if="authStore.user" class="mobile-sidebar__footer">
          <div class="mobile-sidebar__user">
            <div class="mobile-sidebar__avatar">
              {{ authStore.user.username.charAt(0).toUpperCase() }}
            </div>
            <div class="mobile-sidebar__user-info">
              <span class="mobile-sidebar__username">{{ authStore.user.username }}</span>
              <span class="mobile-sidebar__user-status">在线</span>
            </div>
          </div>
        </footer>
      </aside>
    </div>
  </Transition>
</template>

<style scoped>
.mobile-sidebar {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 1000;
  display: flex;
}

.mobile-sidebar__backdrop {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: var(--color-mask);
  backdrop-filter: blur(2px);
}

.mobile-sidebar__content {
  position: relative;
  width: 85%;
  max-width: 320px;
  height: 100%;
  background: var(--color-white);
  display: flex;
  flex-direction: column;
  box-shadow: var(--shadow-xl);
  transition: transform var(--duration-normal) var(--ease-out);
}

/* 头部 */
.mobile-sidebar__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-md) var(--space-lg);
  border-bottom: 1px solid var(--color-divider);
  flex-shrink: 0;
}

.mobile-sidebar__brand {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

.mobile-sidebar__logo {
  width: 36px;
  height: 36px;
  background: var(--color-primary);
  color: var(--color-white);
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  justify-content: center;
}

.mobile-sidebar__title {
  font-size: var(--font-size-h2);
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0;
}

.mobile-sidebar__actions {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

.mobile-sidebar__close {
  width: 36px;
  height: 36px;
  border: none;
  background: var(--color-background);
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all var(--duration-fast);
}

.mobile-sidebar__close:active {
  background: var(--color-divider);
  transform: scale(0.95);
}

/* 搜索栏 */
.mobile-sidebar__search {
  padding: var(--space-md) var(--space-lg);
  flex-shrink: 0;
}

.mobile-sidebar__search-input-wrapper {
  position: relative;
  display: flex;
  align-items: center;
}

.mobile-sidebar__search-icon {
  position: absolute;
  left: var(--space-md);
  color: var(--color-text-tertiary);
}

.mobile-sidebar__search-input {
  width: 100%;
  padding: var(--space-md) var(--space-md) var(--space-md) 40px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  font-size: var(--font-size-body);
  background: var(--color-background-light);
  outline: none;
  transition: all var(--duration-fast);
}

.mobile-sidebar__search-input:focus {
  border-color: var(--color-primary);
  background: var(--color-white);
}

/* 创建按钮 */
.mobile-sidebar__create {
  padding: 0 var(--space-lg) var(--space-md);
  flex-shrink: 0;
}

.mobile-sidebar__create-btn {
  width: 100%;
  padding: var(--space-md);
  border: 2px dashed var(--color-primary);
  border-radius: var(--radius-lg);
  background: var(--color-primary-light);
  color: var(--color-primary);
  font-size: var(--font-size-body);
  font-weight: 500;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-sm);
  cursor: pointer;
  transition: all var(--duration-fast);
}

.mobile-sidebar__create-btn:active {
  background: var(--color-primary);
  color: white;
  transform: scale(0.98);
}

/* 聊天室列表 */
.mobile-sidebar__nav {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 0 var(--space-lg);
}

.mobile-sidebar__section-title {
  font-size: var(--font-size-small);
  color: var(--color-text-tertiary);
  font-weight: 500;
  padding: var(--space-md) 0;
  margin-bottom: var(--space-sm);
}

.mobile-sidebar__loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--space-3xl);
  gap: var(--space-md);
  color: var(--color-text-secondary);
}

.mobile-sidebar__spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--color-divider);
  border-top-color: var(--color-primary);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.mobile-sidebar__empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--space-3xl);
  text-align: center;
  color: var(--color-text-tertiary);
}

.mobile-sidebar__empty-icon {
  margin-bottom: var(--space-md);
  opacity: 0.5;
}

/* 底部用户信息 */
.mobile-sidebar__footer {
  padding: var(--space-md) var(--space-lg);
  border-top: 1px solid var(--color-divider);
  flex-shrink: 0;
}

.mobile-sidebar__user {
  display: flex;
  align-items: center;
  gap: var(--space-md);
}

.mobile-sidebar__avatar {
  width: 40px;
  height: 40px;
  background: var(--color-primary);
  color: white;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  font-size: var(--font-size-h3);
}

.mobile-sidebar__user-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.mobile-sidebar__username {
  font-weight: 500;
  color: var(--color-text-primary);
}

.mobile-sidebar__user-status {
  font-size: var(--font-size-small);
  color: var(--color-success);
}

/* 过渡动画 */
.mobile-sidebar-enter-active,
.mobile-sidebar-leave-active {
  transition: opacity var(--duration-normal);
}

.mobile-sidebar-enter-active .mobile-sidebar__content,
.mobile-sidebar-leave-active .mobile-sidebar__content {
  transition: transform var(--duration-normal) var(--ease-out);
}

.mobile-sidebar-enter-from,
.mobile-sidebar-leave-to {
  opacity: 0;
}

.mobile-sidebar-enter-from .mobile-sidebar__content,
.mobile-sidebar-leave-to .mobile-sidebar__content {
  transform: translateX(-100%);
}
</style>
