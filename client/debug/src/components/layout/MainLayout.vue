<template>
  <div class="main-layout" :style="layoutStore.cssVariables">
    <!-- 背景图片层 -->
    <div v-if="layoutStore.layoutStyles.backgroundImage" class="background-layer" :style="backgroundLayerStyle"></div>

    <!-- 头部导航 -->
    <AppHeader />

    <!-- 侧边栏 -->
    <AppSideBar :is-collapsed="layoutStore.isSidebarCollapsed" :is-mobile-menu-open="layoutStore.isMobileMenuOpen"
      @update:is-collapsed="layoutStore.isSidebarCollapsed = $event"
      @update:is-mobile-menu-open="layoutStore.isMobileMenuOpen = $event" />

    <!-- 主内容区 -->
    <main class="main-content" :style="contentStyle">
      <!-- 内容滚动区域 -->
      <div class="content-scroll-area">
        <div class="content-inner">
          <!-- 页面内容路由视图 -->
          <RouterView />
        </div>

        <!-- 页面级 DockBar -->
        <DockBar :config="currentPageDockConfig" :sidebar-width="layoutStore.sidebarWidth" @missing-params="handleMissingParams" />
      </div>
    </main>

    <!-- 底部 - 移出主内容区，与头部、侧边栏同级 -->
    <AppFooter v-show="layoutStore.isFooterVisible" />

    <!-- 全局弹窗 -->
    <GlobalModal v-model:show="modalState.visible" :title="modalState.title" :content="modalState.content"
      :component="modalState.component" :component-props="modalState.componentProps" :preset="modalState.preset"
      :type="modalState.type" :width="modalState.width" :mask-closable="modalState.maskClosable"
      :closable="modalState.closable" :show-icon="modalState.showIcon" :positive-text="modalState.positiveText"
      :negative-text="modalState.negativeText" :loading="modalState.loading" @positive-click="handleModalPositiveClick"
      @negative-click="handleModalNegativeClick" @close="handleModalClose" />
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted } from 'vue'
import { RouterView, useRoute } from 'vue-router'
import { storeToRefs } from 'pinia'
import AppHeader from './AppHeader.vue'
import AppSideBar from './AppSideBar.vue'
import AppFooter from './AppFooter.vue'
import DockBar from '@/components/common/DockBar.vue'
import GlobalModal from '@/components/common/GlobalModal.vue'
import { useLayoutStore } from '@/store/layout'
import { useUIStore } from '@/store'
import { useConfig } from '@/composables'
import { useGlobalModal } from '@/composables/useGlobalModal'
import { useMessage } from 'naive-ui'


/**
 * MainLayout 组件
 * 主布局容器，整合 Header、SideBar、Footer 和内容区域
 * 支持三端响应式适配和页面级 DockBar
 * 布局策略：固定头部/侧边栏/底部，内容区域独立滚动
 */

const route = useRoute()
const layoutStore = useLayoutStore()
const { config: uiConfig } = useConfig()
const { state: modalState, handlePositiveClick: handleModalPositiveClick, handleNegativeClick: handleModalNegativeClick, handleClose: handleModalClose } = useGlobalModal()
const message = useMessage()

/**
 * 背景图层样式
 */
const backgroundLayerStyle = computed(() => ({
  backgroundImage: `url(${layoutStore.layoutStyles.backgroundImage})`,
  opacity: layoutStore.layoutStyles.backgroundOpacity,
}))

/**
 * 处理 DockBar 缺少参数事件
 */
function handleMissingParams(msg: string) {
  message.warning(msg)
}

// 从 store 获取响应式状态
const { isMobile } = storeToRefs(layoutStore)

/**
 * 监听窗口大小变化
 */
const handleResize = () => {
  layoutStore.updateBreakpoint()
}

onMounted(() => {
  window.addEventListener('resize', handleResize)
})

onUnmounted(() => {
  window.removeEventListener('resize', handleResize)
})

/**
 * 计算主内容区样式
 */
const contentStyle = computed(() => {
  // 底部栏高度（隐藏时为0）
  const footerHeight = layoutStore.isFooterVisible ? 'var(--footer-height)' : '0'

  // 移动端：全宽，无侧边栏
  if (isMobile.value) {
    return {
      marginLeft: '0',
      paddingTop: 'var(--header-height-mobile)',
      paddingBottom: footerHeight,
    }
  }

  return {
    marginLeft: layoutStore.sidebarWidth,
    paddingTop: 'var(--header-height)',
    paddingBottom: footerHeight,
  }
})

/**
 * 当前页面的 Dock 配置
 * 根据路由路径匹配 dock 配置
 */
const currentPageDockConfig = computed(() => {
  if (!uiConfig.value) return null

  // 获取当前路由的第一级路径作为页面 key
  const pathParts = route.path.split('/').filter(Boolean)
  const pageKey = pathParts[0] || 'default'

  // 查找对应页面的配置
  return uiConfig.value.dock[pageKey] || null
})


</script>

<style scoped>
.main-layout {
  /* 固定视口高度，禁止页面级滚动 */
  height: 100vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  position: relative;
}

.background-layer {
  position: fixed;
  inset: 0;
  background-size: cover;
  background-position: center;
  background-repeat: no-repeat;
  pointer-events: none;
  z-index: -1;
  transition: opacity 0.3s ease;
}

.main-content {
  /* 占据剩余空间 */
  flex: 1;
  display: flex;
  flex-direction: column;
  transition: margin-left 0.3s ease;
  /* 禁止主内容区滚动，由子容器处理 */
  overflow: hidden;
}

.content-scroll-area {
  /* 占据剩余空间（减去 footer 高度） */
  flex: 1;
  /* 唯一滚动容器 */
  overflow-y: auto;
  overflow-x: hidden;
  /* 为 DockBar 绝对定位提供参考 */
  position: relative;
}

.content-inner {
  /* 内容内边距 */
  padding: 24px;
  /* 确保内容最小高度，使 DockBar 可以定位到底部 */
  min-height: 100%;
}

/* 移动端适配 */
@media (max-width: 768px) {
  .content-inner {
    padding: 16px;
  }
}
</style>
