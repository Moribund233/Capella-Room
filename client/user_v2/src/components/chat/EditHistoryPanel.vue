<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { messageApi } from '@/api/message'
import { formatTime } from '@/utils/date'
import type { MessageEditResponse } from '@/types/message'

const props = defineProps<{
  messageId: string
}>()

const emit = defineEmits<{
  close: []
}>()

const { t } = useI18n()

const history = ref<MessageEditResponse[]>([])
const loading = ref(true)
const error = ref<string | null>(null)

onMounted(async () => {
  try {
    const res: unknown = await messageApi.getEditHistory(props.messageId)
    history.value = Array.isArray(res) ? res : ((res as Record<string, unknown>).data as MessageEditResponse[] ?? [])
  } catch (e) {
    error.value = '获取编辑历史失败'
    console.error('[EditHistoryPanel]', e)
  } finally {
    loading.value = false
  }
})
</script>

<template>
  <div class="edit-history">
    <div class="edit-history__header">
      <span class="edit-history__title">{{ t('chat.editHistory') }}</span>
      <button class="edit-history__close" @click="emit('close')">&times;</button>
    </div>

    <div v-if="loading" class="edit-history__loading">{{ t('common.loading') }}</div>
    <div v-else-if="error" class="edit-history__error">{{ error }}</div>
    <div v-else-if="history.length === 0" class="edit-history__empty">{{ t('chat.noEditHistory') }}</div>
    <div v-else class="edit-history__list">
      <div
        v-for="(entry, idx) in history"
        :key="entry.id"
        class="edit-history__entry"
      >
        <div class="edit-history__meta">
          <span class="edit-history__index">#{{ history.length - idx }}</span>
          <span class="edit-history__editor">{{ entry.editor.username }}</span>
          <span class="edit-history__time">{{ formatTime(entry.created_at) }}</span>
        </div>
        <div class="edit-history__diff">
          <div class="edit-history__old">
            <span class="edit-history__label">{{ t('chat.before') }}</span>
            <p>{{ entry.old_content }}</p>
          </div>
          <div class="edit-history__new">
            <span class="edit-history__label">{{ t('chat.after') }}</span>
            <p>{{ entry.new_content }}</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.edit-history {
  background: var(--surface, #fff);
  border: 1px solid var(--border, #e0e0e0);
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.15);
  max-width: 480px;
  min-width: 320px;
  max-height: 400px;
  overflow-y: auto;
  z-index: 100;
}

.edit-history__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border, #e0e0e0);
  position: sticky;
  top: 0;
  background: var(--surface, #fff);
}

.edit-history__title {
  font-weight: 600;
  font-size: 14px;
}

.edit-history__close {
  background: none;
  border: none;
  font-size: 20px;
  cursor: pointer;
  color: var(--muted, #999);
  padding: 0 4px;
  line-height: 1;
}

.edit-history__loading,
.edit-history__error,
.edit-history__empty {
  padding: 24px 16px;
  text-align: center;
  color: var(--muted, #999);
  font-size: 13px;
}

.edit-history__list {
  padding: 8px;
}

.edit-history__entry {
  padding: 10px 12px;
  border-radius: 8px;
  margin-bottom: 6px;
}

.edit-history__entry:hover {
  background: var(--message-hover, #f5f5f5);
}

.edit-history__meta {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
  font-size: 12px;
}

.edit-history__index {
  font-weight: 700;
  color: var(--accent, #2080f0);
  font-size: 11px;
}

.edit-history__editor {
  color: var(--fg, #333);
  font-weight: 500;
}

.edit-history__time {
  color: var(--muted, #999);
  margin-left: auto;
}

.edit-history__diff {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.edit-history__label {
  font-size: 11px;
  font-weight: 600;
  color: var(--muted, #999);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.edit-history__old p {
  font-size: 13px;
  color: var(--accent-pink, #e34b6b);
  text-decoration: line-through;
  margin: 2px 0 4px;
  word-break: break-word;
  white-space: pre-wrap;
}

.edit-history__new p {
  font-size: 13px;
  color: var(--accent-green, #22c55e);
  margin: 0;
  word-break: break-word;
  white-space: pre-wrap;
}
</style>
