<template>
  <div class="websocket-test">
    <n-empty description="WebSocket测试功能开发中">
      <template #icon>
        <n-icon :component="Wifi" />
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
import { Wifi } from 'lucide-vue-next'

const router = useRouter()

const goBack = () => {
  router.push('/home')
}
</script>

<style scoped>
.websocket-test {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 400px;
}
</style>
