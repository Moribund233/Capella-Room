<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useAuthStore } from '@/stores/auth'
import { useMessageStore } from '@/stores/message'
import { formatTime } from '@/utils/date'
import { renderMarkdown } from '@/utils/markdown'
import type { Message } from '@/types/message'
import { ElMessageBox } from 'element-plus'
import { ChatRound, Edit, CircleClose, Document } from '@element-plus/icons-vue'
import EmojiPicker from './EmojiPicker.vue'
import EditHistoryPanel from './EditHistoryPanel.vue'

const { t } = useI18n()
const authStore = useAuthStore()
const messageStore = useMessageStore()

const props = defineProps<{
  message: Message
  isOwn: boolean
  highlight?: boolean
}>()

const emit = defineEmits<{
  reply: [message: Message]
  edit: [message: Message]
  delete: [id: string]
  jumpToThread: [messageId: string | undefined]
}>()

const isPinned = computed(() =>
  messageStore.pinnedMessages.some((m) => m.message_id === props.message.id),
)

const showEmojiPicker = ref(false)
const showEditHistory = ref(false)
const imageLoaded = ref(false)
const imageError = ref(false)

const displayName = computed(() => props.message.sender.username)
const displayTime = computed(() => formatTime(props.message.created_at))
const isDeleted = computed(() => props.message.is_deleted)
const isSending = computed(() => props.message.sending)
const isError = computed(() => props.message.error)
const isEdited = computed(() => (props.message.edit_count || 0) > 0)
const isImageUrl = computed(() => /\.(png|jpe?g|gif|webp|bmp|svg)(\?.*)?$/i.test(props.message.content))
const isFileUrl = computed(() => /\/files\//.test(props.message.content) && !isImageUrl.value)
const currentUserId = computed(() => authStore.user?.id ?? '')

watch(() => props.message.id, () => {
  imageLoaded.value = false
  imageError.value = false
})

function hasReacted(emoji: string): boolean {
  const r = props.message.reactions?.find((r) => r.emoji === emoji)
  return r ? r.users.includes(currentUserId.value) : false
}

function handleReactionClick(emoji: string) {
  if (hasReacted(emoji)) {
    messageStore.removeReaction(props.message.id, emoji)
  } else {
    messageStore.addReaction(props.message.id, emoji)
  }
}

function handleEmojiSelect(emoji: string) {
  handleReactionClick(emoji)
}

function getAvatarColor(name: string): string {
  const colors = ['var(--accent)', 'var(--accent-pink)', 'var(--accent-green)', 'var(--accent-orange)', 'var(--accent-blue)']
  let hash = 0
  for (let i = 0; i < name.length; i++) {
    hash = name.charCodeAt(i) + ((hash << 5) - hash)
  }
  return colors[Math.abs(hash) % colors.length]!
}

function getInitial(name: string): string {
  return name.charAt(0).toUpperCase()
}

function handleDelete() {
  ElMessageBox.confirm(t('chat.confirmDelete'), t('common.confirm'), {
    confirmButtonText: t('common.delete'),
    cancelButtonText: t('common.cancel'),
    type: 'warning',
  }).then(() => {
    emit('delete', props.message.id)
  }).catch(() => {
    // 取消删除
  })
}

function handleEdit() {
  emit('edit', props.message)
}
</script>

<template>
  <div
    class="bubble-row"
    :data-message-id="message.id"
    :class="{
      'bubble-row--own': isOwn,
      'bubble-row--deleted': isDeleted,
      'bubble-row--sending': isSending,
      'bubble-row--error': isError,
      'bubble-row--highlight': highlight,
    }"
  >
    <!-- 系统消息（居中简约） -->
    <template v-if="message.is_system">
      <div class="bubble-system">
        <span class="bubble-system__dot" />
        <span class="bubble-system__text">{{ message.content }}</span>
        <span class="bubble-system__time">{{ displayTime }}</span>
      </div>
    </template>

    <!-- 普通消息 -->
    <template v-else>
      <!-- 他人消息：左侧头像 -->
      <div v-if="!isOwn" class="bubble-avatar">
        <div
          class="bubble-avatar__circle"
          :style="{ background: getAvatarColor(displayName) }"
        >
          {{ getInitial(displayName) }}
        </div>
      </div>

      <!-- 气泡主体 -->
      <div class="bubble-content" :class="{ 'bubble-content--own': isOwn }">
        <!-- 操作栏（悬停显示） -->
        <div class="bubble-actions">
          <button :title="t('chat.react')" @click="showEmojiPicker = !showEmojiPicker">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="12" cy="12" r="10"/><path d="M8 14s1.5 2 4 2 4-2 4-2"/><line x1="9" y1="9" x2="9.01" y2="9"/><line x1="15" y1="9" x2="15.01" y2="9"/></svg>
          </button>
          <button
            :title="isPinned ? t('chat.unpinMessage') : t('chat.pinMessage')"
            :class="{ 'pin-btn--active': isPinned }"
            @click="isPinned
              ? messageStore.unpinMessage(message.id, message.room_id)
              : messageStore.pinMessage(message.id, message.room_id)
            "
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="16" height="16"><path d="M12 2L15.09 8.26L22 9.27L17 14.14L18.18 21.02L12 17.77L5.82 21.02L7 14.14L2 9.27L8.91 8.26L12 2Z"/></svg>
          </button>
          <button :title="t('chat.reply')" @click="emit('reply', message)">
            <el-icon :size="16"><ChatRound /></el-icon>
          </button>
          <button
            v-if="isOwn && !isDeleted"
            :title="t('common.edit')"
            @click="handleEdit"
          >
            <el-icon :size="16"><Edit /></el-icon>
          </button>
          <button v-if="isOwn && !isDeleted" title="More" class="more-btn">
            <el-dropdown trigger="click" placement="bottom-end" @command="(cmd: string) => {
              if (cmd === 'delete') handleDelete()
            }">
              <span class="more-trigger">⋯</span>
              <template #dropdown>
                <el-dropdown-menu>
                  <el-dropdown-item command="delete">{{ t('common.delete') }}</el-dropdown-item>
                </el-dropdown-menu>
              </template>
            </el-dropdown>
          </button>
        </div>

        <!-- 气泡本体 -->
        <div class="bubble" :class="{ 'bubble--own': isOwn }">
          <!-- 发送失败标记 -->
          <div v-if="isError" class="bubble-error-icon" :title="t('chat.sendFailed')">
            <el-icon :size="18"><CircleClose /></el-icon>
          </div>

          <!-- 头部：名字 + 时间 -->
          <div class="bubble-header">
            <span class="bubble-author">{{ isOwn ? t('chat.you') : displayName }}</span>
            <span class="bubble-time">{{ displayTime }}</span>
            <span v-if="isEdited" class="bubble-edited" @click.stop="showEditHistory = !showEditHistory">({{ t('chat.edited') }})</span>
            <span v-if="isPinned" class="bubble-pinned" :title="t('chat.pinnedMessage')">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="12" height="12"><path d="M12 2L15.09 8.26L22 9.27L17 14.14L18.18 21.02L12 17.77L5.82 21.02L7 14.14L2 9.27L8.91 8.26L12 2Z"/></svg>
            </span>
          </div>

          <!-- 回复引用（点击跳转到原消息） -->
          <div
            v-if="message.reply_to_message"
            class="bubble-reply"
            :title="t('chat.jumpToMessage')"
            @click.stop="message.reply_to && emit('jumpToThread', message.reply_to)"
          >
            <span class="bubble-reply__line" />
            <div class="bubble-reply__body">
              <span class="bubble-reply__author">{{ message.reply_to_message.sender.username }}</span>
              <span class="bubble-reply__text">{{ message.reply_to_message.content }}</span>
            </div>
          </div>

          <!-- 消息正文 -->
          <div v-if="isDeleted" class="bubble-text bubble-text--deleted">
            {{ t('chat.messageDeleted') }}
          </div>
          <div v-else-if="isImageUrl" class="bubble-image">
            <div v-if="!imageLoaded && !imageError" class="bubble-image__placeholder">
              <el-skeleton animated style="width: 100%; height: 200px" />
            </div>
            <img
              v-show="imageLoaded && !imageError"
              :src="message.content"
              alt=""
              loading="lazy"
              @click.stop
              @load="imageLoaded = true"
              @error="imageError = true"
            />
            <div v-if="imageError" class="bubble-image__error" @click="imageError = false; imageLoaded = false">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" width="24" height="24"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
              <span>加载失败，点击重试</span>
            </div>
          </div>
          <div v-else-if="isFileUrl" class="bubble-file">
            <el-icon :size="28" class="bubble-file__icon"><Document /></el-icon>
            <a :href="message.content" target="_blank" class="bubble-file__name" rel="noopener">
              {{ message.content.split('/').pop() || message.content }}
            </a>
          </div>
          <div v-else class="bubble-text bubble-text--md" v-html="renderMarkdown(message.content)" />

          <!-- 已删除&自己消息下的操作 -->
          <div v-if="isDeleted && isOwn" class="bubble-deleted-actions">
            <button @click="handleDelete">{{ t('common.delete') }}</button>
          </div>
        </div>

        <!-- 反应区域 -->
        <div v-if="!isDeleted && message.reactions && message.reactions.length > 0" class="bubble-reactions">
          <button
            v-for="r in message.reactions"
            :key="r.emoji"
            class="bubble-reactions__pill"
            :class="{ 'bubble-reactions__pill--active': hasReacted(r.emoji) }"
            @click="handleReactionClick(r.emoji)"
          >
            <span class="bubble-reactions__emoji">{{ r.emoji }}</span>
            <span class="bubble-reactions__count">{{ r.count }}</span>
          </button>
        </div>

        <!-- 发送中状态指示 -->
        <div v-if="isSending" class="bubble-sending-indicator">
          <span class="sending-dot" />
          <span class="sending-dot" />
          <span class="sending-dot" />
        </div>

        <!-- 已读回执（自己的消息） -->
        <div v-if="isOwn && !isSending && !isDeleted" class="bubble-read-receipt">
          <span v-if="message.read" class="bubble-read-receipt__read">{{ t('chat.read') }}</span>
          <span v-else class="bubble-read-receipt__sent">{{ t('chat.sent') }}</span>
        </div>

        <!-- Emoji 选择器 -->
        <div style="position: relative">
          <EmojiPicker
            :visible="showEmojiPicker"
            @select="handleEmojiSelect"
            @close="showEmojiPicker = false"
          />
        </div>

        <!-- 编辑历史面板 -->
        <div v-if="showEditHistory" class="edit-history-wrapper" @click.stop>
          <EditHistoryPanel
            :message-id="message.id"
            @close="showEditHistory = false"
          />
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped lang="scss">
.bubble-row {
  display: flex;
  gap: 10px;
  padding: 4px 16px;
  position: relative;
  transition: background 0.1s;

  &:hover {
    background: var(--message-hover);

    .bubble-actions {
      opacity: 1;
      pointer-events: auto;
    }
  }

  &--deleted {
    .bubble {
      opacity: 0.55;
    }
  }

  &--sending {
    .bubble {
      opacity: 0.7;
    }
  }

  &--error {
    .bubble {
      border-color: var(--accent-pink);
    }
  }

  &--highlight {
    animation: highlight-pulse 2s ease-out;
  }
}

@keyframes highlight-pulse {
  0% {
    background: color-mix(in oklch, var(--accent) 20%, transparent);
  }
  100% {
    background: transparent;
  }
}

// ─── 头像 ───────────────────────────────────────
.bubble-avatar {
  flex-shrink: 0;
  padding-top: 8px;

  &__circle {
    width: 34px;
    height: 34px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    font-size: 14px;
    font-weight: 600;
    color: #fff;
  }
}

// ─── 气泡容器 ────────────────────────────────────
.bubble-content {
  display: flex;
  flex-direction: column;
  max-width: 75%;
  position: relative;

  &--own {
    margin-left: auto;
    align-items: flex-end;
  }
}

// ─── 编辑历史面板 ─────────────────────────────
.edit-history-wrapper {
  position: absolute;
  left: 0;
  top: 100%;
  margin-top: 8px;
  z-index: 60;
}

// ─── 悬停操作栏 ─────────────────────────────────
.bubble-actions {
  display: flex;
  gap: 2px;
  padding: 2px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  margin-bottom: 4px;
  opacity: 0;
  pointer-events: none;
  transition: opacity 0.1s;
  align-self: flex-end;

  button {
    background: none;
    border: none;
    color: var(--muted);
    cursor: pointer;
    padding: 4px 6px;
    border-radius: 4px;
    font-size: 13px;
    display: flex;
    align-items: center;

    &:hover {
      color: var(--fg);
      background: var(--message-hover);
    }

    svg {
      width: 16px;
      height: 16px;
    }
  }

  .pin-btn--active {
    color: var(--accent);
  }

  .more-trigger {
    letter-spacing: 1px;
    font-weight: 700;
  }

  :deep(.el-dropdown) {
    display: flex;
    align-items: center;
  }
}

// ─── 气泡本体 ────────────────────────────────────
.bubble {
  position: relative;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 6px 16px 16px 16px;
  padding: 8px 14px;
  min-width: 80px;

  &::before {
    content: '';
    position: absolute;
    left: -7px;
    top: 12px;
    width: 0;
    height: 0;
    border-top: 6px solid transparent;
    border-bottom: 6px solid transparent;
    border-right: 7px solid var(--border);
  }

  &::after {
    content: '';
    position: absolute;
    left: -5px;
    top: 13px;
    width: 0;
    height: 0;
    border-top: 5px solid transparent;
    border-bottom: 5px solid transparent;
    border-right: 6px solid var(--surface);
  }

  &--own {
    background: color-mix(in oklch, var(--accent) 18%, var(--surface));
    border-color: color-mix(in oklch, var(--accent) 30%, var(--border));
    border-radius: 16px 6px 16px 16px;

    &::before {
      left: auto;
      right: -7px;
      border-right: none;
      border-left: 7px solid color-mix(in oklch, var(--accent) 30%, var(--border));
    }

    &::after {
      left: auto;
      right: -5px;
      border-right: none;
      border-left: 6px solid color-mix(in oklch, var(--accent) 18%, var(--surface));
    }
  }
}

// ─── 发送失败图标 ──────────────────────────────
.bubble-error-icon {
  position: absolute;
  left: -28px;
  top: 50%;
  transform: translateY(-50%);
  color: var(--accent-pink);
  cursor: help;
}

// ─── 头部 ─────────────────────────────────────
.bubble-header {
  display: flex;
  align-items: baseline;
  gap: 8px;
  margin-bottom: 4px;
}

.bubble-author {
  font-size: 14px;
  font-weight: 600;
  color: var(--accent);

  &:hover {
    text-decoration: underline;
    cursor: pointer;
  }
}

.bubble--own .bubble-author {
  color: var(--accent-pink);
}

.bubble-time {
  font-size: 11px;
  color: var(--muted);
}

.bubble-edited {
  font-size: 11px;
  color: var(--muted);
}

.bubble-pinned {
  display: inline-flex;
  align-items: center;
  color: var(--accent);

  svg {
    display: block;
  }
}

// ─── 回复引用 ────────────────────────────────
.bubble-reply {
  display: flex;
  gap: 8px;
  margin-bottom: 6px;
  padding: 6px 8px;
  background: color-mix(in oklch, var(--fg) 4%, transparent);
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.15s;

  &:hover {
    background: color-mix(in oklch, var(--fg) 8%, transparent);
  }

  &__line {
    width: 3px;
    min-height: 30px;
    border-radius: 2px;
    background: var(--accent);
    flex-shrink: 0;
  }

  &__body {
    flex: 1;
    min-width: 0;
  }

  &__author {
    font-size: 12px;
    font-weight: 600;
    color: var(--accent);
    display: block;
  }

  &__text {
    font-size: 12px;
    color: var(--muted);
    display: block;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
}

.bubble--own .bubble-reply {
  &__line {
    background: var(--accent-pink);
  }
  &__author {
    color: var(--accent-pink);
  }
}

// ─── 图片消息 ───────────────────────────────
.bubble-image {
  margin: -8px -14px;
  border-radius: 6px 16px 16px 16px;
  overflow: hidden;
  line-height: 0;
  min-height: 60px;
  background: color-mix(in oklch, var(--fg) 4%, transparent);

  img {
    max-width: 100%;
    max-height: 400px;
    object-fit: contain;
    cursor: pointer;
    display: block;
  }

  &__placeholder {
    min-height: 200px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  &__error {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 32px;
    color: var(--muted);
    font-size: 13px;
    cursor: pointer;
    min-height: 100px;

    &:hover {
      color: var(--accent);
    }
  }
}

// ─── 文件消息 ───────────────────────────────
.bubble-file {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px;
  background: color-mix(in oklch, var(--fg) 4%, transparent);
  border-radius: 8px;

  &__icon {
    width: 28px;
    height: 28px;
    flex-shrink: 0;
    color: var(--accent);
  }

  &__name {
    font-size: 14px;
    color: var(--accent);
    text-decoration: none;
    word-break: break-all;
    overflow: hidden;
    text-overflow: ellipsis;

    &:hover {
      text-decoration: underline;
    }
  }
}

// ─── 正文 ─────────────────────────────────────
.bubble-text {
  font-size: 15px;
  line-height: 1.5;
  color: var(--fg);

  p {
    margin: 0;
    white-space: pre-wrap;
    word-break: break-word;
  }

  &--deleted {
    font-style: italic;
    color: var(--muted);
    font-size: 13px;
  }

  &--md {
    p { margin: 0 0 4px; &:last-child { margin: 0; } }
    a { color: var(--accent); text-decoration: underline; }
    code {
      background: color-mix(in oklch, var(--fg) 8%, transparent);
      padding: 1px 4px;
      border-radius: 3px;
      font-size: 13px;
      font-family: 'SF Mono', 'Fira Code', monospace;
    }
    pre {
      background: color-mix(in oklch, var(--fg) 6%, transparent);
      padding: 8px 12px;
      border-radius: 6px;
      overflow-x: auto;
      margin: 6px 0;
      code { background: none; padding: 0; }
    }
    blockquote {
      border-left: 3px solid var(--accent);
      padding-left: 10px;
      margin: 6px 0;
      color: var(--muted);
    }
    ul, ol { margin: 4px 0; padding-left: 20px; }
    li { margin: 2px 0; }
    h1, h2, h3, h4, h5, h6 { margin: 8px 0 4px; font-weight: 600; }
    h1 { font-size: 17px; }
    h2 { font-size: 16px; }
    h3 { font-size: 15px; }
    hr { border: none; border-top: 1px solid var(--border); margin: 8px 0; }
    del { opacity: 0.6; }
  }
}

// ─── 删除消息操作 ─────────────────────────────
.bubble-deleted-actions {
  margin-top: 4px;

  button {
    background: none;
    border: none;
    color: var(--muted);
    font-size: 12px;
    cursor: pointer;
    padding: 2px 4px;

    &:hover {
      color: var(--accent-pink);
    }
  }
}

// ─── 反应区域 ─────────────────────────────
.bubble-reactions {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  margin-top: 6px;

  &__pill {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    padding: 2px 8px;
    border-radius: 12px;
    border: 1px solid var(--border, #e0e0e0);
    background: var(--surface, #fff);
    cursor: pointer;
    font-size: 13px;
    line-height: 1.4;
    transition: background 0.1s, border-color 0.1s;

    &:hover {
      background: color-mix(in oklch, var(--accent) 8%, var(--surface));
      border-color: var(--accent);
    }

    &--active {
      background: color-mix(in oklch, var(--accent) 14%, var(--surface));
      border-color: var(--accent);
    }
  }

  &__emoji {
    font-size: 15px;
  }

  &__count {
    font-size: 11px;
    color: var(--muted);
    font-weight: 500;
  }
}

// ─── 已读回执 ─────────────────────────────
.bubble-read-receipt {
  font-size: 11px;
  line-height: 1;
  margin-top: 2px;

  &__sent {
    color: var(--muted);
  }

  &__read {
    color: var(--accent);
  }
}

// ─── 发送中动画 ─────────────────────────────
.bubble-sending-indicator {
  display: flex;
  gap: 4px;
  padding: 4px 0;
}

.sending-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--muted);
  animation: sendingPulse 1.2s ease-in-out infinite;

  &:nth-child(2) { animation-delay: 0.2s; }
  &:nth-child(3) { animation-delay: 0.4s; }
}

@keyframes sendingPulse {
  0%, 80%, 100% { opacity: 0.3; transform: scale(0.8); }
  40% { opacity: 1; transform: scale(1); }
}

// ─── 系统消息 ─────────────────────────────────
.bubble-system {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 4px 0;
  justify-content: center;

  &__dot {
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: var(--muted);
    flex-shrink: 0;
  }

  &__text {
    font-size: 12px;
    color: var(--muted);
    text-align: center;
  }

  &__time {
    font-size: 11px;
    color: var(--muted);
    opacity: 0.6;
    flex-shrink: 0;
  }
}
</style>
