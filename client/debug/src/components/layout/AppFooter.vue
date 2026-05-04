<template>
  <footer v-if="shouldShowFooter" class="app-footer">
    <StatusBar class="footer-status-bar" />
  </footer>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { StatusBar } from '@/components/common'
import { useStatusBar } from '@/composables/useStatusBar'
import { useLayoutStore } from '@/store'

/**
 * AppFooter 组件
 * 页面底部布局组件，使用 StatusBar 显示状态信息
 * 当 StatusBar 没有内容时自动隐藏
 */

const layoutStore = useLayoutStore()
const { hasContent, visible } = useStatusBar()

/** 是否应该显示 footer */
const shouldShowFooter = computed(() => {
  if (!layoutStore.isFooterVisible) return false
  if (!visible.value) return false
  return hasContent.value
})
</script>

<style scoped>
.app-footer {
  display: flex;
  align-items: center;
  justify-content: center;
  height: var(--footer-height);
  width: var(--footer-compact-width, 100vw);
  margin: 0 var(--footer-margin, 0);
  padding: 0 24px;
  background: var(--bg-container);
  color: var(--text-tertiary);
  font-size: 12px;
  border-radius: var(--footer-border-radius) var(--footer-border-radius) 0 0;
  opacity: var(--footer-opacity);
  position: fixed;
  bottom: 0;
  left: 0;
  z-index: 98;
  transition: var(--transition-base);
  border: var(--layout-border-width) var(--layout-border-style) var(--layout-border-color);
  border-bottom: none;
}

/* StatusBar 样式 */
.footer-status-bar {
  width: 100%;
  max-width: 100%;
}

/* 平板端适配 */
@media screen and (min-width: 768px) and (max-width: 1024px) {
  .app-footer {
    padding: 0 16px;
  }
}

/* 移动端适配 */
@media screen and (max-width: 767px) {
  .app-footer {
    padding: 0 12px;
  }
}
</style>
