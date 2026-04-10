<script setup lang="ts">
import { onMounted } from 'vue'
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
