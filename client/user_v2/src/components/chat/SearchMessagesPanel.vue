<script setup lang="ts">
import { ref, watch, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { messageApi } from '@/api/message'
import { Close } from '@element-plus/icons-vue'
import type { Message } from '@/types/message'

const { t } = useI18n()

const emit = defineEmits<{
  close: []
  jumpToMessage: [messageId: string]
}>()

const query = ref('')
const results = ref<Message[]>([])
const loading = ref(false)
const focusedIndex = ref(-1)
let searchTimer: ReturnType<typeof setTimeout> | null = null

function doSearch() {
  const q = query.value.trim()
  if (!q) {
    results.value = []
    return
  }
  loading.value = true
  messageApi
    .searchMessages({ q, limit: 20 })
    .then((res) => {
      if (res.success && Array.isArray(res.data)) {
        results.value = res.data
      } else {
        results.value = []
      }
    })
    .catch(() => {
      results.value = []
    })
    .finally(() => {
      loading.value = false
    })
}

watch(query, () => {
  if (searchTimer) clearTimeout(searchTimer)
  searchTimer = setTimeout(doSearch, 300)
})

function handleClose() {
  emit('close')
}

function handleSelect(msg: Message) {
  emit('jumpToMessage', msg.id)
  emit('close')
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    handleClose()
    return
  }
  if (e.key === 'ArrowDown') {
    e.preventDefault()
    focusedIndex.value = Math.min(focusedIndex.value + 1, results.value.length - 1)
  } else if (e.key === 'ArrowUp') {
    e.preventDefault()
    focusedIndex.value = Math.max(focusedIndex.value - 1, 0)
  } else if (e.key === 'Enter' && focusedIndex.value >= 0) {
    e.preventDefault()
    handleSelect(results.value[focusedIndex.value]!)
  }
}

function formatTime(iso: string): string {
  try {
    const d = new Date(iso)
    return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
  } catch {
    return ''
  }
}

onUnmounted(() => {
  if (searchTimer) clearTimeout(searchTimer)
})
</script>

<template>
  <div class="search-overlay" @click.self="handleClose">
    <div class="search-panel" @keydown="handleKeydown">
      <div class="search-header">
        <input
          v-model="query"
          class="search-input"
          :placeholder="t('chat.searchMessages')"
          autofocus
        />
        <button class="search-close" @click="handleClose">
          <el-icon :size="18"><Close /></el-icon>
        </button>
      </div>

      <div class="search-body">
        <div v-if="loading" class="search-status">{{ t('common.loading') }}</div>
        <div v-else-if="query.trim() && results.length === 0" class="search-status search-status--empty">
          No results found
        </div>
        <div v-else-if="!query.trim()" class="search-status search-status--hint">
          Type to search messages
        </div>

        <div v-for="(msg, idx) in results" :key="msg.id" class="search-result" :class="{ 'search-result--focused': idx === focusedIndex }" @click="handleSelect(msg)" @mouseenter="focusedIndex = idx">
          <div class="search-result__author">{{ msg.sender.username }}</div>
          <div class="search-result__text">{{ msg.content }}</div>
          <div class="search-result__time">{{ formatTime(msg.created_at) }}</div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">
.search-overlay {
  position: fixed;
  inset: 0;
  z-index: 500;
  background: rgba(0, 0, 0, 0.4);
  display: grid;
  place-items: center;
}

.search-panel {
  width: min(520px, 90vw);
  max-height: 70vh;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.search-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border);
}

.search-input {
  flex: 1;
  background: none;
  border: none;
  color: var(--fg);
  font-size: 15px;
  font-family: inherit;
  outline: none;

  &::placeholder {
    color: var(--muted);
  }
}

.search-close {
  background: none;
  border: none;
  color: var(--muted);
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  display: grid;
  place-items: center;

  &:hover {
    color: var(--fg);
    background: var(--message-hover);
  }
}

.search-body {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}

.search-status {
  padding: 24px 16px;
  text-align: center;
  font-size: 14px;
  color: var(--muted);

  &--empty, &--hint {
    font-style: italic;
  }
}

.search-result {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 10px 16px;
  cursor: pointer;
  border-left: 3px solid transparent;
  transition: background 0.1s, border-color 0.1s;

  &:hover,
  &--focused {
    background: var(--message-hover);
    border-left-color: var(--accent);
  }

  &__author {
    font-size: 13px;
    font-weight: 600;
    color: var(--accent);
  }

  &__text {
    font-size: 14px;
    color: var(--fg);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  &__time {
    font-size: 11px;
    color: var(--muted);
  }
}
</style>
