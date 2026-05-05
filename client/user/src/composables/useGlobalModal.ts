import { ref, markRaw, type Component } from 'vue'
import type { ModalPreset } from '@/components/common/GlobalModal.vue'

/**
 * 弹窗选项
 */
export interface ModalOptions {
  /** 弹窗标题 */
  title?: string
  /** 要渲染的组件 */
  component?: Component
  /** 传递给组件的 props */
  componentProps?: Record<string, unknown>
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
  component: Component | undefined
  componentProps: Record<string, unknown>
  preset: ModalPreset
  maskClosable: boolean
  closable: boolean
  positiveText: string
  negativeText: string
  loading: boolean
}>({
  visible: false,
  title: '',
  component: undefined,
  componentProps: {},
  preset: 'card',
  maskClosable: true,
  closable: true,
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
    title: options.title ?? '',
    component: options.component ? markRaw(options.component) : undefined,
    componentProps: options.componentProps ?? {},
    preset: options.preset ?? 'card',
    maskClosable: options.maskClosable ?? true,
    closable: options.closable ?? true,
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
      preset: 'dialog',
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
    /** 处理确认点击（供 GlobalModal 组件使用） */
    handlePositiveClick,
    /** 处理取消点击（供 GlobalModal 组件使用） */
    handleNegativeClick,
    /** 处理关闭（供 GlobalModal 组件使用） */
    handleClose,
  }
}
