<script setup lang="ts">
import { ref, computed } from 'vue'
import { Plus } from '@element-plus/icons-vue'
import type { QuickGroup, QuickItem } from './types'

/**
 * 组件属性定义
 */
interface Props {
  /** Quick 分组列表 */
  groups: QuickGroup[]
}

const props = defineProps<Props>()

/**
 * 组件事件定义
 */
const emit = defineEmits<{
  /** 点击菜单项 */
  (e: 'select', key: string, childKey?: string): void
}>()

/** 扇形展开状态 */
const isOpen = ref(false)
/** 当前分组索引 */
const currentGroupIndex = ref(0)

/** 当前分组 */
const currentGroup = computed(() => props.groups[currentGroupIndex.value]!)

/** 当前分组的可见项 */
const visibleItems = computed(() =>
  currentGroup.value.items.filter((item) => item.display === 'visible')
)

/** 扇形起始角度（左侧水平） */
const START_ANGLE = 180
/** 扇形展开角度 */
const SPAN_ANGLE = 90
/** 展开半径(px) */
const RADIUS = 110

/**
 * 计算每个扇形项的位置样式
 */
const itemStyles = computed(() => {
  const items = visibleItems.value
  const count = items.length
  if (count === 0) return []

  // 超过5项时自动扩展扇形角度和半径，避免拥挤
  const span = count > 5 ? 140 : SPAN_ANGLE
  const radius = count > 5 ? 130 : RADIUS
  const step = count === 1 ? 0 : span / (count - 1)

  return items.map((_, index) => {
    const angleDeg = START_ANGLE + step * index
    const angleRad = (angleDeg * Math.PI) / 180
    const x = Math.cos(angleRad) * radius
    const y = Math.sin(angleRad) * radius
    return {
      '--dial-x': `${x}px`,
      '--dial-y': `${y}px`,
      '--dial-delay': `${index * 0.03}s`,
    }
  })
})

/**
 * 处理 FAB 点击
 * - 收起状态：展开扇形
 * - 展开状态：切换到下一分组
 */
function handleFabClick(): void {
  if (!isOpen.value) {
    isOpen.value = true
  } else {
    const nextIndex = (currentGroupIndex.value + 1) % props.groups.length
    currentGroupIndex.value = nextIndex
  }
}

/**
 * 点击遮罩层关闭扇形
 */
function handleMaskClick(): void {
  if (isOpen.value) {
    isOpen.value = false
  }
}

/**
 * 处理扇形项点击
 * @param item - Quick 项
 */
function handleItemClick(item: QuickItem): void {
  if (item.onClick) {
    item.onClick()
  }
  emit('select', item.key)
  // 点击项后自动关闭扇形
  isOpen.value = false
}
</script>

<template>
  <div class="quick-dial" :class="{ 'quick-dial--open': isOpen }">
    <!-- 遮罩层：点击关闭 -->
    <div
      v-if="isOpen"
      class="quick-dial__mask"
      @click="handleMaskClick"
    />

    <!-- 扇形菜单项 -->
    <button
      v-for="(item, index) in visibleItems"
      :key="`${currentGroup.key}-${item.key}`"
      class="quick-dial__item"
      :class="{ 'quick-dial__item--active': item.isActive }"
      :style="itemStyles[index]"
      :title="item.label"
      @click="handleItemClick(item)"
    >
      <template v-if="typeof item.icon === 'string'">
        <span class="quick-dial__text">{{ item.icon }}</span>
      </template>
      <el-icon v-else :size="20">
        <component :is="item.icon" />
      </el-icon>
      <span v-if="item.badge" class="quick-dial__badge">{{ item.badge }}</span>
    </button>

    <!-- FAB 主按钮 -->
    <button
      class="quick-dial__fab"
      :class="{ 'quick-dial__fab--open': isOpen }"
      :title="isOpen ? currentGroup.label : '快捷操作'"
      @click="handleFabClick"
    >
      <el-icon :size="24">
        <component :is="currentGroup.icon" v-if="isOpen" />
        <Plus v-else />
      </el-icon>
    </button>
  </div>
</template>

<style scoped lang="scss">
.quick-dial {
  position: fixed;
  right: 16px;
  bottom: 72px;
  z-index: 300;
  width: 56px;
  height: 56px;

  &__mask {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.35);
    z-index: -1;
    animation: mask-fade-in 0.2s ease;
  }

  &__fab {
    position: absolute;
    bottom: 0;
    right: 0;
    width: 56px;
    height: 56px;
    border-radius: 50%;
    border: none;
    background: var(--accent);
    color: #fff;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.25), 0 0 0 1px var(--border);
    transition: transform 0.25s ease, background 0.2s ease, box-shadow 0.2s ease;
    z-index: 2;

    &:active {
      transform: scale(0.92);
    }

    &--open {
      background: var(--surface);
      color: var(--fg);
      box-shadow: 0 2px 10px rgba(0, 0, 0, 0.15), 0 0 0 1px var(--border);
    }
  }

  &__item {
    position: absolute;
    bottom: 6px;
    right: 6px;
    width: 44px;
    height: 44px;
    border-radius: 50%;
    border: none;
    background: var(--surface);
    color: var(--fg);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.12), 0 0 0 1px var(--border);
    z-index: 1;
    transform: translate(0, 0) scale(0.4);
    opacity: 0;
    transition: all 0.35s cubic-bezier(0.34, 1.56, 0.64, 1);
    transition-delay: 0s;

    &:active {
      transform: translate(var(--dial-x), var(--dial-y)) scale(0.9) !important;
    }

    &--active {
      color: var(--accent);
      background: var(--accent-soft);
    }
  }

  &__text {
    font-size: 14px;
    font-weight: 600;
    user-select: none;
  }

  &__badge {
    position: absolute;
    top: -2px;
    right: -2px;
    min-width: 18px;
    height: 18px;
    padding: 0 5px;
    border-radius: 9px;
    background: var(--accent-pink);
    color: #fff;
    font-size: 11px;
    font-weight: 600;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 2px solid var(--surface);
  }

  // 展开状态
  &--open {
    .quick-dial__item {
      transform: translate(var(--dial-x), var(--dial-y)) scale(1);
      opacity: 1;
      transition-delay: var(--dial-delay);
    }
  }
}

@keyframes mask-fade-in {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}
</style>
