<template>
  <div class="not-found-view">
    <n-result
      status="404"
      title="页面未找到"
      description="您访问的页面不存在"
    >
      <template #footer>
        <n-button type="primary" @click="goHome">
          返回首页
        </n-button>
      </template>
    </n-result>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'
import { NResult, NButton } from 'naive-ui'

const router = useRouter()

function goHome() {
  router.push('/home')
}
</script>

<style scoped>
.not-found-view {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
}
</style>
