<script setup lang="ts">
import { ref, watch, nextTick, computed, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useWebSocketStore } from '@/stores/websocket'
import { smartUpload } from '@/api/upload'
import { ElMessage } from 'element-plus'
import type { ReplyToMessage } from '@/types/message'
import { Close, UploadFilled, Promotion, Loading, Comment, Picture } from '@element-plus/icons-vue'
import EmojiPicker from './EmojiPicker.vue'
import GifPicker from './GifPicker.vue'
const { t } = useI18n()

const props = defineProps<{
  roomId: string
  roomName: string
  replyingTo: ReplyToMessage | null
  editingMessage: { id: string; content: string } | null
}>()

const emit = defineEmits<{
  send: [content: string, messageType?: string]
  cancelReply: []
  cancelEdit: []
}>()

const wsStore = useWebSocketStore()

const inputText = ref('')
const textareaRef = ref<HTMLTextAreaElement | null>(null)
const isFocused = ref(false)
const uploading = ref(false)
const uploadProgress = ref(0)
const fileInputRef = ref<HTMLInputElement | null>(null)
const showEmojiPicker = ref(false)
const showGifPicker = ref(false)

function selectGif(url: string) {
  emit('send', url)
  showGifPicker.value = false
}

function toggleGifPicker() {
  showGifPicker.value = !showGifPicker.value
  if (showGifPicker.value) showEmojiPicker.value = false
}

function toggleEmojiPicker() {
  showEmojiPicker.value = !showEmojiPicker.value
  if (showEmojiPicker.value) showGifPicker.value = false
}

function insertEmoji(emoji: string) {
  const ta = textareaRef.value
  if (!ta) {
    inputText.value += emoji
  } else {
    const start = ta.selectionStart
    const end = ta.selectionEnd
    const text = inputText.value
    inputText.value = text.slice(0, start) + emoji + text.slice(end)
    nextTick(() => {
      ta.selectionStart = ta.selectionEnd = start + emoji.length
      ta.focus()
      autoResize()
    })
  }
}
let typingTimer: ReturnType<typeof setTimeout> | null = null
let isTyping = false

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

function sendTyping() {
  if (!wsStore.isConnected) return
  if (!isTyping) {
    isTyping = true
    wsStore.send('Typing', { room_id: props.roomId })
  }
  if (typingTimer) clearTimeout(typingTimer)
  typingTimer = setTimeout(() => {
    sendStopTyping()
  }, 2000)
}

function sendStopTyping() {
  if (!wsStore.isConnected) return
  isTyping = false
  if (typingTimer) clearTimeout(typingTimer)
  typingTimer = null
  wsStore.send('StopTyping', { room_id: props.roomId })
}

function autoResize() {
  const ta = textareaRef.value
  if (!ta) return
  ta.style.height = 'auto'
  ta.style.height = Math.min(ta.scrollHeight, 144) + 'px'
}

function handleInput() {
  autoResize()
  sendTyping()
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault()
    handleSend()
  }
}

const sendState = ref<'idle' | 'sending' | 'error'>('idle')

function handleSend() {
  if (!inputText.value.trim() || sendState.value === 'sending') return
  sendState.value = 'sending'
  emit('send', inputText.value.trim())
  inputText.value = ''
  autoResize()
  setTimeout(() => {
    sendState.value = 'idle'
  }, 300)
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

function handleBlur() {
  isFocused.value = false
  sendStopTyping()
}

function triggerFilePicker() {
  fileInputRef.value?.click()
}

async function handleFileSelected(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  input.value = ''

  uploading.value = true
  uploadProgress.value = 0
  try {
    const isImage = file.type.startsWith('image/')
    const res = await smartUpload(file, {
      endpoint: isImage ? 'image' : 'file',
      usageType: 'message',
      onProgress: (p) => { uploadProgress.value = p },
    })
    if (res.success && res.data) {
      emit('send', res.data.url, isImage ? 'image' : 'file')
    } else {
      ElMessage.error(res.message || t('common.error'))
    }
  } catch {
    ElMessage.error(t('common.error'))
  } finally {
    uploading.value = false
    uploadProgress.value = 0
  }
}

onUnmounted(() => {
  sendStopTyping()
})
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
        <el-icon :size="16"><Close /></el-icon>
      </button>
    </div>

    <!-- 编辑预览 -->
    <div v-if="editingMessage" class="reply-preview reply-preview--edit" @click="focusInput">
      <div class="reply-preview__line" />
      <div class="reply-preview__content">
        <span class="reply-preview__label">{{ t('chat.editingMessage') }}</span>
      </div>
      <button class="reply-preview__close" @click.stop="handleCancelEdit">
        <el-icon :size="16"><Close /></el-icon>
      </button>
    </div>

    <!-- 输入框 -->
    <div
      class="input-wrapper"
      :class="{ 'input-wrapper--focused': isFocused }"
    >
      <div class="input-tools">
        <input
          ref="fileInputRef"
          type="file"
          accept="image/*,.pdf,.doc,.docx,.xls,.xlsx,.txt,.zip"
          style="display:none"
          @change="handleFileSelected"
        />
        <button
          :title="t('chat.uploadFile')"
          :disabled="uploading"
          @click="triggerFilePicker"
        >
          <el-icon :size="20"><UploadFilled /></el-icon>
        </button>
        <div class="emoji-btn-wrapper">
          <button :title="t('chat.emoji')" @click.stop="toggleEmojiPicker">
            <el-icon :size="20"><Comment /></el-icon>
          </button>
          <EmojiPicker
            :visible="showEmojiPicker"
            @select="insertEmoji"
            @close="showEmojiPicker = false"
          />
        </div>
        <span v-if="uploading" class="uploading-spinner" />
        <div v-if="uploading && uploadProgress > 0" class="upload-progress">
          <div class="upload-progress__bar" :style="{ width: uploadProgress + '%' }" />
          <span class="upload-progress__text">{{ uploadProgress }}%</span>
        </div>
      </div>

      <textarea
        ref="textareaRef"
        v-model="inputText"
        :placeholder="t('chat.messagePlaceholder', { room: roomName })"
        rows="1"
        @input="handleInput"
        @keydown="handleKeydown"
        @focus="isFocused = true"
        @blur="handleBlur"
      />

      <div class="input-tools">
        <div class="gif-btn-wrapper">
          <button :title="t('chat.gif')" @click.stop="toggleGifPicker">
            <el-icon :size="20"><Picture /></el-icon>
          </button>
          <GifPicker
            :visible="showGifPicker"
            @select="selectGif"
            @close="showGifPicker = false"
          />
        </div>
        <button
          :title="t('chat.send')"
          class="send-btn"
          :class="{
            'send-btn--active': inputText.trim(),
            'send-btn--sending': sendState === 'sending',
          }"
          :disabled="!inputText.trim() || sendState === 'sending'"
          @click="handleSend"
        >
          <el-icon v-if="sendState === 'sending'" class="is-loading" :size="20"><Loading /></el-icon>
          <el-icon v-else :size="20"><Promotion /></el-icon>
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
  transition: border-color 0.25s, box-shadow 0.25s;

  &--focused {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-soft), 0 0 12px rgba(124, 92, 252, 0.15);
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

.emoji-btn-wrapper,
.gif-btn-wrapper {
  position: relative;
}

.send-btn {
  color: var(--muted) !important;
  transition: transform 0.12s ease;

  &--active {
    color: var(--accent) !important;
  }

  &:active:not(:disabled) {
    transform: scale(0.85);
  }

  &:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  &--sending {
    opacity: 0.7;
    pointer-events: none;
  }
}

.uploading-spinner {
  width: 16px;
  height: 16px;
  border: 2px solid var(--border);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

.upload-progress {
  display: flex;
  align-items: center;
  gap: 6px;
  width: 80px;
  height: 16px;
  background: var(--bg);
  border-radius: 8px;
  overflow: hidden;
  padding: 0 4px;

  &__bar {
    height: 6px;
    background: var(--accent);
    border-radius: 3px;
    transition: width 0.3s ease;
    min-width: 6px;
  }

  &__text {
    font-size: 10px;
    color: var(--muted);
    white-space: nowrap;
    flex-shrink: 0;
  }
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

@media (max-width: 640px) {
  .input-area {
    padding: 0 12px 12px;
  }
}
</style>
