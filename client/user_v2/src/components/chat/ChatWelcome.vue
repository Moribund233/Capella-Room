<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import { useAuthStore } from '@/stores/auth'
import { useRoomStore } from '@/stores/room'
import { useDirectRoomStore } from '@/stores/directRoom'

const authStore = useAuthStore()
const roomStore = useRoomStore()
const directRoomStore = useDirectRoomStore()

const { user } = storeToRefs(authStore)
const { rooms, loading: roomsLoading } = storeToRefs(roomStore)
const { directRooms } = storeToRefs(directRoomStore)

const greeting = computed(() => {
  const h = new Date().getHours()
  if (h < 6) return '凌晨好'
  if (h < 12) return '上午好'
  if (h < 14) return '中午好'
  if (h < 17) return '下午好'
  return '晚上好'
})

const username = computed(() => user.value?.username ?? '')

const totalRooms = computed(() => rooms.value.length + directRooms.value.length)

const totalUnread = computed(() => {
  let n = 0
  for (const r of rooms.value) n += r.unread_count || 0
  for (const r of directRooms.value) n += r.unread_count || 0
  return n
})

const activeRooms = computed(() => {
  let n = 0
  for (const r of rooms.value) if (r.last_message) n++
  for (const r of directRooms.value) if (r.last_message) n++
  return n
})

interface RecentItem {
  id: string
  name: string
  prefix: string
  unreadCount: number
  lastContent: string
  lastSender: string
  lastTime: string
}

const recentItems = computed<RecentItem[]>(() => {
  const items: RecentItem[] = [
    ...rooms.value.map(r => ({
      id: r.id,
      name: r.name,
      prefix: r.is_private ? '🔒' : '#',
      unreadCount: r.unread_count || 0,
      lastContent: r.last_message?.content ?? '',
      lastSender: r.last_message?.sender_name ?? '',
      lastTime: r.last_message?.created_at ?? '',
    })),
    ...directRooms.value.map(r => ({
      id: r.id,
      name: r.target_user.username,
      prefix: '💬',
      unreadCount: r.unread_count || 0,
      lastContent: r.last_message?.content ?? '',
      lastSender: r.last_message?.sender_name ?? '',
      lastTime: r.last_message?.created_at ?? '',
    })),
  ]
  return items
    .filter(i => i.lastTime)
    .sort((a, b) => new Date(b.lastTime).getTime() - new Date(a.lastTime).getTime())
    .slice(0, 10)
})

function formatTime(dateStr: string): string {
  const d = new Date(dateStr)
  const diff = Date.now() - d.getTime()
  const mins = Math.floor(diff / 60000)
  if (mins < 1) return '刚刚'
  if (mins < 60) return `${mins}分钟前`
  const hours = Math.floor(mins / 60)
  if (hours < 24) return `${hours}小时前`
  const days = Math.floor(hours / 24)
  if (days < 7) return `${days}天前`
  return `${d.getMonth() + 1}/${d.getDate()}`
}
</script>

<template>
  <div class="chat-welcome">
    <div class="chat-welcome__inner">
      <div class="chat-welcome__greeting">
        <span class="chat-welcome__greeting-text">{{ greeting }}，{{ username }}</span>
      </div>

      <p class="chat-welcome__summary">
        当前共 <strong>{{ totalRooms }}</strong> 个房间，<strong>{{ totalUnread }}</strong> 条未读消息
      </p>

      <div v-if="totalRooms > 0" class="chat-welcome__stats">
        <div class="stat-card">
          <div class="stat-card__value">{{ totalRooms }}</div>
          <div class="stat-card__label">所有房间</div>
        </div>
        <div class="stat-card">
          <div class="stat-card__value">{{ totalUnread }}</div>
          <div class="stat-card__label">未读消息</div>
        </div>
        <div class="stat-card">
          <div class="stat-card__value">{{ activeRooms }}</div>
          <div class="stat-card__label">近期活跃</div>
        </div>
      </div>

      <div v-if="recentItems.length > 0" class="chat-welcome__recent">
        <div class="chat-welcome__section-title">最近动态</div>
        <div class="recent-list">
          <div
            v-for="(item, idx) in recentItems"
            :key="item.id"
            class="recent-item"
          >
            <div class="recent-item__top">
              <span class="recent-item__prefix">{{ item.prefix }}</span>
              <span class="recent-item__name">{{ item.name }}</span>
              <span v-if="item.unreadCount > 0" class="recent-item__badge">
                {{ item.unreadCount > 99 ? '99+' : item.unreadCount }}
              </span>
            </div>
            <div class="recent-item__bottom">
              <span class="recent-item__preview">
                <span class="recent-item__sender">{{ item.lastSender }}：</span>
                {{ item.lastContent }}
              </span>
              <span class="recent-item__time">{{ formatTime(item.lastTime) }}</span>
            </div>
          </div>
        </div>
        <div class="chat-welcome__recent-footer">
          共 {{ recentItems.length }} 条动态
        </div>
      </div>

      <div v-else-if="!roomsLoading" class="chat-welcome__empty">
        <p>暂无动态，从侧边栏选择一个房间开始聊天</p>
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">
.chat-welcome {
  flex: 1;
  display: flex;
  align-items: flex-start;
  justify-content: center;
  height: 100%;
  overflow-y: auto;
  padding: 48px 24px;

  &__inner {
    width: 100%;
    max-width: 520px;
  }

  &__greeting {
    margin-bottom: 8px;

    &-text {
      font-size: 22px;
      font-weight: 700;
      color: var(--fg);
    }
  }

  &__summary {
    font-size: 14px;
    color: var(--muted);
    margin: 0 0 28px;
    line-height: 1.5;

    strong {
      color: var(--fg);
      font-weight: 600;
    }
  }

  &__stats {
    display: flex;
    gap: 12px;
    margin-bottom: 32px;
  }

  &__section-title {
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--muted);
    margin-bottom: 12px;
  }

  &__recent-footer {
    text-align: center;
    font-size: 12px;
    color: var(--muted);
    margin-top: 16px;
    opacity: 0.6;
  }

  &__empty {
    text-align: center;
    padding: 48px 0;
    font-size: 14px;
    color: var(--muted);
  }
}

.stat-card {
  flex: 1;
  background: var(--sidebar-bg);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 16px;
  text-align: center;

  &__value {
    font-size: 28px;
    font-weight: 700;
    color: var(--fg);
    line-height: 1.2;
  }

  &__label {
    font-size: 12px;
    color: var(--muted);
    margin-top: 4px;
  }
}

.recent-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.recent-item {
  padding: 10px 12px;
  border-radius: 8px;
  cursor: pointer;
  transition: background 0.1s;

  &:hover {
    background: var(--message-hover);
  }

  &__top {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-bottom: 2px;
  }

  &__prefix {
    font-size: 15px;
    flex-shrink: 0;
  }

  &__name {
    font-size: 14px;
    font-weight: 600;
    color: var(--fg);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  &__badge {
    margin-left: auto;
    background: var(--accent);
    color: #fff;
    font-size: 11px;
    font-weight: 600;
    padding: 1px 7px;
    border-radius: var(--radius-full, 999px);
    min-width: 20px;
    text-align: center;
    flex-shrink: 0;
  }

  &__bottom {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  &__preview {
    flex: 1;
    font-size: 13px;
    color: var(--muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  &__sender {
    font-weight: 500;
    color: var(--fg);
  }

  &__time {
    font-size: 12px;
    color: var(--muted);
    white-space: nowrap;
    flex-shrink: 0;
  }
}
</style>
