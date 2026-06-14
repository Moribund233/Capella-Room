<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { RoomMember } from '@/types/room'

const { t } = useI18n()

const props = defineProps<{
  members: RoomMember[]
  totalCount: number
}>()

const onlineMembers = computed(() =>
  props.members.filter((m) => m.user_status === 'online' || m.user_status === 'away'),
)

const offlineMembers = computed(() =>
  props.members.filter((m) => m.user_status === 'offline'),
)

const otherMembers = computed(() =>
  props.members.filter((m) => m.user_status !== 'online' && m.user_status !== 'away' && m.user_status !== 'offline'),
)

function getDotColor(status: string): string {
  switch (status) {
    case 'online': return 'var(--accent-green)'
    case 'away': return 'var(--accent-orange)'
    case 'busy': return 'var(--accent-pink)'
    default: return 'var(--muted)'
  }
}

function getRoleColor(role: string): string {
  switch (role) {
    case 'owner':
    case 'admin': return 'var(--accent)'
    case 'mod': return 'var(--accent-pink)'
    default: return 'var(--muted)'
  }
}
</script>

<template>
  <aside class="member-panel">
    <div class="panel-header">
      {{ t('chat.members') }} — {{ totalCount }}
    </div>

    <div class="member-list">
      <!-- 在线成员 -->
      <div v-if="onlineMembers.length > 0" class="member-group">
        <div class="member-group-label">
          {{ t('chat.online') }} — {{ onlineMembers.length }}
        </div>
        <div
          v-for="member in onlineMembers"
          :key="member.user_id"
          class="member-item"
        >
          <span
            class="member-dot"
            :class="{ 'member-dot--online': member.user_status === 'online' }"
            :style="{ background: getDotColor(member.user_status) }"
          />
          <span class="name">{{ member.username }}</span>
          <span
            v-if="member.role === 'owner' || member.role === 'admin'"
            class="role-tag"
            :style="{ color: getRoleColor(member.role) }"
          >
            {{ member.role === 'owner' ? t('chat.owner') : t('chat.admin') }}
          </span>
        </div>
      </div>

      <!-- 离线成员 -->
      <div v-if="offlineMembers.length > 0" class="member-group">
        <div class="member-group-label">
          {{ t('chat.offline') }} — {{ offlineMembers.length }}
        </div>
        <div
          v-for="member in offlineMembers"
          :key="member.user_id"
          class="member-item"
        >
          <span
            class="member-dot"
            :style="{ background: getDotColor(member.user_status) }"
          />
          <span class="name">{{ member.username }}</span>
        </div>
      </div>

      <!-- 其他状态成员 -->
      <div v-if="otherMembers.length > 0" class="member-group">
        <div class="member-group-label">
          {{ otherMembers.length }}
        </div>
        <div
          v-for="member in otherMembers"
          :key="member.user_id"
          class="member-item"
        >
          <span
            class="member-dot"
            :style="{ background: getDotColor(member.user_status) }"
          />
          <span class="name">{{ member.username }}</span>
        </div>
      </div>

      <!-- 空状态 -->
      <div v-if="members.length === 0" class="member-empty">
        <span>{{ t('chat.noMembers') }}</span>
      </div>
    </div>
  </aside>
</template>

<style scoped lang="scss">
.member-panel {
  width: 280px;
  min-width: 280px;
  height: 100vh;
  background: var(--sidebar-bg);
  border-left: 1px solid var(--border);
  display: flex;
  flex-direction: column;
}

.panel-header {
  height: var(--header-h);
  display: flex;
  align-items: center;
  padding: 0 16px;
  border-bottom: 1px solid var(--border);
  font-weight: 600;
  font-size: 14px;
  flex-shrink: 0;
}

.member-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.member-group {
  margin-bottom: 16px;
}

.member-group-label {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--muted);
  padding: 8px 8px 4px;
}

.member-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 6px 8px;
  border-radius: var(--radius);
  cursor: pointer;

  &:hover {
    background: var(--message-hover);
  }
}

.member-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
  transition: background 0.25s ease, box-shadow 0.25s ease;

  &--online {
    animation: member-status-pulse 2s ease-in-out infinite;
  }
}

@keyframes member-status-pulse {
  0%, 100% { box-shadow: 0 0 0 0 color-mix(in oklch, var(--accent-green) 40%, transparent); }
  50% { box-shadow: 0 0 0 4px color-mix(in oklch, var(--accent-green) 0%, transparent); }
}

.name {
  font-size: 14px;
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.role-tag {
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  padding: 2px 6px;
  border-radius: 4px;
  flex-shrink: 0;
}

.member-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 32px 8px;
  font-size: 13px;
  color: var(--muted);
}

@media (max-width: 860px) {
  .member-panel {
    display: none;
  }
}
</style>
