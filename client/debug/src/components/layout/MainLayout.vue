<script setup lang="ts">
import Sidebar from './Sidebar.vue'
</script>

<template>
  <n-message-provider>
    <n-dialog-provider>
      <div class="main-layout">
        <!-- Sidebar 组件 -->
        <Sidebar />

        <!-- 主内容区域 -->
        <main class="main-content">
          <slot />
        </main>
      </div>
    </n-dialog-provider>
  </n-message-provider>
</template>

<style scoped>
.main-layout {
  min-height: 100vh;
  min-height: 100dvh; /* 动态视口高度 */
  width: 100%;
  display: flex;
}

/* 桌面端：主内容默认有左边距 */
.main-content {
  flex: 1;
  margin-left: 240px;
  min-height: 100vh;
  min-height: 100dvh;
  width: calc(100% - 240px);
  background-color: var(--bg-primary);
  padding: var(--space-lg);
  transition: margin-left 0.3s ease, width 0.3s ease;
  overflow-x: hidden; /* 防止内容溢出 */
}

/* 移动端：主内容无左边距 */
@media screen and (max-width: 767px) {
  .main-layout {
    flex-direction: column;
    width: 100%;
  }

  .main-content {
    margin-left: 0;
    width: 100%;
    padding: var(--space-md);
    padding-top: calc(var(--space-md) + 56px); /* 为右上角按钮留空间 */
    min-height: 100vh;
    min-height: 100dvh;
  }
}
</style>
