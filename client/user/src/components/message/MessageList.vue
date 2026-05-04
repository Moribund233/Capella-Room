<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { isSameDay } from '@/utils/date'
import MessageBubble from './MessageBubble.vue'
import type { Message } from '@/types/message'

const props = defineProps<{
  messages: Message[]
  loading: boolean
  loadingMore: boolean
  hasMore: boolean
  currentUserId: string
}>()

const emit = defineEmits<{
  loadMore: []
  reply: [message: Message]
  edit: [message: Message]
  delete: [message: Message]
}>()

const listRef = ref<HTMLDivElement | null>(null)
const showScrollBtn = ref(false)
const autoScroll = ref(true)

/** 判断是否需要显示日期分隔符 */
function shouldShowDate(index: number): boolean {
  if (index === 0) return true
  const current = props.messages[index]
  const prev = props.messages[index - 1]
  if (!current || !prev) return true
  return !isSameDay(current.created_at, prev.created_at)
}

/** 格式化日期分隔符文本 */
function dateLabel(dateStr: string): string {
  const d = new Date(dateStr)
  const now = new Date()
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate())
  const target = new Date(d.getFullYear(), d.getMonth(), d.getDate())
  const diff = Math.floor((today.getTime() - target.getTime()) / 86400000)

  if (diff === 0) return '今天'
  if (diff === 1) return '昨天'
  if (diff < 7) return (['星期日', '星期一', '星期二', '星期三', '星期四', '星期五', '星期六'])[d.getDay()] ?? ''

  const y = d.getFullYear()
  const m = String(d.getMonth() + 1).padStart(2, '0')
  const day = String(d.getDate()).padStart(2, '0')
  if (y === now.getFullYear()) return `${m}月${day}日`
  return `${y}年${m}月${day}日`
}

/** 滚动到底部 */
function scrollToBottom(smooth = true) {
  nextTick(() => {
    const el = listRef.value
    if (!el) return
    el.scrollTo({
      top: el.scrollHeight,
      behavior: smooth ? 'smooth' : 'instant',
    })
    showScrollBtn.value = false
    autoScroll.value = true
  })
}

/** 处理滚动事件 */
function handleScroll() {
  const el = listRef.value
  if (!el) return

  // 是否在底部附近（阈值 100px）
  const nearBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 100
  autoScroll.value = nearBottom
  showScrollBtn.value = !nearBottom

  // 滚动到顶部时加载更多
  if (el.scrollTop < 80 && props.hasMore && !props.loadingMore && !props.loading) {
    emit('loadMore')
  }
}

// 监听消息变化，自动滚动
watch(
  () => props.messages.length,
  () => {
    if (autoScroll.value) {
      scrollToBottom()
    }
  },
)

// 监听 loadingMore 变化（加载完成后保持滚动位置）
watch(
  () => props.loadingMore,
  (loading) => {
    if (!loading && props.messages.length > 0) {
      // 保留滚动位置，不自动跳转
    }
  },
)

// 首次加载完成后滚动到底部
watch(
  () => props.loading,
  (loading) => {
    if (!loading && props.messages.length > 0) {
      scrollToBottom(false)
    }
  },
)

defineExpose({ scrollToBottom })
</script>

<template>
  <div class="message-list" ref="listRef" @scroll="handleScroll">
    <!-- 加载更多 -->
    <div v-if="loading && messages.length === 0" class="message-list__loading">
      <span>加载中...</span>
    </div>

    <template v-if="!loading || messages.length > 0">
      <!-- 加载更多指示器 -->
      <div v-if="hasMore" class="message-list__more">
        <span v-if="loadingMore" class="message-list__more-text">加载更多...</span>
        <button
          v-else
          class="message-list__more-btn"
          @click="emit('loadMore')"
        >
          加载更多消息
        </button>
      </div>

      <!-- 消息列表 -->
      <div v-if="messages.length === 0 && !loading" class="message-list__empty">
        暂无消息，发送第一条消息吧
      </div>

      <template v-for="(msg, index) in messages" :key="msg.id">
        <!-- 日期分隔符 -->
        <div v-if="shouldShowDate(index)" class="message-list__date">
          <span class="message-list__date-text">{{ dateLabel(msg.created_at) }}</span>
        </div>

        <MessageBubble
          :message="msg"
          :is-own="msg.sender.id === currentUserId"
          @reply="emit('reply', $event)"
          @edit="emit('edit', $event)"
          @delete="emit('delete', $event)"
        />
      </template>
    </template>

    <!-- 滚动到底部按钮 -->
    <button
      v-if="showScrollBtn && messages.length > 0"
      class="message-list__scroll-btn"
      @click="scrollToBottom()"
      title="滚动到底部"
    >
      ↓
    </button>
  </div>
</template>

<style scoped>
.message-list {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-md) var(--space-lg);
  display: flex;
  flex-direction: column;
  gap: 2px;
  position: relative;
}

/* Loading state */
.message-list__loading {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100%;
  color: var(--color-text-tertiary);
  font-size: var(--font-size-small);
}

/* Load more */
.message-list__more {
  display: flex;
  justify-content: center;
  padding: var(--space-sm) 0;
}

.message-list__more-text {
  font-size: var(--font-size-small);
  color: var(--color-text-tertiary);
}

.message-list__more-btn {
  font-size: var(--font-size-small);
  color: var(--color-primary);
  background: none;
  border: none;
  cursor: pointer;
  padding: 4px 12px;
  border-radius: var(--radius-sm);
  transition: background var(--duration-fast);
}

.message-list__more-btn:hover {
  background: var(--color-primary-light);
}

/* Empty state */
.message-list__empty {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100%;
  color: var(--color-text-tertiary);
  font-size: var(--font-size-small);
}

/* Date separator */
.message-list__date {
  display: flex;
  justify-content: center;
  padding: var(--space-md) 0 var(--space-sm);
}

.message-list__date-text {
  font-size: var(--font-size-small);
  color: var(--color-text-tertiary);
  background: var(--color-background);
  padding: 2px 12px;
  border-radius: var(--radius-full);
}

/* Scroll to bottom button */
.message-list__scroll-btn {
  position: sticky;
  bottom: 8px;
  left: 50%;
  transform: translateX(-50%);
  width: 36px;
  height: 36px;
  border-radius: var(--radius-full);
  border: 1px solid var(--color-border);
  background: var(--color-white);
  color: var(--color-text-secondary);
  font-size: 18px;
  cursor: pointer;
  box-shadow: var(--shadow-md);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all var(--duration-fast);
  z-index: 10;
}

.message-list__scroll-btn:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
  box-shadow: var(--shadow-lg);
}
</style>
