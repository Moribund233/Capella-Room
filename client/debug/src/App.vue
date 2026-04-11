<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import { useRoute } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import MainLayout from './components/layout/MainLayout.vue'

const route = useRoute()
const authStore = useAuthStore()

// 应用启动时获取当前用户信息
onMounted(() => {
  if (authStore.isAuthenticated) {
    authStore.fetchCurrentUser()
  }

  // 初始化 Token 过期监听
  const cleanup = authStore.initTokenExpiredListener()
  onUnmounted(cleanup)
})
</script>

<template>
  <n-config-provider>
    <MainLayout v-if="!route.meta.public">
      <router-view />
    </MainLayout>
    <router-view v-else />
  </n-config-provider>
</template>

<style>
/* 导入全局样式 */
@import './style/color.css';
@import './style/base.css';
@import './style/components.css';
@import './style/breakpoints.css';
</style>
