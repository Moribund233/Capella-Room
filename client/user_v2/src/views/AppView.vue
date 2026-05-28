<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useRoomStore } from '@/stores/room'
import { useMessageStore } from '@/stores/message'
import { useWebSocket } from '@/composables/useWebSocket'
import { useMessageActions } from '@/composables/useMessageActions'
import { useResponsive } from '@/composables/useResponsive'
import {
  ChatRoomList,
  ChatHeader,
  ChatMessageList,
  ChatInputArea,
  ChatMemberPanel,
  ChatWelcome,
} from '@/components/chat'

const roomStore = useRoomStore()
const messageStore = useMessageStore()
const { isMobile } = useResponsive()

const { currentRoom, members } = storeToRefs(roomStore)
const hasRoom = ref(false)

// 消息操作
const messageActions = useMessageActions(currentRoom.value?.id || '')

// 面板状态
const showSidebar = ref(!isMobile.value)
const showMemberPanel = ref(false)

// WebSocket 订阅
const ws = useWebSocket()

onMounted(() => {
  // 加载房间列表
  roomStore.fetchMyRooms()

  // WebSocket 消息处理
  ws.onMessage('NewMessage', (payload: unknown) => {
    messageStore.addIncomingMessage(payload as any)
  })

  ws.onMessage('MessageEdited', (payload: unknown) => {
    messageStore.handleMessageEdited(payload as any)
  })

  ws.onMessage('MessageDeleted', (payload: unknown) => {
    messageStore.handleMessageDeleted(payload as any)
  })
})

onUnmounted(() => {
  messageStore.switchRoom('')
})

// 监听当前房间变化
watch(currentRoom, (room) => {
  hasRoom.value = !!room
  if (room) {
    // 移动端选择房间后自动隐藏侧边栏
    if (isMobile.value) {
      showSidebar.value = false
    }
    showMemberPanel.value = false
  }
})

// 发送消息
function handleSend(content: string) {
  if (!currentRoom.value) return
  messageActions.sendMessage(content)
}

// 开始回复
function handleReply(message: any) {
  messageActions.startReply(message)
}

// 开始编辑
function handleEdit(message: any) {
  messageActions.startEdit(message)
}

// 删除消息
function handleDelete(messageId: string) {
  messageStore.deleteMessage(messageId)
}

// 切换侧边栏（移动端）
function toggleSidebar() {
  showSidebar.value = !showSidebar.value
}

function closeMobileSidebar() {
  if (isMobile.value) {
    showSidebar.value = false
  }
}

// 切换成员面板
function toggleMemberPanel() {
  showMemberPanel.value = !showMemberPanel.value
}
</script>

<template>
  <div class="app-view">
    <!-- 侧边栏遮罩（移动端） -->
    <transition name="fade">
      <div
        v-if="isMobile && showSidebar"
        class="sidebar-overlay"
        @click="closeMobileSidebar"
      />
    </transition>

    <!-- 侧边栏 -->
    <transition name="slide-left">
      <div
        v-if="!isMobile || showSidebar"
        class="app-view__sidebar"
        :class="{ 'app-view__sidebar--mobile': isMobile }"
      >
        <ChatRoomList @close-mobile="closeMobileSidebar" />
      </div>
    </transition>

    <!-- 主聊天区 -->
    <div class="app-view__main">
      <template v-if="hasRoom && currentRoom">
        <!-- 聊天头部 -->
        <ChatHeader
          :room="currentRoom"
          :members="members"
          :is-mobile="isMobile"
          @toggle-sidebar="toggleSidebar"
          @toggle-member-panel="toggleMemberPanel"
        />

        <!-- 消息列表 -->
        <ChatMessageList
          :key="currentRoom.id"
          @reply="handleReply"
          @edit="handleEdit"
          @delete="handleDelete"
          @jump-to-thread="(id: string) => {}"
        />

        <!-- 输入区 -->
        <ChatInputArea
          :room-name="currentRoom.name"
          :replying-to="messageActions.replyingTo.value"
          :editing-message="messageActions.editingMessage.value"
          @send="handleSend"
          @cancel-reply="messageActions.cancelReply"
          @cancel-edit="messageActions.cancelEdit"
        />
      </template>

      <!-- 欢迎页（未选择房间时） -->
      <ChatWelcome v-else />
    </div>

    <!-- 成员面板 -->
    <transition name="slide-right">
      <div
        v-if="hasRoom && showMemberPanel && !isMobile"
        class="app-view__member-panel"
      >
        <ChatMemberPanel
          :members="members"
          :total-count="members.length"
        />
      </div>
    </transition>
  </div>
</template>

<style scoped lang="scss">
.app-view {
  display: flex;
  width: 100%;
  height: 100%;
  overflow: hidden;
  position: relative;

  &__sidebar {
    flex-shrink: 0;

    &--mobile {
      position: fixed;
      left: 0;
      top: 0;
      bottom: 0;
      z-index: 200;
      box-shadow: 0 0 40px rgba(0, 0, 0, 0.5);
    }
  }

  &__main {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    background: var(--bg);
  }

  &__member-panel {
    flex-shrink: 0;
  }
}

// 移动端遮罩
.sidebar-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  z-index: 199;
}

// 过渡动画
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.slide-left-enter-active,
.slide-left-leave-active {
  transition: transform 0.2s ease;
}

.slide-left-enter-from,
.slide-left-leave-to {
  transform: translateX(-100%);
}

.slide-right-enter-active,
.slide-right-leave-active {
  transition: transform 0.2s ease;
}

.slide-right-enter-from,
.slide-right-leave-to {
  transform: translateX(100%);
}
</style>
