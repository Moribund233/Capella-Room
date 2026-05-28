<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import type { ReplyToMessage } from '@/types/message'
const { t } = useI18n()

const props = defineProps<{
  roomName: string
  replyingTo: ReplyToMessage | null
  editingMessage: { id: string; content: string } | null
}>()

const emit = defineEmits<{
  send: [content: string]
  cancelReply: []
  cancelEdit: []
}>()

const inputText = ref('')
const textareaRef = ref<HTMLTextAreaElement | null>(null)
const isFocused = ref(false)

// 编辑模式：填充内容
watch(
  () => props.editingMessage,
  (msg) => {
    if (msg) {
      inputText.value = msg.content
      nextTick(() => autoResize())
    }
  },
  { immediate: true },
)

function autoResize() {
  const ta = textareaRef.value
  if (!ta) return
  ta.style.height = 'auto'
  ta.style.height = Math.min(ta.scrollHeight, 144) + 'px'
}

function handleInput() {
  autoResize()
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault()
    handleSend()
  }
}

function handleSend() {
  const text = inputText.value.trim()
  if (!text) return

  emit('send', text)
  inputText.value = ''
  nextTick(() => autoResize())
}

function handleCancelReply() {
  emit('cancelReply')
}

function handleCancelEdit() {
  emit('cancelEdit')
}

function focusInput() {
  textareaRef.value?.focus()
}
</script>

<template>
  <div class="input-area">
    <!-- 回复预览 -->
    <div v-if="replyingTo" class="reply-preview" @click="focusInput">
      <div class="reply-preview__line" />
      <div class="reply-preview__content">
        <span class="reply-preview__label">{{ t('chat.replyingTo') }}</span>
        <span class="reply-preview__author">{{ replyingTo.sender.username }}</span>
        <span class="reply-preview__text">{{ replyingTo.content }}</span>
      </div>
      <button class="reply-preview__close" @click.stop="handleCancelReply">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
      </button>
    </div>

    <!-- 编辑预览 -->
    <div v-if="editingMessage" class="reply-preview reply-preview--edit" @click="focusInput">
      <div class="reply-preview__line" />
      <div class="reply-preview__content">
        <span class="reply-preview__label">{{ t('chat.editingMessage') }}</span>
      </div>
      <button class="reply-preview__close" @click.stop="handleCancelEdit">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
      </button>
    </div>

    <!-- 输入框 -->
    <div
      class="input-wrapper"
      :class="{ 'input-wrapper--focused': isFocused }"
    >
      <div class="input-tools">
        <button :title="t('chat.uploadFile')">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/>
            <polyline points="17 8 12 3 7 8"/>
            <line x1="12" y1="3" x2="12" y2="15"/>
          </svg>
        </button>
        <button :title="t('chat.emoji')">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <circle cx="12" cy="12" r="10"/>
            <path d="M8 14s1.5 2 4 2 4-2 4-2"/>
            <line x1="9" y1="9" x2="9.01" y2="9"/>
            <line x1="15" y1="9" x2="15.01" y2="9"/>
          </svg>
        </button>
      </div>

      <textarea
        ref="textareaRef"
        v-model="inputText"
        :placeholder="t('chat.messagePlaceholder', { room: roomName })"
        rows="1"
        @input="handleInput"
        @keydown="handleKeydown"
        @focus="isFocused = true"
        @blur="isFocused = false"
      />

      <div class="input-tools">
        <button :title="t('chat.gif')">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <rect x="3" y="3" width="18" height="18" rx="2"/>
            <path d="M9 12h6"/>
          </svg>
        </button>
        <button
          :title="t('chat.send')"
          class="send-btn"
          :class="{ 'send-btn--active': inputText.trim() }"
          :disabled="!inputText.trim()"
          @click="handleSend"
        >
          <svg viewBox="0 0 24 24" fill="currentColor" width="20" height="20">
            <path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"/>
          </svg>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">
.input-area {
  padding: 0 20px 20px;
  margin-top: auto;
  flex-shrink: 0;
}

.reply-preview {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 6px 12px;
  margin-bottom: 4px;
  cursor: pointer;
  border-radius: var(--radius) var(--radius) 0 0;

  &:hover {
    background: var(--message-hover);
  }

  &__line {
    width: 3px;
    height: 28px;
    border-radius: 2px;
    background: var(--accent);
    flex-shrink: 0;
  }

  &__content {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: baseline;
    gap: 6px;
    font-size: 13px;
    white-space: nowrap;
    overflow: hidden;
  }

  &__label {
    font-size: 11px;
    color: var(--accent);
    font-weight: 600;
    text-transform: uppercase;
    flex-shrink: 0;
  }

  &__author {
    font-weight: 600;
    color: var(--fg);
    flex-shrink: 0;
  }

  &__text {
    color: var(--muted);
    overflow: hidden;
    text-overflow: ellipsis;
  }

  &__close {
    background: none;
    border: none;
    color: var(--muted);
    cursor: pointer;
    padding: 4px;
    border-radius: 4px;
    display: grid;
    place-items: center;
    flex-shrink: 0;

    &:hover {
      color: var(--fg);
      background: var(--message-hover);
    }

    svg {
      width: 16px;
      height: 16px;
    }
  }

  &--edit .reply-preview__label {
    color: var(--accent-green);
  }

  &--edit .reply-preview__line {
    background: var(--accent-green);
  }
}

.input-wrapper {
  display: flex;
  align-items: flex-end;
  gap: 4px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 4px 8px;
  transition: border-color 0.15s;

  &--focused {
    border-color: var(--accent);
  }

  textarea {
    flex: 1;
    background: none;
    border: none;
    color: var(--fg);
    font: inherit;
    font-size: 15px;
    font-family: var(--font-body);
    padding: 8px 4px;
    resize: none;
    max-height: 144px;
    line-height: 1.4;

    &:focus {
      outline: none;
    }

    &::placeholder {
      color: var(--muted);
    }
  }
}

.input-tools {
  display: flex;
  align-items: center;
  gap: 2px;

  button {
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

    svg {
      width: 20px;
      height: 20px;
    }
  }
}

.send-btn {
  color: var(--muted) !important;

  &--active {
    color: var(--accent) !important;
  }

  &:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
}

@media (max-width: 640px) {
  .input-area {
    padding: 0 12px 12px;
  }
}
</style>
