<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { NavBar } from '@/components/nav'
import { useResponsive } from '@/composables/useResponsive'

const route = useRoute()
const { isMobile } = useResponsive()

// 判断是否在聊天页面（显示侧边栏的页面）
const isChatPage = computed(() => {
  const chatRoutes = ['app']
  return chatRoutes.includes(route.name as string)
})
</script>

<template>
  <div class="main-layout">
    <!-- 桌面端导航栏 -->
    <NavBar v-if="!isMobile" class="main-layout__navbar" />

    <!-- 主内容区 -->
    <main class="main-layout__content" :class="{ 'main-layout--mobile': isMobile }">
      <!-- 页面内容 -->
      <div class="main-layout__page">
        <router-view />
      </div>
    </main>

    <!-- 移动端底部导航栏 -->
    <NavBar v-if="isMobile" class="main-layout__mobile-nav" />
  </div>
</template>

<style scoped lang="scss">
.main-layout {
  display: flex;
  width: 100vw;
  height: 100vh;
  overflow: hidden;
  background: var(--bg);
  color: var(--fg);
}

/* 桌面端导航栏 */
.main-layout__navbar {
  flex-shrink: 0;
}

/* 主内容区 */
.main-layout__content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-width: 0;
}

/* 页面内容 */
.main-layout__page {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

/* 移动端样式 */
.main-layout--mobile {
  /* 移动端底部导航栏空间 */
  padding-bottom: 0;
}

/* 移动端底部导航栏 */
.main-layout__mobile-nav {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  z-index: 100;
}
</style>
