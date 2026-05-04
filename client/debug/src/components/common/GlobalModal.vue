<template>
  <n-modal v-model:show="visible" :title="title" :preset="preset" :style="modalStyle"
    :class="['global-modal', modalClass]" :mask-closable="maskClosable" :closable="closable" :show-icon="showIcon"
    :type="type" :positive-text="positiveText" :negative-text="negativeText" :loading="loading"
    @positive-click="handlePositiveClick" @negative-click="handleNegativeClick" @close="handleClose"
    @after-leave="handleAfterLeave">
    <!-- 自定义图标 -->
    <template v-if="$slots.icon" #icon>
      <slot name="icon" />
    </template>

    <!-- 自定义页头 -->
    <template v-if="$slots.header" #header>
      <slot name="header" />
    </template>

    <!-- 默认内容区域 -->
    <div class="global-modal-content">
      <slot>
        <component :is="component" v-if="component" v-bind="componentProps" />
        <template v-else>{{ content }}</template>
      </slot>
    </div>

    <!-- 自定义页脚 -->
    <template v-if="$slots.footer" #footer>
      <slot name="footer" />
    </template>

    <!-- 自定义操作按钮 -->
    <template v-if="$slots.action" #action>
      <slot name="action" />
    </template>
  </n-modal>
</template>

<script setup lang="ts">
import { computed, ref, watch, type Component } from 'vue'
import { NModal } from 'naive-ui'

/**
 * 弹窗预设类型
 */
export type ModalPreset = 'dialog' | 'card' | 'confirm'

/**
 * 弹窗类型（用于 confirm 预设）
 */
export type ModalType = 'default' | 'info' | 'success' | 'warning' | 'error'

/**
 * 组件属性定义
 */
interface Props {
  /** 是否显示弹窗 */
  show?: boolean
  /** 弹窗标题 */
  title?: string
  /** 弹窗内容（当不使用默认插槽时） */
  content?: string
  /** 要渲染的组件（优先级高于 content） */
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
  /** 是否显示图标（confirm 预设时有效） */
  showIcon?: boolean
  /** 确认按钮文本 */
  positiveText?: string
  /** 取消按钮文本 */
  negativeText?: string
  /** 确认按钮是否加载中 */
  loading?: boolean
  /** 自定义类名 */
  modalClass?: string
}

const props = withDefaults(defineProps<Props>(), {
  show: false,
  title: '提示',
  content: '',
  preset: 'dialog',
  type: 'default',
  width: 420,
  maskClosable: true,
  closable: true,
  showIcon: true,
  positiveText: '确认',
  negativeText: '取消',
  loading: false,
  modalClass: '',
})

/**
 * 组件事件定义
 */
interface Emits {
  /** 更新显示状态 */
  (e: 'update:show', value: boolean): void
  /** 确认按钮点击 */
  (e: 'positive-click'): void
  /** 取消按钮点击 */
  (e: 'negative-click'): void
  /** 弹窗关闭 */
  (e: 'close'): void
  /** 弹窗完全关闭后 */
  (e: 'after-leave'): void
}

const emit = defineEmits<Emits>()

/** 内部显示状态 */
const visible = ref(props.show)

/** 监听外部 show 变化 */
watch(
  () => props.show,
  (newVal) => {
    visible.value = newVal
  }
)

/** 监听内部 visible 变化 */
watch(visible, (newVal) => {
  emit('update:show', newVal)
})

/** 弹窗样式 */
const modalStyle = computed(() => {
  const width = typeof props.width === 'number' ? `${props.width}px` : props.width
  return {
    width,
    maxWidth: '90vw',
  }
})

/**
 * 处理确认按钮点击
 */
function handlePositiveClick(): void {
  emit('positive-click')
}

/**
 * 处理取消按钮点击
 */
function handleNegativeClick(): void {
  emit('negative-click')
  visible.value = false
}

/**
 * 处理弹窗关闭
 */
function handleClose(): void {
  emit('close')
  emit('update:show', false)
}

/**
 * 处理弹窗完全关闭后
 */
function handleAfterLeave(): void {
  emit('after-leave')
}

/**
 * 打开弹窗
 */
function open(): void {
  visible.value = true
}

/**
 * 关闭弹窗
 */
function close(): void {
  visible.value = false
}

/** 暴露方法给父组件 */
defineExpose({
  open,
  close,
  visible,
})
</script>

<style scoped>
.global-modal {
  :deep(.n-modal-body) {
    padding: 20px 24px;
  }

  :deep(.n-modal-header) {
    padding: 16px 24px;
    border-bottom: 1px solid var(--border-color);
  }

  :deep(.n-modal-footer) {
    padding: 12px 24px;
    border-top: 1px solid var(--border-color);
  }
}

.global-modal-content {
  font-size: 14px;
  line-height: 1.6;
  color: var(--text-color);
}

/* 卡片预设样式 */
:deep(.n-card.global-modal) {
  .n-card-header {
    padding: 16px 24px;
    border-bottom: 1px solid var(--border-color);
  }

  .n-card__content {
    padding: 20px 24px;
  }

  .n-card__footer {
    padding: 12px 24px;
    border-top: 1px solid var(--border-color);
  }
}
</style>
