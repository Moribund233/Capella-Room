<script setup lang="ts">
import { computed } from 'vue'

/**
 * 页脚组件
 * 显示版权信息和系统版本
 */

/** 定义props */
const props = defineProps<{
  collapsed?: boolean
}>()

/** 侧边栏宽度 */
const sidebarWidth = computed(() => (props.collapsed ? '64px' : '240px'))

/** 当前年份 */
const currentYear = new Date().getFullYear()

/** 系统版本 */
const version = 'v1.0.0'
</script>

<template>
  <footer class="footer">
    <div class="footer-left">
      <span class="copyright">© {{ currentYear }} Seredeli. All rights reserved.</span>
    </div>
    <div class="footer-right">
      <span class="version">{{ version }}</span>
    </div>
  </footer>
</template>

<style scoped>
.footer {
  position: fixed;
  right: 0;
  bottom: 0;
  left: v-bind(sidebarWidth);
  height: var(--footer-height);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 var(--spacing-6);
  background-color: var(--footer-bg);
  border-top: 1px solid var(--footer-border);
  transition: left var(--transition-normal);
  z-index: calc(var(--z-fixed) - 2);
}

.footer-left,
.footer-right {
  display: flex;
  align-items: center;
  gap: var(--spacing-4);
}

.copyright,
.version {
  font-size: var(--font-size-xs);
  color: var(--footer-text);
}

/* 响应式 */
@media (max-width: 1023px) {
  .footer {
    left: 0 !important;
  }
}

@media (max-width: 767px) {
  .footer {
    justify-content: center;
  }

  .footer-left {
    display: none;
  }
}
</style>
