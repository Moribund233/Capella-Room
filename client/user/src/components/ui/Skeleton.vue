<script setup lang="ts">
/**
 * AppSkeleton 骨架屏组件
 * 用于在内容加载时显示占位符，提升用户体验
 */

defineOptions({
  name: 'AppSkeleton',
})

withDefaults(
  defineProps<{
    /** 是否显示动画 */
    animated?: boolean
    /** 自定义样式 */
    customStyle?: Record<string, string | number>
    /** 圆角大小 */
    radius?: 'none' | 'small' | 'medium' | 'large' | 'full'
  }>(),
  {
    animated: true,
    customStyle: () => ({}),
    radius: 'medium',
  }
)

const radiusMap = {
  none: '0px',
  small: '4px',
  medium: '8px',
  large: '12px',
  full: '50%',
}
</script>

<template>
  <div
    class="skeleton"
    :class="{ 'skeleton--animated': animated }"
    :style="{
      borderRadius: radiusMap[radius],
      ...customStyle,
    }"
  />
</template>

<style scoped>
.skeleton {
  background: var(--skeleton-bg, rgba(0, 0, 0, 0.06));
  min-height: 16px;
}

.skeleton--animated {
  background: linear-gradient(
    90deg,
    var(--skeleton-bg-start, rgba(0, 0, 0, 0.06)) 25%,
    var(--skeleton-bg-middle, rgba(0, 0, 0, 0.1)) 50%,
    var(--skeleton-bg-end, rgba(0, 0, 0, 0.06)) 75%
  );
  background-size: 200% 100%;
  animation: skeleton-loading 1.5s ease-in-out infinite;
}

@keyframes skeleton-loading {
  0% {
    background-position: 200% 0;
  }
  100% {
    background-position: -200% 0;
  }
}
</style>
