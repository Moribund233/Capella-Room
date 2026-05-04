import { ref, markRaw, type Component } from 'vue'
import type { ModalPreset, ModalType, ModalSize } from '@/components/common/GlobalModal.vue'

/**
 * 弹窗选项
 */
export interface ModalOptions {
  /** 弹窗标题 */
  title?: string
  /** 弹窗内容 */
  content?: string
  /** 要渲染的组件 */
  component?: Component
  /** 传递给组件的 props */
  componentProps?: Record<string, unknown>
  /** 预设类型 */
  preset?: ModalPreset
  /** 弹窗类型（confirm 预设时有效） */
  type?: ModalType
  /**
   * 弹窗尺寸预设
   * - small: 小尺寸，适合确认框 (360px)
   * - medium: 中等尺寸，适合普通弹窗 (480px)
   * - large: 大尺寸，适合复杂内容 (720px)
   * - full: 全屏弹窗
   * - auto: 根据内容自适应
   */
  size?: ModalSize
  /** 自定义弹窗宽度（覆盖 size） */
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
  /** 确认按钮回调 */
  onPositiveClick?: () => void | Promise<void>
  /** 取消按钮回调 */
  onNegativeClick?: () => void
  /** 弹窗关闭回调 */
  onClose?: () => void
}

/**
 * 全局弹窗状态
 */
const modalState = ref<{
  visible: boolean
  title: string
  content: string
  component: Component | undefined
  componentProps: Record<string, unknown>
  preset: ModalPreset
  type: ModalType
  size: ModalSize
  width: string | number | undefined
  maxWidth: string | number | undefined
  minWidth: string | number | undefined
  maskClosable: boolean
  closable: boolean
  showIcon: boolean
  positiveText: string
  negativeText: string
  loading: boolean
}>({
  visible: false,
  title: '提示',
  content: '',
  component: undefined,
  componentProps: {},
  preset: 'dialog',
  type: 'default',
  size: 'medium',
  width: undefined,
  maxWidth: undefined,
  minWidth: undefined,
  maskClosable: true,
  closable: true,
  showIcon: true,
  positiveText: '确认',
  negativeText: '取消',
  loading: false,
})

/** 当前弹窗的回调函数 */
let currentCallbacks: {
  onPositiveClick?: () => void | Promise<void>
  onNegativeClick?: () => void
  onClose?: () => void
} = {}

/**
 * 打开弹窗
 * @param options 弹窗选项
 */
function openModal(options: ModalOptions): void {
  modalState.value = {
    ...modalState.value,
    visible: true,
    title: options.title ?? '提示',
    content: options.content ?? '',
    component: options.component ? markRaw(options.component) : undefined,
    componentProps: options.componentProps ?? {},
    preset: options.preset ?? 'dialog',
    type: options.type ?? 'default',
    size: options.size ?? 'medium',
    width: options.width,
    maxWidth: options.maxWidth,
    minWidth: options.minWidth,
    maskClosable: options.maskClosable ?? true,
    closable: options.closable ?? true,
    showIcon: options.showIcon ?? true,
    positiveText: options.positiveText ?? '确认',
    negativeText: options.negativeText ?? '取消',
    loading: false,
  }

  currentCallbacks = {
    onPositiveClick: options.onPositiveClick,
    onNegativeClick: options.onNegativeClick,
    onClose: options.onClose,
  }
}

/**
 * 关闭弹窗
 */
function closeModal(): void {
  modalState.value.visible = false
}

/**
 * 显示确认弹窗
 * @param options 弹窗选项
 * @returns Promise<boolean>
 */
function confirm(options: Omit<ModalOptions, 'preset'>): Promise<boolean> {
  return new Promise((resolve) => {
    openModal({
      ...options,
      preset: 'confirm',
      size: options.size ?? 'small', // 确认框默认小尺寸
      onPositiveClick: async () => {
        await options.onPositiveClick?.()
        resolve(true)
      },
      onNegativeClick: () => {
        options.onNegativeClick?.()
        resolve(false)
      },
      onClose: () => {
        options.onClose?.()
        resolve(false)
      },
    })
  })
}

/**
 * 显示信息弹窗
 * @param options 弹窗选项
 */
function info(options: Omit<ModalOptions, 'preset' | 'type'>): void {
  openModal({
    ...options,
    preset: 'confirm',
    type: 'info',
    size: options.size ?? 'small',
    positiveText: options.positiveText ?? '知道了',
    negativeText: '',
  })
}

/**
 * 显示成功弹窗
 * @param options 弹窗选项
 */
function success(options: Omit<ModalOptions, 'preset' | 'type'>): void {
  openModal({
    ...options,
    preset: 'confirm',
    type: 'success',
    size: options.size ?? 'small',
    positiveText: options.positiveText ?? '知道了',
    negativeText: '',
  })
}

/**
 * 显示警告弹窗
 * @param options 弹窗选项
 */
function warning(options: Omit<ModalOptions, 'preset' | 'type'>): void {
  openModal({
    ...options,
    preset: 'confirm',
    type: 'warning',
    size: options.size ?? 'small',
  })
}

/**
 * 显示错误弹窗
 * @param options 弹窗选项
 */
function error(options: Omit<ModalOptions, 'preset' | 'type'>): void {
  openModal({
    ...options,
    preset: 'confirm',
    type: 'error',
    size: options.size ?? 'small',
    positiveText: options.positiveText ?? '知道了',
    negativeText: '',
  })
}

/**
 * 处理确认点击
 */
async function handlePositiveClick(): Promise<void> {
  if (currentCallbacks.onPositiveClick) {
    modalState.value.loading = true
    try {
      await currentCallbacks.onPositiveClick()
      closeModal()
    } finally {
      modalState.value.loading = false
    }
  } else {
    closeModal()
  }
}

/**
 * 处理取消点击
 */
function handleNegativeClick(): void {
  currentCallbacks.onNegativeClick?.()
  closeModal()
}

/**
 * 处理弹窗关闭
 */
function handleClose(): void {
  currentCallbacks.onClose?.()
  closeModal()
}

/**
 * 使用全局弹窗
 */
export function useGlobalModal() {
  return {
    /** 弹窗状态 */
    modalState,
    /** 打开弹窗 */
    open: openModal,
    /** 关闭弹窗 */
    close: closeModal,
    /** 确认对话框 */
    confirm,
    /** 信息提示 */
    info,
    /** 成功提示 */
    success,
    /** 警告提示 */
    warning,
    /** 错误提示 */
    error,
    /** 处理确认点击（供 GlobalModal 组件使用） */
    handlePositiveClick,
    /** 处理取消点击（供 GlobalModal 组件使用） */
    handleNegativeClick,
    /** 处理关闭（供 GlobalModal 组件使用） */
    handleClose,
  }
}
