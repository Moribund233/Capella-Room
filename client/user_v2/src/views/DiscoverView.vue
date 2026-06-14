<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { useRoomStore } from '@/stores/room'
import { searchApi } from '@/api/search'
import type { Room } from '@/types/room'
import type { UserSearchItem } from '@/types/search'
import {
  Search,
  TrendCharts,
  CollectionTag,
  Lock,
  OfficeBuilding,
  User,
} from '@element-plus/icons-vue'

const router = useRouter()
const { t } = useI18n()
const roomStore = useRoomStore()

const searchQuery = ref('')
const searchActive = ref(false)
const searching = ref(false)
const searchResultsRooms = ref<Room[]>([])
const searchResultsUsers = ref<UserSearchItem[]>([])

const featuredRooms = ref<Room[]>([])
const recentRooms = ref<Room[]>([])
const loadingFeatured = ref(false)
const loadingRecent = ref(false)

let searchTimer: ReturnType<typeof setTimeout> | null = null

async function loadRooms() {
  loadingFeatured.value = true
  loadingRecent.value = true
  try {
    const [featuredRes, recentRes] = await Promise.all([
      searchApi.getPublicRooms({ limit: 6 }),
      searchApi.getRecentPublicRooms({ limit: 10 }),
    ])
    if (featuredRes.success && featuredRes.data) {
      featuredRooms.value = featuredRes.data
    }
    if (recentRes.success && recentRes.data) {
      recentRooms.value = recentRes.data
    }
  } catch {
    console.error('[Discover] Failed to load rooms')
  } finally {
    loadingFeatured.value = false
    loadingRecent.value = false
  }
}

async function doSearch() {
  const q = searchQuery.value.trim()
  if (!q) {
    searchActive.value = false
    return
  }
  searchActive.value = true
  searching.value = true
  try {
    const [roomRes, userRes] = await Promise.all([
      searchApi.searchRooms({ keyword: q, limit: 10 }),
      searchApi.searchUsers({ keyword: q, limit: 10 }),
    ])
    if (roomRes.success && roomRes.data) {
      searchResultsRooms.value = roomRes.data.rooms
    } else {
      searchResultsRooms.value = []
    }
    if (userRes.success && userRes.data) {
      searchResultsUsers.value = userRes.data.users
    } else {
      searchResultsUsers.value = []
    }
  } catch {
    searchResultsRooms.value = []
    searchResultsUsers.value = []
  } finally {
    searching.value = false
  }
}

function onSearchInput() {
  if (searchTimer) clearTimeout(searchTimer)
  searchTimer = setTimeout(doSearch, 400)
}

function clearSearch() {
  searchQuery.value = ''
  searchActive.value = false
  searchResultsRooms.value = []
  searchResultsUsers.value = []
}

async function handleJoinRoom(roomId: string) {
  const ok = await roomStore.joinRoom(roomId)
  if (ok) {
    ElMessage.success(t('chat.joinSuccess'))
    router.push('/app')
  } else {
    ElMessage.error(roomStore.error || t('chat.joinFailed'))
  }
}

function handleViewRoom(roomId: string) {
  roomStore.fetchRoomDetail(roomId)
  router.push('/app')
}

function getInitial(name: string) {
  return name.charAt(0).toUpperCase()
}

onMounted(() => {
  loadRooms()
})
</script>

<template>
  <div class="discover-layout">
    <main class="discover-main">
      <header class="discover-header">
        <div class="header-content">
          <h1 class="discover-title">{{ t('discover.title') }}</h1>
          <p class="discover-subtitle">{{ t('discover.subtitle') }}</p>

          <div class="search-box">
            <el-input
              v-model="searchQuery"
              :placeholder="t('discover.searchPlaceholder')"
              :prefix-icon="Search"
              size="large"
              class="search-input"
              clearable
              @input="onSearchInput"
              @clear="clearSearch"
            />
          </div>
        </div>
      </header>

      <div class="discover-content">
        <!-- 搜索结果 -->
        <section v-if="searchActive" class="content-section">
          <h2 class="section-title">
            <el-icon><Search /></el-icon>
            {{ t('common.search') }}
          </h2>

          <div v-if="searching" class="list-placeholder">{{ t('common.loading') }}</div>

          <template v-else>
            <!-- 房间结果 -->
            <div v-if="searchResultsRooms.length > 0" class="search-group">
              <h3 class="search-group-title">{{ t('chat.rooms') }}</h3>
              <div class="search-results-list">
                <div v-for="room in searchResultsRooms" :key="room.id" class="search-item">
                  <div class="search-item__icon">
                    <el-icon :size="18"><OfficeBuilding /></el-icon>
                  </div>
                  <div class="search-item__body">
                    <span class="search-item__name">{{ room.name }}</span>
                    <span class="search-item__meta">{{ room.member_count }} {{ t('chat.members') }}</span>
                  </div>
                  <div class="search-item__actions">
                    <el-button size="small" type="primary" @click="handleJoinRoom(room.id)">
                      {{ t('discover.join') }}
                    </el-button>
                    <el-button size="small" @click="handleViewRoom(room.id)">
                      {{ t('discover.view') }}
                    </el-button>
                  </div>
                </div>
              </div>
            </div>

            <!-- 用户结果 -->
            <div v-if="searchResultsUsers.length > 0" class="search-group">
              <h3 class="search-group-title">{{ t('friends.title') }}</h3>
              <div class="search-results-list">
                <div v-for="user in searchResultsUsers" :key="user.id" class="search-item">
                  <div class="search-item__avatar">
                    <img v-if="user.avatar_url" :src="user.avatar_url" :alt="user.username" class="avatar-img" />
                    <div v-else class="avatar-placeholder">{{ getInitial(user.username) }}</div>
                  </div>
                  <div class="search-item__body">
                    <span class="search-item__name">{{ user.username }}</span>
                  </div>
                </div>
              </div>
            </div>

            <div v-if="!searching && searchResultsRooms.length === 0 && searchResultsUsers.length === 0" class="list-placeholder">
              {{ t('friends.searchNoResult') }}
            </div>
          </template>
        </section>

        <!-- 推荐房间 -->
        <section v-if="!searchActive" class="content-section">
          <h2 class="section-title">
            <el-icon><TrendCharts /></el-icon>
            {{ t('discover.featured') }}
          </h2>
          <div v-if="loadingFeatured" class="featured-skeleton">
            <div v-for="i in 3" :key="i" class="room-card-skeleton">
              <div class="skeleton-icon" />
              <div class="skeleton-body">
                <div class="skeleton-title" />
                <div class="skeleton-meta" />
                <div class="skeleton-desc" />
              </div>
            </div>
          </div>
          <div v-else-if="featuredRooms.length === 0" class="list-placeholder">
            {{ t('chat.noRooms') }}
          </div>
          <div v-else class="featured-grid">
            <el-card
              v-for="room in featuredRooms"
              :key="room.id"
              class="room-card featured"
              shadow="never"
            >
              <div class="room-card-header">
                <div class="room-icon" :class="{ 'room-icon--private': room.is_private }">
                  <el-icon :size="22"><OfficeBuilding /></el-icon>
                </div>
                <div class="room-info">
                  <h3 class="room-name">{{ room.name }}</h3>
                  <div class="room-meta">
                    <el-icon><User /></el-icon>
                    <span>{{ room.member_count }} {{ t('chat.members') }}</span>
                    <el-icon v-if="!room.is_private"><OfficeBuilding /></el-icon>
                    <el-icon v-else><Lock /></el-icon>
                  </div>
                </div>
              </div>
              <p v-if="room.description" class="room-description">{{ room.description }}</p>
              <div class="room-actions">
                <el-button type="primary" @click="handleJoinRoom(room.id)">
                  {{ t('discover.join') }}
                </el-button>
                <el-button text @click="handleViewRoom(room.id)">
                  {{ t('discover.view') }}
                </el-button>
              </div>
            </el-card>
          </div>
        </section>

        <!-- 最新活跃房间 -->
        <section v-if="!searchActive" class="content-section">
          <h2 class="section-title">
            <el-icon><CollectionTag /></el-icon>
            {{ t('discover.trending') }}
          </h2>
          <div v-if="loadingRecent" class="trending-skeleton">
            <div v-for="i in 4" :key="i" class="room-row-skeleton">
              <div class="skeleton-icon" />
              <div class="skeleton-body">
                <div class="skeleton-title" />
                <div class="skeleton-meta" />
              </div>
            </div>
          </div>
          <div v-else-if="recentRooms.length === 0" class="list-placeholder">
            {{ t('chat.noRooms') }}
          </div>
          <div v-else class="trending-list">
            <el-card
              v-for="room in recentRooms"
              :key="room.id"
              class="room-card trending"
              shadow="never"
            >
              <div class="trending-content">
                <div class="trending-info">
                  <h3 class="room-name">{{ room.name }}</h3>
                  <p v-if="room.description" class="room-description">{{ room.description }}</p>
                  <div class="room-meta">
                    <el-icon><User /></el-icon>
                    <span>{{ room.member_count }} {{ t('chat.members') }}</span>
                  </div>
                </div>
                <el-button type="primary" @click="handleJoinRoom(room.id)">
                  {{ t('discover.join') }}
                </el-button>
              </div>
            </el-card>
          </div>
        </section>
      </div>
    </main>
  </div>
</template>

<style scoped lang="scss">
.discover-layout {
  display: flex;
  height: 100vh;
  background: var(--bg);
  color: var(--fg);
  overflow: hidden;
}

.discover-main {
  flex: 1;
  overflow-y: auto;
}

.discover-header {
  padding: 48px 48px 32px;
  background: linear-gradient(
    180deg,
    color-mix(in oklch, var(--accent) 8%, var(--bg)) 0%,
    var(--bg) 100%
  );
  border-bottom: 1px solid var(--border);
}

.header-content {
  max-width: 960px;
  margin: 0 auto;
}

.discover-title {
  font-family: var(--font-display);
  font-size: 36px;
  font-weight: 700;
  margin: 0 0 8px;
  background: linear-gradient(135deg, var(--accent), var(--accent-pink));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.discover-subtitle {
  font-size: 16px;
  color: var(--muted);
  margin: 0 0 24px;
}

.search-box {
  margin-bottom: 24px;
}

.search-input {
  :deep(.el-input__wrapper) {
    background-color: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    padding: 8px 16px;
    box-shadow: none;

    &.is-focus {
      border-color: var(--accent);
    }
  }

  :deep(.el-input__inner) {
    font-size: 16px;
    height: 40px;
  }
}

.discover-content {
  padding: 32px 48px 48px;
  max-width: 960px;
  margin: 0 auto;
}

.content-section {
  margin-bottom: 48px;

  &:last-child {
    margin-bottom: 0;
  }
}

.section-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 18px;
  font-weight: 600;
  margin: 0 0 20px;
  color: var(--fg);
}

.list-placeholder {
  text-align: center;
  padding: 48px 16px;
  color: var(--muted);
  font-size: 14px;
}

.featured-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 20px;
}

.room-card {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);

  :deep(.el-card__body) {
    padding: 20px;
  }

  &.featured {
    .room-card-header {
      display: flex;
      align-items: flex-start;
      gap: 12px;
      margin-bottom: 12px;
    }

    .room-icon {
      width: 48px;
      height: 48px;
      border-radius: 12px;
      display: grid;
      place-items: center;
      font-size: 20px;
      font-weight: 700;
      color: #fff;
      flex-shrink: 0;
      background: var(--accent);
    }

    .room-info {
      flex: 1;
      min-width: 0;
    }

    .room-name {
      font-size: 16px;
      font-weight: 600;
      margin: 0 0 4px;
    }

    .room-meta {
      display: flex;
      align-items: center;
      gap: 8px;
      font-size: 13px;
      color: var(--muted);

      .el-icon {
        font-size: 14px;
      }
    }

    .room-description {
      font-size: 14px;
      color: var(--muted);
      margin: 0 0 12px;
      line-height: 1.5;
    }

    .room-actions {
      display: flex;
      gap: 8px;
    }
  }
}

.trending-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.room-card.trending {
  :deep(.el-card__body) {
    padding: 16px 20px;
  }

  .trending-content {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .trending-info {
    flex: 1;
    min-width: 0;
  }

  .room-name {
    font-size: 15px;
    font-weight: 600;
    margin: 0 0 4px;
  }

  .room-description {
    font-size: 13px;
    color: var(--muted);
    margin: 0 0 8px;
    line-height: 1.4;
  }

  .room-meta {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 13px;
    color: var(--muted);

    .el-icon {
      font-size: 14px;
    }
  }
}

.search-group {
  margin-bottom: 24px;

  &:last-child {
    margin-bottom: 0;
  }
}

.search-group-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--muted);
  margin: 0 0 12px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.search-results-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.search-item {
  display: flex;
  align-items: center;
  padding: 10px 14px;
  border-radius: 8px;
  background: var(--surface);
  border: 1px solid var(--border);
  gap: 10px;

  &__icon {
    width: 36px;
    height: 36px;
    border-radius: 8px;
    display: grid;
    place-items: center;
    background: var(--accent);
    color: #fff;
    flex-shrink: 0;
  }

  &__avatar {
    flex-shrink: 0;

    .avatar-img, .avatar-placeholder {
      width: 36px;
      height: 36px;
      border-radius: 50%;
      object-fit: cover;
    }

    .avatar-placeholder {
      display: flex;
      align-items: center;
      justify-content: center;
      background: var(--accent);
      color: #fff;
      font-size: 14px;
      font-weight: 600;
    }
  }

  &__body {
    flex: 1;
    min-width: 0;
  }

  &__name {
    display: block;
    font-size: 14px;
    font-weight: 500;
    color: var(--fg);
  }

  &__meta {
    font-size: 12px;
    color: var(--muted);
  }

  &__actions {
    flex-shrink: 0;
    display: flex;
    gap: 6px;
  }
}

.room-icon--private {
  background: var(--accent-pink);
}

// ─── 骨架屏 ─────────────────────────────
.featured-skeleton,
.trending-skeleton {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.room-card-skeleton,
.room-row-skeleton {
  display: flex;
  gap: 14px;
  padding: 16px;
  border-radius: 12px;
  border: 1px solid var(--border, #eee);
  background: var(--surface, #fff);
}

.room-row-skeleton {
  align-items: center;
}

.skeleton-icon {
  width: 44px;
  height: 44px;
  border-radius: 10px;
  background: var(--message-hover, #f0f0f0);
  flex-shrink: 0;
  animation: skeleton-pulse 1.5s ease-in-out infinite;
}

.skeleton-body {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.skeleton-title {
  height: 16px;
  width: 60%;
  border-radius: 4px;
  background: var(--message-hover, #f0f0f0);
  animation: skeleton-pulse 1.5s ease-in-out infinite;
}

.skeleton-meta {
  height: 12px;
  width: 40%;
  border-radius: 4px;
  background: var(--message-hover, #f0f0f0);
  animation: skeleton-pulse 1.5s ease-in-out infinite;
}

.skeleton-desc {
  height: 12px;
  width: 80%;
  border-radius: 4px;
  background: var(--message-hover, #f0f0f0);
  animation: skeleton-pulse 1.5s ease-in-out infinite;
}

@keyframes skeleton-pulse {
  0%, 100% { opacity: 0.4; }
  50% { opacity: 0.8; }
}
</style>
