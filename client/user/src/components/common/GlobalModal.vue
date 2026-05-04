<script setup lang="ts">
import { computed } from 'vue'
import { X, AlertCircle, CheckCircle, AlertTriangle, Info } from 'lucide-vue-next'

/**
 * 弹窗预设类型
 */
export type ModalPreset = 'dialog' | 'confirm' | 'card'

/**
 * 弹窗类型（用于 confirm 预设）
 */
export type ModalType = 'default' | 'info' | 'success' | 'warning' | 'error'

/**
 * 弹窗尺寸预设
 * - small: 小尺寸，适合确认框 (360px)
 * - medium: 中等尺寸，适合普通弹窗 (480px)
 * - large: 大尺寸，适合复杂内容 (720px)
 * - full: 全屏弹窗
 * - auto: 根据内容自适应
 */
export type ModalSize = 'small' | 'medium' | 'large' | 'full' | 'auto'

/**
 * 组件属性定义
 */
interface Props {
  /** 是否显示 */
  visible: boolean
  /** 弹窗标题 */
  title?: string
  /** 弹窗内容 */
  content?: string
  /** 预设类型 */
  preset?: ModalPreset
  /** 弹窗类型（confirm 预设时有效） */
  type?: ModalType
  /**
   * 弹窗尺寸预设
   * 推荐使用 size 而非直接设置 width，以获得更好的响应式支持
   */
  size?: ModalSize
  /**
   * 自定义弹窗宽度
   * 设置后将覆盖 size 的宽度值，但仍受响应式限制
   */
  width?: string | number
  /** 弹窗最大宽度 */
  maxWidth?: string | number
  /** 弹窗最小宽度 */
  minWidth?: string | number
  /** 是否允许点击遮罩关闭 */
  maskClosable?: boolean
  /** 是否显示关闭按钮 */
  closable?: boolean
  /** 是否显示图标 */
  showIcon?: boolean
  /** 确认按钮文本 */
  positiveText?: string
  /** 取消按钮文本 */
  negativeText?: string
  /** 是否加载中 */
  loading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  title: '提示',
  content: '',
  preset: 'dialog',
  type: 'default',
  size: 'medium',
  maskClosable: true,
  closable: true,
  showIcon: true,
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
 * 尺寸映射表
 * 定义各尺寸在不同断点下的宽度
 */
const sizeMap: Record<ModalSize, { default: string; mobile: string; tablet?: string }> = {
  small: { default: '360px', mobile: '90vw' },
  medium: { default: '480px', mobile: '92vw', tablet: '520px' },
  large: { default: '720px', mobile: '96vw', tablet: '680px' },
  full: { default: '100vw', mobile: '100vw' },
  auto: { default: 'auto', mobile: '90vw' },
}

/**
 * 弹窗宽度样式
 * 优先使用自定义 width，否则使用 size 预设
 */
const widthStyle = computed(() => {
  // 如果设置了自定义 width，使用自定义值
  if (props.width !== undefined) {
    const width = typeof props.width === 'number' ? `${props.width}px` : props.width
    return { width }
  }

  // 使用 size 预设
  const sizeConfig = sizeMap[props.size]
  return {
    '--modal-width': sizeConfig.default,
    '--modal-width-mobile': sizeConfig.mobile,
    '--modal-width-tablet': sizeConfig.tablet || sizeConfig.default,
  }
})

/**
 * 最大/最小宽度样式
 */
const constraintStyle = computed(() => {
  const style: Record<string, string> = {}
  if (props.maxWidth !== undefined) {
    style['--modal-max-width'] = typeof props.maxWidth === 'number' ? `${props.maxWidth}px` : props.maxWidth
  }
  if (props.minWidth !== undefined) {
    style['--modal-min-width'] = typeof props.minWidth === 'number' ? `${props.minWidth}px` : props.minWidth
  }
  return style
})

/**
 * 合并样式
 */
const containerStyle = computed(() => ({
  ...widthStyle.value,
  ...constraintStyle.value,
}))

/**
 * 图标组件
 */
const iconComponent = computed(() => {
  switch (props.type) {
    case 'success':
      return CheckCircle
    case 'warning':
      return AlertTriangle
    case 'error':
      return AlertCircle
    case 'info':
      return Info
    default:
      return AlertCircle
  }
})

/**
 * 图标颜色类
 */
const iconClass = computed(() => {
  switch (props.type) {
    case 'success':
      return 'is-success'
    case 'warning':
      return 'is-warning'
    case 'error':
      return 'is-error'
    case 'info':
      return 'is-info'
    default:
      return ''
  }
})

/**
 * 是否显示底部按钮
 */
const showFooter = computed(() => props.preset === 'confirm' || props.preset === 'dialog')

/**
 * 是否显示取消按钮
 */
const showCancelButton = computed(() => props.preset === 'confirm' && props.negativeText)

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
        <div
          class="modal-container"
          :class="[`modal--${preset}`, `modal--${size}`]"
          :style="containerStyle"
        >
          <!-- 头部 -->
          <div class="modal-header">
            <div class="modal-title-wrapper">
              <component
                :is="iconComponent"
                v-if="showIcon && preset === 'confirm'"
                class="modal-icon"
                :class="iconClass"
                :size="24"
              />
              <h3 class="modal-title">{{ title }}</h3>
            </div>
            <button
              v-if="closable"
              class="modal-close"
              @click="handleClose"
            >
              <X :size="18" />
            </button>
          </div>

          <!-- 内容 -->
          <div class="modal-content">
            <slot>
              <p v-if="content" class="modal-text">{{ content }}</p>
            </slot>
          </div>

          <!-- 底部按钮 -->
          <div v-if="showFooter" class="modal-footer">
            <button
              v-if="showCancelButton"
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
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: var(--space-lg);
}

.modal-container {
  background: var(--color-white);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-xl);
  max-width: 95vw;
  max-height: 90vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;

  /* 响应式宽度 */
  width: var(--modal-width, 480px);
  max-width: var(--modal-max-width, 95vw);
  min-width: var(--modal-min-width, auto);
}

/* 全屏模式 */
.modal--full {
  width: 100vw;
  height: 100vh;
  max-width: 100vw;
  max-height: 100vh;
  border-radius: 0;
}

/* 自适应模式 */
.modal--auto {
  width: auto;
  min-width: 320px;
}

/* 移动端响应式 */
@media (max-width: 640px) {
  .modal-container:not(.modal--full) {
    width: var(--modal-width-mobile, 92vw) !important;
  }
}

/* 平板响应式 */
@media (min-width: 641px) and (max-width: 1024px) {
  .modal-container:not(.modal--full) {
    width: var(--modal-width-tablet, var(--modal-width, 480px));
  }
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-md) var(--space-lg);
  border-bottom: 1px solid var(--color-border);
  flex-shrink: 0;
}

.modal-title-wrapper {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

.modal-icon {
  flex-shrink: 0;
}

.modal-icon.is-success {
  color: var(--color-success);
}

.modal-icon.is-warning {
  color: var(--color-warning);
}

.modal-icon.is-error {
  color: var(--color-danger);
}

.modal-icon.is-info {
  color: var(--color-info);
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

.modal-content {
  padding: var(--space-lg);
  overflow-y: auto;
  flex: 1;
}

.modal-text {
  margin: 0;
  color: var(--color-text-secondary);
  line-height: 1.6;
}

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
