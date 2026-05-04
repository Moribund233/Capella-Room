<template>
  <div class="debug-view">
    <h1 class="page-title">调试工具</h1>
    <p class="page-description">开发和测试辅助工具集</p>

    <!-- 子路由视图 -->
    <div class="content-area">
      <RouterView />
    </div>
  </div>
</template>

<script setup lang="ts">
import { RouterView } from 'vue-router'
</script>

<style scoped>
.debug-view {
  padding: 24px;
  min-height: 100%;
}

.page-title {
  margin-bottom: 8px;
  color: var(--text-primary);
  font-size: 24px;
  font-weight: 600;
}

.page-description {
  color: var(--text-secondary);
  margin-bottom: 24px;
  font-size: 14px;
}

.content-area {
  background: var(--bg-container);
  border-radius: 12px;
  padding: 24px;
  min-height: 400px;
  /* 确保子组件可以正确计算高度 */
  height: calc(100vh - 200px);
  overflow: hidden;
}

/* 确保 RouterView 内的内容占满高度 */
.content-area > :deep(*) {
  height: 100%;
}
</style>
