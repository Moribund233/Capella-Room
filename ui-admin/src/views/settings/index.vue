<script setup lang="ts">
/**
 * 系统设置页面
 * 系统配置和参数设置
 */
</script>

<template>
  <div class="settings-page">
    <div class="page-header">
      <h2 class="page-title">系统设置</h2>
      <p class="page-desc">配置系统参数和个性化选项</p>
    </div>

    <div class="card">
      <div class="card-body">
        <p class="placeholder-text">设置选项将在这里显示</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-page {
  max-width: 1400px;
}

.page-header {
  margin-bottom: var(--spacing-6);
}

.page-title {
  font-size: var(--font-size-2xl);
  font-weight: var(--font-weight-bold);
  color: var(--text-primary);
  margin-bottom: var(--spacing-2);
}

.page-desc {
  font-size: var(--font-size-base);
  color: var(--text-secondary);
}

.card {
  background-color: var(--bg-card);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-sm);
}

.card-body {
  padding: var(--spacing-6);
}

.placeholder-text {
  color: var(--text-tertiary);
  text-align: center;
  padding: var(--spacing-12) 0;
}
</style>
