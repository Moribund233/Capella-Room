<template>
  <div
    v-if="hasContent"
    ref="statusBarRef"
    class="status-bar"
    :class="{ 'is-scrolling': isScrolling && !isResizing }"
  >
    <div class="status-bar-track" :style="trackStyle">
      <div class="status-bar-content" :style="contentStyle">
        <!-- 自定义渲染 -->
        <template v-if="content?.render">
          <RenderFunction :render="content.render" />
        </template>
        <!-- 状态项数组 -->
        <template v-else-if="content?.items && content.items.length > 0">
          <span
            v-for="(item, index) in content.items"
            :key="index"
            class="status-item"
            :class="{ 'is-link': item.onClick || item.href }"
            @click="handleItemClick(item)"
          >
            <component
              :is="item.icon"
              v-if="item.icon"
              class="status-icon"
              :size="14"
            />
            <span v-if="item.label" class="status-label">{{ item.label }}</span>
            <span v-if="item.value !== undefined" class="status-value" :class="item.valueClass">
              {{ item.value }}
            </span>
            <span v-if="index < content.items.length - 1" class="status-divider">|</span>
          </span>
        </template>
        <!-- 纯文本 -->
        <span v-else-if="content?.text" class="status-text">{{ content.text }}</span>
      </div>
      <!-- 复制一份内容用于无缝滚动 -->
      <div
        v-show="shouldScroll && !isResizing"
        class="status-bar-content status-bar-content--clone"
        :style="contentStyle"
      >
        <template v-if="content?.render">
          <RenderFunction :render="content.render" />
        </template>
        <template v-else-if="content?.items && content.items.length > 0">
          <span
            v-for="(item, index) in content.items"
            :key="index"
            class="status-item"
            :class="{ 'is-link': item.onClick || item.href }"
            @click="handleItemClick(item)"
          >
            <component
              :is="item.icon"
              v-if="item.icon"
              class="status-icon"
              :size="14"
            />
            <span v-if="item.label" class="status-label">{{ item.label }}</span>
            <span v-if="item.value !== undefined" class="status-value" :class="item.valueClass">
              {{ item.value }}
            </span>
            <span v-if="index < content.items.length - 1" class="status-divider">|</span>
          </span>
        </template>
        <span v-else-if="content?.text" class="status-text">{{ content.text }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue'
import type { VNode, FunctionalComponent } from 'vue'
import type { LucideProps } from 'lucide-vue-next'
import { useStatusBar } from '@/composables/useStatusBar'

/**
 * StatusBar 组件
 * 单行状态栏组件，当内容超出容器宽度时自动滚动显示
 * 内容通过 useStatusBar composable 管理
 */

/**
 * 状态项类型
 */
export interface StatusItem {
  /** 标签文本 */
  label?: string
  /** 值文本 */
  value?: string | number
  /** 值的样式类 */
  valueClass?: string
  /** 图标组件 */
  icon?: FunctionalComponent<LucideProps>
  /** 点击跳转链接 */
  href?: string
  /** 点击回调 */
  onClick?: () => void
}

/**
 * 状态栏内容配置
 */
export interface StatusBarContent {
  /** 显示的文本内容 */
  text?: string
  /** 状态项数组 */
  items?: StatusItem[]
  /** 自定义渲染函数 */
  render?: () => VNode | VNode[]
}

const statusBarRef = ref<HTMLElement | null>(null)
const contentWidth = ref(0)
const containerWidth = ref(0)
const isScrolling = ref(false)
const isResizing = ref(false)
let resizeTimeout: number | null = null
let resizeObserver: ResizeObserver | null = null

const { content, hasContent, config } = useStatusBar()

/** 是否需要滚动 */
const shouldScroll = computed(() => {
  if (!hasContent.value) return false
  return contentWidth.value > containerWidth.value
})

/** 轨道样式 */
const trackStyle = computed(() => {
  if (!shouldScroll.value || isResizing.value) return {}
  return {
    animationDuration: `${(contentWidth.value + containerWidth.value) / config.value.speed}s`,
    animationDelay: `${config.value.pauseDuration}ms`,
  }
})

/** 内容样式 */
const contentStyle = computed(() => {
  return {
    paddingRight: shouldScroll.value ? '50px' : '0',
  }
})

/**
 * 处理状态项点击
 */
const handleItemClick = (item: StatusItem) => {
  if (item.onClick) {
    item.onClick()
  } else if (item.href) {
    window.open(item.href, '_blank')
  }
}

/**
 * 测量内容尺寸
 */
const measureDimensions = () => {
  nextTick(() => {
    if (!statusBarRef.value) return

    const container = statusBarRef.value
    const contentEl = container.querySelector('.status-bar-content') as HTMLElement

    if (contentEl) {
      containerWidth.value = container.clientWidth
      contentWidth.value = contentEl.scrollWidth
    }
  })
}

/**
 * 处理尺寸变化（带防抖）
 */
const handleResize = () => {
  // 标记正在调整大小，隐藏克隆内容和动画
  isResizing.value = true
  isScrolling.value = false

  // 清除之前的定时器
  if (resizeTimeout) {
    clearTimeout(resizeTimeout)
  }

  // 重新测量尺寸
  measureDimensions()

  // 延迟恢复滚动，等待布局稳定
  resizeTimeout = window.setTimeout(() => {
    isResizing.value = false
    if (shouldScroll.value) {
      isScrolling.value = true
    }
  }, 150)
}

/**
 * 开始滚动动画
 */
const startScroll = () => {
  if (shouldScroll.value && !isResizing.value) {
    isScrolling.value = true
  }
}

/**
 * 停止滚动动画
 */
const stopScroll = () => {
  isScrolling.value = false
}

/**
 * 重新计算并启动滚动
 */
const resetScroll = () => {
  stopScroll()
  measureDimensions()
  if (shouldScroll.value) {
    setTimeout(() => {
      startScroll()
    }, 100)
  }
}

// 监听内容变化
watch(() => content.value, resetScroll, { deep: true })

onMounted(() => {
  measureDimensions()

  // 使用 ResizeObserver 监听尺寸变化
  if (statusBarRef.value && typeof ResizeObserver !== 'undefined') {
    resizeObserver = new ResizeObserver(() => {
      handleResize()
    })
    resizeObserver.observe(statusBarRef.value)
  } else {
    // 降级到 window resize
    window.addEventListener('resize', handleResize)
  }

  if (shouldScroll.value) {
    setTimeout(() => {
      startScroll()
    }, config.value.pauseDuration)
  }
})

onUnmounted(() => {
  if (resizeObserver) {
    resizeObserver.disconnect()
    resizeObserver = null
  } else {
    window.removeEventListener('resize', handleResize)
  }

  if (resizeTimeout) {
    clearTimeout(resizeTimeout)
  }
})

/**
 * 渲染函数组件
 */
const RenderFunction = {
  props: {
    render: {
      type: Function as unknown as () => (() => VNode | VNode[]),
      required: true,
    },
  },
  setup(props: { render: () => VNode | VNode[] }) {
    return () => props.render()
  },
}
</script>

<style scoped>
.status-bar {
  width: 100%;
  height: 100%;
  overflow: hidden;
  white-space: nowrap;
  display: flex;
  align-items: center;
}

.status-bar-track {
  display: flex;
  align-items: center;
  will-change: transform;
}

.status-bar.is-scrolling .status-bar-track {
  animation-name: statusBarScroll;
  animation-timing-function: linear;
  animation-iteration-count: infinite;
}

.status-bar-content {
  display: flex;
  align-items: center;
  flex-shrink: 0;
}

.status-bar-content--clone {
  /* 克隆内容用于无缝滚动 */
}

.status-text {
  display: inline-block;
}

/* 状态项样式 */
.status-item {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.status-item.is-link {
  cursor: pointer;
  transition: var(--transition-fast);
}

.status-item.is-link:hover {
  color: var(--color-primary);
}

.status-icon {
  flex-shrink: 0;
}

.status-label {
  opacity: 0.8;
}

.status-value {
  font-weight: 500;
}

.status-divider {
  margin: 0 12px;
  opacity: 0.4;
}

@keyframes statusBarScroll {
  0% {
    transform: translateX(0);
  }
  100% {
    transform: translateX(-50%);
  }
}

/* 悬停时暂停滚动 */
.status-bar:hover .status-bar-track {
  animation-play-state: paused;
}
</style>
