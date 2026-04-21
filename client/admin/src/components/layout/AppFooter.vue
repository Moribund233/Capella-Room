<template>
  <footer class="app-footer">
    <div class="footer-left">
      <span class="copyright">© 2026 SeredeliUI. All rights reserved.</span>
    </div>
    <div class="footer-center">
      <nav class="footer-links">
        <a href="#" class="footer-link">关于我们</a>
        <span class="divider">|</span>
        <a href="#" class="footer-link">隐私政策</a>
        <span class="divider">|</span>
        <a href="#" class="footer-link">使用条款</a>
        <span class="divider">|</span>
        <a href="#" class="footer-link">帮助中心</a>
      </nav>
    </div>
    <div class="footer-right">
      <span class="version">v{{ version }}</span>
    </div>
  </footer>
</template>

<script setup lang="ts">
import { useAppConfig } from '@/composables'

/**
 * AppFooter 组件
 * 页面底部布局组件，显示版权信息、链接和版本号
 */
const { version } = useAppConfig()
</script>

<style scoped>
.app-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
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

/* 左侧区域 */
.footer-left {
  display: flex;
  align-items: center;
}

.copyright {
  white-space: nowrap;
}

/* 中间区域 */
.footer-center {
  display: flex;
  align-items: center;
  justify-content: center;
  flex: 1;
  padding: 0 16px;
}

.footer-links {
  display: flex;
  align-items: center;
  gap: 12px;
}

.footer-link {
  color: var(--text-tertiary);
  text-decoration: none;
  transition: var(--transition-fast);
  white-space: nowrap;
}

.footer-link:hover {
  color: var(--color-primary);
}

.divider {
  color: var(--border-color-base);
}

/* 右侧区域 */
.footer-right {
  display: flex;
  align-items: center;
}

.version {
  padding: 2px 8px;
  background: var(--bg-layout);
  border-radius: 4px;
  font-family: monospace;
}

/* 平板端适配 */
@media screen and (min-width: 768px) and (max-width: 1024px) {
  .app-footer {
    padding: 0 16px;
  }

  .footer-center {
    display: none;
  }
}

/* 移动端适配 */
@media screen and (max-width: 767px) {
  .app-footer {
    flex-direction: column;
    justify-content: center;
    gap: 8px;
    height: auto;
    padding: 12px;
    text-align: center;
  }

  .footer-left,
  .footer-center,
  .footer-right {
    width: 100%;
    justify-content: center;
  }

  .footer-center {
    padding: 0;
    order: -1;
  }

  .footer-links {
    flex-wrap: wrap;
    justify-content: center;
  }
}
</style>
