<script setup lang="ts">
import { ref, nextTick, watch } from 'vue'
import MessageReply from './MessageReply.vue'
import type { ReplyToMessage } from '@/types/message'

const props = defineProps<{
  disabled: boolean
  placeholder?: string
  replyTo?: ReplyToMessage | null
  editingMessage?: { id: string; content: string } | null
}>()

const emit = defineEmits<{
  send: [content: string]
  cancelReply: []
  edit: [messageId: string, content: string]
  cancelEdit: []
}>()

const text = ref('')
const textareaRef = ref<HTMLTextAreaElement | null>(null)

watch(
  () => props.editingMessage,
  (editing) => {
    if (editing) {
      text.value = editing.content
      nextTick(() => {
        adjustHeight()
        textareaRef.value?.focus()
      })
    }
  },
  { immediate: true }
)

function adjustHeight() {
  const el = textareaRef.value
  if (!el) return
  el.style.height = 'auto'
  el.style.height = Math.min(el.scrollHeight, 120) + 'px'
}

function handleSend() {
  const content = text.value.trim()
  if (!content || props.disabled) return

  if (props.editingMessage) {
    emit('edit', props.editingMessage.id, content)
  } else {
    emit('send', content)
  }

  text.value = ''
  nextTick(() => {
    adjustHeight()
    textareaRef.value?.focus()
  })
}

function handleCancel() {
  if (props.editingMessage) {
    text.value = ''
    emit('cancelEdit')
  } else if (props.replyTo) {
    emit('cancelReply')
  }
  nextTick(() => {
    adjustHeight()
  })
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault()
    handleSend()
  }
  if (e.key === 'Escape') {
    handleCancel()
  }
}
</script>

<template>
  <div class="message-input">
    <!-- 回复预览 -->
    <MessageReply
      v-if="replyTo && !editingMessage"
      :reply-to="replyTo"
      @cancel="emit('cancelReply')"
    />

    <!-- 编辑提示 -->
    <div v-if="editingMessage" class="message-input__editing">
      <div class="message-input__editing-content">
        <span class="message-input__editing-label">编辑消息</span>
      </div>
      <button class="message-input__editing-cancel" @click="handleCancel">
        ✕
      </button>
    </div>

    <div class="message-input__container">
      <textarea
        ref="textareaRef"
        v-model="text"
        class="message-input__textarea"
        :placeholder="placeholder || (editingMessage ? '编辑消息...' : '输入消息...')"
        :disabled="disabled"
        rows="1"
        @input="adjustHeight"
        @keydown="onKeydown"
      />
      <button
        class="message-input__send"
        :disabled="disabled || !text.trim()"
        @click="handleSend"
      >
        {{ editingMessage ? '保存' : '发送' }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.message-input {
  display: flex;
  flex-direction: column;
  border-top: 1px solid var(--color-border);
  background: var(--color-white);
  flex-shrink: 0;
}

.message-input__container {
  display: flex;
  gap: var(--space-sm);
  padding: var(--space-md) var(--space-lg);
  align-items: flex-end;
}

.message-input__textarea {
  flex: 1;
  padding: 8px 14px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  font-size: var(--font-size-body);
  font-family: inherit;
  line-height: var(--line-height-body);
  outline: none;
  resize: none;
  max-height: 120px;
  transition: border-color var(--duration-fast);
}

.message-input__textarea:focus {
  border-color: var(--color-primary);
}

.message-input__textarea:disabled {
  background: var(--color-background);
  cursor: not-allowed;
}

.message-input__textarea::placeholder {
  color: var(--color-text-tertiary);
}

.message-input__send {
  padding: 8px 20px;
  background: var(--color-primary);
  color: var(--color-white);
  border: none;
  border-radius: var(--radius-md);
  font-size: var(--font-size-body);
  cursor: pointer;
  transition: opacity var(--duration-fast);
  white-space: nowrap;
  height: 36px;
}

.message-input__send:hover:not(:disabled) {
  opacity: 0.9;
}

.message-input__send:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Editing indicator */
.message-input__editing {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  padding: var(--space-sm) var(--space-md);
  background: var(--color-background);
  border-top: 1px solid var(--color-border);
  border-left: 3px solid var(--color-warning);
}

.message-input__editing-content {
  flex: 1;
}

.message-input__editing-label {
  font-size: var(--font-size-small);
  font-weight: 600;
  color: var(--color-warning);
}

.message-input__editing-cancel {
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

.message-input__editing-cancel:hover {
  background: var(--color-border);
  color: var(--color-text-primary);
}
</style>
