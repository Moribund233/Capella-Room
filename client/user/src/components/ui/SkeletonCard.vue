<script setup lang="ts">
import Skeleton from './Skeleton.vue'

/**
 * 卡片骨架屏组件
 * 用于卡片内容加载时的占位显示
 */

withDefaults(
  defineProps<{
    /** 是否显示动画 */
    animated?: boolean
    /** 是否有封面图 */
    hasCover?: boolean
    /** 行数 */
    lines?: number
    /** 是否有操作按钮 */
    hasAction?: boolean
  }>(),
  {
    animated: true,
    hasCover: false,
    lines: 2,
    hasAction: false,
  }
)
</script>

<template>
  <div class="skeleton-card">
    <!-- 封面图 -->
    <Skeleton
      v-if="hasCover"
      :animated="animated"
      :custom-style="{ width: '100%', height: '160px' }"
      radius="small"
    />

    <div class="skeleton-card__content">
      <!-- 标题 -->
      <Skeleton
        :animated="animated"
        :custom-style="{ width: '60%', height: '20px' }"
        radius="small"
      />

      <!-- 内容行 -->
      <div class="skeleton-card__lines">
        <Skeleton
          v-for="i in lines"
          :key="i"
          :animated="animated"
          :custom-style="{ width: i === lines ? '40%' : '100%', height: '14px' }"
          radius="small"
        />
      </div>

      <!-- 操作按钮 -->
      <div v-if="hasAction" class="skeleton-card__action">
        <Skeleton
          :animated="animated"
          :custom-style="{ width: '80px', height: '32px' }"
          radius="small"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.skeleton-card {
  background: var(--card-bg, #fff);
  border-radius: 12px;
  overflow: hidden;
  border: 1px solid var(--border-color, rgba(0, 0, 0, 0.06));
}

.skeleton-card__content {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.skeleton-card__lines {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.skeleton-card__action {
  display: flex;
  justify-content: flex-end;
  margin-top: 8px;
  padding-top: 12px;
  border-top: 1px solid var(--border-color, rgba(0, 0, 0, 0.06));
}
</style>
