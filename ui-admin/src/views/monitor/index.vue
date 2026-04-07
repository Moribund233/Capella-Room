<script setup lang="ts">
/**
 * 系统监控页面
 * 系统运行状态和性能监控
 */
</script>

<template>
  <div class="monitor-page">
    <div class="page-header">
      <h2 class="page-title">系统监控</h2>
      <p class="page-desc">实时监控系统运行状态</p>
    </div>

    <div class="card">
      <div class="card-body">
        <p class="placeholder-text">监控图表将在这里显示</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.monitor-page {
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
