<script setup lang="ts">
import { ElDialog, ElButton } from 'element-plus'

export type ModalPreset = 'confirm' | 'info' | 'success' | 'warning' | 'error' | 'card' | 'dialog'

interface Props {
  visible: boolean
  title?: string
  preset?: ModalPreset
  content?: string
  confirmText?: string
  cancelText?: string
  showCancel?: boolean
}

withDefaults(defineProps<Props>(), {
  title: '',
  preset: 'info',
  content: '',
  confirmText: '确定',
  cancelText: '取消',
  showCancel: true,
})

const emit = defineEmits<{
  confirm: []
  cancel: []
  update: [visible: boolean]
}>()

function handleConfirm() {
  emit('confirm')
  emit('update', false)
}

function handleCancel() {
  emit('cancel')
  emit('update', false)
}
</script>

<template>
  <ElDialog
    :model-value="visible"
    :title="title"
    width="400px"
    @update:model-value="$emit('update', $event)"
  >
    <p>{{ content }}</p>
    <template #footer>
      <div class="dialog-footer">
        <ElButton v-if="showCancel" @click="handleCancel">{{ cancelText }}</ElButton>
        <ElButton type="primary" @click="handleConfirm">{{ confirmText }}</ElButton>
      </div>
    </template>
  </ElDialog>
</template>
