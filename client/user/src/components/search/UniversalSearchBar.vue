<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { Search, X, Clock } from 'lucide-vue-next'
import { useDebounceFn } from '@vueuse/core'
import type { SearchType } from '@/types/search'

/**
 * 组件属性定义
 */
interface Props {
  /** 当前搜索类型 */
  searchType?: SearchType
  /** 当前关键词 */
  modelValue?: string
  /** 加载状态 */
  loading?: boolean
  /** 搜索历史 */
  history?: string[]
  /** 占位符文本 */
  placeholder?: string
}

const props = withDefaults(defineProps<Props>(), {
  searchType: 'room',
  modelValue: '',
  loading: false,
  history: () => [],
  placeholder: '搜索...',
})

/**
 * 组件事件定义
 */
interface Emits {
  /** 更新关键词 */
  (e: 'update:modelValue', value: string): void
  /** 搜索 */
  (e: 'search', keyword: string): void
  /** 切换搜索类型 */
  (e: 'changeType', type: SearchType): void
  /** 选择历史记录 */
  (e: 'selectHistory', keyword: string): void
  /** 清除历史记录 */
  (e: 'clearHistory'): void
  /** 移除单条历史 */
  (e: 'removeHistory', keyword: string): void
}

const emit = defineEmits<Emits>()

/** 输入框引用 */
const inputRef = ref<HTMLInputElement>()
/** 是否显示历史下拉 */
const showHistory = ref(false)
/** 本地输入值 */
const localValue = ref(props.modelValue)

/** 搜索类型选项 */
const typeOptions: { value: SearchType; label: string }[] = [
  { value: 'room', label: '房间' },
  { value: 'user', label: '用户' },
]

/**
 * 监听外部值变化
 */
watch(
  () => props.modelValue,
  (newVal) => {
    localValue.value = newVal
  }
)

/**
 * 防抖搜索
 */
const debouncedSearch = useDebounceFn((keyword: string) => {
  if (keyword.trim()) {
    emit('search', keyword.trim())
  }
}, 300)

/**
 * 处理输入
 */
function handleInput(e: Event) {
  const target = e.target as HTMLInputElement
  const value = target.value
  localValue.value = value
  emit('update:modelValue', value)

  if (value.trim()) {
    debouncedSearch(value)
  }
}

/**
 * 处理回车搜索
 */
function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter' && localValue.value.trim()) {
    emit('search', localValue.value.trim())
    showHistory.value = false
  }
}

/**
 * 清除输入
 */
function handleClear() {
  localValue.value = ''
  emit('update:modelValue', '')
  nextTick(() => {
    inputRef.value?.focus()
  })
}

/**
 * 切换搜索类型
 */
function handleTypeChange(type: SearchType) {
  emit('changeType', type)
  // 清空输入
  handleClear()
}

/**
 * 选择历史记录
 */
function selectHistory(keyword: string) {
  localValue.value = keyword
  emit('update:modelValue', keyword)
  emit('selectHistory', keyword)
  showHistory.value = false
}

/**
 * 移除历史记录
 */
function removeHistory(e: Event, keyword: string) {
  e.stopPropagation()
  emit('removeHistory', keyword)
}

/**
 * 清除所有历史
 */
function clearAllHistory(e: Event) {
  e.stopPropagation()
  emit('clearHistory')
}

/**
 * 聚焦时显示历史
 */
function handleFocus() {
  if (props.history.length > 0 && !localValue.value.trim()) {
    showHistory.value = true
  }
}

/**
 * 失去焦点时延迟隐藏历史
 */
function handleBlur() {
  setTimeout(() => {
    showHistory.value = false
  }, 200)
}
</script>

<template>
  <div class="universal-search-bar">
    <!-- 搜索类型切换 -->
    <div class="search-type-tabs">
      <button
        v-for="option in typeOptions"
        :key="option.value"
        class="search-type-tab"
        :class="{ 'search-type-tab--active': searchType === option.value }"
        @click="handleTypeChange(option.value)"
      >
        {{ option.label }}
      </button>
    </div>

    <!-- 搜索输入框 -->
    <div class="search-input-wrapper">
      <Search class="search-icon" :size="18" />
      <input
        ref="inputRef"
        v-model="localValue"
        type="text"
        class="search-input"
        :placeholder="placeholder"
        @input="handleInput"
        @keydown="handleKeydown"
        @focus="handleFocus"
        @blur="handleBlur"
      />
      <!-- 清除按钮 -->
      <button
        v-if="localValue"
        class="clear-btn"
        @click="handleClear"
      >
        <X :size="16" />
      </button>
      <!-- 加载指示器 -->
      <span v-else-if="loading" class="loading-indicator" />
    </div>

    <!-- 搜索历史下拉 -->
    <div
      v-if="showHistory && history.length > 0"
      class="search-history-dropdown"
    >
      <div class="history-header">
        <span class="history-title">搜索历史</span>
        <button class="history-clear" @click="clearAllHistory">
          清除全部
        </button>
      </div>
      <div class="history-list">
        <div
          v-for="item in history"
          :key="item"
          class="history-item"
          @click="selectHistory(item)"
        >
          <Clock :size="14" class="history-icon" />
          <span class="history-text">{{ item }}</span>
          <button
            class="history-remove"
            @click="(e) => removeHistory(e, item)"
          >
            <X :size="14" />
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.universal-search-bar {
  position: relative;
}

.search-type-tabs {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
}

.search-type-tab {
  padding: 6px 16px;
  border-radius: 16px;
  border: 1px solid var(--color-border, #e0e0e0);
  background: var(--color-white, #fff);
  color: var(--color-text-secondary, #666);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.search-type-tab:hover {
  border-color: var(--color-primary, #2080f0);
  color: var(--color-primary, #2080f0);
}

.search-type-tab--active {
  background: var(--color-primary, #2080f0);
  border-color: var(--color-primary, #2080f0);
  color: var(--color-white, #fff);
}

.search-input-wrapper {
  position: relative;
  display: flex;
  align-items: center;
}

.search-icon {
  position: absolute;
  left: 12px;
  color: var(--color-text-tertiary, #999);
  pointer-events: none;
}

.search-input {
  width: 100%;
  padding: 10px 40px 10px 40px;
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 24px;
  font-size: 14px;
  background: var(--color-white, #fff);
  transition: all 0.2s ease;
}

.search-input:focus {
  outline: none;
  border-color: var(--color-primary, #2080f0);
  box-shadow: 0 0 0 3px rgba(32, 128, 240, 0.1);
}

.search-input::placeholder {
  color: var(--color-text-tertiary, #999);
}

.clear-btn {
  position: absolute;
  right: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  border: none;
  background: var(--color-background-soft, #f0f0f0);
  color: var(--color-text-secondary, #666);
  cursor: pointer;
  transition: all 0.2s ease;
}

.clear-btn:hover {
  background: var(--color-text-tertiary, #999);
  color: var(--color-white, #fff);
}

.loading-indicator {
  position: absolute;
  right: 12px;
  width: 16px;
  height: 16px;
  border: 2px solid var(--color-border, #e0e0e0);
  border-top-color: var(--color-primary, #2080f0);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.search-history-dropdown {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  margin-top: 8px;
  background: var(--color-white, #fff);
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 12px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  z-index: 100;
  overflow: hidden;
}

.history-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid var(--color-border, #f0f0f0);
}

.history-title {
  font-size: 12px;
  color: var(--color-text-tertiary, #999);
}

.history-clear {
  font-size: 12px;
  color: var(--color-primary, #2080f0);
  background: none;
  border: none;
  cursor: pointer;
}

.history-clear:hover {
  text-decoration: underline;
}

.history-list {
  max-height: 240px;
  overflow-y: auto;
}

.history-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 16px;
  cursor: pointer;
  transition: background-color 0.2s ease;
}

.history-item:hover {
  background-color: var(--color-background-soft, #f5f5f5);
}

.history-icon {
  color: var(--color-text-tertiary, #999);
  flex-shrink: 0;
}

.history-text {
  flex: 1;
  font-size: 14px;
  color: var(--color-text, #333);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.history-remove {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  border: none;
  background: transparent;
  color: var(--color-text-tertiary, #999);
  cursor: pointer;
  opacity: 0;
  transition: all 0.2s ease;
}

.history-item:hover .history-remove {
  opacity: 1;
}

.history-remove:hover {
  background: var(--color-background-soft, #f0f0f0);
  color: var(--color-text-secondary, #666);
}
</style>
