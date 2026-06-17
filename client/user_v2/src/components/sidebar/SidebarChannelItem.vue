<script setup lang="ts">
export interface ChannelItemData {
  id: string
  name: string
  type: 'channel' | 'dm'
  unreadCount: number
  isActive: boolean
  isPrivate?: boolean
  userStatus?: 'online' | 'offline' | 'away'
}

defineProps<{
  item: ChannelItemData
}>()

const emit = defineEmits<{
  select: []
  close: []
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
  >
    <span class="channel-item__drag" title="拖拽排序">
      <svg viewBox="0 0 24 24" fill="currentColor" width="12" height="12">
        <circle cx="9" cy="5" r="1.5" />
        <circle cx="15" cy="5" r="1.5" />
        <circle cx="9" cy="12" r="1.5" />
        <circle cx="15" cy="12" r="1.5" />
        <circle cx="9" cy="19" r="1.5" />
        <circle cx="15" cy="19" r="1.5" />
      </svg>
    </span>

    <template v-if="item.type === 'channel'">
      <span v-if="item.isPrivate" class="channel-item__prefix" title="私密房间">🔒</span>
      <span v-else class="channel-item__prefix">#</span>
    </template>

    <template v-else>
      <span
        class="channel-item__status"
        :class="`channel-item__status--${item.userStatus || 'offline'}`"
      />
    </template>

    <span class="channel-item__name">{{ item.name }}</span>

    <span
      v-if="item.unreadCount > 0"
      class="channel-item__badge"
    >
      {{ item.unreadCount > 99 ? '99+' : item.unreadCount }}
    </span>

    <button
      v-if="item.isActive"
      class="channel-item__close"
      title="关闭房间"
      @click.stop="emit('close')"
    >
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" width="14" height="14">
        <line x1="18" y1="6" x2="6" y2="18" />
        <line x1="6" y1="6" x2="18" y2="18" />
      </svg>
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

  &:hover {
    background: var(--message-hover);
    color: var(--fg);

    .channel-item__drag {
      opacity: 1;
    }
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

  &__name {
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
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
