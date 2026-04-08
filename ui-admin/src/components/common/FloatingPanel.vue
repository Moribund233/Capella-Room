<script setup lang="ts">
import { Close } from '@element-plus/icons-vue'

/**
 * 通用浮窗组件
 * 提供顶层浮窗容器，包含关闭按钮和统一样式
 * 内部不维护具体业务逻辑，只提供内容渲染区
 *
 * @example
 * <FloatingPanel
 *   v-model:visible="isVisible"
 *   title="通知列表"
 *   width="480px"
 * >
 *   <template #default>
 *     <!-- 自定义内容 -->
 *   </template>
 * </FloatingPanel>
 */

/** 组件属性定义 */
interface Props {
  /** 是否显示浮窗 */
  visible: boolean
  /** 浮窗标题 */
  title?: string
  /** 浮窗宽度，默认 480px */
  width?: string
  /** 是否显示遮罩层，默认 true */
  showOverlay?: boolean
  /** 点击遮罩层是否关闭，默认 true */
  closeOnOverlayClick?: boolean
  /** 是否显示关闭按钮，默认 true */
  showCloseButton?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  visible: false,
  title: '',
  width: '480px',
  showOverlay: true,
  closeOnOverlayClick: true,
  showCloseButton: true,
})

/** 组件事件定义 */
interface Emits {
  /** 更新显示状态 */
  'update:visible': [value: boolean]
  /** 关闭回调 */
  close: []
}

const emit = defineEmits<Emits>()

/**
 * 关闭浮窗
 */
const handleClose = () => {
  emit('update:visible', false)
  emit('close')
}

/**
 * 处理遮罩层点击
 */
const handleOverlayClick = () => {
  if (props.closeOnOverlayClick) {
    handleClose()
  }
}

/**
 * 处理内容区点击，阻止冒泡
 */
const handleContentClick = (event: MouseEvent) => {
  event.stopPropagation()
}
</script>

<template>
  <Teleport to="body">
    <Transition name="floating-panel">
      <div v-show="visible" class="floating-panel-wrapper" @click="handleOverlayClick">
        <!-- 遮罩层 -->
        <div v-if="showOverlay" class="floating-panel-overlay"></div>

        <!-- 浮窗主体 -->
        <div
          class="floating-panel"
          :style="{ width }"
          @click="handleContentClick"
        >
          <!-- 头部 -->
          <div class="floating-panel-header">
            <h3 class="floating-panel-title">{{ title }}</h3>
            <button
              v-if="showCloseButton"
              class="floating-panel-close"
              @click="handleClose"
              aria-label="关闭"
            >
              <Close class="close-icon" />
            </button>
          </div>

          <!-- 内容区 -->
          <div class="floating-panel-body">
            <slot></slot>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
/* 浮窗包装器 - 全屏定位 */
.floating-panel-wrapper {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: var(--z-modal);
  pointer-events: auto;
}

/* 遮罩层 */
.floating-panel-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(2px);
}

/* 浮窗主体 */
.floating-panel {
  position: relative;
  display: flex;
  flex-direction: column;
  max-height: 80vh;
  background-color: var(--bg-primary);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-xl);
  overflow: hidden;
  z-index: 1;
}

/* 头部 */
.floating-panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--spacing-4) var(--spacing-6);
  border-bottom: 1px solid var(--border-secondary);
  flex-shrink: 0;
}

.floating-panel-title {
  font-size: var(--font-size-lg);
  font-weight: var(--font-weight-semibold);
  color: var(--text-primary);
  margin: 0;
}

.floating-panel-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  padding: 0;
  border: none;
  border-radius: var(--radius-md);
  background-color: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.floating-panel-close:hover {
  background-color: var(--bg-secondary);
  color: var(--text-primary);
}

.close-icon {
  width: 18px;
  height: 18px;
}

/* 内容区 */
.floating-panel-body {
  flex: 1;
  overflow-y: auto;
  padding: var(--spacing-4) var(--spacing-6);
}

/* 滚动条样式 */
.floating-panel-body::-webkit-scrollbar {
  width: 6px;
}

.floating-panel-body::-webkit-scrollbar-track {
  background: transparent;
}

.floating-panel-body::-webkit-scrollbar-thumb {
  background-color: var(--border-primary);
  border-radius: var(--radius-full);
}

.floating-panel-body::-webkit-scrollbar-thumb:hover {
  background-color: var(--text-muted);
}

/* 过渡动画 */
.floating-panel-enter-active,
.floating-panel-leave-active {
  transition: opacity var(--transition-normal);
}

.floating-panel-enter-from,
.floating-panel-leave-to {
  opacity: 0;
}

.floating-panel-enter-active .floating-panel,
.floating-panel-leave-active .floating-panel {
  transition: transform var(--transition-normal), opacity var(--transition-normal);
}

.floating-panel-enter-from .floating-panel,
.floating-panel-leave-to .floating-panel {
  transform: scale(0.95) translateY(-10px);
  opacity: 0;
}

/* 暗色主题适配 */
:global(.dark) .floating-panel-overlay {
  background-color: rgba(0, 0, 0, 0.7);
}
</style>
