<template>
  <div class="quick-bar">
    <!-- 外显按钮区域 -->
    <template v-for="item in visibleItems" :key="item.key">
      <!-- Action 类型：直接点击执行 -->
      <n-tooltip v-if="item.type === 'action'" :show-arrow="false" placement="bottom">
        <template #trigger>
          <button class="action-btn" @click="item.onClick" :aria-label="item.label">
            <component :is="getIconComponent(item.currentIcon)" class="icon" :size="18" />
            <span v-if="item.badge" class="badge">{{ item.badge }}</span>
          </button>
        </template>
        {{ item.label }}
      </n-tooltip>

      <!-- Menu 类型：带下拉菜单 -->
      <n-dropdown v-else :options="getDropdownOptions(item)" placement="bottom" trigger="click"
        @select="(key: string) => handleMenuSelect(item, key)">
        <button class="action-btn" :aria-label="item.label">
          <component :is="getIconComponent(item.currentIcon)" class="icon" :size="18" />
          <span v-if="item.badge" class="badge">{{ item.badge }}</span>
        </button>
      </n-dropdown>
    </template>

    <!-- 下拉菜单按钮（聚合所有 dropdown 类型的按钮） -->
    <n-dropdown v-if="dropdownItems.length > 0" :options="aggregatedOptions" placement="bottom-end" trigger="click"
      @select="handleAggregatedSelect">
      <button class="action-btn" aria-label="更多操作">
        <MoreVertical class="icon" :size="18" />
      </button>
    </n-dropdown>
  </div>
</template>

<script setup lang="ts">
import { computed, h, ref, watchEffect } from 'vue'
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
  type QuickConfigItem,
  type QuickRuntimeItem,
  type QuickContext,
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
  // 使用缓存的图标组件
  if (iconCache.has(iconName)) {
    return iconCache.get(iconName)!
  }

  // 直接从 lucide-vue-next 获取图标组件
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
 * Quick 组合式函数映射表
 * 注册所有可用的 Quick 组合式函数
 */
const quickFactories: Record<string, typeof useQuickTheme> = {
  theme: useQuickTheme,
  sidebar: useQuickLayout,
  footer: useQuickLayout,
  user: useQuickUser,
  personalization: useQuickPersonalization,
}

/**
 * Quick 上下文
 * 使用 emit 触发 action 事件给父组件处理
 */
const quickContext: QuickContext = {
  emitAction: (key: string, childKey?: string) => {
    emit('action', key, childKey)
  },
}

/**
 * 运行时 Quick 项列表
 */
const runtimeItems = ref<QuickRuntimeItem[]>([])

/**
 * 根据配置生成运行时项
 */
watchEffect(() => {
  runtimeItems.value = props.items.map((config): QuickRuntimeItem => {
    const factory = quickFactories[config.key]

    if (!factory) {
      // 未注册的 Quick，使用默认行为（直接触发 action 事件）
      return {
        ...config,
        isActive: false,
        currentIcon: config.icon,
        onClick: () => emit('action', config.key),
        onSelect: config.type === 'menu'
          ? (childKey: string) => emit('action', config.key, childKey)
          : undefined,
      }
    }

    // 调用对应的 Quick 组合式函数
    const quickReturn = factory(config, quickContext)

    return {
      ...config,
      isActive: quickReturn.isActive.value,
      currentIcon: quickReturn.currentIcon.value,
      onClick: quickReturn.onClick,
      onSelect: quickReturn.onSelect,
    }
  })
})

/**
 * 外显按钮列表
 */
const visibleItems = computed(() =>
  runtimeItems.value.filter(item => item.display === 'visible')
)

/**
 * 下拉菜单按钮列表
 */
const dropdownItems = computed(() =>
  runtimeItems.value.filter(item => item.display === 'dropdown')
)

/**
 * 获取下拉菜单选项（用于 Menu 类型的外显按钮）
 */
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

/**
 * 处理外显菜单项选择
 */
function handleMenuSelect(item: QuickRuntimeItem, childKey: string) {
  if (item.onSelect) {
    item.onSelect(childKey)
  } else {
    emit('action', item.key, childKey)
  }
}

/**
 * 聚合下拉菜单选项
 */
const aggregatedOptions = computed(() => {
  const options: Array<{
    key: string
    label: string
    icon?: () => ReturnType<typeof h>
    children?: Array<{ key: string; label: string; icon?: () => ReturnType<typeof h> }>
  }> = []

  for (const item of dropdownItems.value) {
    if (item.type === 'menu' && item.children && item.children.length > 0) {
      // Menu 类型：渲染为级联菜单
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
      // Action 类型：渲染为普通菜单项
      options.push({
        key: item.key,
        label: item.label,
        icon: () => h(getIconComponent(item.currentIcon), { size: 16 }),
      })
    }
  }

  return options
})

/**
 * 处理聚合菜单选择
 */
function handleAggregatedSelect(key: string) {
  // 检查是否是级联菜单项（格式：parentKey:childKey）
  if (key.includes(':')) {
    const [parentKey, childKey] = key.split(':') as [string, string]
    const parent = runtimeItems.value.find(item => item.key === parentKey)
    if (parent?.onSelect) {
      parent.onSelect(childKey)
    } else {
      emit('action', parentKey, childKey)
    }
  } else {
    // 普通菜单项
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
  gap: 8px;
}

.action-btn {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border: none;
  background: transparent;
  border-radius: 6px;
  cursor: pointer;
  color: var(--text-secondary);
  transition: var(--transition-fast);
}

.action-btn:hover {
  background: var(--bg-layout);
  color: var(--color-primary);
}

.action-btn .icon {
  transition: var(--transition-fast);
}

.action-btn .badge {
  position: absolute;
  top: 4px;
  right: 4px;
  min-width: 16px;
  height: 16px;
  padding: 0 4px;
  background: var(--color-error);
  color: white;
  font-size: 10px;
  font-weight: 600;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
}
</style>
