<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useRoomStore } from '@/stores/room'
import { useMessageStore } from '@/stores/message'
import { useWebSocket } from '@/composables/useWebSocket'
import { useMessageActions } from '@/composables/useMessageActions'
import { useResponsive } from '@/composables/useResponsive'
import { useBrowserNotification } from '@/composables/useBrowserNotification'
import { useSwipeGesture } from '@/composables/useSwipeGesture'
import {
  ChatRoomList,
  ChatHeader,
  ChatMessageList,
  ChatInputArea,
  ChatMemberPanel,
  ChatWelcome,
  SearchMessagesPanel,
  RoomSettingsModal,
  PinnedMessagesPanel,
} from '@/components/chat'
import ChatMessageListComponent from '@/components/chat/ChatMessageList.vue'
import type { Message } from '@/types/message'
import type {
  NewMessagePayload,
  MessageEditedPayload,
  MessageDeletedPayload,
  MissedMessagesPayload,
  ReactionAddedPayload,
  ReactionRemovedPayload,
  UserStatusChangedPayload,
  MessagePinnedPayload,
  MessageUnpinnedPayload,
  SystemMessagePayload,
} from '@/types/websocket'

const roomStore = useRoomStore()
const messageStore = useMessageStore()
const { isMobile, showMemberPanel: canShowMemberPanel, sidebarCollapsed, toggleSidebar } = useResponsive()

const { currentRoom, members } = storeToRefs(roomStore)
const hasRoom = ref(false)

// 消息操作
const messageActions = useMessageActions()

// 面板状态
const showMemberPanel = ref(false)

// 消息列表组件引用（用于滚动到指定消息）
const chatMessageListRef = ref<InstanceType<typeof ChatMessageListComponent> | null>(null)

// WebSocket 订阅
const ws = useWebSocket()

// 移动端滑动手势
const mainContentRef = ref<HTMLElement | null>(null)
useSwipeGesture(mainContentRef, {
  onSwipeRight: () => {
    if (isMobile.value && sidebarCollapsed.value) {
      sidebarCollapsed.value = false
    }
  },
  onSwipeLeft: () => {
    if (isMobile.value && !sidebarCollapsed.value) {
      sidebarCollapsed.value = true
    }
  },
})

onMounted(() => {
  // 移动端默认收起侧边栏
  if (isMobile.value) {
    sidebarCollapsed.value = true
  }

  // 加载房间列表
  roomStore.fetchMyRooms()

  // WebSocket 消息处理
  ws.onMessage('NewMessage', (payload: unknown) => {
    const p = payload as NewMessagePayload
    messageStore.addIncomingMessage(p)

    if (document.hidden || p.room_id !== currentRoom.value?.id) {
      browserNotify(p.sender_name, {
        body: p.content,
        tag: `room-${p.room_id}`,
      })
    }
  })

  ws.onMessage('MessageEdited', (payload: unknown) => {
    messageStore.handleMessageEdited(payload as MessageEditedPayload)
  })

  ws.onMessage('MessageDeleted', (payload: unknown) => {
    messageStore.handleMessageDeleted(payload as MessageDeletedPayload)
  })

  ws.onMessage('UserTyping', (payload: unknown) => {
    const p = payload as { room_id: string; user_id: string; username: string }
    if (p.room_id === currentRoom.value?.id) {
      messageActions.addTypingUser(p.user_id, p.username)
    }
  })

  ws.onMessage('UserStopTyping', (payload: unknown) => {
    const p = payload as { room_id: string; user_id: string }
    if (p.room_id === currentRoom.value?.id) {
      messageActions.removeTypingUser(p.user_id)
    }
  })

  ws.onMessage('UserStatusChanged', (payload: unknown) => {
    const p = payload as UserStatusChangedPayload
    roomStore.updateMemberStatus(p.user_id, p.status)
  })

  ws.onMessage('MessageReadReceipt', (payload: unknown) => {
    const p = payload as { message_id: string; user_id: string }
    messageStore.markMessageAsRead(p.message_id)
  })

  ws.onMessage('MissedMessages', (payload: unknown) => {
    messageStore.addMissedMessages(payload as MissedMessagesPayload)
  })

  ws.onMessage('ReactionAdded', (payload: unknown) => {
    messageStore.handleReactionAdded(payload as ReactionAddedPayload)
  })

  ws.onMessage('ReactionRemoved', (payload: unknown) => {
    messageStore.handleReactionRemoved(payload as ReactionRemovedPayload)
  })

  ws.onMessage('MessagePinned', (payload: unknown) => {
    messageStore.handleMessagePinned(payload as MessagePinnedPayload)
  })

  ws.onMessage('MessageUnpinned', (payload: unknown) => {
    messageStore.handleMessageUnpinned(payload as MessageUnpinnedPayload)
  })

  ws.onMessage('SystemMessage', (payload: unknown) => {
    const p = payload as SystemMessagePayload
    messageStore.addSystemMessage(p.content)
  })
})

onUnmounted(() => {
  messageStore.switchRoom('')
})

// 监听当前房间变化
watch(currentRoom, (room) => {
  hasRoom.value = !!room
  messageActions.clearTypingUsers()
  if (room) {
    // 移动端选择房间后自动隐藏侧边栏
    if (isMobile.value) {
      sidebarCollapsed.value = true
    }
    showMemberPanel.value = false

    // 加载置顶消息
    messageStore.fetchPinnedMessages(room.id)
  }
})

// 发送消息
function handleSend(content: string, messageType?: string) {
  if (!currentRoom.value) return
  messageActions.sendMessage(content, messageType)
}

// 开始回复
function handleReply(message: Message) {
  messageActions.startReply(message)
}

function handleEdit(message: Message) {
  messageActions.startEdit(message)
}

// 删除消息
function handleDelete(messageId: string) {
  messageStore.deleteMessage(messageId)
}

// 滚动到指定消息（回复引用点击跳转）
function handleJumpToThread(msgId: string | undefined) {
  if (!msgId) return
  chatMessageListRef.value?.scrollToMessage(msgId)
}

function closeMobileSidebar() {
  if (isMobile.value) {
    sidebarCollapsed.value = true
  }
}

// 浏览器通知
const { notify: browserNotify } = useBrowserNotification()

// 切换成员面板
function toggleMemberPanel() {
  showMemberPanel.value = !showMemberPanel.value
}

// 置顶消息
const showPinned = ref(false)
function togglePinned() {
  showPinned.value = !showPinned.value
}

// 消息搜索
const showSearch = ref(false)
const showRoomSettings = ref(false)
function toggleSearch() {
  showSearch.value = !showSearch.value
}

function handleJumpToSearch(msgId: string) {
  chatMessageListRef.value?.scrollToMessage(msgId)
}
</script>

<template>
  <div class="app-view">
    <!-- 侧边栏遮罩（移动端） -->
    <transition name="fade">
      <div
        v-if="isMobile && !sidebarCollapsed"
        class="sidebar-overlay"
        @click="closeMobileSidebar"
      />
    </transition>

    <!-- 侧边栏 -->
    <transition name="slide-left">
      <div
        v-if="!sidebarCollapsed"
        class="app-view__sidebar"
        :class="{ 'app-view__sidebar--mobile': isMobile }"
      >
        <ChatRoomList />
      </div>
    </transition>

    <!-- 主聊天区 -->
    <div ref="mainContentRef" class="app-view__main">
      <template v-if="hasRoom && currentRoom">
        <!-- 聊天头部 -->
        <ChatHeader
          :room="currentRoom"
          :members="members"
          :is-mobile="isMobile"
          @toggle-sidebar="toggleSidebar"
          @toggle-member-panel="toggleMemberPanel"
          @toggle-pinned="togglePinned"
          @toggle-search="toggleSearch"
          @toggle-settings="showRoomSettings = !showRoomSettings"
        />

        <!-- 消息列表 -->
        <ChatMessageList
          ref="chatMessageListRef"
          :key="currentRoom.id"
          :typing-users="messageActions.typingUsers.value"
          @reply="handleReply"
          @edit="handleEdit"
          @delete="handleDelete"
          @jump-to-thread="handleJumpToThread"
        />

        <!-- 输入区 -->
        <ChatInputArea
          :room-id="currentRoom.id"
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
        v-if="hasRoom && showMemberPanel && canShowMemberPanel"
        class="app-view__member-panel"
      >
        <ChatMemberPanel
          :members="members"
          :total-count="members.length"
        />
      </div>
    </transition>

    <!-- 置顶消息面板 -->
    <transition name="slide-right">
      <div
        v-if="hasRoom && showPinned && canShowMemberPanel"
        class="app-view__pinned-panel"
      >
        <PinnedMessagesPanel @jump-to-message="handleJumpToSearch" />
      </div>
    </transition>

    <!-- 消息搜索面板 -->
    <SearchMessagesPanel
      v-if="showSearch"
      @close="showSearch = false"
      @jump-to-message="handleJumpToSearch"
    />

    <!-- 房间设置 -->
    <RoomSettingsModal
      v-if="showRoomSettings && currentRoom"
      :room="currentRoom"
      @close="showRoomSettings = false"
    />
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

  &__pinned-panel {
    flex-shrink: 0;
    width: 320px;
    border-left: 1px solid var(--border);
    background: var(--bg);
    overflow-y: auto;
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
  transition: transform 0.25s cubic-bezier(0.4, 0, 0.2, 1), opacity 0.25s ease;
}

.slide-left-enter-from,
.slide-left-leave-to {
  transform: translateX(-100%);
  opacity: 0;
}

.slide-right-enter-active,
.slide-right-leave-active {
  transition: transform 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.slide-right-enter-from,
.slide-right-leave-to {
  transform: translateX(100%);
}
</style>
