<script setup lang="ts">
import { computed } from 'vue'
import { MoreVertical } from 'lucide-vue-next'
import type { QuickRuntimeItem } from '@/config/quick'
import QuickAction from './QuickAction.vue'

/**
 * 组件属性定义
 */
interface Props {
  /** Quick 运行时项列表 */
  items: QuickRuntimeItem[]
  /** 位置：header 或 mobile-header */
  position?: 'header' | 'mobile-header'
}

const props = withDefaults(defineProps<Props>(), {
  position: 'header',
})

/**
 * 组件事件定义
 */
const emit = defineEmits<{
  /** 点击菜单项 */
  (e: 'select', key: string, childKey?: string): void
}>()

/**
 * 外显的项
 */
const visibleItems = computed(() =>
  props.items.filter((item) => item.display === 'visible')
)

/**
 * 下拉菜单的项
 */
const dropdownItems = computed(() =>
  props.items.filter((item) => item.display === 'dropdown')
)

/**
 * 处理菜单选择
 */
function handleMenuSelect(item: QuickRuntimeItem, childKey: string) {
  if (item.onSelect) {
    item.onSelect(childKey)
  }
  emit('select', item.key, childKey)
}

/**
 * 处理聚合菜单选择
 */
function handleAggregatedSelect(key: string) {
  // 查找对应的项并执行点击
  const item = props.items.find((i) => i.key === key)
  if (item) {
    item.onClick()
  }
  emit('select', key)
}
</script>

<template>
  <div class="quick-bar" :class="`quick-bar--${position}`">
    <!-- 外显按钮区域 -->
    <template v-for="item in visibleItems" :key="item.key">
      <!-- Action 类型：直接点击执行 -->
      <QuickAction
        v-if="item.type === 'action'"
        :item="item"
        @click="item.onClick"
      />

      <!-- Menu 类型：带下拉菜单 -->
      <div v-else class="quick-menu">
        <QuickAction
          :item="item"
          @click="item.onClick"
        />
        <!-- 下拉菜单 -->
        <div v-if="item.children && item.children.length > 0" class="quick-dropdown">
          <button
            v-for="child in item.children"
            :key="child.key"
            class="quick-dropdown__item"
            :disabled="child.disabled"
            @click="handleMenuSelect(item, child.key)"
          >
            <span class="quick-dropdown__label">{{ child.label }}</span>
          </button>
        </div>
      </div>
    </template>

    <!-- 下拉菜单按钮（聚合所有 dropdown 类型的按钮） -->
    <div v-if="dropdownItems.length > 0" class="quick-menu">
      <button
        class="quick-action more-btn"
        aria-label="更多操作"
      >
        <MoreVertical :size="18" />
      </button>
      <div class="quick-dropdown">
        <button
          v-for="item in dropdownItems"
          :key="item.key"
          class="quick-dropdown__item"
          @click="handleAggregatedSelect(item.key)"
        >
          <span class="quick-dropdown__label">{{ item.label }}</span>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.quick-bar {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
}

.quick-bar--header {
  /* 桌面端头部样式 - 垂直排列 */
  flex-direction: column;
}

.quick-bar--mobile-header {
  /* 移动端头部样式 - 水平排列 */
  flex-direction: row;
  gap: var(--space-sm);
}

/* 菜单容器 */
.quick-menu {
  position: relative;
}

/* 下拉菜单 */
.quick-dropdown {
  position: absolute;
  top: 100%;
  right: 0;
  margin-top: var(--space-xs);
  background: var(--color-white);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-lg);
  min-width: 160px;
  padding: var(--space-xs);
  opacity: 0;
  visibility: hidden;
  transform: translateY(-10px);
  transition: all var(--duration-fast);
  z-index: 100;
}

.quick-menu:hover .quick-dropdown,
.quick-dropdown:hover {
  opacity: 1;
  visibility: visible;
  transform: translateY(0);
}

/* 下拉菜单项 */
.quick-dropdown__item {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  width: 100%;
  padding: var(--space-sm) var(--space-md);
  border: none;
  background: transparent;
  color: var(--color-text-primary);
  font-size: var(--font-size-body);
  text-align: left;
  cursor: pointer;
  border-radius: var(--radius-sm);
  transition: background var(--duration-fast);
}

.quick-dropdown__item:hover {
  background: var(--color-background);
}

.quick-dropdown__item:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* 更多按钮 */
.more-btn {
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

.more-btn:hover {
  background: var(--color-background);
  color: var(--color-text-primary);
}
</style>
