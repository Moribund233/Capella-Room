<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { Reply, Pencil, Trash2 } from 'lucide-vue-next'
import type { Message } from '@/types/message'

const props = defineProps<{
  message: Message
  isOwn: boolean
  visible: boolean
}>()

const emit = defineEmits<{
  reply: [message: Message]
  edit: [message: Message]
  delete: [message: Message]
  close: []
}>()

const menuRef = ref<HTMLDivElement | null>(null)

function handleReply() {
  emit('reply', props.message)
  emit('close')
}

function handleEdit() {
  emit('edit', props.message)
  emit('close')
}

function handleDelete() {
  emit('delete', props.message)
  emit('close')
}

function handleClickOutside(event: MouseEvent) {
  if (menuRef.value && !menuRef.value.contains(event.target as Node)) {
    emit('close')
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>

<template>
  <div
    v-if="visible"
    ref="menuRef"
    class="message-actions"
    @click.stop
  >
    <button class="message-actions__item" @click="handleReply">
      <Reply class="message-actions__icon" :size="16" />
      <span class="message-actions__text">回复</span>
    </button>
    <template v-if="isOwn && !message.is_deleted">
      <button class="message-actions__item" @click="handleEdit">
        <Pencil class="message-actions__icon" :size="16" />
        <span class="message-actions__text">编辑</span>
      </button>
      <button class="message-actions__item message-actions__item--danger" @click="handleDelete">
        <Trash2 class="message-actions__icon" :size="16" />
        <span class="message-actions__text">删除</span>
      </button>
    </template>
  </div>
</template>

<style scoped>
.message-actions {
  position: absolute;
  right: 0;
  top: 100%;
  margin-top: 4px;
  background: var(--color-white);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  z-index: 100;
  min-width: 120px;
  padding: 4px 0;
}

.message-actions__item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 8px 12px;
  border: none;
  background: none;
  cursor: pointer;
  font-size: var(--font-size-body);
  color: var(--color-text-primary);
  transition: background-color var(--duration-fast);
}

.message-actions__item:hover {
  background: var(--color-background);
}

.message-actions__item--danger {
  color: var(--color-error);
}

.message-actions__item--danger:hover {
  background: var(--color-error-light, #fee2e2);
}

.message-actions__icon {
  flex-shrink: 0;
}

.message-actions__text {
  font-size: 14px;
}
</style>
