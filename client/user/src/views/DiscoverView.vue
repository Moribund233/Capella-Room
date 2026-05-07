<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { Compass, TrendingUp } from 'lucide-vue-next'
import UniversalSearchBar from '@/components/search/UniversalSearchBar.vue'
import RoomSearchResults from '@/components/search/RoomSearchResults.vue'
import UserSearchResults from '@/components/search/UserSearchResults.vue'
import UserProfileModal from '@/components/user/UserProfileModal.vue'
import StartDirectChatModal from '@/components/user/StartDirectChatModal.vue'
import { useSearchStore } from '@/stores/search'
import { useFriendStore } from '@/stores/friend'
import type { SearchType } from '@/types/search'
import type { UserSearchItem } from '@/types/search'
import type { User } from '@/types/user'

const router = useRouter()
const searchStore = useSearchStore()
const friendStore = useFriendStore()

/** 当前激活的标签 */
const activeTab = ref<'discover' | 'search'>('discover')
/** 是否显示用户资料弹窗 */
const showUserProfile = ref(false)
/** 当前选中的用户ID */
const selectedUserId = ref('')
/** 当前选中的用户信息 */
const selectedUserInfo = ref<UserSearchItem | undefined>()
/** 是否显示发起私聊弹窗 */
const showStartDirectChat = ref(false)
/** 私聊目标用户 */
const directChatTargetUser = ref<User | null>(null)

/** 是否正在搜索 */
const isSearching = computed(() => {
  return searchStore.keyword.trim().length > 0
})

/** 搜索栏占位符 */
const searchPlaceholder = computed(() => {
  return searchStore.searchType === 'room' ? '搜索房间名称...' : '搜索用户名...'
})

/**
 * 处理搜索类型切换
 */
function handleSearchTypeChange(type: SearchType) {
  searchStore.setSearchType(type)
  searchStore.clearResults()
}

/**
 * 处理搜索
 */
async function handleSearch(keyword: string) {
  if (!keyword.trim()) return
  activeTab.value = 'search'
  await searchStore.search({ keyword: keyword.trim() })
}

/**
 * 处理选择历史记录
 */
function handleSelectHistory(keyword: string) {
  searchStore.setKeyword(keyword)
  handleSearch(keyword)
}

/**
 * 处理房间点击
 */
function handleRoomClick(roomId: string) {
  router.push(`/room/${roomId}`)
}

/**
 * 处理用户点击
 */
function handleUserClick(user: UserSearchItem) {
  selectedUserId.value = user.id
  selectedUserInfo.value = user
  showUserProfile.value = true
}

/**
 * 处理加载更多
 */
async function handleLoadMore() {
  await searchStore.loadMore()
}

/**
 * 处理发送私信
 */
function handleSendMessage() {
  // 将 UserSearchItem 转换为 User 类型
  if (selectedUserInfo.value) {
    directChatTargetUser.value = {
      id: selectedUserInfo.value.id,
      username: selectedUserInfo.value.username,
      avatar_url: selectedUserInfo.value.avatar_url,
      email: '',
      status: selectedUserInfo.value.status,
      is_active: true,
      role: 'user',
      created_at: '',
    }
    showStartDirectChat.value = true
  }
}

/**
 * 处理私聊创建成功
 */
function handleDirectChatStarted(roomId: string) {
  router.push(`/room/${roomId}`)
}

/**
 * 处理添加好友
 */
async function handleAddFriend(userId: string) {
  await friendStore.sendFriendRequest({ target_user_id: userId })
}

/**
 * 初始化
 */
onMounted(async () => {
  // 加载推荐房间
  await searchStore.fetchPublicRooms(20)
})
</script>

<template>
  <div class="discover-view">
    <!-- 页面头部 -->
    <div class="discover-header">
      <div class="header-title">
        <Compass :size="24" />
        <h1>发现</h1>
      </div>
      <p class="header-subtitle">探索公开房间，发现有趣的人</p>
    </div>

    <!-- 搜索栏 -->
    <div class="discover-search">
      <UniversalSearchBar
        v-model="searchStore.keyword"
        :search-type="searchStore.searchType"
        :loading="searchStore.loading"
        :history="searchStore.history"
        :placeholder="searchPlaceholder"
        @search="handleSearch"
        @change-type="handleSearchTypeChange"
        @select-history="handleSelectHistory"
        @clear-history="searchStore.clearHistory"
        @remove-history="searchStore.removeFromHistory"
      />
    </div>

    <!-- 内容区域 -->
    <div class="discover-content">
      <!-- 发现标签页 -->
      <template v-if="activeTab === 'discover' && !isSearching">
        <!-- 推荐房间 -->
        <div class="section">
          <div class="section-header">
            <TrendingUp :size="18" />
            <h2>推荐房间</h2>
          </div>
          <RoomSearchResults
            :rooms="searchStore.roomResults"
            :loading="searchStore.loading"
            @click="handleRoomClick"
          />
        </div>
      </template>

      <!-- 搜索结果标签页 -->
      <template v-else>
        <div class="search-results">
          <!-- 房间搜索结果 -->
          <template v-if="searchStore.searchType === 'room'">
            <div class="results-header">
              <h2>
                "{{ searchStore.keyword }}" 的搜索结果
                <span class="results-count">({{ searchStore.roomTotal }})</span>
              </h2>
            </div>
            <RoomSearchResults
              :rooms="searchStore.roomResults"
              :loading="searchStore.loading"
              :has-more="searchStore.hasMore"
              :keyword="searchStore.keyword"
              @click="handleRoomClick"
              @load-more="handleLoadMore"
            />
          </template>

          <!-- 用户搜索结果 -->
          <template v-else>
            <div class="results-header">
              <h2>
                "{{ searchStore.keyword }}" 的搜索结果
                <span class="results-count">({{ searchStore.userTotal }})</span>
              </h2>
            </div>
            <UserSearchResults
              :users="searchStore.userResults"
              :loading="searchStore.loading"
              :has-more="searchStore.hasMore"
              :keyword="searchStore.keyword"
              @click="handleUserClick"
              @load-more="handleLoadMore"
            />
          </template>
        </div>
      </template>
    </div>

    <!-- 用户资料弹窗 -->
    <UserProfileModal
      v-model:visible="showUserProfile"
      :user-id="selectedUserId"
      :user-info="selectedUserInfo"
      @send-message="handleSendMessage"
      @add-friend="handleAddFriend"
    />

    <!-- 发起私聊弹窗 -->
    <StartDirectChatModal
      :show="showStartDirectChat"
      :target-user="directChatTargetUser"
      @close="showStartDirectChat = false"
      @started="handleDirectChatStarted"
    />
  </div>
</template>

<style scoped>
.discover-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--color-background, #f5f5f5);
}

.discover-header {
  padding: 24px 24px 16px;
  background: var(--color-white, #fff);
  border-bottom: 1px solid var(--color-border, #eee);
}

.header-title {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 8px;
}

.header-title h1 {
  font-size: 24px;
  font-weight: 600;
  color: var(--color-text, #333);
  margin: 0;
}

.header-title :deep(svg) {
  color: var(--color-primary, #2080f0);
}

.header-subtitle {
  font-size: 14px;
  color: var(--color-text-secondary, #666);
  margin: 0;
}

.discover-search {
  padding: 16px 24px;
  background: var(--color-white, #fff);
  border-bottom: 1px solid var(--color-border, #eee);
}

.discover-content {
  flex: 1;
  overflow-y: auto;
  padding: 16px 24px;
}

.section {
  margin-bottom: 24px;
}

.section-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 16px;
}

.section-header h2 {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text, #333);
  margin: 0;
}

.section-header :deep(svg) {
  color: var(--color-primary, #2080f0);
}

.search-results {
  background: var(--color-white, #fff);
  border-radius: 12px;
  overflow: hidden;
}

.results-header {
  padding: 16px 20px;
  border-bottom: 1px solid var(--color-border, #eee);
}

.results-header h2 {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text, #333);
  margin: 0;
}

.results-count {
  font-size: 14px;
  color: var(--color-text-secondary, #666);
  font-weight: normal;
}

/* 响应式适配 */
@media (max-width: 768px) {
  .discover-header {
    padding: 16px 16px 12px;
  }

  .header-title h1 {
    font-size: 20px;
  }

  .discover-search {
    padding: 12px 16px;
  }

  .discover-content {
    padding: 12px 16px;
  }
}
</style>
