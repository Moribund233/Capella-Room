<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { Menu } from 'lucide-vue-next'
import { useResponsive } from '@/composables/useResponsive'
import { useRoomStore } from '@/stores/room'
import { useGlobalModal } from '@/composables/useGlobalModal'
import { QuickBar } from '@/components/quick'
import { useQuickBar } from '@/composables/quick'
import { quickBarConfig } from '@/config/quick'
import NavBar from '@/components/nav/NavBar.vue'
import MobileSidebar from '@/components/layout/MobileSidebar.vue'
import DesktopSidebar from '@/components/layout/DesktopSidebar.vue'
import CreateRoomModal from '@/components/room/CreateRoomModal.vue'
import PageTransition from '@/components/ui/PageTransition.vue'
import GlobalModal from '@/components/common/GlobalModal.vue'

const router = useRouter()
const { isMobile, isTablet, isDesktop } = useResponsive()
const roomStore = useRoomStore()

// 初始化全局弹窗
const { modalState, handlePositiveClick, handleNegativeClick, handleClose } = useGlobalModal()

// 初始化移动端 QuickBar
const { items: mobileQuickItems } = useQuickBar(quickBarConfig)

const showMobileSidebar = ref(false)
const showCreateModal = ref(false)

function handleRoomCreated(roomId: string) {
  showCreateModal.value = false
  if (roomId) {
    router.push(`/room/${roomId}`)
  }
}

function openMobileSidebar() {
  showMobileSidebar.value = true
}

function closeMobileSidebar() {
  showMobileSidebar.value = false
}

function openCreateModal() {
  showCreateModal.value = true
  // 如果在移动端，关闭侧边栏
  if (isMobile.value) {
    showMobileSidebar.value = false
  }
}

// 键盘快捷键
function onKeyDown(e: KeyboardEvent) {
  // Ctrl/Cmd + B 切换侧边栏
  if ((e.ctrlKey || e.metaKey) && e.key === 'b') {
    e.preventDefault()
    if (isMobile.value) {
      showMobileSidebar.value = !showMobileSidebar.value
    }
  }
  // ESC 关闭移动端侧边栏
  if (e.key === 'Escape' && showMobileSidebar.value) {
    showMobileSidebar.value = false
  }
}

onMounted(() => {
  roomStore.fetchMyRooms()
  window.addEventListener('keydown', onKeyDown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', onKeyDown)
})
</script>

<template>
  <div class="main-layout">
    <!-- 桌面端导航栏 -->
    <NavBar v-if="!isMobile" class="main-layout__navbar" />

    <!-- 桌面端侧边栏 -->
    <DesktopSidebar
      v-if="isDesktop || isTablet"
      class="main-layout__sidebar-desktop"
      @create-room="openCreateModal"
    />

    <!-- 移动端侧边栏 -->
    <MobileSidebar
      v-if="isMobile"
      :show="showMobileSidebar"
      @close="closeMobileSidebar"
      @create-room="openCreateModal"
    />

    <!-- 主内容区 -->
    <main class="main-layout__content" :class="{ 'main-layout--mobile': isMobile }">
      <!-- 移动端顶部栏 -->
      <header v-if="isMobile" class="main-layout__mobile-header">
        <button
          class="main-layout__menu-btn"
          @click="openMobileSidebar"
          aria-label="打开菜单"
        >
          <Menu :size="24" />
        </button>
        <h1 class="main-layout__mobile-title">Seredeli</h1>
        <!-- 移动端 QuickBar -->
        <div class="main-layout__mobile-quick">
          <QuickBar :items="mobileQuickItems" position="mobile-header" />
        </div>
      </header>

      <!-- 页面内容 -->
      <div class="main-layout__page">
        <router-view v-slot="{ Component }">
          <PageTransition name="slide">
            <component :is="Component" />
          </PageTransition>
        </router-view>
      </div>
    </main>

    <!-- 移动端底部导航栏 -->
    <NavBar v-if="isMobile" class="main-layout__mobile-nav" />

    <!-- 创建聊天室弹窗 -->
    <CreateRoomModal
      :show="showCreateModal"
      @close="showCreateModal = false"
      @created="handleRoomCreated"
    />

    <!-- 全局弹窗 -->
    <GlobalModal
      v-model:visible="modalState.visible"
      :title="modalState.title"
      :content="modalState.content"
      :preset="modalState.preset"
      :type="modalState.type"
      :size="modalState.size"
      :width="modalState.width"
      :max-width="modalState.maxWidth"
      :min-width="modalState.minWidth"
      :mask-closable="modalState.maskClosable"
      :closable="modalState.closable"
      :show-icon="modalState.showIcon"
      :positive-text="modalState.positiveText"
      :negative-text="modalState.negativeText"
      :loading="modalState.loading"
      @positive-click="handlePositiveClick"
      @negative-click="handleNegativeClick"
      @close="handleClose"
    >
      <component
        :is="modalState.component"
        v-if="modalState.component"
        v-bind="modalState.componentProps"
      />
    </GlobalModal>
  </div>
</template>

<style scoped>
.main-layout {
  display: flex;
  width: 100vw;
  height: 100vh;
  overflow: hidden;
  background: var(--color-background);
}

/* 桌面端导航栏 */
.main-layout__navbar {
  flex-shrink: 0;
}

/* 桌面端侧边栏 */
.main-layout__sidebar-desktop {
  flex-shrink: 0;
  height: 100%;
}

/* 主内容区 */
.main-layout__content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-width: 0;
}

/* 移动端样式 */
.main-layout--mobile {
  padding-bottom: 56px; /* 底部导航栏高度 */
}

.main-layout__mobile-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-md) var(--space-lg);
  background: var(--color-white);
  border-bottom: 1px solid var(--color-border);
  flex-shrink: 0;
  height: 56px;
}

.main-layout__menu-btn {
  width: 40px;
  height: 40px;
  border: none;
  background: transparent;
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-primary);
  cursor: pointer;
  transition: background var(--duration-fast);
}

.main-layout__menu-btn:active {
  background: var(--color-background);
}

.main-layout__mobile-title {
  font-size: var(--font-size-h3);
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0;
}

.main-layout__mobile-spacer {
  width: 40px;
}

/* 移动端 QuickBar */
.main-layout__mobile-quick {
  display: flex;
  align-items: center;
}

/* 页面内容 */
.main-layout__page {
  flex: 1;
  overflow: hidden;
  position: relative;
}

/* 移动端底部导航栏 */
.main-layout__mobile-nav {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  z-index: 200;
}

/* 响应式调整 */
@media (max-width: 767px) {
  .main-layout {
    flex-direction: column;
  }
}
</style>
