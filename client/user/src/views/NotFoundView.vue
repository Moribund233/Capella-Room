<script setup lang="ts">
import { useRouter } from 'vue-router'
import { NButton, NResult } from 'naive-ui'
import { ROUTE_PATHS } from '@/constants'

const router = useRouter()

function goHome() {
  router.push(ROUTE_PATHS.CHAT)
}
</script>

<template>
  <div class="not-found">
    <NResult status="404" title="页面不存在" description="请检查访问地址是否正确">
      <template #footer>
        <NButton type="primary" @click="goHome">返回首页</NButton>
      </template>
    </NResult>
  </div>
</template>

<style scoped>
.not-found {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
}
</style>
