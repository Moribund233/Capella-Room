<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { Lock, User, Close } from '@element-plus/icons-vue'
import { GripVertical } from '@lucide/vue'
import BaseBadge from '@/components/common/BaseBadge.vue'

const { t } = useI18n()

export interface ChannelItemData {
  id: string
  name: string
  type: 'channel' | 'dm'
  unreadCount: number
  isActive: boolean
  isPrivate?: boolean
  userStatus?: 'online' | 'offline' | 'away'
  lastMessage?: string
  memberCount?: number
}

defineProps<{
  item: ChannelItemData
}>()

const emit = defineEmits<{
  select: []
  close: []
  focus: []
}>()
</script>

<template>
  <div
    class="channel-item"
    :class="{
      'channel-item--active': item.isActive,
      'channel-item--unread': item.unreadCount > 0,
    }"
    @click="emit('select')"
    @focus="emit('focus')"
  >
    <span class="channel-item__drag" :title="t('sidebar.dragToReorder')">
      <el-icon :size="12"><GripVertical /></el-icon>
    </span>

    <template v-if="item.type === 'channel'">
      <span v-if="item.isPrivate" class="channel-item__prefix channel-item__prefix--lock">
        <Lock width="14" height="14" />
      </span>
      <span v-else class="channel-item__prefix">#</span>
    </template>

    <template v-else>
      <span
        class="channel-item__status"
        :class="`channel-item__status--${item.userStatus || 'offline'}`"
      />
    </template>

    <div class="channel-item__content">
      <div class="channel-item__row">
        <span class="channel-item__name">{{ item.name }}</span>
        <BaseBadge
          v-if="item.unreadCount > 0"
          variant="primary"
          size="sm"
        >
          {{ item.unreadCount > 99 ? '99+' : item.unreadCount }}
        </BaseBadge>
      </div>
      <div v-if="item.lastMessage || item.memberCount" class="channel-item__meta">
        <span v-if="item.lastMessage" class="channel-item__last-message">{{ item.lastMessage }}</span>
        <span v-if="item.memberCount" class="channel-item__member-count">
          <el-icon :size="10"><User /></el-icon>
          {{ item.memberCount }}
        </span>
      </div>
    </div>

    <button
      v-if="item.isActive"
      class="channel-item__close"
      :title="t('sidebar.closeRoom')"
      @click.stop="emit('close')"
    >
      <el-icon :size="14"><Close /></el-icon>
    </button>
  </div>
</template>

<style scoped lang="scss">
.channel-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px 6px 4px;
  border-radius: var(--radius);
  font-size: 15px;
  color: var(--muted);
  cursor: pointer;
  transition: background 0.1s, color 0.1s;
  user-select: none;
  min-height: 36px;
  outline: none;

  &:hover {
    background: var(--message-hover);
    color: var(--fg);

    .channel-item__drag {
      opacity: 1;
    }
  }

  &:focus-visible {
    background: var(--message-hover);
    color: var(--fg);
    box-shadow: inset 0 0 0 2px var(--accent);
  }

  &--active {
    background: var(--accent-soft);
    color: var(--fg);

    .channel-item__prefix {
      color: var(--accent);
    }
  }

  &--unread {
    .channel-item__name {
      color: var(--fg);
      font-weight: 600;
    }
  }

  &__drag {
    opacity: 0;
    color: var(--muted);
    flex-shrink: 0;
    width: 12px;
    display: flex;
    align-items: center;
    cursor: grab;
    transition: opacity 0.15s;
  }

  &__prefix {
    flex-shrink: 0;
    width: 18px;
    text-align: center;
    font-weight: 300;
    color: var(--muted);
    opacity: 0.7;
    font-size: 16px;

    &--lock {
      display: flex;
      align-items: center;
      justify-content: center;
      color: var(--accent-orange);
      opacity: 1;
    }
  }

  &__status {
    flex-shrink: 0;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    margin: 0 5px;

    &--online {
      background: var(--accent-green);
    }

    &--away {
      background: var(--accent-orange);
    }

    &--offline {
      background: var(--muted);
      opacity: 0.5;
    }
  }

  &__content {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  &__row {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  &__name {
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  &__meta {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    color: var(--muted);
    opacity: 0.7;
  }

  &__last-message {
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 140px;
  }

  &__member-count {
    display: flex;
    align-items: center;
    gap: 3px;
    flex-shrink: 0;

    svg {
      opacity: 0.6;
    }
  }

  &__badge {
    background: var(--accent);
    color: #fff;
    font-size: 11px;
    font-weight: 600;
    padding: 1px 7px;
    border-radius: var(--radius-full);
    min-width: 20px;
    text-align: center;
    flex-shrink: 0;
  }

  &__close {
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: none;
    background: transparent;
    border-radius: 6px;
    color: var(--muted);
    cursor: pointer;
    flex-shrink: 0;
    transition: all 0.15s;
    margin-left: 2px;

    &:hover {
      background: var(--message-hover);
      color: var(--fg);
    }
  }
}
</style>
