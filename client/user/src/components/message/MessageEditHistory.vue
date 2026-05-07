<script setup lang="ts">
import { ref, watch } from 'vue'
import { messageApi } from '@/api/message'
import { X, History } from 'lucide-vue-next'
import type { Message } from '@/types/message'

/**
 * 组件属性定义
 */
interface Props {
  /** 消息ID */
  messageId: string | null
  /** 是否显示 */
  visible: boolean
}

const props = defineProps<Props>()

/**
 * 组件事件定义
 */
interface Emits {
  /** 关闭弹窗 */
  (e: 'close'): void
}

const emit = defineEmits<Emits>()

/** 编辑历史列表 */
const editHistory = ref<Message[]>([])
/** 加载状态 */
const loading = ref(false)
/** 错误信息 */
const error = ref<string | null>(null)

/**
 * 加载编辑历史
 */
async function loadEditHistory() {
  if (!props.messageId) return

  loading.value = true
  error.value = null

  try {
    const res = await messageApi.getEditHistory(props.messageId)
    if (res.success && res.data) {
      // 按时间倒序排列，最新的在前
      editHistory.value = res.data.reverse()
    }
  } catch (err) {
    error.value = '加载编辑历史失败'
    console.error('[MessageEditHistory] load error:', err)
  } finally {
    loading.value = false
  }
}

/**
 * 格式化日期时间
 */
function formatDateTime(dateStr: string): string {
  return new Date(dateStr).toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  })
}

/**
 * 获取版本标签
 */
function getVersionLabel(index: number, total: number): string {
  if (index === 0) return '当前版本'
  if (index === total - 1) return '原始版本'
  return `版本 ${total - index}`
}

/**
 * 获取版本标签类型
 */
function getVersionTagType(index: number): 'success' | 'default' | 'info' {
  if (index === 0) return 'success'
  return 'default'
}

// 监听 visible 变化，打开时加载数据
watch(
  () => props.visible,
  (visible) => {
    if (visible && props.messageId) {
      loadEditHistory()
    } else {
      editHistory.value = []
    }
  }
)

/**
 * 处理键盘事件
 */
function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    emit('close')
  }
}
</script>

<template>
  <div
    v-if="visible"
    class="message-edit-history"
    @click.self="emit('close')"
    @keydown="handleKeydown"
  >
    <div class="message-edit-history__panel">
      <!-- 头部 -->
      <div class="message-edit-history__header">
        <div class="message-edit-history__title">
          <History :size="20" />
          <span>编辑历史</span>
        </div>
        <button class="message-edit-history__close" @click="emit('close')">
          <X :size="20" />
        </button>
      </div>

      <!-- 内容区域 -->
      <div class="message-edit-history__content">
        <!-- 加载中 -->
        <div v-if="loading" class="message-edit-history__loading">
          <div class="message-edit-history__spinner"></div>
          <span>加载中...</span>
        </div>

        <!-- 错误提示 -->
        <div v-else-if="error" class="message-edit-history__error">
          {{ error }}
        </div>

        <!-- 空状态 -->
        <div v-else-if="editHistory.length === 0" class="message-edit-history__empty">
          暂无编辑历史
        </div>

        <!-- 历史列表 -->
        <div v-else class="message-edit-history__list">
          <div
            v-for="(item, index) in editHistory"
            :key="item.id + index"
            class="message-edit-history__item"
          >
            <div class="message-edit-history__item-header">
              <span
                class="message-edit-history__version-tag"
                :class="`message-edit-history__version-tag--${getVersionTagType(index)}`"
              >
                {{ getVersionLabel(index, editHistory.length) }}
              </span>
              <span class="message-edit-history__time">
                {{ formatDateTime(item.created_at) }}
              </span>
            </div>
            <div class="message-edit-history__item-content">
              {{ item.content }}
            </div>
            <div v-if="item.edited_at" class="message-edit-history__item-edited">
              编辑于 {{ formatDateTime(item.edited_at) }}
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.message-edit-history {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: var(--color-mask, rgba(0, 0, 0, 0.5));
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.message-edit-history__panel {
  width: 90%;
  max-width: 500px;
  max-height: 80vh;
  background: var(--color-white);
  border-radius: var(--radius-lg, 12px);
  box-shadow: 0 8px 32px var(--color-shadow-dark, rgba(0, 0, 0, 0.2));
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.message-edit-history__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-md, 16px) var(--space-lg, 20px);
  border-bottom: 1px solid var(--color-border);
}

.message-edit-history__title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: var(--font-size-h4, 16px);
  font-weight: 600;
  color: var(--color-text-primary);
}

.message-edit-history__close {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: none;
  border-radius: var(--radius-sm, 6px);
  cursor: pointer;
  color: var(--color-text-secondary);
  transition: all 0.2s;
}

.message-edit-history__close:hover {
  background: var(--color-background-soft);
  color: var(--color-text-primary);
}

.message-edit-history__content {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-md, 16px);
}

.message-edit-history__loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 40px;
  color: var(--color-text-secondary);
}

.message-edit-history__spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--color-border);
  border-top-color: var(--color-primary);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.message-edit-history__error {
  padding: 40px;
  text-align: center;
  color: var(--color-error);
}

.message-edit-history__empty {
  padding: 40px;
  text-align: center;
  color: var(--color-text-secondary);
}

.message-edit-history__list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.message-edit-history__item {
  padding: 16px;
  background: var(--color-background-soft, #f5f5f5);
  border-radius: var(--radius-md, 8px);
  border-left: 4px solid var(--color-primary);
}

.message-edit-history__item:first-child {
  border-left-color: var(--color-success, #52c41a);
  background: var(--color-success-light, rgba(82, 196, 26, 0.1));
}

.message-edit-history__item-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.message-edit-history__version-tag {
  padding: 2px 8px;
  border-radius: var(--radius-sm, 6px);
  font-size: var(--font-size-small, 12px);
  font-weight: 500;
}

.message-edit-history__version-tag--success {
  background: var(--color-success-light, rgba(82, 196, 26, 0.1));
  color: var(--color-success, #52c41a);
}

.message-edit-history__version-tag--default {
  background: var(--color-background-mute);
  color: var(--color-text-secondary);
}

.message-edit-history__time {
  font-size: var(--font-size-small, 12px);
  color: var(--color-text-tertiary);
}

.message-edit-history__item-content {
  font-size: var(--font-size-body, 14px);
  line-height: 1.6;
  color: var(--color-text-primary);
  word-break: break-word;
  white-space: pre-wrap;
}

.message-edit-history__item-edited {
  margin-top: 8px;
  font-size: var(--font-size-small, 12px);
  color: var(--color-text-tertiary);
  font-style: italic;
}
</style>
