<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { messageApi } from '@/api/message'
import type { Message } from '@/types/message'

const props = defineProps<{
  roomId: string
  visible: boolean
}>()

const emit = defineEmits<{
  close: []
  jumpToMessage: [messageId: string]
}>()

const searchQuery = ref('')
const searchResults = ref<Message[]>([])
const loading = ref(false)
const inputRef = ref<HTMLInputElement | null>(null)

watch(
  () => props.visible,
  (visible) => {
    if (visible) {
      nextTick(() => {
        inputRef.value?.focus()
      })
    } else {
      searchQuery.value = ''
      searchResults.value = []
    }
  }
)

async function handleSearch() {
  const query = searchQuery.value.trim()
  if (!query) {
    searchResults.value = []
    return
  }

  loading.value = true
  try {
    const results = await messageApi.searchMessages({
      q: query,
      room_id: props.roomId,
      limit: 20,
    })
    searchResults.value = results
  } catch (err) {
    console.error('[MessageSearch] search error:', err)
    searchResults.value = []
  } finally {
    loading.value = false
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    emit('close')
  }
  if (e.key === 'Enter') {
    handleSearch()
  }
}

function handleResultClick(message: Message) {
  emit('jumpToMessage', message.id)
  emit('close')
}

function formatContent(content: string, query: string): string {
  if (!query) return content
  const maxLength = 80
  const lowerContent = content.toLowerCase()
  const lowerQuery = query.toLowerCase()
  const index = lowerContent.indexOf(lowerQuery)

  if (index === -1) {
    return content.length > maxLength ? content.slice(0, maxLength) + '...' : content
  }

  const start = Math.max(0, index - 20)
  const end = Math.min(content.length, index + query.length + 20)
  let result = content.slice(start, end)

  if (start > 0) result = '...' + result
  if (end < content.length) result = result + '...'

  return result
}

function highlightMatch(content: string, query: string): string {
  if (!query) return content
  const escapedQuery = query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  const regex = new RegExp(`(${escapedQuery})`, 'gi')
  return content.replace(regex, '<mark>$1</mark>')
}
</script>

<template>
  <div v-if="visible" class="message-search" @click.self="emit('close')">
    <div class="message-search__panel">
      <div class="message-search__header">
        <h3 class="message-search__title">搜索消息</h3>
        <button class="message-search__close" @click="emit('close')">✕</button>
      </div>

      <div class="message-search__input-wrapper">
        <input
          ref="inputRef"
          v-model="searchQuery"
          type="text"
          class="message-search__input"
          placeholder="输入关键词搜索..."
          @keydown="handleKeydown"
        />
        <button
          class="message-search__button"
          :disabled="loading || !searchQuery.trim()"
          @click="handleSearch"
        >
          {{ loading ? '搜索中...' : '搜索' }}
        </button>
      </div>

      <div class="message-search__results">
        <div v-if="loading" class="message-search__loading">搜索中...</div>
        <div v-else-if="searchResults.length === 0 && searchQuery" class="message-search__empty">
          未找到匹配的消息
        </div>
        <div v-else-if="searchResults.length === 0" class="message-search__hint">
          输入关键词开始搜索
        </div>
        <div v-else class="message-search__list">
          <div
            v-for="message in searchResults"
            :key="message.id"
            class="message-search__item"
            @click="handleResultClick(message)"
          >
            <div class="message-search__item-header">
              <span class="message-search__author">{{ message.sender.username }}</span>
              <span class="message-search__time">{{ new Date(message.created_at).toLocaleString() }}</span>
            </div>
            <div
              class="message-search__content"
              v-html="highlightMatch(formatContent(message.content, searchQuery), searchQuery)"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.message-search {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.message-search__panel {
  width: 90%;
  max-width: 600px;
  max-height: 80vh;
  background: var(--color-white);
  border-radius: var(--radius-lg);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.message-search__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-md) var(--space-lg);
  border-bottom: 1px solid var(--color-border);
}

.message-search__title {
  font-size: var(--font-size-h3);
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0;
}

.message-search__close {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: none;
  cursor: pointer;
  color: var(--color-text-tertiary);
  font-size: 18px;
  border-radius: var(--radius-sm);
  transition: all var(--duration-fast);
}

.message-search__close:hover {
  background: var(--color-background);
  color: var(--color-text-primary);
}

.message-search__input-wrapper {
  display: flex;
  gap: var(--space-sm);
  padding: var(--space-md) var(--space-lg);
  border-bottom: 1px solid var(--color-border);
}

.message-search__input {
  flex: 1;
  padding: 10px 14px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  font-size: var(--font-size-body);
  outline: none;
  transition: border-color var(--duration-fast);
}

.message-search__input:focus {
  border-color: var(--color-primary);
}

.message-search__button {
  padding: 10px 20px;
  background: var(--color-primary);
  color: #fff;
  border: none;
  border-radius: var(--radius-md);
  font-size: var(--font-size-body);
  cursor: pointer;
  transition: opacity var(--duration-fast);
  white-space: nowrap;
}

.message-search__button:hover:not(:disabled) {
  opacity: 0.9;
}

.message-search__button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.message-search__results {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-md);
  min-height: 200px;
  max-height: 400px;
}

.message-search__loading,
.message-search__empty,
.message-search__hint {
  text-align: center;
  padding: var(--space-xl);
  color: var(--color-text-secondary);
  font-size: var(--font-size-body);
}

.message-search__list {
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
}

.message-search__item {
  padding: var(--space-md);
  background: var(--color-background);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: background-color var(--duration-fast);
}

.message-search__item:hover {
  background: var(--color-border);
}

.message-search__item-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--space-xs);
}

.message-search__author {
  font-size: var(--font-size-small);
  font-weight: 600;
  color: var(--color-primary);
}

.message-search__time {
  font-size: var(--font-size-tiny);
  color: var(--color-text-tertiary);
}

.message-search__content {
  font-size: var(--font-size-body);
  color: var(--color-text-primary);
  line-height: var(--line-height-body);
  word-break: break-word;
}

.message-search__content :deep(mark) {
  background: var(--color-primary-light, rgba(7, 193, 96, 0.2));
  color: var(--color-primary);
  padding: 0 2px;
  border-radius: 2px;
}
</style>
