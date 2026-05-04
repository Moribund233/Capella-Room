<script setup lang="ts">
import type { ReplyToMessage } from '@/types/message'

defineProps<{
  replyTo: ReplyToMessage
}>()

const emit = defineEmits<{
  cancel: []
}>()
</script>

<template>
  <div class="message-reply">
    <div class="message-reply__content">
      <div class="message-reply__author">
        回复 {{ replyTo.sender.username }}
      </div>
      <div class="message-reply__preview">
        {{ replyTo.content }}
      </div>
    </div>
    <button class="message-reply__cancel" @click="emit('cancel')">
      ✕
    </button>
  </div>
</template>

<style scoped>
.message-reply {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  padding: var(--space-sm) var(--space-md);
  background: var(--color-background);
  border-top: 1px solid var(--color-border);
  border-left: 3px solid var(--color-primary);
}

.message-reply__content {
  flex: 1;
  min-width: 0;
}

.message-reply__author {
  font-size: var(--font-size-small);
  font-weight: 600;
  color: var(--color-primary);
  margin-bottom: 2px;
}

.message-reply__preview {
  font-size: var(--font-size-small);
  color: var(--color-text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.message-reply__cancel {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: none;
  cursor: pointer;
  color: var(--color-text-tertiary);
  font-size: 14px;
  border-radius: var(--radius-sm);
  transition: all var(--duration-fast);
}

.message-reply__cancel:hover {
  background: var(--color-border);
  color: var(--color-text-primary);
}
</style>
