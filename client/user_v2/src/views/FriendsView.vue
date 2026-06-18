<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useFriendStore } from '@/stores/friend'
import { useDirectRoomStore } from '@/stores/directRoom'
import { useRoomStore } from '@/stores/room'
import type { Friend } from '@/types/friend'
import type { Room } from '@/types/room'
import {
  UserFilled,
  Delete,
  CircleCheck,
  CloseBold,
} from '@element-plus/icons-vue'

type TabType = 'friends' | 'rooms' | 'requests'

const router = useRouter()
const { t } = useI18n()
const friendStore = useFriendStore()
const directRoomStore = useDirectRoomStore()
const roomStore = useRoomStore()

const activeTab = ref<TabType>('friends')

const friends = computed(() => friendStore.friends)
const receivedRequests = computed(() => friendStore.receivedRequests)
const sentRequests = computed(() => friendStore.sentRequests)
const loading = computed(() => friendStore.loading)

const onlineFriends = computed(() =>
  friends.value.filter(f => f.friend.status === 'online')
)
const offlineFriends = computed(() =>
  friends.value.filter(f => f.friend.status !== 'online')
)

const myRooms = computed(() => roomStore.rooms)

// detail panel
const selectedFriend = ref<Friend | null>(null)
const showDetail = ref(false)

function openDetail(friend: Friend) {
  selectedFriend.value = friend
  showDetail.value = true
}

function closeDetail() {
  showDetail.value = false
  selectedFriend.value = null
}

const hasPendingRequests = computed(() =>
  receivedRequests.value.some(r => r.status === 'pending')
)

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
    if (ok) {
      ElMessage.success(t('friends.deleteFriend'))
      closeDetail()
    }
  } catch { }
}

async function handleSendMessage(friend: Friend) {
  const room = await directRoomStore.createOrGetDirectRoom(friend.friend.id)
  if (room) {
    router.push('/app')
  }
}

function handleEnterRoom(roomId: string) {
  roomStore.fetchRoomDetail(roomId)
  router.push('/app')
}

function getInitial(name: string) {
  return name.charAt(0).toUpperCase()
}

function getStatusLabel(status: string) {
  switch (status) {
    case 'online': return '在线'
    case 'away': return '离开'
    case 'busy': return '忙碌'
    default: return '离线'
  }
}

function getStatusTagClass(status: string) {
  switch (status) {
    case 'online': return 'tag-online'
    case 'away': return 'tag-idle'
    case 'busy': return 'tag-busy'
    default: return 'tag-offline'
  }
}

function getDotClass(status: string) {
  switch (status) {
    case 'online': return 'dot-online'
    case 'away': return 'dot-idle'
    case 'busy': return 'dot-busy'
    default: return 'dot-offline'
  }
}

onMounted(() => {
  friendStore.fetchFriends()
  friendStore.fetchReceivedRequests()
  friendStore.fetchSentRequests()
  friendStore.clearUnreadRequestCount()
  roomStore.fetchMyRooms()
})
</script>

<template>
  <div class="social-page">
    <!-- ambient glow -->
    <div class="ambient" />

    <div class="social-inner">
      <!-- masthead -->
      <div class="masthead">
        <div class="masthead-eyebrow">{{ t('social.title') }}</div>
        <h1>
          <span class="grad">{{ t('social.title') }}</span> · 你的人际网络
        </h1>
        <p>管理你的好友和房间，从这里开始每一次对话</p>
      </div>

      <!-- segmented tabs -->
      <div class="segmented">
        <button
          class="seg-item"
          :class="{ active: activeTab === 'friends' }"
          @click="activeTab = 'friends'"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/></svg>
          {{ t('friends.friends') }}
          <span v-if="friends.length > 0" class="seg-badge">{{ friends.length }}</span>
        </button>
        <button
          class="seg-item"
          :class="{ active: activeTab === 'rooms' }"
          @click="activeTab = 'rooms'"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/></svg>
          {{ t('chat.rooms') }}
          <span v-if="myRooms.length > 0" class="seg-badge">{{ myRooms.length }}</span>
        </button>
        <button
          class="seg-item"
          :class="{ active: activeTab === 'requests' }"
          @click="activeTab = 'requests'"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
          {{ t('friends.received') }}
          <span v-if="hasPendingRequests" class="seg-badge seg-badge-warn">{{ receivedRequests.length }}</span>
        </button>
      </div>

      <!-- === FRIENDS TAB === -->
      <div v-if="activeTab === 'friends'" class="tab-content">
        <div v-if="loading" class="list-placeholder">{{ t('common.loading') }}</div>
        <div v-else-if="friends.length === 0" class="empty">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/></svg>
          <p>{{ t('friends.noFriends') }}</p>
        </div>
        <template v-else>
          <div v-if="onlineFriends.length > 0" class="section">
            <div class="section-header">
              <h2 class="section-title">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
                在线 · {{ onlineFriends.length }}
              </h2>
            </div>
            <div class="card-list">
              <div
                v-for="friend in onlineFriends"
                :key="friend.id"
                class="friend-card"
                @click="openDetail(friend)"
              >
                <div class="friend-card-avatar" :class="getDotClass(friend.friend.status)">
                  <span class="avatar-text">{{ getInitial(friend.friend.username) }}</span>
                </div>
                <div class="friend-card-body">
                  <div class="friend-card-name">{{ friend.friend.username }}</div>
                  <div class="friend-card-meta">
                    <span :class="['status-tag', getStatusTagClass(friend.friend.status)]">
                      {{ getStatusLabel(friend.friend.status) }}
                    </span>
                  </div>
                </div>
                <div class="friend-card-actions" @click.stop>
                  <span class="btn btn-primary btn-xs" @click="handleSendMessage(friend)">
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>
                    聊天
                  </span>
                </div>
              </div>
            </div>
          </div>

          <div v-if="offlineFriends.length > 0" class="section">
            <div class="section-header">
              <h2 class="section-title">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
                离线 · {{ offlineFriends.length }}
              </h2>
            </div>
            <div class="card-list">
              <div
                v-for="friend in offlineFriends"
                :key="friend.id"
                class="friend-card"
                @click="openDetail(friend)"
              >
                <div class="friend-card-avatar" :class="getDotClass(friend.friend.status)">
                  <span class="avatar-text">{{ getInitial(friend.friend.username) }}</span>
                </div>
                <div class="friend-card-body">
                  <div class="friend-card-name">{{ friend.friend.username }}</div>
                  <div class="friend-card-meta">
                    <span :class="['status-tag', getStatusTagClass(friend.friend.status)]">
                      {{ getStatusLabel(friend.friend.status) }}
                    </span>
                  </div>
                </div>
                <div class="friend-card-actions" @click.stop>
                  <span class="btn btn-primary btn-xs" @click="handleSendMessage(friend)">
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>
                    聊天
                  </span>
                </div>
              </div>
            </div>
          </div>
        </template>
      </div>

      <!-- === ROOMS TAB === -->
      <div v-if="activeTab === 'rooms'" class="tab-content">
        <div v-if="myRooms.length === 0" class="empty">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/></svg>
          <p>{{ t('chat.noRooms') }}</p>
        </div>
        <div v-else class="card-list">
          <div
            v-for="room in myRooms"
            :key="room.id"
            class="room-card"
          >
            <div class="room-card-icon">
              {{ getInitial(room.name) }}
            </div>
            <div class="room-card-body">
              <div class="room-card-name">{{ room.name }}</div>
              <div class="room-card-meta">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/></svg>
                {{ room.member_count }} 人
              </div>
            </div>
            <div class="friend-card-actions">
              <span class="btn btn-primary btn-xs" @click="handleEnterRoom(room.id)">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4"/><polyline points="10 17 15 12 10 7"/><line x1="15" y1="12" x2="3" y2="12"/></svg>
                进入
              </span>
            </div>
          </div>
        </div>
      </div>

      <!-- === REQUESTS TAB === -->
      <div v-if="activeTab === 'requests'" class="tab-content">
        <div v-if="loading" class="list-placeholder">{{ t('common.loading') }}</div>
        <div v-else-if="receivedRequests.length === 0 && sentRequests.length === 0" class="empty">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
          <p>{{ t('friends.noReceived') }}</p>
        </div>
        <template v-else>
          <div v-if="receivedRequests.length > 0" class="section">
            <div class="section-header">
              <h2 class="section-title">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
                {{ t('friends.received') }}
              </h2>
            </div>
            <div class="card-list">
              <div v-for="req in receivedRequests" :key="req.id" class="request-card">
                <div class="friend-card-avatar dot-online">
                  <span class="avatar-text">{{ getInitial(req.sender.username) }}</span>
                </div>
                <div class="friend-card-body">
                  <div class="friend-card-name">{{ req.sender.username }}</div>
                  <div v-if="req.message" class="request-message">{{ req.message }}</div>
                  <div class="friend-card-meta">
                    <span v-if="req.status === 'pending'" class="status-tag tag-idle">{{ t('friends.waiting') }}</span>
                    <span v-else-if="req.status === 'accepted'" class="status-tag tag-online">{{ t('friends.accepted') }}</span>
                    <span v-else class="status-tag tag-offline">{{ t('friends.rejected') }}</span>
                  </div>
                </div>
                <div v-if="req.status === 'pending'" class="friend-card-actions">
                  <span class="btn btn-primary btn-xs" @click="handleAccept(req.id)">{{ t('friends.accept') }}</span>
                  <span class="btn btn-ghost btn-xs" @click="handleReject(req.id)">{{ t('friends.reject') }}</span>
                </div>
              </div>
            </div>
          </div>

          <div v-if="sentRequests.length > 0" class="section">
            <div class="section-header">
              <h2 class="section-title">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 2L11 13"/><path d="M22 2l-7 20-4-9-9-4 20-7z"/></svg>
                {{ t('friends.sent') }}
              </h2>
            </div>
            <div class="card-list">
              <div v-for="req in sentRequests" :key="req.id" class="request-card">
                <div class="friend-card-avatar dot-offline">
                  <span class="avatar-text">{{ getInitial(req.receiver.username) }}</span>
                </div>
                <div class="friend-card-body">
                  <div class="friend-card-name">{{ req.receiver.username }}</div>
                  <div class="friend-card-meta">
                    <span v-if="req.status === 'pending'" class="status-tag tag-idle">{{ t('friends.pending') }}</span>
                    <span v-else-if="req.status === 'accepted'" class="status-tag tag-online">{{ t('friends.accepted') }}</span>
                    <span v-else class="status-tag tag-offline">{{ t('friends.rejected') }}</span>
                  </div>
                </div>
                <div v-if="req.status === 'pending'" class="friend-card-actions">
                  <span class="btn btn-ghost btn-xs" @click="handleCancel(req.id)">{{ t('friends.cancelRequest') }}</span>
                </div>
              </div>
            </div>
          </div>
        </template>
      </div>
    </div>

    <!-- detail slide-over -->
    <Transition name="slide">
      <div v-if="showDetail" class="detail-overlay" @click="closeDetail" />
    </Transition>
    <Transition name="slide">
      <div v-if="showDetail && selectedFriend" class="detail-panel">
        <div class="detail-header">
          <span class="detail-header-title">{{ t('friends.viewProfile') }}</span>
          <button class="detail-close" @click="closeDetail">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
          </button>
        </div>
        <div class="detail-body">
          <div class="detail-avatar">
            {{ getInitial(selectedFriend.friend.username) }}
          </div>
          <h2 class="detail-name">{{ selectedFriend.friend.username }}</h2>
          <div class="detail-status">
            <span :class="['detail-dot', getDotClass(selectedFriend.friend.status)]" />
            {{ getStatusLabel(selectedFriend.friend.status) }}
          </div>

          <div class="detail-stats">
            <div class="detail-stat">
              <div class="detail-stat-value">{{ myRooms.length }}</div>
              <div class="detail-stat-label">共同房间</div>
            </div>
            <div class="detail-stat">
              <div class="detail-stat-value">—</div>
              <div class="detail-stat-label">消息</div>
            </div>
            <div class="detail-stat">
              <div class="detail-stat-value">—</div>
              <div class="detail-stat-label">成为好友</div>
            </div>
          </div>

          <div class="detail-actions">
            <button class="detail-action-btn primary" @click="handleSendMessage(selectedFriend)">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>
              {{ t('friends.sendMessage') }}
            </button>
            <button class="detail-action-btn danger" @click="handleDeleteFriend(selectedFriend)">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6h18"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
              {{ t('friends.deleteFriend') }}
            </button>
          </div>
        </div>
      </div>
    </Transition>

  </div>
</template>

<style scoped lang="scss">
.social-page {
  height: 100%;
  overflow-y: auto;
  position: relative;
}

.ambient {
  position: fixed;
  top: -30vh;
  left: -10vw;
  width: 60vw;
  height: 60vh;
  background: radial-gradient(ellipse, color-mix(in oklch, var(--accent) 5%, transparent), transparent 70%);
  pointer-events: none;
  z-index: 0;
}

.social-inner {
  max-width: 1200px;
  margin: 0 auto;
  padding: 40px 48px 80px;
  position: relative;
  z-index: 1;
}

.masthead {
  margin-bottom: 28px;
}

.masthead-eyebrow {
  font-size: 13px;
  font-weight: 600;
  color: var(--accent);
  text-transform: uppercase;
  letter-spacing: 2px;
  margin-bottom: 6px;
}

.masthead h1 {
  font-family: var(--font-display);
  font-size: clamp(28px, 3.5vw, 38px);
  font-weight: 700;
  margin: 0 0 6px;
  letter-spacing: -0.03em;
  line-height: 1.08;
}

.masthead .grad {
  background: linear-gradient(135deg, var(--accent-pink) 0%, var(--accent) 60%, var(--accent-blue) 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.masthead p {
  font-size: 15px;
  color: var(--muted);
  margin: 0;
  max-width: 420px;
}

/* ─── Segmented tabs ─── */
.segmented {
  display: inline-flex;
  gap: 4px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 4px;
  margin-bottom: 28px;
}

.seg-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 18px;
  border-radius: 8px;
  border: none;
  background: transparent;
  color: var(--muted);
  font-size: 14px;
  font-weight: 500;
  font-family: var(--font-body);
  cursor: pointer;
  transition: all 0.15s;
  white-space: nowrap;
}

.seg-item:hover {
  color: var(--fg);
}

.seg-item.active {
  background: var(--accent);
  color: #fff;
}

.seg-item svg {
  width: 16px;
  height: 16px;
}

.seg-badge {
  font-size: 11px;
  font-weight: 600;
  background: var(--accent-soft);
  color: var(--accent);
  padding: 1px 7px;
  border-radius: var(--radius-full);
}

.seg-item.active .seg-badge {
  background: color-mix(in oklch, white 20%, transparent);
  color: #fff;
}

.seg-badge-warn {
  background: color-mix(in oklch, var(--accent-orange) 18%, transparent);
  color: var(--accent-orange);
}

.seg-item.active .seg-badge-warn {
  background: color-mix(in oklch, white 20%, transparent);
  color: #fff;
}

/* ─── Sections ─── */
.tab-content {
  min-height: 200px;
}

.section {
  margin-bottom: 28px;
}

.section:last-child {
  margin-bottom: 0;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  margin: 0;
  color: var(--muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  display: flex;
  align-items: center;
  gap: 6px;
}

.section-title svg {
  width: 14px;
  height: 14px;
}

/* ─── Card list ─── */
.card-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

/* ─── Friend card ─── */
.friend-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  transition: all 0.15s;
  cursor: default;
}

.friend-card:hover {
  border-color: var(--accent);
  background: color-mix(in oklch, var(--accent) 4%, var(--surface));
}

.friend-card-avatar {
  width: 44px;
  height: 44px;
  border-radius: 50%;
  display: grid;
  place-items: center;
  font-size: 16px;
  font-weight: 600;
  color: #fff;
  flex-shrink: 0;
  position: relative;
  background: linear-gradient(135deg, var(--accent), var(--accent-pink));
}

.friend-card-avatar::after {
  content: '';
  position: absolute;
  bottom: 1px;
  right: 1px;
  width: 11px;
  height: 11px;
  border-radius: 50%;
  border: 2px solid var(--surface);
}

.dot-online::after { background: var(--accent-green); }
.dot-idle::after { background: var(--accent-orange); }
.dot-busy::after { background: var(--accent-pink); }
.dot-offline::after { background: var(--muted); }

.avatar-text {
  font-size: 16px;
  font-weight: 600;
}

.friend-card-body {
  flex: 1;
  min-width: 0;
}

.friend-card-name {
  font-size: 15px;
  font-weight: 600;
  margin: 0 0 2px;
}

.friend-card-meta {
  font-size: 13px;
  color: var(--muted);
  display: flex;
  align-items: center;
  gap: 8px;
}

.friend-card-actions {
  display: flex;
  gap: 6px;
  flex-shrink: 0;
  opacity: 0;
  transition: opacity 0.15s;
}

.friend-card:hover .friend-card-actions {
  opacity: 1;
}

/* ─── Status tags ─── */
.status-tag {
  font-size: 11px;
  padding: 1px 8px;
  border-radius: var(--radius-full);
}

.tag-online { background: color-mix(in oklch, var(--accent-green) 15%, transparent); color: var(--accent-green); }
.tag-idle { background: color-mix(in oklch, var(--accent-orange) 15%, transparent); color: var(--accent-orange); }
.tag-busy { background: color-mix(in oklch, var(--accent-pink) 15%, transparent); color: var(--accent-pink); }
.tag-offline { background: var(--message-hover); color: var(--muted); }

/* ─── Room card ─── */
.room-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  transition: all 0.15s;
  cursor: default;
}

.room-card:hover {
  border-color: var(--accent);
  background: color-mix(in oklch, var(--accent) 4%, var(--surface));
}

.room-card:hover .friend-card-actions {
  opacity: 1;
}

.room-card-icon {
  width: 44px;
  height: 44px;
  border-radius: var(--radius);
  display: grid;
  place-items: center;
  font-size: 18px;
  font-weight: 700;
  color: #fff;
  flex-shrink: 0;
  background: linear-gradient(135deg, var(--accent), var(--accent-pink));
}

.room-card-body {
  flex: 1;
  min-width: 0;
}

.room-card-name {
  font-size: 15px;
  font-weight: 600;
  margin: 0 0 2px;
}

.room-card-meta {
  font-size: 13px;
  color: var(--muted);
  display: flex;
  align-items: center;
  gap: 6px;
}

.room-card-meta svg {
  width: 14px;
  height: 14px;
}

/* ─── Request card ─── */
.request-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  transition: all 0.15s;
}

.request-card:hover {
  border-color: var(--accent);
}

.request-message {
  font-size: 12px;
  color: var(--muted);
  margin: 0 0 2px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* ─── Detail panel ─── */
.detail-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  z-index: 100;
}

.detail-panel {
  position: fixed;
  top: 0;
  right: 0;
  width: 380px;
  height: 100vh;
  background: var(--surface);
  border-left: 1px solid var(--border);
  z-index: 101;
  display: flex;
  flex-direction: column;
}

.detail-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 24px 24px 0;
}

.detail-header-title {
  font-size: 16px;
  font-weight: 600;
}

.detail-close {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  border: 1px solid var(--border);
  background: transparent;
  color: var(--muted);
  cursor: pointer;
  display: grid;
  place-items: center;
  transition: all 0.15s;
}

.detail-close:hover {
  border-color: var(--fg);
  color: var(--fg);
}

.detail-body {
  flex: 1;
  overflow-y: auto;
  padding: 24px;
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
}

.detail-avatar {
  width: 80px;
  height: 80px;
  border-radius: 50%;
  display: grid;
  place-items: center;
  font-size: 28px;
  font-weight: 600;
  color: #fff;
  margin-bottom: 16px;
  background: linear-gradient(135deg, var(--accent), var(--accent-pink));
}

.detail-name {
  font-size: 22px;
  font-weight: 600;
  margin: 0 0 6px;
}

.detail-status {
  font-size: 14px;
  color: var(--muted);
  margin: 0 0 24px;
  display: flex;
  align-items: center;
  gap: 6px;
}

.detail-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  display: inline-block;
}

.detail-dot.dot-online { background: var(--accent-green); }
.detail-dot.dot-idle { background: var(--accent-orange); }
.detail-dot.dot-busy { background: var(--accent-pink); }
.detail-dot.dot-offline { background: var(--muted); }

.detail-stats {
  display: grid;
  grid-template-columns: 1fr 1fr 1fr;
  gap: 16px;
  width: 100%;
  margin-bottom: 24px;
}

.detail-stat {
  text-align: center;
  padding: 12px;
  border-radius: var(--radius);
  background: var(--bg);
}

.detail-stat-value {
  font-size: 18px;
  font-weight: 700;
  margin-bottom: 2px;
}

.detail-stat-label {
  font-size: 12px;
  color: var(--muted);
}

.detail-actions {
  display: flex;
  flex-direction: column;
  gap: 10px;
  width: 100%;
}

.detail-action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 12px;
  border-radius: var(--radius);
  border: 1px solid var(--border);
  background: transparent;
  color: var(--fg);
  font-size: 15px;
  font-weight: 500;
  font-family: var(--font-body);
  cursor: pointer;
  transition: all 0.15s;
}

.detail-action-btn:hover {
  border-color: var(--accent);
  background: var(--accent-soft);
}

.detail-action-btn.primary {
  background: var(--accent);
  border-color: var(--accent);
  color: #fff;
}

.detail-action-btn.primary:hover {
  background: color-mix(in oklch, var(--accent) 85%, black);
}

.detail-action-btn.danger {
  color: var(--accent-pink);
}

.detail-action-btn.danger:hover {
  border-color: var(--accent-pink);
  background: color-mix(in oklch, var(--accent-pink) 10%, transparent);
}

.detail-action-btn svg {
  width: 18px;
  height: 18px;
}

/* ─── Transitions ─── */
.slide-enter-active,
.slide-leave-active {
  transition: all 0.25s ease;
}

.slide-enter-from,
.slide-leave-to {
  opacity: 0;
}

.slide-enter-from .detail-panel,
.slide-leave-to .detail-panel {
  transform: translateX(100%);
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
  padding: 64px 16px;
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
  .social-inner {
    padding: 24px 16px 80px;
  }

  .detail-panel {
    width: 100vw;
  }
}
</style>
