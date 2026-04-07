<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useThemeStore } from '@/stores/theme'
import AppSidebar from './components/AppSidebar.vue'
import AppHeader from './components/AppHeader.vue'
import AppFooter from './components/AppFooter.vue'

/**
 * 主布局组件
 * 整合侧边栏、顶部导航和页脚
 */

/** 侧边栏折叠状态 */
const sidebarCollapsed = ref(false)

/** 移动端侧边栏打开状态 */
const mobileSidebarOpen = ref(false)

/** 主题状态 */
const themeStore = useThemeStore()

onMounted(() => {
  themeStore.initTheme()
})

/**
 * 处理侧边栏切换
 * 桌面端：折叠/展开侧边栏
 * 移动端：打开/关闭浮层侧边栏
 */
const handleToggleSidebar = () => {
  // 检测是否为移动端（小于1024px）
  const isMobile = window.innerWidth < 1024
  if (isMobile) {
    mobileSidebarOpen.value = !mobileSidebarOpen.value
  } else {
    sidebarCollapsed.value = !sidebarCollapsed.value
  }
}
</script>

<template>
  <div class="layout">
    <!-- 侧边栏 -->
    <AppSidebar v-model:collapsed="sidebarCollapsed" v-model:mobile-open="mobileSidebarOpen" />

    <!-- 主内容区 -->
    <div class="main-wrapper" :class="{ 'sidebar-collapsed': sidebarCollapsed }">
      <!-- 顶部导航 -->
      <AppHeader :collapsed="sidebarCollapsed" @toggle-sidebar="handleToggleSidebar" />

      <!-- 内容区域 -->
      <main class="main-content">
        <div class="content-wrapper">
          <RouterView />
        </div>
      </main>

      <!-- 页脚 -->
      <AppFooter :collapsed="sidebarCollapsed" />
    </div>
  </div>
</template>

<style scoped>
.layout {
  min-height: 100vh;
  background-color: var(--bg-primary);
}

.main-wrapper {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
  margin-left: var(--sidebar-width);
  transition: margin-left var(--transition-normal);
}

.main-wrapper.sidebar-collapsed {
  margin-left: var(--sidebar-collapsed-width);
}

.main-content {
  flex: 1;
  padding-top: var(--header-height);
  padding-bottom: var(--footer-height);
  overflow-x: hidden;
}

.content-wrapper {
  padding: var(--spacing-6);
  min-height: calc(100vh - var(--header-height) - var(--footer-height));
}

/* 响应式 */
@media (max-width: 1023px) {
  .main-wrapper {
    margin-left: 0;
  }

  .main-wrapper.sidebar-collapsed {
    margin-left: 0;
  }
}

@media (max-width: 767px) {
  .content-wrapper {
    padding: var(--spacing-4);
  }
}
</style>
