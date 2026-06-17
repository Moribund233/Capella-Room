<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  Search, Setting, UserFilled,
} from '@element-plus/icons-vue'
import type { Room, RoomMember } from '@/types/room'

const { t } = useI18n()

const props = defineProps<{
  room: Room
  members: RoomMember[]
  isMobile: boolean
}>()

const emit = defineEmits<{
  toggleSidebar: []
  toggleMemberPanel: []
  toggleSearch: []
  toggleSettings: []
  togglePinned: []
}>()

function getInitial(name: string) {
  return name.charAt(0).toUpperCase()
}

function getColor(index: number) {
  const colors = ['var(--accent)', 'var(--accent-pink)', 'var(--accent-green)', 'var(--accent-orange)', 'var(--accent-blue)']
  return colors[index % colors.length]
}

const onlineCount = computed(() =>
  props.members.filter((m) => m.user_status === 'online' || m.user_status === 'away').length,
)

</script>

<template>
  <div class="chat-header">
    <!-- 移动端菜单切换 -->
    <button v-if="isMobile" class="mobile-toggle" @click="emit('toggleSidebar')">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
        <line x1="3" y1="6" x2="21" y2="6" />
        <line x1="3" y1="12" x2="21" y2="12" />
        <line x1="3" y1="18" x2="21" y2="18" />
      </svg>
    </button>

    <!-- 房间信息 -->
    <div class="channel-info">
      <span class="channel-hash">#</span>
      <span class="channel-name">{{ room.name }}</span>
      <span class="channel-topic">· {{ room.description || t('chat.noDescription') }}</span>
    </div>

    <!-- 右侧操作 -->
    <div class="chat-header-right">
      <div v-if="members.length > 0" class="member-avatars">
        <div
          v-for="(member, idx) in members.slice(0, 5)"
          :key="member.user_id"
          class="mini-avatar"
          :style="{ background: getColor(idx), zIndex: 5 - idx }"
        >
          {{ getInitial(member.username) }}
        </div>
      </div>
      <span class="member-count">
        <span class="status-dot status-dot--online" />
        {{ onlineCount }}/{{ members.length }}
      </span>

      <el-tooltip :content="t('chat.pinnedMessages')" placement="bottom">
        <button class="header-btn" @click="emit('togglePinned')">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="20" height="20">
            <path d="M12 2L15.09 8.26L22 9.27L17 14.14L18.18 21.02L12 17.77L5.82 21.02L7 14.14L2 9.27L8.91 8.26L12 2Z" />
          </svg>
        </button>
      </el-tooltip>

      <el-tooltip :content="t('chat.searchMessages')" placement="bottom">
        <button class="header-btn" @click="emit('toggleSearch')">
          <el-icon :size="20"><Search /></el-icon>
        </button>
      </el-tooltip>

      <el-tooltip :content="t('chat.members')" placement="bottom">
        <button class="header-btn" @click="emit('toggleMemberPanel')">
          <el-icon :size="20"><UserFilled /></el-icon>
        </button>
      </el-tooltip>

      <el-tooltip content="Room settings" placement="bottom">
        <button class="header-btn" @click="emit('toggleSettings')">
          <el-icon :size="20"><Setting /></el-icon>
        </button>
      </el-tooltip>
    </div>
  </div>
</template>

<style scoped lang="scss">
.chat-header {
  height: var(--header-h);
  display: flex;
  align-items: center;
  padding: 0 20px;
  border-bottom: 1px solid var(--border);
  gap: 12px;
  background: var(--bg);
  flex-shrink: 0;
}

.mobile-toggle {
  display: none;
  background: none;
  border: none;
  color: var(--muted);
  padding: 6px;
  border-radius: 6px;
  cursor: pointer;

  &:hover {
    color: var(--fg);
    background: var(--message-hover);
  }

  svg {
    width: 22px;
    height: 22px;
  }
}

.channel-info {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  min-width: 0;
}

.channel-hash {
  color: var(--muted);
  font-weight: 300;
  font-size: 20px;
}

.channel-name {
  font-size: 16px;
  font-weight: 600;
  white-space: nowrap;
}

.channel-topic {
  font-size: 13px;
  color: var(--muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.chat-header-right {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-shrink: 0;
}

.member-avatars {
  display: flex;
}

.mini-avatar {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  border: 2px solid var(--bg);
  margin-left: -6px;
  display: grid;
  place-items: center;
  font-size: 11px;
  font-weight: 600;
  color: #fff;

  &:first-child {
    margin-left: 0;
  }
}

.member-count {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: var(--muted);
  white-space: nowrap;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
  transition: background 0.25s ease, box-shadow 0.25s ease;

  &--online {
    background: var(--accent-green);
    animation: status-pulse 2s ease-in-out infinite;
  }
}

@keyframes status-pulse {
  0%, 100% { box-shadow: 0 0 0 0 color-mix(in oklch, var(--accent-green) 40%, transparent); }
  50% { box-shadow: 0 0 0 4px color-mix(in oklch, var(--accent-green) 0%, transparent); }
}

.header-btn {
  background: none;
  border: none;
  color: var(--muted);
  cursor: pointer;
  padding: 6px;
  border-radius: 6px;
  display: grid;
  place-items: center;

  &:hover {
    color: var(--fg);
    background: var(--message-hover);
  }

  svg, .el-icon {
    font-size: 20px;
  }
}

@media (max-width: 640px) {
  .chat-header {
    padding: 0 12px;
  }

  .mobile-toggle {
    display: grid;
    place-items: center;
  }

  .member-avatars, .member-count {
    display: none;
  }

  .channel-topic {
    display: none;
  }
}
</style>
