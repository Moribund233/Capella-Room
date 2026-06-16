<script setup lang="ts">
import { ElDialog, ElButton } from 'element-plus'
import { useGlobalModal } from '@/composables/useGlobalModal'

export type ModalPreset = 'dialog' | 'card'

const {
  modalState,
  handlePositiveClick,
  handleNegativeClick,
  handleClose,
} = useGlobalModal()

function onUpdateVisible(val: boolean) {
  if (!val) handleClose()
}
</script>

<template>
  <ElDialog
    :model-value="modalState.visible"
    :title="modalState.title"
    :width="modalState.preset === 'dialog' ? '420px' : '520px'"
    :close-on-click-modal="modalState.maskClosable"
    :show-close="modalState.closable"
    :align-center="true"
    :class="{
      'global-modal--dialog': modalState.preset === 'dialog',
      'global-modal--card': modalState.preset === 'card',
    }"
    :destroy-on-close="true"
    @update:model-value="onUpdateVisible"
  >
    <template v-if="modalState.component">
      <component :is="modalState.component" v-bind="modalState.componentProps" />
    </template>
    <template v-else>
      <slot />
    </template>

    <template v-if="modalState.preset === 'dialog'" #footer>
      <div class="global-modal__footer">
        <ElButton :disabled="modalState.loading" @click="handleNegativeClick">
          {{ modalState.negativeText }}
        </ElButton>
        <ElButton
          type="primary"
          :loading="modalState.loading"
          @click="handlePositiveClick"
        >
          {{ modalState.positiveText }}
        </ElButton>
      </div>
    </template>
  </ElDialog>
</template>

<style scoped lang="scss">
.global-modal__footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

:deep(.el-dialog__body) {
  padding: 0;
  max-height: 70vh;
  overflow-y: auto;
}

.global-modal--dialog :deep(.el-dialog__body) {
  padding: 20px;
}
</style>
