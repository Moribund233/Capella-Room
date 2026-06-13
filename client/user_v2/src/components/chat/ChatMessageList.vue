<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { storeToRefs } from 'pinia'
import { useMessageStore } from '@/stores/message'
import { useAuthStore } from '@/stores/auth'
import { shouldShowTimeSeparator, formatTime } from '@/utils/date'
import { Loading, WarningFilled, ArrowDown } from '@element-plus/icons-vue'
import ChatMessageBubble from './ChatMessageBubble.vue'
import type { Message } from '@/types/message'

const { t } = useI18n()
const messageStore = useMessageStore()
const authStore = useAuthStore()

const { messages, loading, loadingMore, hasMore, error } = storeToRefs(messageStore)

const props = withDefaults(defineProps<{
  typingUsers?: Array<{ id: string; username: string }>
}>(), {
  typingUsers: () => [],
})

const emit = defineEmits<{
  reply: [message: Message]
  edit: [message: Message]
  delete: [id: string]
  jumpToThread: [messageId: string | undefined]
}>()

const listRef = ref<HTMLElement | null>(null)
const autoScroll = ref(true)
const newMessageCount = ref(0)

// 监听消息变化，自动滚动到底部
watch(
  () => messages.value.length,
  () => {
    if (autoScroll.value) {
      nextTick(() => {
        scrollToBottom()
      })
    } else {
      newMessageCount.value++
    }
  },
)

// 滚动事件 - 检测用户是否在底部
function onScroll() {
  if (!listRef.value) return
  const el = listRef.value
  const threshold = 100
  const atBottom = el.scrollHeight - el.scrollTop - el.clientHeight < threshold
  autoScroll.value = atBottom
  if (atBottom) {
    newMessageCount.value = 0
  }
}

function scrollToBottom() {
  if (!listRef.value) return
  listRef.value.scrollTop = listRef.value.scrollHeight
}

function scrollToNewMessages() {
  scrollToBottom()
  autoScroll.value = true
  newMessageCount.value = 0
}

// 加载更多
async function loadMore() {
  if (loadingMore.value || !hasMore.value) return
  const prevHeight = listRef.value?.scrollHeight || 0
  await messageStore.fetchMore()
  nextTick(() => {
    if (listRef.value) {
      const newHeight = listRef.value.scrollHeight
      listRef.value.scrollTop = newHeight - prevHeight
    }
  })
}

// 滚动加载更多
function onScrollTop() {
  if (listRef.value && listRef.value.scrollTop < 50 && hasMore.value && !loadingMore.value) {
    loadMore()
  }
}

// 消息分组 - 按天分隔
interface MessageGroup {
  date: string
  messages: Message[]
  showDate: boolean
}

const messageGroups = computed(() => {
  const groups: MessageGroup[] = []
  let currentDate = ''

  for (const msg of messages.value) {
    const msgDate = new Date(msg.created_at).toLocaleDateString()
    if (msgDate !== currentDate) {
      currentDate = msgDate
      groups.push({
        date: msgDate,
        messages: [msg],
        showDate: true,
      })
    } else {
      groups[groups.length - 1]!.messages.push(msg)
    }
  }

  return groups
})

function isOwnMessage(message: Message): boolean {
  return message.sender.id === authStore.user?.id
}

function formatDateSeparator(dateStr: string): string {
  const date = new Date(dateStr)
  const now = new Date()
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate())
  const target = new Date(date.getFullYear(), date.getMonth(), date.getDate())
  const diffDays = Math.floor((today.getTime() - target.getTime()) / (1000 * 60 * 60 * 24))

  if (diffDays === 0) return t('chat.today')
  if (diffDays === 1) return t('chat.yesterday')
  return date.toLocaleDateString()
}

const currentHighlight = ref<string | null>(null)
let highlightTimer: ReturnType<typeof setTimeout> | null = null

function scrollToMessage(msgId: string) {
  if (!listRef.value) return
  const el = listRef.value.querySelector(`[data-message-id="${msgId}"]`) as HTMLElement | null
  if (!el) return
  el.scrollIntoView({ block: 'center', behavior: 'smooth' })

  if (highlightTimer) clearTimeout(highlightTimer)
  currentHighlight.value = msgId
  highlightTimer = setTimeout(() => {
    currentHighlight.value = null
  }, 2000)
}

defineExpose({ scrollToMessage })
</script>

<template>
  <div
    ref="listRef"
    class="messages"
    @scroll="onScroll; onScrollTop()"
  >
    <!-- 加载更多 -->
    <div v-if="hasMore" class="load-more">
      <el-button
        v-if="!loadingMore"
        text
        size="small"
        type="primary"
        @click="loadMore"
      >
        {{ t('chat.loadMore') }}
      </el-button>
      <el-icon v-else class="is-loading">
        <Loading />
      </el-icon>
    </div>

    <!-- 首次加载中 -->
    <div v-if="loading && messages.length === 0" class="messages-loading">
      <div v-for="i in 5" :key="i" class="message-skeleton">
        <div class="skeleton-avatar" />
        <div class="skeleton-body">
          <div class="skeleton-header" :style="{ width: 80 + i * 20 + 'px' }" />
          <div class="skeleton-line" :style="{ width: 150 + i * 30 + 'px' }" />
        </div>
      </div>
    </div>

    <!-- 错误状态 -->
    <div v-if="error && messages.length === 0" class="messages-error">
      <el-result
        :title="t('common.error')"
        :sub-title="error"
      >
        <template #icon>
          <el-icon :size="48" color="var(--accent-pink)"><WarningFilled /></el-icon>
        </template>
        <template #extra>
          <el-button type="primary" @click="messageStore.fetchMessages(messageStore.currentRoomId!)">
            {{ t('common.retry') }}
          </el-button>
        </template>
      </el-result>
    </div>

    <!-- 空状态 -->
    <div v-if="!loading && !error && messages.length === 0" class="messages-empty">
      <span>{{ t('chat.noMessages') }}</span>
    </div>

    <!-- 消息列表 -->
    <template v-for="group in messageGroups" :key="group.date">
      <!-- 时间分隔符 -->
      <div class="message-divider">
        <span>{{ formatDateSeparator(group.date) }}</span>
      </div>

      <!-- 消息 -->
      <ChatMessageBubble
        v-for="msg in group.messages"
        :key="msg.id"
        :message="msg"
        :is-own="isOwnMessage(msg)"
        :highlight="currentHighlight === msg.id"
        @reply="emit('reply', $event)"
        @edit="emit('edit', $event)"
        @delete="emit('delete', $event)"
        @jump-to-thread="emit('jumpToThread', $event)"
      />
    </template>

    <!-- 正在输入指示器 -->
    <div v-if="typingUsers.length > 0" class="typing-indicator">
      <span class="typing-dots">
        <span class="dot" /><span class="dot" /><span class="dot" />
      </span>
      <span class="typing-text">
        {{ typingUsers.map(u => u.username).join(', ') }}
        {{ typingUsers.length === 1 ? t('chat.isTyping') : t('chat.areTyping') }}
      </span>
    </div>

    <!-- 新消息提示 -->
    <div
      v-if="newMessageCount > 0 && !autoScroll"
      class="new-message-bar"
      @click="scrollToNewMessages"
    >
      <el-icon><ArrowDown /></el-icon>
      {{ newMessageCount }} {{ t('chat.newMessages') }}
    </div>
  </div>
</template>

<style scoped lang="scss">
.messages {
  flex: 1;
  overflow-y: auto;
  padding: 20px 24px 8px;
  display: flex;
  flex-direction: column;
}

.load-more {
  text-align: center;
  padding: 8px 0;
}

.messages-loading {
  padding: 8px 0;
}

.message-skeleton {
  display: flex;
  gap: 12px;
  padding: 8px 12px;
}

.skeleton-avatar {
  width: 40px;
  height: 40px;
  min-width: 40px;
  border-radius: 50%;
  background: var(--message-hover);
  animation: pulse 1.5s ease-in-out infinite;
}

.skeleton-body {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.skeleton-header {
  height: 14px;
  border-radius: 4px;
  background: var(--message-hover);
  animation: pulse 1.5s ease-in-out infinite;
}

.skeleton-line {
  height: 12px;
  border-radius: 4px;
  background: var(--message-hover);
  animation: pulse 1.5s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 0.4; }
  50% { opacity: 0.8; }
}

.messages-error {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.messages-empty {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--muted);
  font-size: 14px;
}

.message-divider {
  text-align: center;
  font-size: 12px;
  color: var(--muted);
  border-top: 1px solid var(--border);
  margin: 24px 0;
  padding-top: 8px;

  span {
    background: var(--bg);
    padding: 0 12px;
    position: relative;
    top: -16px;
  }
}

.typing-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  font-size: 13px;
  color: var(--muted);
}

.typing-dots {
  display: flex;
  align-items: center;
  gap: 3px;
}

.dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--muted);
  animation: typingBounce 1.4s ease-in-out infinite;
}

.dot:nth-child(2) {
  animation-delay: 0.2s;
}

.dot:nth-child(3) {
  animation-delay: 0.4s;
}

@keyframes typingBounce {
  0%, 60%, 100% {
    transform: translateY(0);
    opacity: 0.4;
  }
  30% {
    transform: translateY(-4px);
    opacity: 1;
  }
}

.typing-text {
  font-style: italic;
}

.new-message-bar {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 0;
  font-size: 12px;
  color: var(--accent);
  cursor: pointer;
  justify-content: center;

  &:hover {
    opacity: 1;
  }
}

@media (max-width: 640px) {
  .messages {
    padding: 12px 8px;
  }
}
</style>
