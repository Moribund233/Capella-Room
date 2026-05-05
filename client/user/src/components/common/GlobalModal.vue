<script setup lang="ts">
import { X } from 'lucide-vue-next'

/**
 * 弹窗预设类型
 * - dialog: 对话框形式（带底部按钮）
 * - card: 卡片形式（纯内容容器）
 */
export type ModalPreset = 'dialog' | 'card'

/**
 * 组件属性定义
 */
interface Props {
  /** 是否显示 */
  visible: boolean
  /** 弹窗标题 */
  title?: string
  /** 预设类型 */
  preset?: ModalPreset
  /** 是否允许点击遮罩关闭 */
  maskClosable?: boolean
  /** 是否显示关闭按钮 */
  closable?: boolean
  /** 确认按钮文本 */
  positiveText?: string
  /** 取消按钮文本 */
  negativeText?: string
  /** 是否加载中 */
  loading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  title: '',
  preset: 'card',
  maskClosable: true,
  closable: true,
  positiveText: '确认',
  negativeText: '取消',
  loading: false,
})

/**
 * 组件事件定义
 */
const emit = defineEmits<{
  /** 更新显示状态 */
  (e: 'update:visible', value: boolean): void
  /** 点击确认按钮 */
  (e: 'positiveClick'): void
  /** 点击取消按钮 */
  (e: 'negativeClick'): void
  /** 弹窗关闭 */
  (e: 'close'): void
}>()

/**
 * 处理点击遮罩
 */
function handleMaskClick() {
  if (props.maskClosable && !props.loading) {
    handleClose()
  }
}

/**
 * 处理关闭
 */
function handleClose() {
  emit('update:visible', false)
  emit('close')
}

/**
 * 处理确认
 */
function handlePositiveClick() {
  emit('positiveClick')
}

/**
 * 处理取消
 */
function handleNegativeClick() {
  emit('negativeClick')
  handleClose()
}
</script>

<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="visible" class="modal-overlay" @click.self="handleMaskClick">
        <div class="modal-container" :class="`modal--${preset}`">
          <!-- 头部 -->
          <div v-if="title || closable" class="modal-header">
            <h3 v-if="title" class="modal-title">{{ title }}</h3>
            <button v-if="closable" class="modal-close" @click="handleClose">
              <X :size="18" />
            </button>
          </div>

          <!-- 内容区域 - 纯容器，不处理内部排版 -->
          <div class="modal-content">
            <slot />
          </div>

          <!-- 底部按钮 - 仅 dialog 预设显示 -->
          <div v-if="preset === 'dialog'" class="modal-footer">
            <button
              v-if="negativeText"
              class="modal-btn modal-btn--default"
              :disabled="loading"
              @click="handleNegativeClick"
            >
              {{ negativeText }}
            </button>
            <button
              class="modal-btn modal-btn--primary"
              :class="{ 'is-loading': loading }"
              :disabled="loading"
              @click="handlePositiveClick"
            >
              {{ positiveText }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
/* 遮罩层 */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: var(--color-mask);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: var(--space-lg);
}

/* 弹窗容器 - 纯 flex 容器，不限制尺寸 */
.modal-container {
  display: flex;
  flex-direction: column;
  background: var(--color-white);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-xl);
  max-height: 90vh;
  overflow: hidden;
  /* 尺寸完全由内容或外部组件决定 */
  width: auto;
  height: auto;
}

/* 头部 */
.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-md) var(--space-lg);
  border-bottom: 1px solid var(--color-border);
  flex-shrink: 0;
}

.modal-title {
  font-size: var(--font-size-h5);
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0;
}

.modal-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: none;
  background: transparent;
  color: var(--color-text-secondary);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--duration-fast);
}

.modal-close:hover {
  background: var(--color-background);
  color: var(--color-text-primary);
}

/* 内容区域 - 纯容器 */
.modal-content {
  flex: 1;
  overflow-y: auto;
  /* 不设置 padding，由内容组件自行控制 */
}

/* 底部按钮 */
.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-sm);
  padding: var(--space-md) var(--space-lg);
  border-top: 1px solid var(--color-border);
  flex-shrink: 0;
}

.modal-btn {
  padding: var(--space-sm) var(--space-lg);
  border-radius: var(--radius-md);
  font-size: var(--font-size-body);
  font-weight: 500;
  cursor: pointer;
  transition: all var(--duration-fast);
  border: 1px solid transparent;
}

.modal-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.modal-btn--default {
  background: var(--color-white);
  border-color: var(--color-border);
  color: var(--color-text-primary);
}

.modal-btn--default:hover:not(:disabled) {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.modal-btn--primary {
  background: var(--color-primary);
  color: white;
}

.modal-btn--primary:hover:not(:disabled) {
  background: var(--color-primary-hover);
}

.modal-btn.is-loading {
  position: relative;
  color: transparent;
}

.modal-btn.is-loading::after {
  content: '';
  position: absolute;
  top: 50%;
  left: 50%;
  width: 16px;
  height: 16px;
  margin: -8px 0 0 -8px;
  border: 2px solid transparent;
  border-top-color: white;
  border-radius: 50%;
  animation: modal-spin 0.8s linear infinite;
}

@keyframes modal-spin {
  to {
    transform: rotate(360deg);
  }
}

/* 过渡动画 */
.modal-enter-active,
.modal-leave-active {
  transition: opacity var(--duration-normal);
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-active .modal-container,
.modal-leave-active .modal-container {
  transition: transform var(--duration-normal) var(--ease-out);
}

.modal-enter-from .modal-container,
.modal-leave-to .modal-container {
  transform: scale(0.95);
}
</style>
