<script setup lang="ts">
import { computed } from 'vue'
import * as LucideIcons from 'lucide-vue-next'
import type { FunctionalComponent } from 'vue'
import type { LucideProps } from 'lucide-vue-next'
import type { QuickRuntimeItem } from '@/config/quick'

/**
 * 组件属性定义
 */
interface Props {
  /** Quick 运行时项 */
  item: QuickRuntimeItem
}

const props = defineProps<Props>()

/**
 * 组件事件定义
 */
const emit = defineEmits<{
  (e: 'click'): void
}>()

/**
 * 图标组件缓存
 */
const iconCache = new Map<string, FunctionalComponent<LucideProps>>()

/**
 * 获取图标组件
 * @param iconName 图标名称
 */
function getIconComponent(iconName: string): FunctionalComponent<LucideProps> {
  if (iconCache.has(iconName)) {
    return iconCache.get(iconName)!
  }
  const component = (LucideIcons as unknown as Record<string, FunctionalComponent<LucideProps>>)[iconName]
    || LucideIcons.Circle
  iconCache.set(iconName, component)
  return component
}

/**
 * 当前图标组件
 */
const IconComponent = computed(() => getIconComponent(props.item.currentIcon))

/**
 * 计算徽章数值（支持普通值和 Ref）
 */
const badgeValue = computed(() => {
  const badge = props.item.badge
  // 检查是否是 Ref 对象
  if (badge && typeof badge === 'object' && 'value' in badge) {
    return (badge as { value: number }).value || 0
  }
  return (badge as number) || 0
})

/**
 * 处理点击
 */
function handleClick() {
  if (!props.item.disabled) {
    emit('click')
  }
}
</script>

<template>
  <button
    class="quick-action"
    :class="{
      'is-active': item.isActive,
      'is-disabled': item.disabled,
    }"
    :aria-label="item.label"
    :disabled="item.disabled"
    :title="item.label"
    @click="handleClick"
  >
    <div class="quick-action__icon-wrapper">
      <IconComponent class="quick-action__icon" :size="18" />
    </div>
    <!-- 徽标 -->
    <span v-if="badgeValue > 0" class="quick-action__badge">
      {{ badgeValue > 99 ? '99+' : badgeValue }}
    </span>
    <!-- 激活指示器 -->
    <div v-if="item.isActive" class="quick-action__active-dot" />
  </button>
</template>

<style scoped>
.quick-action {
  position: relative;
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: transparent;
  color: var(--color-text-secondary);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--duration-fast);
}

.quick-action:hover {
  background: var(--color-background);
  color: var(--color-text-primary);
}

.quick-action:active {
  transform: scale(0.95);
}

.quick-action.is-active {
  color: var(--color-primary);
  background: var(--color-primary-light);
}

.quick-action.is-disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.quick-action.is-disabled:hover {
  background: transparent;
}

.quick-action__icon-wrapper {
  display: flex;
  align-items: center;
  justify-content: center;
}

.quick-action__icon {
  flex-shrink: 0;
}

/* 徽标 */
.quick-action__badge {
  position: absolute;
  top: 2px;
  right: 2px;
  min-width: 16px;
  height: 16px;
  padding: 0 4px;
  background: var(--color-danger);
  color: white;
  font-size: 10px;
  font-weight: 600;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
}

/* 激活指示器 */
.quick-action__active-dot {
  position: absolute;
  bottom: 4px;
  width: 4px;
  height: 4px;
  background: var(--color-primary);
  border-radius: 50%;
}
</style>
