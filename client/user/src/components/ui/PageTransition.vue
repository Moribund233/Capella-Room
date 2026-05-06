<script setup lang="ts">
import { computed } from 'vue'

type TransitionName = 'slide' | 'fade' | 'scale' | 'slide-up' | 'slide-down' | 'tab-slide'

const props = withDefaults(defineProps<{
  name?: TransitionName
  mode?: 'in-out' | 'out-in' | 'default'
  duration?: number
}>(), {
  name: 'slide',
  mode: 'out-in',
  duration: 300,
})

const transitionStyle = computed(() => ({
  '--transition-duration': `${props.duration}ms`,
}))
</script>

<template>
  <Transition
    :name="`page-${name}`"
    :mode="mode"
    :style="transitionStyle"
  >
    <slot />
  </Transition>
</template>

<style scoped>
/* 滑动过渡 */
.page-slide-enter-active,
.page-slide-leave-active {
  transition: all var(--transition-duration, 300ms) var(--ease-out);
}

.page-slide-enter-from {
  opacity: 0;
  transform: translateX(30px);
}

.page-slide-leave-to {
  opacity: 0;
  transform: translateX(-30px);
}

/* 淡入淡出 */
.page-fade-enter-active,
.page-fade-leave-active {
  transition: opacity var(--transition-duration, 300ms) var(--ease-default);
}

.page-fade-enter-from,
.page-fade-leave-to {
  opacity: 0;
}

/* 缩放过渡 */
.page-scale-enter-active,
.page-scale-leave-active {
  transition: all var(--transition-duration, 300ms) var(--ease-spring);
}

.page-scale-enter-from {
  opacity: 0;
  transform: scale(0.95);
}

.page-scale-leave-to {
  opacity: 0;
  transform: scale(1.05);
}

/* 向上滑动 */
.page-slide-up-enter-active,
.page-slide-up-leave-active {
  transition: all var(--transition-duration, 300ms) var(--ease-out);
}

.page-slide-up-enter-from {
  opacity: 0;
  transform: translateY(30px);
}

.page-slide-up-leave-to {
  opacity: 0;
  transform: translateY(-30px);
}

/* 向下滑动 */
.page-slide-down-enter-active,
.page-slide-down-leave-active {
  transition: all var(--transition-duration, 300ms) var(--ease-out);
}

.page-slide-down-enter-from {
  opacity: 0;
  transform: translateY(-30px);
}

.page-slide-down-leave-to {
  opacity: 0;
  transform: translateY(30px);
}

/* Tab 滑动切换 - 更流畅的左右滑动效果 */
.page-tab-slide-enter-active,
.page-tab-slide-leave-active {
  transition: all var(--transition-duration, 300ms) cubic-bezier(0.4, 0, 0.2, 1);
}

.page-tab-slide-enter-from {
  opacity: 0;
  transform: translateX(20px);
}

.page-tab-slide-leave-to {
  opacity: 0;
  transform: translateX(-20px);
}

.page-tab-slide-enter-to,
.page-tab-slide-leave-from {
  opacity: 1;
  transform: translateX(0);
}
</style>
