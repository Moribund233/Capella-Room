<script setup lang="ts">
import { computed } from 'vue'

type TransitionName = 'slide' | 'fade' | 'scale' | 'slide-up' | 'slide-down'

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
</style>
