<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { storeToRefs } from 'pinia'
import { Users, UserPlus, Inbox, Send } from 'lucide-vue-next'
import { useFriendStore } from '@/stores/friend'
import { useDirectRoomStore } from '@/stores/directRoom'
import FriendList from '@/components/friend/FriendList.vue'
import FriendRequestList from '@/components/friend/FriendRequestList.vue'
import AddFriendModal from '@/components/friend/AddFriendModal.vue'
import UserProfileModal from '@/components/user/UserProfileModal.vue'
import type { Friend } from '@/types/friend'

type TabType = 'friends' | 'received' | 'sent'

const router = useRouter()
const friendStore = useFriendStore()
const directRoomStore = useDirectRoomStore()
const {
  friends,
  receivedRequests,
  sentRequests,
  loading,
  pendingReceivedCount,
} = storeToRefs(friendStore)

const activeTab = ref<TabType>('friends')

// 添加好友弹窗
const showAddFriend = ref(false)

// 用户资料弹窗
const showUserProfile = ref(false)
const selectedUserId = ref('')

const tabs = computed(() => [
  { key: 'friends' as TabType, label: '好友', icon: Users, count: friends.value.length },
  { key: 'received' as TabType, label: '收到的请求', icon: Inbox, count: pendingReceivedCount.value },
  { key: 'sent' as TabType, label: '已发送', icon: Send, count: 0 },
])

async function handleAccept(requestId: string) {
  await friendStore.handleFriendRequest(requestId, true)
}

async function handleReject(requestId: string) {
  await friendStore.handleFriendRequest(requestId, false)
}

async function handleCancel(requestId: string) {
  await friendStore.cancelFriendRequest(requestId)
}

function handleFriendClick(friend: Friend) {
  selectedUserId.value = friend.friend.id
  showUserProfile.value = true
}

async function handleSendMessage() {
  showUserProfile.value = false
  const room = await directRoomStore.createOrGetDirectRoom(selectedUserId.value)
  if (room) {
    router.push(`/room/${room.id}`)
  }
}

function handleAddFriendDone() {
  showAddFriend.value = false
  friendStore.fetchReceivedRequests()
}

onMounted(() => {
  friendStore.fetchFriends()
  friendStore.fetchReceivedRequests()
  friendStore.fetchSentRequests()
  friendStore.clearUnreadRequestCount()
})
</script>

<template>
  <div class="friends-view">
    <div class="friends-view__header">
      <div class="friends-view__header-title">
        <Users :size="24" />
        <h1>好友</h1>
      </div>
      <button class="friends-view__add-btn" @click="showAddFriend = true">
        <UserPlus :size="18" />
        <span>添加好友</span>
      </button>
    </div>

    <div class="friends-view__tabs">
      <button
        v-for="tab in tabs"
        :key="tab.key"
        class="friends-view__tab"
        :class="{ 'friends-view__tab--active': activeTab === tab.key }"
        @click="activeTab = tab.key"
      >
        <component :is="tab.icon" :size="16" />
        <span>{{ tab.label }}</span>
        <span v-if="tab.count > 0" class="friends-view__tab-badge">{{ tab.count }}</span>
      </button>
    </div>

    <div class="friends-view__content">
      <FriendList
        v-if="activeTab === 'friends'"
        :friends="friends"
        :loading="loading"
        @click="handleFriendClick"
      />
      <FriendRequestList
        v-else-if="activeTab === 'received'"
        :requests="receivedRequests"
        :loading="loading"
        :is-received="true"
        @accept="handleAccept"
        @reject="handleReject"
      />
      <FriendRequestList
        v-else
        :requests="sentRequests"
        :loading="loading"
        :is-received="false"
        @cancel="handleCancel"
      />
    </div>

    <AddFriendModal
      :show="showAddFriend"
      @close="showAddFriend = false"
      @done="handleAddFriendDone"
    />

    <UserProfileModal
      v-model:visible="showUserProfile"
      :user-id="selectedUserId"
      @send-message="handleSendMessage"
    />
  </div>
</template>

<style scoped>
.friends-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--color-background, #f5f5f5);
}

.friends-view__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 24px 16px;
  background: var(--color-white, #fff);
  border-bottom: 1px solid var(--color-border, #eee);
  flex-shrink: 0;
}

.friends-view__header-title {
  display: flex;
  align-items: center;
  gap: 10px;
}

.friends-view__header-title h1 {
  font-size: 22px;
  font-weight: 600;
  color: var(--color-text, #333);
  margin: 0;
}

.friends-view__header-title :deep(svg) {
  color: var(--color-primary, #2080f0);
}

.friends-view__add-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  background: var(--color-primary, #2080f0);
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: opacity var(--duration-fast, 0.15s);
}

.friends-view__add-btn:hover {
  opacity: 0.9;
}

.friends-view__tabs {
  display: flex;
  gap: 4px;
  padding: 8px 16px;
  background: var(--color-white, #fff);
  border-bottom: 1px solid var(--color-border, #eee);
  flex-shrink: 0;
}

.friends-view__tab {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: var(--color-text-secondary, #666);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--duration-fast, 0.15s);
  position: relative;
}

.friends-view__tab:hover {
  background: var(--color-background, #f5f5f5);
  color: var(--color-text, #333);
}

.friends-view__tab--active {
  background: var(--color-primary-soft, #e8f4ff);
  color: var(--color-primary, #2080f0);
}

.friends-view__tab-badge {
  min-width: 18px;
  height: 18px;
  padding: 0 5px;
  border-radius: 9px;
  background: var(--color-error, #f5222d);
  color: white;
  font-size: 11px;
  font-weight: 600;
  display: flex;
  align-items: center;
  justify-content: center;
}

.friends-view__content {
  flex: 1;
  overflow-y: auto;
  background: var(--color-white, #fff);
  margin: 0;
}

@media (max-width: 768px) {
  .friends-view__header {
    padding: 16px;
  }

  .friends-view__header-title h1 {
    font-size: 18px;
  }
}
</style>
