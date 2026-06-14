<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useMessageStore } from '@/stores/message'

const { t } = useI18n()
const messageStore = useMessageStore()

const emit = defineEmits<{
  jumpToMessage: [messageId: string]
}>()

const pinned = computed(() => messageStore.pinnedMessages)

function handleUnpin(messageId: string, roomId: string) {
  messageStore.unpinMessage(messageId, roomId)
}

function getInitial(name: string): string {
  return name.charAt(0).toUpperCase()
}
</script>

<template>
  <div v-if="pinned.length > 0" class="pinned-bar">
    <div class="pinned-bar__header">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="14" height="14"><path d="M12 2L15.09 8.26L22 9.27L17 14.14L18.18 21.02L12 17.77L5.82 21.02L7 14.14L2 9.27L8.91 8.26L12 2Z"/></svg>
      <span class="pinned-bar__count">{{ pinned.length }}</span>
    </div>
    <div class="pinned-bar__list">
      <div
        v-for="pin in pinned"
        :key="pin.message_id"
        class="pinned-item"
        @click="emit('jumpToMessage', pin.message_id)"
      >
        <div class="pinned-item__avatar" :title="pin.sender_name">
          {{ getInitial(pin.sender_name) }}
        </div>
        <div class="pinned-item__body">
          <span class="pinned-item__sender">{{ pin.sender_name }}</span>
          <span class="pinned-item__content">{{ pin.content }}</span>
        </div>
        <button
          class="pinned-item__unpin"
          :title="t('chat.unpinMessage')"
          @click.stop="handleUnpin(pin.message_id, pin.room_id)"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="13" height="13"><path d="M12 2L15.09 8.26L22 9.27L17 14.14L18.18 21.02L12 17.77L5.82 21.02L7 14.14L2 9.27L8.91 8.26L12 2Z"/></svg>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">
.pinned-bar {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 6px 16px;
  border-bottom: 1px solid var(--border);
  background: color-mix(in oklch, var(--accent) 6%, var(--bg));
  flex-shrink: 0;

  &__header {
    display: flex;
    align-items: center;
    gap: 4px;
    color: var(--accent);
    padding-top: 2px;
    flex-shrink: 0;

    svg {
      display: block;
    }
  }

  &__count {
    font-size: 11px;
    font-weight: 600;
  }

  &__list {
    display: flex;
    gap: 6px;
    overflow-x: auto;
    flex: 1;
    min-width: 0;
    scrollbar-width: none;

    &::-webkit-scrollbar {
      display: none;
    }
  }
}

.pinned-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 8px;
  border-radius: 6px;
  background: color-mix(in oklch, var(--accent) 8%, var(--surface));
  border: 1px solid color-mix(in oklch, var(--accent) 16%, var(--border));
  cursor: pointer;
  flex-shrink: 0;
  max-width: 260px;
  transition: background 0.1s;

  &:hover {
    background: color-mix(in oklch, var(--accent) 14%, var(--surface));

    .pinned-item__unpin {
      opacity: 1;
      pointer-events: auto;
    }
  }

  &__avatar {
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: var(--accent);
    color: #fff;
    font-size: 10px;
    font-weight: 600;
    display: grid;
    place-items: center;
    flex-shrink: 0;
  }

  &__body {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  &__sender {
    font-size: 11px;
    font-weight: 600;
    color: var(--accent);
    line-height: 1.2;
  }

  &__content {
    font-size: 11px;
    color: var(--muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    line-height: 1.3;
  }

  &__unpin {
    background: none;
    border: none;
    color: var(--muted);
    cursor: pointer;
    padding: 2px;
    border-radius: 3px;
    opacity: 0;
    pointer-events: none;
    transition: opacity 0.1s;
    flex-shrink: 0;

    svg {
      display: block;
    }

    &:hover {
      color: var(--accent-pink);
      background: color-mix(in oklch, var(--accent-pink) 12%, transparent);
    }
  }
}
</style>
