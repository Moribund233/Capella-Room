<template>
  <n-card class="chart-card" :class="{ 'is-loading': loading, 'is-empty': isEmpty }" :bordered="bordered"
    :segmented="segmented">
    <template v-if="title || $slots.header" #header>
      <slot name="header">
        <div class="chart-card-header">
          <div class="chart-card-title-wrapper">
            <span class="chart-card-title">{{ title }}</span>
            <span v-if="subtitle" class="chart-card-subtitle">{{ subtitle }}</span>
          </div>
          <!-- 装饰性渐变条 -->
          <div class="title-accent"></div>
        </div>
      </slot>
    </template>

    <template v-if="$slots['header-extra']" #header-extra>
      <slot name="header-extra" />
    </template>

    <div class="chart-card-body" :style="bodyStyle">
      <n-spin v-if="loading" :size="spinSize" />
      <n-empty v-else-if="isEmpty" :description="emptyText" />
      <slot v-else />
    </div>

    <template v-if="$slots.footer" #footer>
      <slot name="footer" />
    </template>
  </n-card>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { NCard, NSpin, NEmpty } from 'naive-ui'

/**
 * 图表卡片组件属性
 */
interface Props {
  /** 卡片标题 */
  title?: string
  /** 副标题 */
  subtitle?: string
  /** 是否显示边框 */
  bordered?: boolean
  /** 是否显示分段线 */
  segmented?: boolean
  /** 是否加载中 */
  loading?: boolean
  /** 加载指示器大小 */
  spinSize?: 'small' | 'medium' | 'large'
  /** 是否数据为空 */
  empty?: boolean
  /** 空状态提示文本 */
  emptyText?: string
  /** 内容区域最小高度 */
  minHeight?: string | number
}

const props = withDefaults(defineProps<Props>(), {
  title: '',
  subtitle: '',
  bordered: false,
  segmented: false,
  loading: false,
  spinSize: 'medium',
  empty: false,
  emptyText: '暂无数据',
  minHeight: '280px',
})

/**
 * 内容区域样式
 */
const bodyStyle = computed(() => ({
  minHeight: typeof props.minHeight === 'number' ? `${props.minHeight}px` : props.minHeight,
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
}))

/**
 * 是否为空状态
 */
const isEmpty = computed(() => props.empty)
</script>

<style scoped>
.chart-card {
  background: var(--glass-bg);
  backdrop-filter: var(--glass-backdrop);
  -webkit-backdrop-filter: var(--glass-backdrop);
  border: 1px solid var(--border-color-base);
  border-radius: var(--radius-xl);
  box-shadow: var(--shadow-sm);
  transition: all var(--duration-slow) var(--ease-out-expo);
  overflow: hidden;
  position: relative;
}

/* 顶部渐变装饰 */
.chart-card::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 1px;
  background: linear-gradient(90deg,
      transparent 0%,
      rgba(99, 102, 241, 0.3) 20%,
      rgba(139, 92, 246, 0.3) 80%,
      transparent 100%);
  opacity: 0;
  transition: opacity var(--duration-normal) var(--ease-smooth);
}

.chart-card:hover {
  transform: translateY(-4px);
  box-shadow: var(--shadow-xl);
  border-color: rgba(99, 102, 241, 0.15);
}

.chart-card:hover::before {
  opacity: 1;
}

/* 加载状态 */
.chart-card.is-loading {
  opacity: 0.8;
}

/* 空状态 */
.chart-card.is-empty {
  background: var(--bg-secondary);
}

.chart-card-header {
  display: flex;
  flex-direction: column;
  gap: 4px;
  position: relative;
}

.chart-card-title-wrapper {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.chart-card-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
  letter-spacing: -0.01em;
}

.chart-card-subtitle {
  font-size: 12px;
  color: var(--text-tertiary);
  font-weight: 400;
}

/* 标题装饰条 */
.title-accent {
  position: absolute;
  bottom: 0;
  left: 0;
  width: 24px;
  height: 2px;
  background: var(--color-primary-gradient);
  border-radius: var(--radius-full);
  transition: width var(--duration-slow) var(--ease-out-expo);
}

.chart-card:hover .title-accent {
  width: 48px;
}

.chart-card-body {
  width: 100%;
}

.chart-card-body > :deep(*) {
  width: 100%;
}
</style>
