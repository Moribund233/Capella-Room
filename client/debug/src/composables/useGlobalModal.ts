import { ref, markRaw, type Component } from 'vue'
import type { ModalPreset, ModalType } from '@/components/common/GlobalModal.vue'

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
  /** 弹窗宽度 */
  width?: string | number
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
  width: string | number
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
  width: 420,
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
    width: options.width ?? 420,
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
    positiveText: options.positiveText ?? '知道了',
    negativeText: '',
  })
}

/**
 * 处理确认按钮点击
 */
async function handlePositiveClick(): Promise<void> {
  modalState.value.loading = true
  try {
    await currentCallbacks.onPositiveClick?.()
    closeModal()
  } finally {
    modalState.value.loading = false
  }
}

/**
 * 处理取消按钮点击
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
}

/**
 * 使用全局弹窗
 * @returns 全局弹窗方法
 */
export function useGlobalModal() {
  return {
    /** 弹窗状态 */
    state: modalState,
    /** 打开弹窗 */
    open: openModal,
    /** 关闭弹窗 */
    close: closeModal,
    /** 确认弹窗 */
    confirm,
    /** 信息弹窗 */
    info,
    /** 成功弹窗 */
    success,
    /** 警告弹窗 */
    warning,
    /** 错误弹窗 */
    error,
    /** 确认按钮点击处理 */
    handlePositiveClick,
    /** 取消按钮点击处理 */
    handleNegativeClick,
    /** 弹窗关闭处理 */
    handleClose,
  }
}

export default useGlobalModal
