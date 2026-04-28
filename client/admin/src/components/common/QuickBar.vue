<template>
  <div class="quick-bar">
    <!-- 外显按钮区域 -->
    <template v-for="item in visibleItems" :key="item.key">
      <!-- Action 类型：直接点击执行 -->
      <n-tooltip v-if="item.type === 'action'" :show-arrow="false" placement="bottom" :disabled="item.disabled">
        <template #trigger>
          <button class="action-btn" :class="{ 'is-disabled': item.disabled }" @click="item.onClick"
            :aria-label="item.label" :disabled="item.disabled">
            <component :is="getIconComponent(item.currentIcon)" class="icon" :size="18" />
            <span v-if="item.badge" class="badge">{{ item.badge }}</span>
          </button>
        </template>
        {{ item.disabled ? '暂不可用' : item.label }}
      </n-tooltip>

      <!-- Menu 类型：带下拉菜单 -->
      <n-dropdown v-else :options="getDropdownOptions(item)" placement="bottom" trigger="click"
        @select="(key: string) => handleMenuSelect(item, key)" :disabled="item.disabled">
        <button class="action-btn" :class="{ 'is-disabled': item.disabled }" :aria-label="item.label"
          :disabled="item.disabled">
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
import { computed, h, ref, watch } from 'vue'
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
  type UseQuickReturn,
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
 * Quick 上下文
 * 使用 emit 触发 action 事件给父组件处理
 */
const quickContext: QuickContext = {
  emitAction: (key: string, childKey?: string) => {
    emit('action', key, childKey)
  },
}

// ==================== 在 setup 同步期间创建所有 Quick 实例 ====================

/**
 * 主题 Quick
 */
const themeQuick = useQuickTheme({ key: 'theme', type: 'action', icon: 'Sun', iconAlt: 'Moon', label: '主题', display: 'visible' }, quickContext)

/**
 * 侧边栏 Quick
 */
const sidebarQuick = useQuickLayout({ key: 'sidebar', type: 'action', icon: 'PanelLeft', label: '侧边栏', display: 'visible' }, quickContext)

/**
 * 页脚 Quick
 */
const footerQuick = useQuickLayout({ key: 'footer', type: 'action', icon: 'PanelBottom', label: '页脚', display: 'visible' }, quickContext)

/**
 * 用户 Quick
 */
const userQuick = useQuickUser({ key: 'user', type: 'menu', icon: 'UserCircle', iconAlt: 'User', label: '用户中心', display: 'visible', children: [] }, quickContext)

/**
 * 个性化 Quick
 */
const personalizationQuick = useQuickPersonalization({ key: 'personalization', type: 'action', icon: 'Palette', label: '个性化', display: 'visible' }, quickContext)

/**
 * Quick 实例映射表
 */
const quickInstances: Record<string, UseQuickReturn> = {
  theme: themeQuick,
  sidebar: sidebarQuick,
  footer: footerQuick,
  user: userQuick,
  personalization: personalizationQuick,
}

/**
 * 运行时 Quick 项列表
 */
const runtimeItems = ref<QuickRuntimeItem[]>([])

/**
 * 根据配置生成运行时项
 * 在 setup 同步期间执行，确保 inject() 在正确上下文中
 */
const generateRuntimeItems = (items: QuickConfigItem[]): QuickRuntimeItem[] => {
  return items.map((config): QuickRuntimeItem => {
    const quickInstance = quickInstances[config.key]

    if (!quickInstance) {
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

    // 使用已创建的 Quick 实例
    return {
      ...config,
      isActive: quickInstance.isActive.value,
      currentIcon: quickInstance.currentIcon.value,
      disabled: quickInstance.disabled?.value ?? false,
      onClick: quickInstance.onClick,
      onSelect: quickInstance.onSelect,
    }
  })
}

// 初始化运行时项
runtimeItems.value = generateRuntimeItems(props.items)

/**
 * 监听配置变化，更新运行时项
 * 注意：这里只更新值，不重新创建 Quick 实例
 */
watch(
  () => props.items,
  (newItems) => {
    runtimeItems.value = generateRuntimeItems(newItems)
  },
  { deep: true }
)

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

.action-btn.is-disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.action-btn.is-disabled:hover {
  background: transparent;
}

.icon {
  flex-shrink: 0;
}

.badge {
  position: absolute;
  top: 2px;
  right: 2px;
  min-width: 16px;
  height: 16px;
  padding: 0 4px;
  font-size: 10px;
  font-weight: 600;
  line-height: 16px;
  text-align: center;
  color: white;
  background: var(--color-error);
  border-radius: 8px;
}
</style>
