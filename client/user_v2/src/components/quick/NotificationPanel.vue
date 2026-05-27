<script setup lang="ts">
import { ElDrawer, ElEmpty } from 'element-plus'

interface Props {
  visible: boolean
}

defineProps<Props>()

const emit = defineEmits<{
  close: []
}>()

function handleClose() {
  emit('close')
}
</script>

<template>
  <ElDrawer
    :model-value="visible"
    title="通知"
    size="400px"
    @close="handleClose"
  >
    <ElEmpty description="暂无通知" />
  </ElDrawer>
</template>
