<template>
  <div class="quick-bar">
    <!-- 外显按钮区域 -->
    <template v-for="item in visibleItems" :key="item.key">
      <!-- Action 类型：直接点击执行 -->
      <n-tooltip v-if="item.type === 'action'" :show-arrow="false" placement="bottom" :disabled="item.disabled">
        <template #trigger>
          <button class="action-btn" :class="{ 'is-disabled': item.disabled, 'is-active': item.isActive }"
            @click="item.onClick" :aria-label="item.label" :disabled="item.disabled">
            <div class="icon-wrapper">
              <component :is="getIconComponent(item.currentIcon)" class="icon" :size="18" />
            </div>
            <span v-if="item.badge" class="badge">{{ item.badge }}</span>
            <!-- 激活指示器 -->
            <div class="active-dot"></div>
          </button>
        </template>
        {{ item.disabled ? '暂不可用' : item.label }}
      </n-tooltip>

      <!-- Menu 类型：带下拉菜单 -->
      <n-dropdown v-else :options="getDropdownOptions(item)" placement="bottom" trigger="click"
        @select="(key: string) => handleMenuSelect(item, key)" :disabled="item.disabled">
        <button class="action-btn" :class="{ 'is-disabled': item.disabled, 'is-active': item.isActive }"
          :aria-label="item.label" :disabled="item.disabled">
          <div class="icon-wrapper">
            <component :is="getIconComponent(item.currentIcon)" class="icon" :size="18" />
          </div>
          <span v-if="item.badge" class="badge">{{ item.badge }}</span>
          <div class="active-dot"></div>
        </button>
      </n-dropdown>
    </template>

    <!-- 下拉菜单按钮（聚合所有 dropdown 类型的按钮） -->
    <n-dropdown v-if="dropdownItems.length > 0" :options="aggregatedOptions" placement="bottom-end" trigger="click"
      @select="handleAggregatedSelect">
      <button class="action-btn more-btn" aria-label="更多操作">
        <div class="icon-wrapper">
          <MoreVertical class="icon" :size="18" />
        </div>
      </button>
    </n-dropdown>
  </div>
</template>

<script setup lang="ts">
import { computed, h, unref } from 'vue'
import { NDropdown, NTooltip } from 'naive-ui'
import { MoreVertical } from 'lucide-vue-next'
import * as LucideIcons from 'lucide-vue-next'
import type { FunctionalComponent } from 'vue'
import type { LucideProps } from 'lucide-vue-next'
import {
  useQuickTheme,
  useQuickLayout,
  useQuickUser,
  useQuickPersonalization,
  useQuickNotification,
  type QuickConfigItem,
  type QuickContext,
  type QuickRuntimeItem,
  type UseQuickReturn,
  type QuickFactory,
} from '@/composables/quick'

/**
 * 图标组件缓存
 */
const iconCache = new Map<string, FunctionalComponent<LucideProps>>()

/**
 * 获取图标组件
 * @param iconName 图标名称（直接使用 lucide-vue-next 的组件名，如 'Sun'）
 * @returns 图标组件
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
 * 组件属性定义
 */
interface Props {
  /** QuickBar 配置项列表 */
  items: QuickConfigItem[]
}

const props = withDefaults(defineProps<Props>(), {
  items: () => [],
})

/**
 * 组件事件定义
 */
const emit = defineEmits<{
  /** 触发 action 事件 */
  (e: 'action', key: string, childKey?: string): void
}>()

/**
 * Quick 上下文
 */
const quickContext: QuickContext = {
  emitAction: (key: string, childKey?: string) => {
    emit('action', key, childKey)
  },
}

/**
 * Quick 工厂映射表
 * 注册所有可用的 Quick 组合式函数
 */
const quickFactories: Record<string, QuickFactory> = {
  theme: useQuickTheme,
  sidebar: useQuickLayout,
  footer: useQuickLayout,
  user: useQuickUser,
  personalization: useQuickPersonalization,
  notification: useQuickNotification,
}

// ==================== 在 setup 同步期间创建所有 Quick 实例 ====================

/**
 * 根据配置创建 Quick 实例
 * 在 setup 同步期间执行，确保 inject/onMounted 在正确上下文执行
 */
function createQuickInstances(items: QuickConfigItem[]): Record<string, UseQuickReturn> {
  const instances: Record<string, UseQuickReturn> = {}

  for (const item of items) {
    const factory = quickFactories[item.key]
    if (factory && !instances[item.key]) {
      // 在 setup 同步期间调用 composable，确保生命周期钩子正确注册
      instances[item.key] = factory(item, quickContext)
    }
  }

  return instances
}

// 创建 Quick 实例（setup 同步期间）
const quickInstances = createQuickInstances(props.items)

/**
 * 将 UseQuickReturn 转换为 QuickRuntimeItem
 * 提取 Ref 的值用于渲染
 */
function toRuntimeItem(config: QuickConfigItem, quickReturn: UseQuickReturn): QuickRuntimeItem {
  return {
    ...config,
    isActive: unref(quickReturn.isActive),
    currentIcon: unref(quickReturn.currentIcon),
    disabled: quickReturn.disabled ? unref(quickReturn.disabled) : false,
    onClick: quickReturn.onClick,
    onSelect: quickReturn.onSelect,
  }
}

/**
 * 运行时 Quick 项列表
 * 使用 computed 确保响应式更新（图标、激活状态等）
 */
const runtimeItems = computed<QuickRuntimeItem[]>(() => {
  return props.items.map((config): QuickRuntimeItem => {
    const quickReturn = quickInstances[config.key]

    if (quickReturn) {
      // 使用已创建的 Quick 实例，unref 获取当前值
      return toRuntimeItem(config, quickReturn)
    }

    // 默认行为：直接触发 action 事件
    return {
      ...config,
      isActive: false,
      currentIcon: config.icon,
      onClick: () => emit('action', config.key),
      onSelect: config.type === 'menu'
        ? (childKey: string) => emit('action', config.key, childKey)
        : undefined,
    }
  })
})

const visibleItems = computed(() =>
  runtimeItems.value.filter(item => item.display === 'visible')
)

const dropdownItems = computed(() =>
  runtimeItems.value.filter(item => item.display === 'dropdown')
)

function getDropdownOptions(item: QuickRuntimeItem) {
  if (!item.children || item.children.length === 0) {
    return []
  }
  return item.children.map(child => ({
    key: child.key,
    label: child.label,
    icon: child.icon
      ? () => h(getIconComponent(child.icon!), { size: 16 })
      : undefined,
    disabled: child.disabled,
  }))
}

function handleMenuSelect(item: QuickRuntimeItem, childKey: string) {
  if (item.onSelect) {
    item.onSelect(childKey)
  } else {
    emit('action', item.key, childKey)
  }
}

const aggregatedOptions = computed(() => {
  const options: Array<{
    key: string
    label: string
    icon?: () => ReturnType<typeof h>
    children?: Array<{ key: string; label: string; icon?: () => ReturnType<typeof h> }>
  }> = []

  for (const item of dropdownItems.value) {
    if (item.type === 'menu' && item.children && item.children.length > 0) {
      options.push({
        key: item.key,
        label: item.label,
        icon: () => h(getIconComponent(item.currentIcon), { size: 16 }),
        children: item.children.map(child => ({
          key: `${item.key}:${child.key}`,
          label: child.label,
          icon: child.icon
            ? () => h(getIconComponent(child.icon!), { size: 16 })
            : undefined,
        })),
      })
    } else {
      options.push({
        key: item.key,
        label: item.label,
        icon: () => h(getIconComponent(item.currentIcon), { size: 16 }),
      })
    }
  }
  return options
})

function handleAggregatedSelect(key: string) {
  if (key.includes(':')) {
    const [parentKey, childKey] = key.split(':') as [string, string]
    const parent = runtimeItems.value.find(item => item.key === parentKey)
    if (parent?.onSelect) {
      parent.onSelect(childKey)
    } else {
      emit('action', parentKey, childKey)
    }
  } else {
    const item = runtimeItems.value.find(item => item.key === key)
    if (item) {
      item.onClick()
    } else {
      emit('action', key)
    }
  }
}
</script>

<style scoped>
.quick-bar {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  padding: var(--space-1);
  background: rgba(0, 0, 0, 0.02);
  border-radius: var(--radius-lg);
}

.action-btn {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 38px;
  height: 38px;
  border: none;
  background: transparent;
  border-radius: var(--radius-md);
  cursor: pointer;
  color: var(--text-secondary);
  transition: all var(--duration-fast) var(--ease-smooth);
  overflow: hidden;
}

.action-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
  transform: translateY(-1px);
}

.action-btn:active {
  transform: scale(0.95);
}

.action-btn.is-disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.action-btn.is-disabled:hover {
  background: transparent;
  color: var(--text-secondary);
  transform: none;
}

.action-btn.is-active {
  color: var(--color-primary);
}

.action-btn.is-active .icon-wrapper {
  background: var(--color-primary-light);
}

.icon-wrapper {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: var(--radius-md);
  transition: all var(--duration-fast) var(--ease-smooth);
}

.action-btn:hover .icon-wrapper {
  background: rgba(0, 0, 0, 0.04);
}

.action-btn.is-active:hover .icon-wrapper {
  background: var(--color-primary-light);
}

.icon {
  transition: transform var(--duration-fast) var(--ease-smooth);
}

.action-btn:hover .icon {
  transform: scale(1.1);
}

.action-btn .badge {
  position: absolute;
  top: 4px;
  right: 4px;
  min-width: 18px;
  height: 18px;
  padding: 0 5px;
  background: var(--color-error);
  color: white;
  font-size: 11px;
  font-weight: 600;
  border-radius: var(--radius-full);
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 2px 6px rgba(239, 68, 68, 0.4);
  animation: badgePulse 2s ease-in-out infinite;
}

@keyframes badgePulse {

  0%,
  100% {
    box-shadow: 0 2px 6px rgba(239, 68, 68, 0.4);
  }

  50% {
    box-shadow: 0 2px 10px rgba(239, 68, 68, 0.6);
  }
}

.active-dot {
  position: absolute;
  bottom: 4px;
  left: 50%;
  transform: translateX(-50%) scale(0);
  width: 4px;
  height: 4px;
  background: var(--color-primary);
  border-radius: var(--radius-full);
  transition: transform var(--duration-fast) var(--ease-spring);
}

.action-btn.is-active .active-dot {
  transform: translateX(-50%) scale(1);
}
</style>
