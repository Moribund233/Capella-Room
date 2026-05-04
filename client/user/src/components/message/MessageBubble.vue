<script setup lang="ts">
import { ref, onUnmounted } from 'vue'
import { formatTime } from '@/utils/date'
import MessageActions from './MessageActions.vue'
import type { Message } from '@/types/message'

const props = defineProps<{
  message: Message
  isOwn: boolean
}>()

const emit = defineEmits<{
  reply: [message: Message]
  edit: [message: Message]
  delete: [message: Message]
}>()

const showActions = ref(false)
const bubbleRef = ref<HTMLElement>()

// 长按相关
let longPressTimer: ReturnType<typeof setTimeout> | null = null
const LONG_PRESS_DELAY = 500
let touchStartX = 0
let touchStartY = 0

function handleContextMenu(event: MouseEvent) {
  event.preventDefault()
  showActions.value = true
}

// 触摸开始 - 开始长按计时
function onTouchStart(e: TouchEvent) {
  const touch = e.touches[0]
  if (!touch) return
  touchStartX = touch.clientX
  touchStartY = touch.clientY

  longPressTimer = setTimeout(() => {
    showActions.value = true
    // 触发震动反馈（如果支持）
    if (navigator.vibrate) {
      navigator.vibrate(50)
    }
  }, LONG_PRESS_DELAY)
}

// 触摸移动 - 如果移动距离过大，取消长按
function onTouchMove(e: TouchEvent) {
  if (!longPressTimer) return

  const touch = e.touches[0]
  if (!touch) return
  const deltaX = Math.abs(touch.clientX - touchStartX)
  const deltaY = Math.abs(touch.clientY - touchStartY)

  // 如果移动超过 10px，取消长按
  if (deltaX > 10 || deltaY > 10) {
    clearTimeout(longPressTimer)
    longPressTimer = null
  }
}

// 触摸结束 - 清除长按计时器
function onTouchEnd() {
  if (longPressTimer) {
    clearTimeout(longPressTimer)
    longPressTimer = null
  }
}

// 触摸取消 - 清除长按计时器
function onTouchCancel() {
  if (longPressTimer) {
    clearTimeout(longPressTimer)
    longPressTimer = null
  }
}

function handleReply() {
  emit('reply', props.message)
  showActions.value = false
}

function handleEdit() {
  emit('edit', props.message)
  showActions.value = false
}

function handleDelete() {
  emit('delete', props.message)
  showActions.value = false
}

onUnmounted(() => {
  if (longPressTimer) {
    clearTimeout(longPressTimer)
  }
})
</script>

<template>
  <div
    ref="bubbleRef"
    class="message-bubble"
    :class="{
      'message-bubble--own': isOwn,
      'message-bubble--other': !isOwn,
      'message-bubble--failed': message.error,
    }"
    @contextmenu="handleContextMenu"
    @touchstart="onTouchStart"
    @touchmove="onTouchMove"
    @touchend="onTouchEnd"
    @touchcancel="onTouchCancel"
  >
    <!-- 头像 -->
    <div class="message-bubble__avatar">
      {{ message.sender.username.charAt(0).toUpperCase() }}
    </div>

    <div class="message-bubble__body">
      <!-- 发送者名称 -->
      <span class="message-bubble__sender">
        {{ message.sender.username }}
      </span>

      <!-- 回复预览 -->
      <div
        v-if="message.reply_to_message"
        class="message-bubble__reply"
        @click.stop
      >
        <div class="message-bubble__reply-author">
          {{ message.reply_to_message.sender.username }}
        </div>
        <div class="message-bubble__reply-preview">
          {{ message.reply_to_message.content }}
        </div>
      </div>

      <!-- 已删除消息 -->
      <div v-if="message.is_deleted" class="message-bubble__deleted">
        此消息已被删除
      </div>

      <!-- 消息内容 -->
      <div v-else class="message-bubble__content-wrapper">
        <div class="message-bubble__content">
          {{ message.content }}
        </div>
        <!-- 操作菜单 -->
        <MessageActions
          :message="message"
          :is-own="isOwn"
          :visible="showActions"
          @reply="handleReply"
          @edit="handleEdit"
          @delete="handleDelete"
          @close="showActions = false"
        />
      </div>

      <!-- 底部信息 -->
      <div class="message-bubble__footer">
        <span class="message-bubble__time">{{ formatTime(message.created_at) }}</span>
        <span v-if="message.edited_at" class="message-bubble__edited">已编辑</span>
        <span v-if="message.sending" class="message-bubble__status message-bubble__status--sending">
          发送中...
        </span>
        <span v-else-if="message.error" class="message-bubble__status message-bubble__status--error">
          发送失败
        </span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.message-bubble {
  display: flex;
  gap: 8px;
  max-width: 75%;
  margin-bottom: 2px;
  position: relative;
}

.message-bubble--own {
  align-self: flex-end;
  flex-direction: row-reverse;
}

.message-bubble--other {
  align-self: flex-start;
}

/* Avatar */
.message-bubble__avatar {
  width: 32px;
  height: 32px;
  border-radius: var(--radius-full);
  background: var(--color-primary);
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 13px;
  font-weight: 600;
  flex-shrink: 0;
  margin-top: 4px;
}

/* Body */
.message-bubble__body {
  display: inline-flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
  max-width: 100%;
  position: relative;
}

.message-bubble--own .message-bubble__body {
  align-items: flex-end;
}

.message-bubble__sender {
  font-size: var(--font-size-small);
  color: var(--color-text-tertiary);
  margin-bottom: 2px;
  padding: 0 4px;
}

.message-bubble--own .message-bubble__sender {
  text-align: right;
}

/* Reply preview */
.message-bubble__reply {
  padding: 6px 10px;
  margin-bottom: 2px;
  border-left: 3px solid var(--color-primary);
  border-radius: 4px;
  background: rgba(7, 193, 96, 0.06);
  cursor: default;
}

.message-bubble__reply-author {
  font-size: var(--font-size-small);
  font-weight: 600;
  color: var(--color-primary);
  margin-bottom: 2px;
}

.message-bubble__reply-preview {
  font-size: var(--font-size-small);
  color: var(--color-text-tertiary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* Content wrapper */
.message-bubble__content-wrapper {
  position: relative;
  display: inline-flex;
  flex-direction: column;
}

/* Content */
.message-bubble__content {
  padding: 8px 14px;
  font-size: var(--font-size-body);
  line-height: var(--line-height-body);
  word-break: break-word;
  white-space: pre-wrap;
  width: fit-content;
  min-width: min-content;
}

.message-bubble--other .message-bubble__content {
  background: var(--color-white);
  border: 1px solid var(--color-border);
  border-radius: 4px 12px 12px 12px;
  color: var(--color-text-primary);
}

.message-bubble--own .message-bubble__content {
  background: var(--color-primary);
  border-radius: 12px 4px 12px 12px;
  color: #fff;
}

.message-bubble--failed .message-bubble__content {
  opacity: 0.6;
}

/* Deleted message */
.message-bubble__deleted {
  padding: 6px 14px;
  font-size: var(--font-size-small);
  color: var(--color-text-quaternary);
  font-style: italic;
  background: var(--color-background);
  border-radius: 4px 12px 12px 12px;
}

.message-bubble--own .message-bubble__deleted {
  border-radius: 12px 4px 12px 12px;
}

/* Footer */
.message-bubble__footer {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 0 4px;
  margin-top: 1px;
}

.message-bubble--own .message-bubble__footer {
  justify-content: flex-end;
}

.message-bubble__time {
  font-size: var(--font-size-tiny);
  color: var(--color-text-quaternary);
}

.message-bubble--own .message-bubble__time {
  color: rgba(255, 255, 255, 0.6);
}

.message-bubble__edited {
  font-size: var(--font-size-tiny);
  color: var(--color-text-quaternary);
}

.message-bubble__status {
  font-size: var(--font-size-tiny);
}

.message-bubble__status--sending {
  color: var(--color-text-quaternary);
}

.message-bubble__status--error {
  color: var(--color-error);
}

.message-bubble--own .message-bubble__status--sending {
  color: rgba(255, 255, 255, 0.6);
}

.message-bubble--own .message-bubble__status--error {
  color: #ffccc7;
}
</style>
