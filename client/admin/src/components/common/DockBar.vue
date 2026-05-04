<template>
  <div v-if="visible" ref="dockRef" class="dock-bar" :class="[`position-${position}`, { 'is-expanded': isExpanded }]"
    :style="dockStyle" tabindex="-1" @click="handleDockClick" @blur="handleBlur">
    <!-- 线条指示器 -->
    <div class="dock-indicator" :class="`indicator-${position}`">
      <div class="indicator-glow"></div>
    </div>

    <!-- 悬浮面板容器 -->
    <n-card class="dock-panel" :bordered="false" size="small">
      <!-- Dock 项目列表 -->
      <n-space :vertical="isVertical" :size="8" align="center" justify="center">
        <n-tooltip v-for="item in items" :key="item.key" placement="top" trigger="hover">
          <template #trigger>
            <n-button class="dock-item" :class="{ 'is-active': activeKey === item.key }"
              :type="activeKey === item.key ? 'primary' : 'default'" quaternary :circle="true" size="large"
              @click.stop="handleItemClick(item)">
              <template #icon>
                <component :is="item.icon" :size="22" />
              </template>
            </n-button>
          </template>
          {{ item.label }}
        </n-tooltip>
      </n-space>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { NCard, NSpace, NButton, NTooltip } from 'naive-ui'
import type { FunctionalComponent } from 'vue'
import type { LucideProps } from 'lucide-vue-next'
import * as LucideIcons from 'lucide-vue-next'
import type { DockPageConfig } from '@/config'

/**
 * Dock 项类型
 */
export interface DockItem {
  /** 唯一标识 */
  key: string
  /** 显示标签 */
  label: string
  /** 图标组件 */
  icon: FunctionalComponent<LucideProps>
  /** 路由路径（可以是模板，如 /room/chat/:id） */
  path: string
  /** 是否禁用 */
  disabled?: boolean
  /** 是否需要路由参数 */
  requiresParams?: boolean
  /** 参数名列表 */
  paramKeys?: string[]
  /** 缺少参数时的提示消息 */
  missingParamsMessage?: string
}

/**
 * 获取图标组件
 * @param iconName 图标名称
 * @returns 图标组件
 */
function getIconComponent(iconName: string): FunctionalComponent<LucideProps> {
  return (LucideIcons as unknown as Record<string, FunctionalComponent<LucideProps>>)[iconName]
    || LucideIcons.Layout
}

/**
 * Dock 位置类型
 */
export type DockPosition = 'bottom' | 'left' | 'right'

/**
 * 组件属性定义
 */
interface Props {
  /** 页面级配置 */
  config?: DockPageConfig | null
  /** 侧边栏宽度（用于计算水平居中位置） */
  sidebarWidth?: string
}

const props = withDefaults(defineProps<Props>(), {
  config: null,
  sidebarWidth: '0px',
})

/**
 * 组件事件定义
 */
interface Emits {
  /** 展开/收起状态变化 */
  (e: 'update:expanded', value: boolean): void
  /** 缺少必要参数时触发，传递提示消息 */
  (e: 'missingParams', message: string): void
}

const emit = defineEmits<Emits>()

const router = useRouter()
const route = useRoute()
const dockRef = ref<HTMLElement | null>(null)

/** 展开状态 */
const isExpanded = ref(false)

/** 缓存的路由参数 - 用于在参数缺失时保留上一个有效值 */
const cachedParams = ref<Record<string, string>>({})

/** 是否显示 */
const visible = computed(() => props.config?.enabled ?? false)

/** 位置 */
const position = computed(() => props.config?.position ?? 'bottom')

/** 距离边缘的距离 */
const offset = computed(() => props.config?.offset ?? 24)

/** 自动收起延迟 */
const autoCollapseDelay = computed(() => 300)

/** 是否垂直布局 */
const isVertical = computed(() => position.value === 'left' || position.value === 'right')

/** Dock 项列表 */
const items = computed<DockItem[]>(() => {
  if (!props.config?.items) return []

  return props.config.items.map(item => ({
    key: item.key,
    label: item.label,
    icon: getIconComponent(item.icon),
    path: item.path,
    disabled: item.disabled,
    requiresParams: item.requiresParams,
    paramKeys: item.paramKeys,
    missingParamsMessage: item.missingParamsMessage,
  }))
})

/**
 * 解析路径模板，替换参数
 * 优先使用当前路由参数，如果缺失则使用缓存的参数
 * @param path 路径模板（如 /room/chat/:id）
 * @returns 解析后的实际路径
 */
function resolvePath(path: string): string {
  // 从当前路由获取参数，并更新缓存
  const params = route.params
  const mergedParams: Record<string, string> = { ...cachedParams.value }

  // 更新缓存：使用当前路由的有效参数
  Object.keys(params).forEach(key => {
    const value = params[key]
    if (value) {
      const replacement = Array.isArray(value) ? value[0] : value
      if (replacement) {
        mergedParams[key] = replacement
        cachedParams.value[key] = replacement
      }
    }
  })

  // 替换路径中的参数占位符
  let resolvedPath = path
  Object.keys(mergedParams).forEach(key => {
    const value = mergedParams[key]
    if (value) {
      resolvedPath = resolvedPath.replace(`:${key}`, value)
    }
  })

  return resolvedPath
}

/** 当前激活的 key */
const activeKey = computed(() => {
  const currentPath = route.path
  const matched = items.value.find((item) => {
    // 解析路径模板后比较
    const resolvedPath = resolvePath(item.path)
    return resolvedPath === currentPath
  })
  return matched?.key || ''
})

/**
 * Dock 样式
 * 桌面端使用 fixed 定位，通过 calc 计算在内容区域水平居中
 */
const dockStyle = computed(() => {
  const offsetValue = `${offset.value}px`
  const sidebarWidth = props.sidebarWidth

  switch (position.value) {
    case 'bottom':
      return {
        // 使用 calc 计算：侧边栏宽度 + (剩余宽度 / 2)
        left: `calc(${sidebarWidth} + (100% - ${sidebarWidth}) / 2)`,
        // 使用 CSS 变量避开 footer
        bottom: 'var(--dock-bottom-offset, 80px)',
        transform: 'translateX(-50%)',
      }
    case 'left':
      return {
        left: `calc(${sidebarWidth} + ${offsetValue})`,
        top: '50%',
        transform: 'translateY(-50%)',
      }
    case 'right':
      return {
        right: offsetValue,
        top: '50%',
        transform: 'translateY(-50%)',
      }
    default:
      return {}
  }
})

/**
 * 处理 Dock 点击 - 展开面板
 */
const handleDockClick = () => {
  if (!isExpanded.value) {
    isExpanded.value = true
    emit('update:expanded', true)
    // 聚焦以接收 blur 事件
    dockRef.value?.focus()
  }
}

/**
 * 处理失焦 - 收起面板
 */
const handleBlur = () => {
  // 延迟收起，避免点击子元素时立即收起
  setTimeout(() => {
    if (isExpanded.value) {
      isExpanded.value = false
      emit('update:expanded', false)
    }
  }, autoCollapseDelay.value)
}

/**
 * 处理 Dock 项点击
 */
const handleItemClick = (item: DockItem) => {
  if (item.disabled) return

  // 解析路径模板，替换参数（优先使用当前路由参数，缺失时使用缓存）
  const resolvedPath = resolvePath(item.path)

  // 如果解析后的路径仍然缺少必要参数，则触发事件并返回
  if (item.requiresParams && resolvedPath.includes(':')) {
    console.warn(`[DockBar] 无法跳转，缺少必要参数: ${item.path}`)
    const message = item.missingParamsMessage || '缺少必要参数，无法跳转'
    emit('missingParams', message)
    return
  }

  // 路由跳转
  router.push(resolvedPath)

  // 点击后收起面板
  setTimeout(() => {
    isExpanded.value = false
    emit('update:expanded', false)
  }, 150)
}

/**
 * 处理外部点击
 */
const handleOutsideClick = (event: MouseEvent) => {
  if (dockRef.value && !dockRef.value.contains(event.target as Node)) {
    if (isExpanded.value) {
      isExpanded.value = false
      emit('update:expanded', false)
    }
  }
}

/**
 * 处理键盘事件
 */
const handleKeyDown = (event: KeyboardEvent) => {
  if (event.key === 'Escape' && isExpanded.value) {
    isExpanded.value = false
    emit('update:expanded', false)
  }
}

onMounted(() => {
  document.addEventListener('click', handleOutsideClick)
  document.addEventListener('keydown', handleKeyDown)
})

onUnmounted(() => {
  document.removeEventListener('click', handleOutsideClick)
  document.removeEventListener('keydown', handleKeyDown)
})
</script>

<style scoped>
.dock-bar {
  /* 统一使用 fixed 定位，通过 calc 计算位置实现内容区域居中 */
  position: fixed;
  z-index: 100;
  outline: none;
}

/* 线条指示器 */
.dock-indicator {
  position: absolute;
  background: var(--color-primary);
  border-radius: var(--radius-full);
  cursor: pointer;
  transition: all var(--duration-normal) var(--ease-smooth);
  overflow: hidden;
}

/* 发光效果 */
.indicator-glow {
  position: absolute;
  inset: 0;
  background: var(--color-primary-gradient);
  opacity: 0.6;
  filter: blur(4px);
  animation: pulse-glow 2s ease-in-out infinite;
}

@keyframes pulse-glow {
  0%, 100% {
    opacity: 0.4;
    transform: scale(1);
  }
  50% {
    opacity: 0.8;
    transform: scale(1.1);
  }
}

/* 底部位置 - 水平线条 */
.position-bottom .dock-indicator {
  left: 50%;
  bottom: -10px;
  transform: translateX(-50%);
  width: 40px;
  height: 4px;
}

/* 左侧位置 - 垂直线条 */
.position-left .dock-indicator {
  left: -10px;
  top: 50%;
  transform: translateY(-50%);
  width: 4px;
  height: 40px;
}

/* 右侧位置 - 垂直线条 */
.position-right .dock-indicator {
  right: -10px;
  top: 50%;
  transform: translateY(-50%);
  width: 4px;
  height: 40px;
}

/* 展开时隐藏线条 */
.is-expanded .dock-indicator {
  opacity: 0;
  transform: scale(0.8);
}

/* 悬停时线条高亮 */
.dock-bar:hover .indicator-bottom {
  width: 56px;
  box-shadow: 0 0 12px var(--color-primary);
}

.dock-bar:hover .indicator-left,
.dock-bar:hover .indicator-right {
  height: 56px;
  box-shadow: 0 0 12px var(--color-primary);
}

.dock-panel {
  background: var(--dock-bg);
  backdrop-filter: var(--glass-backdrop);
  -webkit-backdrop-filter: var(--glass-backdrop);
  border: 1px solid var(--glass-border);
  border-radius: var(--radius-2xl);
  box-shadow: var(--dock-shadow);
  padding: var(--space-3);
  transition: all var(--duration-slow) var(--ease-out-expo);
}

/* 收起状态 - 面板隐藏 */
.dock-bar:not(.is-expanded) .dock-panel {
  opacity: 0;
  transform: scale(0.8);
  pointer-events: none;
}

/* 展开状态 - 面板显示 */
.is-expanded .dock-panel {
  opacity: 1;
  transform: scale(1);
  pointer-events: auto;
}

.dock-item {
  width: 44px;
  height: 44px;
  border-radius: var(--radius-xl);
  transition: all var(--duration-normal) var(--ease-spring);
}

.dock-item:hover {
  transform: scale(1.1) translateY(-2px);
  background: var(--color-primary-light);
}

.dock-item.is-active {
  background: var(--color-primary-gradient);
  box-shadow: 0 4px 12px rgba(99, 102, 241, 0.4);
}

/* 位置样式 */
.position-bottom .dock-panel {
  display: flex;
  flex-direction: row;
}

.position-left .dock-panel,
.position-right .dock-panel {
  display: flex;
  flex-direction: column;
}

/* 移动端适配 */
@media (max-width: 768px) {
  .dock-indicator {
    width: 40px;
    height: 3px;
  }

  .position-left .dock-indicator,
  .position-right .dock-indicator {
    width: 3px;
    height: 40px;
  }
}
</style>
