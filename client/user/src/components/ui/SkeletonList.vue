<script setup lang="ts">
import Skeleton from './Skeleton.vue'

/**
 * 列表骨架屏组件
 * 用于列表内容加载时的占位显示
 */

withDefaults(
  defineProps<{
    /** 是否显示动画 */
    animated?: boolean
    /** 行数 */
    rows?: number
    /** 是否有头像 */
    hasAvatar?: boolean
    /** 是否有副标题 */
    hasSubtitle?: boolean
  }>(),
  {
    animated: true,
    rows: 5,
    hasAvatar: true,
    hasSubtitle: true,
  }
)
</script>

<template>
  <div class="skeleton-list">
    <div
      v-for="i in rows"
      :key="i"
      class="skeleton-list__item"
    >
      <!-- 头像 -->
      <Skeleton
        v-if="hasAvatar"
        :animated="animated"
        :custom-style="{ width: '48px', height: '48px', flexShrink: 0 }"
        radius="full"
      />

      <!-- 内容 -->
      <div class="skeleton-list__content">
        <Skeleton
          :animated="animated"
          :custom-style="{ width: '40%', height: '16px' }"
          radius="small"
        />
        <Skeleton
          v-if="hasSubtitle"
          :animated="animated"
          :custom-style="{ width: '70%', height: '14px' }"
          radius="small"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.skeleton-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.skeleton-list__item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  background: var(--list-item-bg, transparent);
  border-radius: 8px;
}

.skeleton-list__content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
</style>
