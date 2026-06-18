<script setup lang="ts">
import { watch } from 'vue'
import { useRoute } from 'vue-router'
import { NavBar } from '@/components/nav'
import { useResponsive } from '@/composables/useResponsive'
import { useRoomStore } from '@/stores/room'
import { storeToRefs } from 'pinia'
import GlobalModal from '@/components/common/GlobalModal.vue'
import { useGlobalModal } from '@/composables/useGlobalModal'

const { isMobile } = useResponsive()
const route = useRoute()
const roomStore = useRoomStore()
const { currentRoom } = storeToRefs(roomStore)
useGlobalModal()

watch(
  () => route.path,
  (path) => {
    if (path !== '/app') {
      roomStore.clearCurrentRoom()
    }
  },
)
</script>

<template>
  <div class="main-layout">
    <!-- 桌面端导航栏 -->
    <NavBar v-if="!isMobile" class="main-layout__navbar" />

    <!-- 主内容区 -->
    <main class="main-layout__content">
      <!-- 页面内容 -->
      <div class="main-layout__page">
        <router-view v-slot="{ Component }">
          <transition name="page" mode="out-in">
            <component :is="Component" />
          </transition>
        </router-view>
      </div>
    </main>

    <!-- 移动端底部导航栏（进入房间后全屏隐藏） -->
    <NavBar v-if="isMobile && !currentRoom" class="main-layout__mobile-nav" />

    <!-- 全局弹窗 -->
    <GlobalModal />
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

/* 移动端底部导航栏 */
.main-layout__mobile-nav {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  z-index: 100;
}

// ─── 页面转场动画 ─────────────────────────────
.page-enter-active,
.page-leave-active {
  transition: opacity 0.18s ease, transform 0.18s ease;
}

.page-enter-from {
  opacity: 0;
  transform: translateY(6px);
}

.page-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}
</style>
