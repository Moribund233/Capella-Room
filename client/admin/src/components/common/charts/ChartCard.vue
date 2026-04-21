<template>
  <n-card class="chart-card" :bordered="bordered" :segmented="segmented">
    <template v-if="title || $slots.header" #header>
      <slot name="header">
        <div class="chart-card-header">
          <span class="chart-card-title">{{ title }}</span>
          <span v-if="subtitle" class="chart-card-subtitle">{{ subtitle }}</span>
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
  transition: var(--transition-base);
}

.chart-card-header {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.chart-card-title {
  font-size: 16px;
  font-weight: 500;
  color: var(--text-primary);
}

.chart-card-subtitle {
  font-size: 12px;
  color: var(--text-tertiary);
}

.chart-card-body {
  width: 100%;
}

.chart-card-body > :deep(*) {
  width: 100%;
}
</style>
