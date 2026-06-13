<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useFriendStore } from '@/stores/friend'
import { useDirectRoomStore } from '@/stores/directRoom'
import { useRoomStore } from '@/stores/room'
import { searchApi } from '@/api/search'
import type { Friend } from '@/types/friend'
import type { UserSearchItem } from '@/types/search'
import {
  UserFilled,
  Plus,
  ChatDotRound,
  Delete,
  CircleCheck,
  CloseBold,
  Search,
} from '@element-plus/icons-vue'

type TabType = 'friends' | 'received' | 'sent'

const router = useRouter()
const { t } = useI18n()
const friendStore = useFriendStore()
const directRoomStore = useDirectRoomStore()
const roomStore = useRoomStore()

const activeTab = ref<TabType>('friends')

const tabs = computed(() => [
  { key: 'friends' as TabType, label: t('friends.friends'), icon: UserFilled, count: friendStore.friends.length },
  { key: 'received' as TabType, label: t('friends.received'), icon: CircleCheck, count: friendStore.pendingReceivedCount },
  { key: 'sent' as TabType, label: t('friends.sent'), icon: CloseBold, count: 0 },
])

const friends = computed(() => friendStore.friends)
const receivedRequests = computed(() => friendStore.receivedRequests)
const sentRequests = computed(() => friendStore.sentRequests)
const loading = computed(() => friendStore.loading)

const showAddDialog = ref(false)
const searchKeyword = ref('')
const searchResults = ref<UserSearchItem[]>([])
const searching = ref(false)

async function handleAccept(requestId: string) {
  const ok = await friendStore.handleFriendRequest(requestId, true)
  if (ok) ElMessage.success(t('friends.accepted'))
}

async function handleReject(requestId: string) {
  await friendStore.handleFriendRequest(requestId, false)
}

async function handleCancel(requestId: string) {
  const ok = await friendStore.cancelFriendRequest(requestId)
  if (ok) ElMessage.success(t('friends.cancelRequest'))
}

async function handleDeleteFriend(friend: Friend) {
  try {
    await ElMessageBox.confirm(t('friends.deleteConfirm'), t('friends.deleteFriend'), {
      confirmButtonText: t('common.confirm'),
      cancelButtonText: t('common.cancel'),
      type: 'warning',
    })
    const ok = await friendStore.deleteFriend(friend.id)
    if (ok) ElMessage.success(t('friends.deleteFriend'))
  } catch { }
}

async function handleSendMessage(friend: Friend) {
  const room = await directRoomStore.createOrGetDirectRoom(friend.friend.id)
  if (room) {
    router.push('/app')
  }
}

let searchTimer: ReturnType<typeof setTimeout> | null = null

async function doSearch() {
  const q = searchKeyword.value.trim()
  if (!q) {
    searchResults.value = []
    return
  }
  searching.value = true
  try {
    const res = await searchApi.searchUsers({ keyword: q, limit: 10 })
    if (res.success && res.data) {
      searchResults.value = res.data.users
    } else {
      searchResults.value = []
    }
  } catch {
    searchResults.value = []
  } finally {
    searching.value = false
  }
}

function onSearchInput() {
  if (searchTimer) clearTimeout(searchTimer)
  searchTimer = setTimeout(doSearch, 400)
}

async function handleAddFriend(userId: string) {
  const ok = await friendStore.sendFriendRequest({ target_user_id: userId })
  if (ok) {
    ElMessage.success(t('friends.requestSent'))
    searchKeyword.value = ''
    searchResults.value = []
    friendStore.fetchSentRequests()
  } else {
    ElMessage.error(friendStore.error || t('friends.sendRequest'))
  }
}

function getStatusClass(status: string) {
  switch (status) {
    case 'online': return 'status-online'
    case 'away': return 'status-away'
    case 'busy': return 'status-busy'
    default: return 'status-offline'
  }
}

function getStatusLabel(status: string) {
  return status.charAt(0).toUpperCase() + status.slice(1)
}

function getInitial(name: string) {
  return name.charAt(0).toUpperCase()
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
      <div class="friends-view__header-left">
        <el-icon :size="24"><UserFilled /></el-icon>
        <h1>{{ t('friends.title') }}</h1>
      </div>
      <el-button type="primary" :icon="Plus" @click="showAddDialog = true">
        {{ t('friends.addFriend') }}
      </el-button>
    </div>

    <el-tabs v-model="activeTab" class="friends-view__tabs">
      <el-tab-pane
        v-for="tab in tabs"
        :key="tab.key"
        :name="tab.key"
        :label="tab.label"
      >
        <template #label>
          <span class="tab-label">
            <el-icon><component :is="tab.icon" /></el-icon>
            <span>{{ tab.label }}</span>
            <el-tag v-if="tab.count > 0" size="small" type="danger" class="tab-badge">
              {{ tab.count }}
            </el-tag>
          </span>
        </template>
      </el-tab-pane>
    </el-tabs>

    <div class="friends-view__content">
      <!-- 好友列表 -->
      <div v-if="activeTab === 'friends'" class="friend-list">
        <div v-if="loading" class="list-placeholder">{{ t('common.loading') }}</div>
        <div v-else-if="friends.length === 0" class="list-placeholder">
          {{ t('friends.noFriends') }}
        </div>
        <div v-else class="friend-cards">
          <div v-for="friend in friends" :key="friend.id" class="friend-card">
            <div class="friend-card__avatar">
              <img v-if="friend.friend.avatar_url" :src="friend.friend.avatar_url" :alt="friend.friend.username" class="avatar-img" />
              <div v-else class="avatar-placeholder">{{ getInitial(friend.friend.username) }}</div>
              <span class="status-dot" :class="getStatusClass(friend.friend.status)" />
            </div>
            <div class="friend-card__body">
              <span class="friend-name">{{ friend.friend.username }}</span>
              <span class="friend-status" :class="getStatusClass(friend.friend.status)">
                {{ getStatusLabel(friend.friend.status) }}
              </span>
            </div>
            <div class="friend-card__actions">
              <el-tooltip :content="t('friends.sendMessage')" placement="top">
                <el-button circle size="small" :icon="ChatDotRound" @click="handleSendMessage(friend)" />
              </el-tooltip>
              <el-tooltip :content="t('friends.deleteFriend')" placement="top">
                <el-button circle size="small" :icon="Delete" type="danger" plain @click="handleDeleteFriend(friend)" />
              </el-tooltip>
            </div>
          </div>
        </div>
      </div>

      <!-- 收到的请求 -->
      <div v-if="activeTab === 'received'" class="friend-list">
        <div v-if="loading" class="list-placeholder">{{ t('common.loading') }}</div>
        <div v-else-if="receivedRequests.length === 0" class="list-placeholder">
          {{ t('friends.noReceived') }}
        </div>
        <div v-else class="friend-cards">
          <div v-for="req in receivedRequests" :key="req.id" class="friend-card">
            <div class="friend-card__avatar">
              <img v-if="req.sender.avatar_url" :src="req.sender.avatar_url" :alt="req.sender.username" class="avatar-img" />
              <div v-else class="avatar-placeholder">{{ getInitial(req.sender.username) }}</div>
            </div>
            <div class="friend-card__body">
              <span class="friend-name">{{ req.sender.username }}</span>
              <span v-if="req.message" class="friend-message">{{ req.message }}</span>
              <span v-if="req.status === 'pending'" class="friend-status status-pending">{{ t('friends.waiting') }}</span>
              <span v-else-if="req.status === 'accepted'" class="friend-status status-online">{{ t('friends.accepted') }}</span>
              <span v-else class="friend-status status-offline">{{ t('friends.rejected') }}</span>
            </div>
            <div v-if="req.status === 'pending'" class="friend-card__actions">
              <el-button type="primary" size="small" :icon="CircleCheck" @click="handleAccept(req.id)">
                {{ t('friends.accept') }}
              </el-button>
              <el-button size="small" :icon="CloseBold" @click="handleReject(req.id)">
                {{ t('friends.reject') }}
              </el-button>
            </div>
          </div>
        </div>
      </div>

      <!-- 已发送的请求 -->
      <div v-if="activeTab === 'sent'" class="friend-list">
        <div v-if="loading" class="list-placeholder">{{ t('common.loading') }}</div>
        <div v-else-if="sentRequests.length === 0" class="list-placeholder">
          {{ t('friends.noSent') }}
        </div>
        <div v-else class="friend-cards">
          <div v-for="req in sentRequests" :key="req.id" class="friend-card">
            <div class="friend-card__avatar">
              <img v-if="req.receiver.avatar_url" :src="req.receiver.avatar_url" :alt="req.receiver.username" class="avatar-img" />
              <div v-else class="avatar-placeholder">{{ getInitial(req.receiver.username) }}</div>
            </div>
            <div class="friend-card__body">
              <span class="friend-name">{{ req.receiver.username }}</span>
              <span v-if="req.status === 'pending'" class="friend-status status-pending">{{ t('friends.pending') }}</span>
              <span v-else-if="req.status === 'accepted'" class="friend-status status-online">{{ t('friends.accepted') }}</span>
              <span v-else class="friend-status status-offline">{{ t('friends.rejected') }}</span>
            </div>
            <div v-if="req.status === 'pending'" class="friend-card__actions">
              <el-button size="small" @click="handleCancel(req.id)">
                {{ t('friends.cancelRequest') }}
              </el-button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 添加好友弹窗 -->
    <el-dialog
      v-model="showAddDialog"
      :title="t('friends.addFriend')"
      width="400px"
      :close-on-click-modal="false"
    >
      <div class="add-friend-body">
        <el-input
          v-model="searchKeyword"
          :placeholder="t('friends.searchPlaceholder')"
          :prefix-icon="Search"
          @input="onSearchInput"
          clearable
        />
        <div v-if="searching" class="search-status">{{ t('common.loading') }}</div>
        <div v-else-if="searchKeyword && !searching && searchResults.length === 0 && !searching" class="search-status">
          {{ t('friends.searchNoResult') }}
        </div>
        <div v-if="searchResults.length > 0" class="search-results">
          <div v-for="user in searchResults" :key="user.id" class="search-result-item">
            <div class="result-avatar">
              <img v-if="user.avatar_url" :src="user.avatar_url" :alt="user.username" class="avatar-img" />
              <div v-else class="avatar-placeholder-sm">{{ getInitial(user.username) }}</div>
            </div>
            <span class="result-name">{{ user.username }}</span>
            <el-button size="small" type="primary" @click="handleAddFriend(user.id)">
              {{ t('friends.addFriend') }}
            </el-button>
          </div>
        </div>
      </div>
    </el-dialog>
  </div>
</template>

<style scoped lang="scss">
.friends-view {
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 24px;
  overflow: hidden;
  max-width: 720px;
  margin: 0 auto;
  width: 100%;
}

.friends-view__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 20px;
  flex-shrink: 0;

  &-left {
    display: flex;
    align-items: center;
    gap: 10px;

    h1 {
      font-size: 20px;
      font-weight: 600;
      margin: 0;
      color: var(--el-text-color-primary);
    }
  }
}

.friends-view__tabs {
  flex-shrink: 0;
}

.tab-label {
  display: flex;
  align-items: center;
  gap: 6px;
}

.tab-badge {
  margin-left: 4px;
}

.friends-view__content {
  flex: 1;
  overflow-y: auto;
  margin-top: 12px;
}

.list-placeholder {
  text-align: center;
  padding: 48px 16px;
  color: var(--el-text-color-placeholder);
  font-size: 14px;
}

.friend-cards {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.friend-card {
  display: flex;
  align-items: center;
  padding: 12px 16px;
  border-radius: 8px;
  background: var(--el-bg-color-page);
  border: 1px solid var(--el-border-color-light);
  transition: background 0.2s;

  &:hover {
    background: var(--el-color-primary-light-9);
  }

  &__avatar {
    position: relative;
    flex-shrink: 0;
    margin-right: 12px;

    .avatar-img, .avatar-placeholder {
      width: 40px;
      height: 40px;
      border-radius: 50%;
      object-fit: cover;
    }

    .avatar-placeholder {
      display: flex;
      align-items: center;
      justify-content: center;
      background: var(--el-color-primary-light-8);
      color: var(--el-color-primary);
      font-size: 16px;
      font-weight: 600;
    }

    .status-dot {
      position: absolute;
      bottom: 0;
      right: 0;
      width: 10px;
      height: 10px;
      border-radius: 50%;
      border: 2px solid var(--el-bg-color-page);
    }
  }

  &__body {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;

    .friend-name {
      font-size: 14px;
      font-weight: 500;
      color: var(--el-text-color-primary);
    }

    .friend-message {
      font-size: 12px;
      color: var(--el-text-color-secondary);
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }

    .friend-status {
      font-size: 12px;
    }
  }

  &__actions {
    flex-shrink: 0;
    display: flex;
    gap: 6px;
    align-items: center;
    margin-left: 12px;
  }
}

.status-online { color: var(--el-color-success); }
.status-away { color: var(--el-color-warning); }
.status-busy { color: var(--el-color-danger); }
.status-offline { color: var(--el-text-color-placeholder); }
.status-pending { color: var(--el-color-warning); }

.status-online { background: var(--el-color-success); }
.status-away { background: var(--el-color-warning); }
.status-busy { background: var(--el-color-danger); }
.status-offline { background: var(--el-text-color-placeholder); }

.add-friend-body {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.search-status {
  text-align: center;
  padding: 24px;
  color: var(--el-text-color-placeholder);
  font-size: 13px;
}

.search-results {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: 320px;
  overflow-y: auto;
}

.search-result-item {
  display: flex;
  align-items: center;
  padding: 8px 12px;
  border-radius: 8px;
  background: var(--el-bg-color-page);
  gap: 10px;

  .result-avatar {
    flex-shrink: 0;

    .avatar-img {
      width: 36px;
      height: 36px;
      border-radius: 50%;
      object-fit: cover;
    }

    .avatar-placeholder-sm {
      width: 36px;
      height: 36px;
      border-radius: 50%;
      display: flex;
      align-items: center;
      justify-content: center;
      background: var(--el-color-primary-light-8);
      color: var(--el-color-primary);
      font-size: 14px;
      font-weight: 600;
    }
  }

  .result-name {
    flex: 1;
    font-size: 14px;
    color: var(--el-text-color-primary);
  }
}
</style>
