<script setup lang="ts">
import { ElDialog, ElForm, ElFormItem, ElSelect, ElOption } from 'element-plus'

interface Props {
  visible: boolean
}

defineProps<Props>()

const emit = defineEmits<{
  close: []
  update: [visible: boolean]
}>()

function handleClose() {
  emit('close')
  emit('update', false)
}
</script>

<template>
  <ElDialog
    :model-value="visible"
    title="个性化设置"
    width="500px"
    @update:model-value="$emit('update', $event)"
  >
    <ElForm label-width="100px">
      <ElFormItem label="主题">
        <ElSelect placeholder="选择主题">
          <ElOption label="浅色" value="light" />
          <ElOption label="深色" value="dark" />
          <ElOption label="跟随系统" value="auto" />
        </ElSelect>
      </ElFormItem>
    </ElForm>
    <template #footer>
      <div class="dialog-footer">
        <ElButton @click="handleClose">取消</ElButton>
        <ElButton type="primary" @click="handleClose">确定</ElButton>
      </div>
    </template>
  </ElDialog>
</template>
