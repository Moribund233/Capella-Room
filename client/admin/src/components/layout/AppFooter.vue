<template>
  <footer v-if="shouldShowFooter" class="app-footer" :class="{ 'has-status-bar': showStatusBar }">
    <!-- StatusBar 区域 -->
    <status-bar v-if="showStatusBar" @update:has-content="handleStatusBarContentChange">
      <template v-if="isVNodeContent">
        <component :is="statusBarContentVNode" />
      </template>
      <template v-else>{{ statusBarContent }}</template>
    </status-bar>

    <!-- Footer 主内容 -->
    <div v-if="showFooterContent" class="footer-content">
      <!-- 自定义 Footer 内容区域 -->
      <slot />
    </div>
  </footer>
</template>

<script setup lang="ts">
import { computed, h } from 'vue'
import { storeToRefs } from 'pinia'
import { useLayoutStore } from '@/store'
import StatusBar from '@/components/common/StatusBar.vue'
import { getStatusBarState } from '@/composables/useStatusBar'

/**
 * AppFooter 组件
 * 页面底部布局组件，支持自定义内容和 StatusBar 状态栏显示
 */
const layoutStore = useLayoutStore()

/**
 * 从 store 获取状态
 */
const { isFooterVisible, isStatusBarVisible, hasStatusBarContent } = storeToRefs(layoutStore)

/**
 * 获取 StatusBar 全局状态
 */
const { content: statusBarContent } = getStatusBarState()

/**
 * 是否显示 StatusBar
 */
const showStatusBar = computed(() => isStatusBarVisible.value && hasStatusBarContent.value)

/**
 * 是否显示 Footer 内容
 */
const showFooterContent = computed(() => isFooterVisible.value)

/**
 * 是否应该显示整个 Footer 组件
 * 当 StatusBar 有内容或 Footer 内容可见时显示
 */
const shouldShowFooter = computed(() => showStatusBar.value || showFooterContent.value)

/**
 * 是否为 VNode 类型内容
 */
const isVNodeContent = computed(() => {
  const content = statusBarContent.value
  return typeof content === 'object' && content !== null
})

/**
 * StatusBar 内容（VNode 形式）
 */
const statusBarContentVNode = computed(() => {
  const content = statusBarContent.value
  if (typeof content === 'object' && content !== null) {
    // 如果是 VNode 或 VNode 数组，包装在 div 中
    return {
      render() {
        return h('div', {}, content)
      }
    }
  }
  return null
})

/**
 * 处理 StatusBar 内容变化
 * @param hasContent 是否有内容
 */
function handleStatusBarContentChange(hasContent: boolean): void {
  layoutStore.setStatusBarContent(hasContent)
}
</script>

<style scoped>
.app-footer {
  display: flex;
  flex-direction: column;
  width: var(--footer-compact-width, 100vw);
  margin: 0 var(--footer-margin, 0);
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
  overflow: hidden;
}

/* Footer 主内容 */
.footer-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: var(--footer-height);
  padding: 0 24px;
}

/* 有 StatusBar 时的样式 */
.app-footer.has-status-bar .footer-content {
  border-top: 1px solid var(--border-color-base);
}

/* 平板端适配 */
@media screen and (min-width: 768px) and (max-width: 1024px) {
  .footer-content {
    padding: 0 16px;
  }
}

/* 移动端适配 */
@media screen and (max-width: 767px) {
  .footer-content {
    flex-direction: column;
    justify-content: center;
    gap: 8px;
    height: auto;
    padding: 12px;
    text-align: center;
  }
}
</style>
