<template>
  <div v-if="visible" ref="dockRef" class="dock-bar" :class="[`position-${position}`, { 'is-expanded': isExpanded }]"
    :style="dockStyle" tabindex="-1" @click="handleDockClick" @blur="handleBlur">
    <!-- 线条指示器 -->
    <div class="dock-indicator" :class="`indicator-${position}`"></div>

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
  /** 路由路径 */
  path: string
  /** 是否禁用 */
  disabled?: boolean
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
}

const emit = defineEmits<Emits>()

const router = useRouter()
const route = useRoute()
const dockRef = ref<HTMLElement | null>(null)

/** 展开状态 */
const isExpanded = ref(false)

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
  }))
})

/** 当前激活的 key */
const activeKey = computed(() => {
  const currentPath = route.path
  const matched = items.value.find((item) => item.path === currentPath)
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

  // 路由跳转
  router.push(item.path)

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
  background: var(--dock-indicator-color, rgba(128, 128, 128, 0.5));
  border-radius: 2px;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

/* 底部位置 - 水平线条 */
.position-bottom .dock-indicator {
  left: 50%;
  bottom: -8px;
  transform: translateX(-50%);
  width: 60px;
  height: 4px;
}

/* 左侧位置 - 垂直线条 */
.position-left .dock-indicator {
  left: -8px;
  top: 50%;
  transform: translateY(-50%);
  width: 4px;
  height: 60px;
}

/* 右侧位置 - 垂直线条 */
.position-right .dock-indicator {
  right: -8px;
  top: 50%;
  transform: translateY(-50%);
  width: 4px;
  height: 60px;
}

/* 展开时隐藏线条 */
.is-expanded .dock-indicator {
  opacity: 0;
  transform: scale(0.8);
}

/* 悬停时线条高亮 */
.dock-bar:hover .dock-indicator {
  background: var(--dock-indicator-hover-color, rgba(128, 128, 128, 0.8));
  transform: translateX(-50%) scaleX(1.2);
}

.position-left .dock-bar:hover .dock-indicator,
.position-right .dock-bar:hover .dock-indicator {
  transform: translateY(-50%) scaleY(1.2);
}

.dock-panel {
  backdrop-filter: blur(10px);
  border-radius: 16px;
  box-shadow: var(--dock-shadow);
  overflow: hidden;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
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
  transition: all 0.2s ease;
}

.dock-item:hover {
  transform: scale(1.1);
}

.dock-item.is-active {
  transform: scale(1.05);
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
