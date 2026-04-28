<template>
  <div v-if="hasContent" class="status-bar" :class="{ 'is-scrolling': isOverflow }">
    <div ref="contentRef" class="status-content" :class="{ 'scroll-animation': isOverflow }">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue'

/**
 * StatusBar 组件
 * 单行状态栏组件，支持动态内容，移动端自动滚动显示
 *
 * @example
 * <status-bar>
 *   <span>系统状态: 正常运行</span>
 *   <a href="#">查看详情</a>
 * </status-bar>
 */

/**
 * 组件属性定义
 */
interface Props {
  /** 是否启用滚动动画 */
  scrollable?: boolean
  /** 滚动速度（像素/秒） */
  scrollSpeed?: number
}

const props = withDefaults(defineProps<Props>(), {
  scrollable: true,
  scrollSpeed: 30,
})

/**
 * 组件事件定义
 */
const emit = defineEmits<{
  /** 内容变化事件 */
  (e: 'update:hasContent', value: boolean): void
}>()

/**
 * 内容容器引用
 */
const contentRef = ref<HTMLElement | null>(null)

/**
 * 容器宽度
 */
const containerWidth = ref(0)

/**
 * 内容宽度
 */
const contentWidth = ref(0)

/**
 * 是否有内容
 */
const hasContent = computed(() => {
  if (!contentRef.value) return false
  // 检查是否有实际内容（排除空白字符）
  const text = contentRef.value.textContent?.trim() || ''
  return text.length > 0 || contentRef.value.children.length > 0
})

/**
 * 内容是否溢出（需要滚动）
 */
const isOverflow = computed(() => {
  if (!props.scrollable) return false
  return contentWidth.value > containerWidth.value
})

/**
 * 更新尺寸
 */
function updateDimensions(): void {
  if (!contentRef.value) return

  const parent = contentRef.value.parentElement
  if (parent) {
    containerWidth.value = parent.clientWidth
  }

  // 计算内容实际宽度
  contentWidth.value = contentRef.value.scrollWidth
}

/**
 * 观察内容变化
 */
let resizeObserver: ResizeObserver | null = null
let mutationObserver: MutationObserver | null = null

onMounted(() => {
  nextTick(() => {
    updateDimensions()

    // 监听尺寸变化
    if (window.ResizeObserver) {
      resizeObserver = new ResizeObserver(() => {
        updateDimensions()
      })
      if (contentRef.value) {
        resizeObserver.observe(contentRef.value)
      }
    }

    // 监听内容变化
    mutationObserver = new MutationObserver(() => {
      nextTick(() => {
        updateDimensions()
      })
    })

    if (contentRef.value) {
      mutationObserver.observe(contentRef.value, {
        childList: true,
        subtree: true,
        characterData: true,
      })
    }
  })
})

onUnmounted(() => {
  resizeObserver?.disconnect()
  mutationObserver?.disconnect()
})

/**
 * 监听 hasContent 变化，通知父组件
 */
watch(hasContent, (newValue) => {
  emit('update:hasContent', newValue)
}, { immediate: true })

/**
 * 监听内容宽度变化
 */
watch([containerWidth, contentWidth], () => {
  // 强制更新以触发重新计算
})
</script>

<style scoped>
.status-bar {
  width: 100%;
  height: 28px;
  background: var(--bg-layout);
  border-top: 1px solid var(--border-color-base);
  overflow: hidden;
  position: relative;
  display: flex;
  align-items: center;
}

.status-content {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 0 16px;
  white-space: nowrap;
  font-size: 12px;
  color: var(--text-tertiary);
}

/* 滚动动画 */
.scroll-animation {
  animation: scroll linear infinite;
  animation-duration: var(--scroll-duration, 10s);
  padding-right: 50px; /* 添加间距，让内容完全滚出后再出现 */
}

@keyframes scroll {
  0% {
    transform: translateX(0);
  }
  100% {
    transform: translateX(calc(-100% + var(--container-width, 100%)));
  }
}

/* 悬停暂停 */
.status-bar:hover .scroll-animation {
  animation-play-state: paused;
}

/* 链接样式 */
.status-content :deep(a) {
  color: var(--color-primary);
  text-decoration: none;
  transition: color 0.2s;
}

.status-content :deep(a:hover) {
  color: var(--color-primary-hover);
  text-decoration: underline;
}

/* 状态指示器 */
.status-content :deep(.status-indicator) {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.status-content :deep(.status-dot) {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--color-success);
}

.status-content :deep(.status-dot.warning) {
  background: var(--color-warning);
}

.status-content :deep(.status-dot.error) {
  background: var(--color-error);
}
</style>
