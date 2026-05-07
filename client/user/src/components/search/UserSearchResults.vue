<script setup lang="ts">
import { computed } from 'vue'
import UserCard from '@/components/user/UserCard.vue'
import EmptyState from '@/components/ui/EmptyState.vue'
import SkeletonCard from '@/components/ui/SkeletonCard.vue'
import type { UserSearchItem } from '@/types/search'

/**
 * 组件属性定义
 */
interface Props {
  /** 用户列表 */
  users: UserSearchItem[]
  /** 加载状态 */
  loading?: boolean
  /** 是否还有更多 */
  hasMore?: boolean
  /** 搜索关键词 */
  keyword?: string
}

const props = withDefaults(defineProps<Props>(), {
  loading: false,
  hasMore: false,
  keyword: '',
})

/**
 * 组件事件定义
 */
interface Emits {
  /** 点击用户 */
  (e: 'click', user: UserSearchItem): void
  /** 加载更多 */
  (e: 'loadMore'): void
}

const emit = defineEmits<Emits>()

/**
 * 是否显示空状态
 */
const showEmpty = computed(() => {
  return !props.loading && props.users.length === 0
})

/**
 * 处理用户点击
 */
function handleUserClick(user: UserSearchItem) {
  emit('click', user)
}

/**
 * 处理加载更多
 */
function handleLoadMore() {
  if (!props.loading && props.hasMore) {
    emit('loadMore')
  }
}
</script>

<template>
  <div class="user-search-results">
    <!-- 加载中骨架屏 -->
    <div v-if="loading && users.length === 0" class="results-skeleton">
      <SkeletonCard v-for="i in 5" :key="i" />
    </div>

    <!-- 空状态 -->
    <EmptyState
      v-else-if="showEmpty"
      :title="keyword ? '未找到相关用户' : '暂无用户'"
      :description="keyword ? `没有找到包含「${keyword}」的用户` : '还没有任何用户'"
      icon="users"
    />

    <!-- 结果列表 -->
    <template v-else>
      <div class="results-list">
        <UserCard
          v-for="user in users"
          :key="user.id"
          :user="user"
          @click="handleUserClick"
        />
      </div>

      <!-- 加载更多 -->
      <div v-if="hasMore" class="load-more">
        <button
          class="load-more-btn"
          :disabled="loading"
          @click="handleLoadMore"
        >
          <span v-if="loading" class="loading-spinner" />
          <span v-else>加载更多</span>
        </button>
      </div>

      <!-- 已加载全部 -->
      <div v-else-if="users.length > 0" class="all-loaded">
        已加载全部 {{ users.length }} 个用户
      </div>
    </template>
  </div>
</template>

<style scoped>
.user-search-results {
  flex: 1;
  overflow-y: auto;
}

.results-skeleton {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.results-list {
  padding: 8px 16px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.load-more {
  padding: 16px;
  text-align: center;
}

.load-more-btn {
  padding: 8px 24px;
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 20px;
  background: var(--color-white, #fff);
  color: var(--color-text-secondary, #666);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.load-more-btn:hover:not(:disabled) {
  border-color: var(--color-primary, #2080f0);
  color: var(--color-primary, #2080f0);
}

.load-more-btn:disabled {
  cursor: not-allowed;
  opacity: 0.6;
}

.loading-spinner {
  display: inline-block;
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

.all-loaded {
  padding: 16px;
  text-align: center;
  font-size: 12px;
  color: var(--color-text-tertiary, #999);
}
</style>
