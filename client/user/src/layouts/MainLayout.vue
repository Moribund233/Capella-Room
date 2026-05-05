<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, h } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { Menu, MessageCircle, X, ArrowLeft } from 'lucide-vue-next'
import { useNotification } from 'naive-ui'
import { useResponsive } from '@/composables/useResponsive'
import { useRoomStore } from '@/stores/room'
import { useAuthStore } from '@/stores/auth'
import { useWebSocketStore } from '@/stores/websocket'
import { useGlobalModal } from '@/composables/useGlobalModal'
import { usePersonalizationStore } from '@/stores/personalization'
import { QuickBar } from '@/components/quick'
import { useQuickBar } from '@/composables/quick'
import { quickBarConfig } from '@/config/quick'
import { WSMessageType, type RoomMessageSummaryPayload } from '@/types/websocket'
import NavBar from '@/components/nav/NavBar.vue'
import MobileSidebar from '@/components/layout/MobileSidebar.vue'
import DesktopSidebar from '@/components/layout/DesktopSidebar.vue'
import CreateRoomModal from '@/components/room/CreateRoomModal.vue'
import PageTransition from '@/components/ui/PageTransition.vue'
import GlobalModal from '@/components/common/GlobalModal.vue'

const router = useRouter()
const route = useRoute()
const notification = useNotification()
const { isMobile, isTablet, isDesktop } = useResponsive()
const roomStore = useRoomStore()
const authStore = useAuthStore()
const wsStore = useWebSocketStore()
const personalizationStore = usePersonalizationStore()

// 判断是否在聊天页面（显示侧边栏的页面）
const isChatPage = computed(() => {
  const chatRoutes = ['chat', 'chat-room']
  return chatRoutes.includes(route.name as string)
})

// 初始化全局弹窗
const { modalState, handlePositiveClick, handleNegativeClick, handleClose } = useGlobalModal()

// 初始化移动端 QuickBar
const { items: mobileQuickItems } = useQuickBar(quickBarConfig)

const showMobileSidebar = ref(false)
const showCreateModal = ref(false)

function handleRoomCreated(roomId: string) {
  showCreateModal.value = false
  if (roomId) {
    router.push(`/room/${roomId}`)
  }
}

function openMobileSidebar() {
  showMobileSidebar.value = true
}

function closeMobileSidebar() {
  showMobileSidebar.value = false
}

function openCreateModal() {
  showCreateModal.value = true
  // 如果在移动端，关闭侧边栏
  if (isMobile.value) {
    showMobileSidebar.value = false
  }
}

// 键盘快捷键
function onKeyDown(e: KeyboardEvent) {
  // Ctrl/Cmd + B 切换侧边栏
  if ((e.ctrlKey || e.metaKey) && e.key === 'b') {
    e.preventDefault()
    if (isMobile.value) {
      showMobileSidebar.value = !showMobileSidebar.value
    }
  }
  // ESC 关闭移动端侧边栏
  if (e.key === 'Escape' && showMobileSidebar.value) {
    showMobileSidebar.value = false
  }
}

/**
 * 处理房间消息摘要 - 更新房间列表并显示通知
 * @param payload - 消息摘要数据
 */
function handleRoomMessageSummary(payload: RoomMessageSummaryPayload) {
  // 更新房间列表的最新消息
  roomStore.updateRoomLastMessage(payload.room_id, payload.last_message, true)

  // 如果不在当前房间，显示通知
  const currentRoomId = route.params.roomId as string
  if (currentRoomId !== payload.room_id) {
    showMessageNotification(payload)
  }
}

/**
 * 显示消息通知 - 现代化样式
 * @param payload - 消息摘要数据
 */
function showMessageNotification(payload: RoomMessageSummaryPayload) {
  const room = roomStore.rooms.find(r => r.id === payload.room_id)
  const roomName = room?.name || '未知房间'
  const senderName = payload.last_message.sender_name
  const content = payload.last_message.content

  // 截断过长的内容
  const maxLength = 60
  const displayContent = content.length > maxLength
    ? content.slice(0, maxLength) + '...'
    : content

  // 使用自定义渲染创建现代化通知
  const n = notification.create({
    content: () =>
      h('div', {
        class: 'custom-message-notification',
        onClick: () => {
          n.destroy()
          router.push(`/room/${payload.room_id}`)
        }
      }, [
        // 图标区域
        h('div', { class: 'notification-icon' }, [
          h(MessageCircle, { size: 20 })
        ]),
        // 内容区域
        h('div', { class: 'notification-content' }, [
          h('div', { class: 'notification-header' }, [
            h('span', { class: 'notification-room' }, roomName),
            h('span', { class: 'notification-meta' }, '新消息')
          ]),
          h('div', { class: 'notification-body' }, [
            h('span', { class: 'notification-sender' }, `${senderName}:`),
            h('span', { class: 'notification-text' }, displayContent)
          ])
        ]),
        // 关闭按钮
        h('button', {
          class: 'notification-close',
          onClick: (e: Event) => {
            e.stopPropagation()
            n.destroy()
          }
        }, [
          h(X, { size: 14 })
        ])
      ]),
    duration: 5000,
    closable: false,
    type: 'default',
  })
}

// 订阅 WebSocket 消息
function subscribeWebSocketMessages() {
  wsStore.onMessage<RoomMessageSummaryPayload>(
    WSMessageType.ROOM_MESSAGE_SUMMARY,
    handleRoomMessageSummary,
  )
}

// 取消订阅 WebSocket 消息
function unsubscribeWebSocketMessages() {
  wsStore.offMessage<RoomMessageSummaryPayload>(
    WSMessageType.ROOM_MESSAGE_SUMMARY,
    handleRoomMessageSummary,
  )
}

// 确保 WebSocket 连接（全局单例）
function ensureWebSocketConnection() {
  if (authStore.isAuthenticated && !wsStore.isConnected && !wsStore.isConnecting) {
    console.log('[MainLayout] Ensuring WebSocket connection...')
    wsStore.connect()
  }
}

onMounted(() => {
  // 仅在用户已登录时获取房间列表
  if (authStore.isAuthenticated) {
    roomStore.fetchMyRooms()
  }
  // 确保 WebSocket 连接
  ensureWebSocketConnection()
  // 订阅房间消息摘要（全局）
  subscribeWebSocketMessages()
  // 注意：通知系统在 App.vue 中全局初始化，不在此处重复初始化
  window.addEventListener('keydown', onKeyDown)
})

onUnmounted(() => {
  unsubscribeWebSocketMessages()
  // 注意：不在这里调用 notificationStore.cleanup()
  // 通知系统是全局的，应该在应用级别管理，避免路由切换时断开通知
  window.removeEventListener('keydown', onKeyDown)
})
</script>

<template>
  <div
    class="main-layout"
    :style="{
      backgroundImage: personalizationStore.backgroundImage && personalizationStore.enableBackgroundImage
        ? `url(${personalizationStore.backgroundImage})`
        : 'none',
    }"
  >
    <!-- 背景遮罩层（用于透明度效果） -->
    <div
      v-if="personalizationStore.backgroundImage && personalizationStore.enableBackgroundImage"
      class="main-layout__bg-overlay"
      :style="{ opacity: 1 - personalizationStore.backgroundOpacity }"
    />

    <!-- 桌面端导航栏 -->
    <NavBar v-if="!isMobile" class="main-layout__navbar" />

    <!-- 桌面端侧边栏 - 仅在聊天页面显示 -->
    <DesktopSidebar
      v-if="(isDesktop || isTablet) && isChatPage"
      class="main-layout__sidebar-desktop"
      @create-room="openCreateModal"
    />

    <!-- 移动端侧边栏 - 仅在聊天页面显示 -->
    <MobileSidebar
      v-if="isMobile && isChatPage"
      :show="showMobileSidebar"
      @close="closeMobileSidebar"
      @create-room="openCreateModal"
    />

    <!-- 主内容区 -->
    <main class="main-layout__content" :class="{ 'main-layout--mobile': isMobile }">
      <!-- 移动端顶部栏 -->
      <header v-if="isMobile" class="main-layout__mobile-header">
        <!-- 菜单按钮 - 仅在聊天页面显示 -->
        <button
          v-if="isChatPage"
          class="main-layout__menu-btn"
          @click="openMobileSidebar"
          aria-label="打开菜单"
        >
          <Menu :size="24" />
        </button>
        <!-- 非聊天页面显示返回按钮 -->
        <button
          v-else
          class="main-layout__menu-btn"
          @click="router.push('/')"
          aria-label="返回"
        >
          <ArrowLeft :size="24" />
        </button>
        <h1 class="main-layout__mobile-title">Seredeli</h1>
        <!-- 移动端 QuickBar -->
        <div class="main-layout__mobile-quick">
          <QuickBar :items="mobileQuickItems" position="mobile-header" />
        </div>
      </header>

      <!-- 页面内容 -->
      <div class="main-layout__page">
        <router-view v-slot="{ Component }">
          <PageTransition name="slide">
            <component :is="Component" />
          </PageTransition>
        </router-view>
      </div>
    </main>

    <!-- 移动端底部导航栏 -->
    <NavBar v-if="isMobile" class="main-layout__mobile-nav" />

    <!-- 创建聊天室弹窗 -->
    <CreateRoomModal
      :show="showCreateModal"
      @close="showCreateModal = false"
      @created="handleRoomCreated"
    />

    <!-- 全局弹窗 -->
    <GlobalModal
      v-model:visible="modalState.visible"
      :title="modalState.title"
      :preset="modalState.preset"
      :mask-closable="modalState.maskClosable"
      :closable="modalState.closable"
      :positive-text="modalState.positiveText"
      :negative-text="modalState.negativeText"
      :loading="modalState.loading"
      @positive-click="handlePositiveClick"
      @negative-click="handleNegativeClick"
      @close="handleClose"
    >
      <component
        :is="modalState.component"
        v-if="modalState.component"
        v-bind="modalState.componentProps"
      />
    </GlobalModal>
  </div>
</template>

<style scoped>
.main-layout {
  display: flex;
  width: 100vw;
  height: 100vh;
  overflow: hidden;
  background: var(--color-background);
  background-size: cover;
  background-position: center;
  background-repeat: no-repeat;
  position: relative;
}

/* 背景遮罩层 */
.main-layout__bg-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: var(--color-background);
  pointer-events: none;
  z-index: 0;
}

/* 确保所有子元素在遮罩层之上 */
.main-layout > *:not(.main-layout__bg-overlay) {
  position: relative;
  z-index: 1;
}

/* 桌面端导航栏 */
.main-layout__navbar {
  flex-shrink: 0;
}

/* 桌面端侧边栏 */
.main-layout__sidebar-desktop {
  flex-shrink: 0;
  height: 100%;
}

/* 主内容区 */
.main-layout__content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-width: 0;
}

/* 移动端样式 */
.main-layout--mobile {
  /* 注意：不在主布局添加 padding-bottom，由各页面自行控制 */
  /* 聊天页面等需要底部空间的页面应自行处理 */
}

.main-layout__mobile-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-md) var(--space-lg);
  background: var(--color-white);
  border-bottom: 1px solid var(--color-border);
  flex-shrink: 0;
  height: 56px;
}

.main-layout__menu-btn {
  width: 40px;
  height: 40px;
  border: none;
  background: transparent;
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-primary);
  cursor: pointer;
  transition: background var(--duration-fast);
}

.main-layout__menu-btn:active {
  background: var(--color-background);
}

.main-layout__mobile-title {
  font-size: var(--font-size-h3);
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0;
}

.main-layout__mobile-spacer {
  width: 40px;
}

/* 移动端 QuickBar */
.main-layout__mobile-quick {
  display: flex;
  align-items: center;
}

/* 页面内容 */
.main-layout__page {
  flex: 1;
  overflow: hidden;
  position: relative;
}

/* 移动端底部导航栏 */
.main-layout__mobile-nav {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  z-index: 200;
}

/* 响应式调整 */
@media (max-width: 767px) {
  .main-layout {
    flex-direction: column;
  }
}
</style>

<style>
/* 全局通知样式 - 现代化设计 */
.custom-message-notification {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 14px 16px;
  background: var(--color-surface);
  border-radius: 16px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.12), 0 2px 8px rgba(0, 0, 0, 0.08);
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  min-width: 320px;
  max-width: 420px;
  border: 1px solid var(--color-border);
}

.custom-message-notification:hover {
  transform: translateY(-2px);
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.15), 0 4px 12px rgba(0, 0, 0, 0.1);
}

.notification-icon {
  flex-shrink: 0;
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, var(--color-primary) 0%, var(--color-primary-light) 100%);
  border-radius: 12px;
  color: white;
}

.notification-content {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.notification-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.notification-room {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.notification-meta {
  font-size: 11px;
  color: var(--color-primary);
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.notification-body {
  display: flex;
  align-items: baseline;
  gap: 6px;
  flex-wrap: wrap;
}

.notification-sender {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-secondary);
  flex-shrink: 0;
}

.notification-text {
  font-size: 13px;
  color: var(--color-text);
  line-height: 1.5;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

.notification-close {
  flex-shrink: 0;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  border-radius: 6px;
  color: var(--color-text-tertiary);
  cursor: pointer;
  transition: all 0.2s ease;
  margin-top: 2px;
}

.notification-close:hover {
  background: var(--color-background);
  color: var(--color-text);
}

/* Naive UI 通知容器样式覆盖 */
.n-notification-container.n-notification-container--top {
  top: 20px !important;
}

.n-notification-container.n-notification-container--top .n-notification-wrapper {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

.n-notification-container.n-notification-container--top .n-notification {
  background: transparent !important;
  box-shadow: none !important;
  padding: 0 !important;
  margin: 0 !important;
  animation: notification-slide-in 0.4s cubic-bezier(0.4, 0, 0.2, 1);
}

@keyframes notification-slide-in {
  from {
    opacity: 0;
    transform: translateY(-20px) scale(0.95);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

.n-notification-container.n-notification-container--top .n-notification.n-notification--closable {
  padding: 0 !important;
}

.n-notification-container.n-notification-container--top .n-notification__content {
  padding: 0 !important;
}

/* 移动端适配 */
@media (max-width: 767px) {
  .custom-message-notification {
    min-width: calc(100vw - 32px);
    max-width: calc(100vw - 32px);
    border-radius: 12px;
    padding: 12px 14px;
  }

  .notification-icon {
    width: 36px;
    height: 36px;
    border-radius: 10px;
  }

  .notification-room {
    font-size: 13px;
  }

  .notification-sender,
  .notification-text {
    font-size: 12px;
  }

  .n-notification-container.n-notification-container--top {
    top: 12px !important;
    left: 16px !important;
    right: 16px !important;
  }

  .n-notification-container.n-notification-container--top .n-notification-wrapper {
    align-items: stretch;
  }
}
</style>
