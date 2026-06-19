<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { useRoomStore } from '@/stores/room'
import { searchApi } from '@/api/search'
import { roomApi } from '@/api/room'
import { userApi } from '@/api/user'
import type { Room } from '@/types/room'
import type { UserSearchItem } from '@/types/search'
import type { UserInfo } from '@/types/user'
import { getAvatarGradient, getAvatarShadow } from '@/utils/avatar'
import { Search, User, UserFilled, Clock, Grid, TrendCharts } from '@element-plus/icons-vue'

const router = useRouter()
const { t } = useI18n()
const roomStore = useRoomStore()

const recommendedUsers = ref<UserInfo[]>([])

const searchQuery = ref('')
const searchActive = ref(false)
const searching = ref(false)
const searchResultsRooms = ref<Room[]>([])
const searchResultsUsers = ref<UserSearchItem[]>([])

const featuredRooms = ref<Room[]>([])
const recentRooms = ref<Room[]>([])
const loadingFeatured = ref(false)
const loadingRecent = ref(false)
const loadingRecommended = ref(false)

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
    console.error('[Explore] Failed to load rooms')
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
    if (roomRes.success && Array.isArray(roomRes.data)) {
      searchResultsRooms.value = roomRes.data as Room[]
    } else if (roomRes.success && roomRes.data && 'rooms' in roomRes.data) {
      searchResultsRooms.value = (roomRes.data as { rooms: Room[] }).rooms
    } else {
      searchResultsRooms.value = []
    }
    if (userRes.success && userRes.data) {
      searchResultsUsers.value = userRes.data.users || []
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

function getRoomCount(room: Room): number {
  return room.member_count || 0
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

async function handleChat(userId: string) {
  const res = await roomApi.createDirectRoom({ target_user_id: userId })
  if (res.success && res.data) {
    await roomStore.joinRoom(res.data.id)
    router.push('/app')
  } else {
    ElMessage.error(t('discover.createPrivateFailed'))
  }
}

function formatOnline(room: Room): string {
  const count = getRoomCount(room)
  if (count >= 1000) return `${Math.floor(count / 1000)}k ${t('discover.onlineCount')}`
  return `${count} ${t('discover.onlineCount')}`
}

// 使用统一的渐变工具函数
function getRoomGradientLocal(name: string) {
  return { background: getAvatarGradient(name) }
}

function getBannerGradientLocal(name: string) {
  return { background: getAvatarGradient(name + '_banner') }
}

function getUserGradientLocal(name: string) {
  return { background: getAvatarGradient(name + '_user') }
}

onMounted(async () => {
  loadRooms()
  loadRecommendedUsers()
})

async function loadRecommendedUsers() {
  loadingRecommended.value = true
  try {
    const res = await userApi.getRecommendedUsers(12)
    if (res.success && res.data) {
      recommendedUsers.value = res.data
    }
  } catch {
    console.error('[Explore] Failed to load recommended users')
  } finally {
    loadingRecommended.value = false
  }
}
</script>

<template>
  <div class="explore-page">
    <div class="ambient" />

    <div class="explore-inner">
      <!-- masthead -->
      <div class="masthead">
        <div class="masthead-eyebrow">{{ t('discover.title') }}</div>
        <h1>{{ t('discover.findYour') }}<br /><span class="grad">{{ t('discover.nextBase') }}</span></h1>
        <p>{{ t('discover.subtitle') }}</p>

        <div class="search-box">
          <el-icon :size="20"><Search /></el-icon>
          <input
            v-model="searchQuery"
            type="text"
            :placeholder="t('discover.searchPlaceholder')"
            @input="onSearchInput"
          />
        </div>
      </div>

      <!-- search results -->
      <template v-if="searchActive">
        <div class="content-section">
          <div class="section-header">
            <h2 class="section-title">
              <el-icon :size="18"><Search /></el-icon>
              {{ t('discover.searchResults') }} "{{ searchQuery }}"
            </h2>
            <span class="section-action" @click="clearSearch">{{ t('discover.clearSearch') }}</span>
          </div>

          <div v-if="searching" class="list-placeholder">{{ t('common.loading') }}</div>
          <template v-else>
            <div v-if="searchResultsRooms.length > 0" class="carousel">
              <div
                v-for="room in searchResultsRooms"
                :key="room.id"
                class="hero-card"
              >
                <div class="hero-card-banner" :style="getBannerGradientLocal(room.name)">
                  <div class="hero-card-icon" :style="getRoomGradientLocal(room.name)">{{ getInitial(room.name) }}</div>
                </div>
                <div class="hero-card-body">
                  <h3 class="hero-card-name">{{ room.name }}</h3>
                  <p v-if="room.description" class="hero-card-desc">{{ room.description }}</p>
                </div>
                <hr class="glow-divider" />
                <div class="hero-card-footer">
                  <div class="hero-card-stats">
                    <el-icon :size="14"><User /></el-icon>
                    {{ getRoomCount(room) }}
                  </div>
                  <span class="btn btn-primary btn-xs" @click="handleJoinRoom(room.id)">{{ t('discover.join') }}</span>
                </div>
              </div>
            </div>

            <div v-if="searchResultsUsers.length > 0" class="search-users-section">
              <h3 class="search-group-title">{{ t('discover.users') }}</h3>
              <div class="card-list">
                <div
                  v-for="user in searchResultsUsers"
                  :key="user.id"
                  class="search-user-row"
                >
                  <div class="search-user-avatar" :style="getUserGradientLocal(user.username)">{{ getInitial(user.username) }}</div>
                  <span class="search-user-name">{{ user.username }}</span>
                </div>
              </div>
            </div>

            <div v-if="!searching && searchResultsRooms.length === 0 && searchResultsUsers.length === 0" class="empty">
              <el-icon :size="40"><Search /></el-icon>
              <p>{{ t('discover.noResult') }}</p>
            </div>
          </template>
        </div>
      </template>

      <!-- browse mode -->
      <template v-else>
        <!-- 热门房间 -->
        <div class="content-section">
          <div class="section-header">
            <h2 class="section-title">
              <el-icon :size="18"><TrendCharts /></el-icon>
              {{ t('discover.trendingRooms') }}
            </h2>
            <span class="section-action">{{ t('discover.viewMore') }} →</span>
          </div>

          <div v-if="loadingFeatured" class="list-placeholder">{{ t('common.loading') }}</div>
          <div v-else-if="featuredRooms.length === 0" class="empty">
            <el-icon :size="40"><Grid /></el-icon>
            <p>{{ t('chat.noRooms') }}</p>
          </div>
          <div v-else class="trending-stack">
            <div
              v-for="(room, idx) in featuredRooms.slice(0, 6)"
              :key="room.id"
              class="trending-card"
            >
              <span class="trending-rank" :class="`r${idx + 1}`">{{ idx + 1 }}</span>
              <div class="trending-icon" :style="getRoomGradientLocal(room.name)">{{ getInitial(room.name) }}</div>
              <div class="trending-body">
                <div class="trending-name">{{ room.name }}</div>
                <div class="trending-online">
                  <el-icon :size="12"><Clock /></el-icon>
                  {{ formatOnline(room) }}
                </div>
              </div>
              <span class="btn btn-primary btn-xs" @click="handleJoinRoom(room.id)">{{ t('discover.join') }}</span>
            </div>
          </div>
        </div>

        <!-- 可能感兴趣的人 -->
        <div class="content-section">
          <div class="section-header">
            <h2 class="section-title">
              <el-icon :size="18"><UserFilled /></el-icon>
              {{ t('discover.recommendedUsers') }}
            </h2>
          </div>

          <div v-if="loadingRecommended" class="list-placeholder">{{ t('common.loading') }}</div>
          <div v-else-if="recommendedUsers.length === 0" class="empty" style="padding: 28px 16px;">
            <el-icon :size="40"><UserFilled /></el-icon>
            <p>{{ t('discover.noRecommendedUsers') }}</p>
          </div>
          <div v-else class="user-carousel">
            <div
              v-for="user in recommendedUsers"
              :key="user.id"
              class="user-card"
            >
              <div class="user-card-avatar" :style="getUserGradientLocal(user.username)">
                <div class="user-card-online" />
                {{ getInitial(user.username) }}
              </div>
              <div class="user-card-name">{{ user.username }}</div>
              <span class="btn btn-primary btn-xs" @click="handleChat(user.id)">{{ t('discover.sendMessage') }}</span>
            </div>
          </div>
        </div>

        <!-- 最近活跃 -->
        <div class="content-section">
          <div class="section-header">
            <h2 class="section-title">
              <el-icon :size="18"><TrendCharts /></el-icon>
              {{ t('discover.recentActive') }}
            </h2>
          </div>

          <div v-if="loadingRecent" class="list-placeholder">{{ t('common.loading') }}</div>
          <div v-else-if="recentRooms.length === 0" class="empty">
            <el-icon :size="40"><TrendCharts /></el-icon>
            <p>{{ t('chat.noRooms') }}</p>
          </div>
          <div v-else class="carousel">
            <div
              v-for="room in recentRooms"
              :key="room.id"
              class="compact-card"
            >
              <div class="compact-card-header">
                <div class="compact-card-icon" :style="getRoomGradientLocal(room.name)">{{ getInitial(room.name) }}</div>
                <div class="compact-card-info">
                  <div class="compact-card-name">{{ room.name }}</div>
                  <div class="compact-card-meta">{{ getRoomCount(room) }} {{ t('discover.members') }}</div>
                </div>
              </div>
              <p v-if="room.description" class="compact-card-desc">{{ room.description }}</p>
              <div class="compact-card-actions">
                <span class="btn btn-primary btn-xs" @click="handleJoinRoom(room.id)">{{ t('discover.join') }}</span>
                <span class="btn btn-ghost btn-xs" @click="handleViewRoom(room.id)">{{ t('discover.preview') }}</span>
              </div>
            </div>
          </div>
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped lang="scss">
.explore-page {
  height: 100%;
  overflow-y: auto;
  position: relative;
}

.ambient {
  position: fixed;
  top: -40vh;
  right: -20vw;
  width: 80vw;
  height: 80vh;
  background: radial-gradient(ellipse, color-mix(in oklch, var(--accent) 6%, transparent), transparent 70%);
  pointer-events: none;
  z-index: 0;
}

.explore-inner {
  max-width: 1200px;
  margin: 0 auto;
  padding: 40px 48px 80px;
  position: relative;
  z-index: 1;
}

/* ─── Masthead ─── */
.masthead {
  margin-bottom: 32px;
}

.masthead-eyebrow {
  font-size: 13px;
  font-weight: 600;
  color: var(--accent);
  text-transform: uppercase;
  letter-spacing: 2px;
  margin-bottom: 8px;
}

.masthead h1 {
  font-family: var(--font-display);
  font-size: clamp(36px, 4.5vw, 48px);
  font-weight: 700;
  margin: 0 0 6px;
  letter-spacing: -0.03em;
  line-height: 1.05;
}

.masthead .grad {
  background: linear-gradient(135deg, var(--accent) 0%, var(--accent-pink) 50%, var(--accent-orange) 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.masthead p {
  font-size: 16px;
  color: var(--muted);
  margin: 0 0 28px;
  max-width: 480px;
}

/* ─── Search ─── */
.search-box {
  display: flex;
  align-items: center;
  gap: 12px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: 14px 22px;
  max-width: 600px;
  transition: border-color 0.25s, box-shadow 0.25s;
}

.search-box:focus-within {
  border-color: var(--accent);
  box-shadow: 0 0 0 4px var(--accent-soft);
}

.search-box svg {
  width: 20px;
  height: 20px;
  color: var(--muted);
  flex-shrink: 0;
}

.search-box input {
  flex: 1;
  border: none;
  background: transparent;
  color: var(--fg);
  font-size: 16px;
  font-family: var(--font-body);
  outline: none;
}

.search-box input::placeholder {
  color: var(--muted);
}

/* ─── Sections ─── */
.content-section {
  margin-bottom: 48px;
}

.content-section:last-child {
  margin-bottom: 0;
}

.section-header {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  margin-bottom: 20px;
}

.section-title {
  font-size: 18px;
  font-weight: 600;
  margin: 0;
  display: flex;
  align-items: center;
  gap: 8px;
}

.section-title svg {
  width: 20px;
  height: 20px;
}

.section-action {
  font-size: 14px;
  color: var(--muted);
  cursor: pointer;
  transition: color 0.15s;
}

.section-action:hover {
  color: var(--accent);
}

/* ─── Carousel ─── */
.carousel {
  display: flex;
  gap: 14px;
  overflow-x: auto;
  padding: 4px 0 8px;
  scroll-snap-type: x mandatory;
}

.carousel > * {
  scroll-snap-align: start;
  flex-shrink: 0;
}

/* ─── Hero card ─── */
.hero-card {
  width: 340px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  overflow: hidden;
  transition: all 0.25s;
  cursor: default;
  flex-shrink: 0;
}

.hero-card:hover {
  border-color: var(--accent);
  transform: translateY(-3px);
  box-shadow: 0 8px 32px rgba(124, 92, 252, 0.15);
}

.hero-card-banner {
  height: 90px;
  background: linear-gradient(135deg, var(--accent), var(--accent-blue));
  position: relative;
}

.hero-card-icon {
  position: absolute;
  bottom: -20px;
  left: 20px;
  width: 48px;
  height: 48px;
  border-radius: var(--radius);
  background: var(--surface);
  border: 2px solid var(--border);
  display: grid;
  place-items: center;
  font-size: 20px;
  font-weight: 700;
}

.hero-card-body {
  padding: 28px 20px 12px;
}

.hero-card-name {
  font-size: 17px;
  font-weight: 600;
  margin: 0 0 6px;
}

.hero-card-desc {
  font-size: 13px;
  color: var(--muted);
  margin: 0;
  line-height: 1.5;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.glow-divider {
  height: 1px;
  border: none;
  background: linear-gradient(90deg, transparent, var(--accent), transparent);
  opacity: 0.2;
  margin: 0;
}

.hero-card-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 20px 16px;
}

.hero-card-stats {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: var(--muted);
}

.hero-card-stats svg {
  width: 14px;
  height: 14px;
}

/* ─── Search users ─── */
.search-users-section {
  margin-top: 20px;
}

.search-group-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin: 0 0 12px;
}

.card-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.search-user-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
}

.search-user-avatar {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  display: grid;
  place-items: center;
  color: #fff;
  font-size: 14px;
  font-weight: 600;
  flex-shrink: 0;
}

.search-user-name {
  font-size: 14px;
  font-weight: 500;
}

/* ─── User carousel (recommendations) ─── */
.user-carousel {
  display: flex;
  gap: 14px;
  overflow-x: auto;
  padding: 4px 0 8px;
  scroll-snap-type: x mandatory;
}

.user-card {
  scroll-snap-align: start;
  flex-shrink: 0;
  width: 140px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: 20px 14px 14px;
  text-align: center;
  transition: all 0.2s;
  cursor: default;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
}

.user-card:hover {
  border-color: var(--accent);
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(124, 92, 252, 0.12);
}

.user-card-avatar {
  position: relative;
  width: 48px;
  height: 48px;
  border-radius: 50%;
  display: grid;
  place-items: center;
  font-size: 18px;
  font-weight: 700;
  color: #fff;
  margin-bottom: 4px;
}

.user-card-online {
  position: absolute;
  bottom: 0;
  right: 0;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: var(--accent-green);
  border: 2px solid var(--surface);
}

.user-card-name {
  font-size: 14px;
  font-weight: 600;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 100%;
}

.user-card-status {
  font-size: 11px;
  color: var(--muted);
  margin-bottom: 4px;
}

/* ─── Trending stack ─── */
.trending-stack {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
}

.trending-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 16px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  transition: all 0.2s;
  cursor: default;
}

.trending-card:hover {
  border-color: var(--accent);
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(124, 92, 252, 0.12);
}

.trending-rank {
  width: 24px;
  font-size: 14px;
  font-weight: 700;
  color: var(--muted);
  text-align: center;
  flex-shrink: 0;
}

.trending-rank.r1 { color: var(--accent-orange); }
.trending-rank.r2 { color: var(--accent); }
.trending-rank.r3 { color: var(--accent-blue); }

.trending-icon {
  width: 36px;
  height: 36px;
  border-radius: 8px;
  display: grid;
  place-items: center;
  font-size: 13px;
  font-weight: 600;
  color: #fff;
  flex-shrink: 0;
}

.trending-body {
  flex: 1;
  min-width: 0;
}

.trending-name {
  font-size: 14px;
  font-weight: 500;
  margin: 0 0 2px;
}

.trending-online {
  font-size: 12px;
  color: var(--muted);
  display: flex;
  align-items: center;
  gap: 4px;
}

.trending-online svg {
  width: 12px;
  height: 12px;
}

/* ─── Compact card ─── */
.compact-card {
  width: 260px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 16px;
  transition: all 0.2s;
  cursor: default;
  flex-shrink: 0;
}

.compact-card:hover {
  border-color: var(--accent);
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(124, 92, 252, 0.12);
}

.compact-card-header {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 10px;
}

.compact-card-icon {
  width: 36px;
  height: 36px;
  border-radius: 8px;
  display: grid;
  place-items: center;
  font-size: 14px;
  font-weight: 600;
  color: #fff;
  flex-shrink: 0;
}

.compact-card-info {
  flex: 1;
  min-width: 0;
}

.compact-card-name {
  font-size: 14px;
  font-weight: 600;
  margin: 0 0 2px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.compact-card-meta {
  font-size: 12px;
  color: var(--muted);
}

.compact-card-desc {
  font-size: 12px;
  color: var(--muted);
  margin: 0 0 10px;
  line-height: 1.4;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.compact-card-actions {
  display: flex;
  gap: 6px;
}

/* ─── Buttons ─── */
.btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 7px 16px;
  border-radius: var(--radius-full);
  border: 1px solid transparent;
  font-size: 13px;
  font-weight: 500;
  font-family: var(--font-body);
  cursor: pointer;
  transition: all 0.12s;
}

.btn:active {
  transform: translateY(1px);
}

.btn-primary {
  background: var(--accent);
  color: #fff;
  border-color: var(--accent);
}

.btn-primary:hover {
  background: color-mix(in oklch, var(--accent) 85%, black);
}

.btn-ghost {
  background: transparent;
  color: var(--muted);
  border-color: var(--border);
}

.btn-ghost:hover {
  border-color: var(--fg);
  color: var(--fg);
}

.btn-xs {
  padding: 4px 10px;
  font-size: 11px;
  gap: 4px;
}

/* ─── Empty / placeholder ─── */
.empty {
  text-align: center;
  padding: 48px 16px;
  color: var(--muted);
}

.empty svg {
  width: 40px;
  height: 40px;
  margin: 0 auto 12px;
  opacity: 0.2;
}

.empty p {
  margin: 0;
  font-size: 14px;
}

.list-placeholder {
  text-align: center;
  padding: 48px 16px;
  color: var(--muted);
  font-size: 14px;
}

/* ─── Responsive ─── */
@media (max-width: 820px) {
  .explore-inner {
    padding: 24px 16px 48px;
  }

  .trending-stack {
    grid-template-columns: 1fr;
  }

  .hero-card {
    width: 280px;
  }

  .compact-card {
    width: 240px;
  }
}
</style>
