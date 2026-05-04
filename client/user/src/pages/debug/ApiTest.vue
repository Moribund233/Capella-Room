<template>
  <div class="api-test">
    <n-empty description="API测试功能开发中">
      <template #icon>
        <n-icon :component="Terminal" />
      </template>
      <template #extra>
        <n-button type="primary" @click="goBack">返回</n-button>
      </template>
    </n-empty>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'
import { NEmpty, NIcon, NButton } from 'naive-ui'
import { Terminal } from 'lucide-vue-next'

const router = useRouter()

const goBack = () => {
  router.push('/home')
}
</script>

<style scoped>
.api-test {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 400px;
}
</style>
