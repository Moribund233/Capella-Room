<script setup lang="ts">
/**
 * 文档管理页面
 * 文档上传、管理和查看
 */
</script>

<template>
  <div class="documents-page">
    <div class="page-header">
      <h2 class="page-title">文档管理</h2>
      <p class="page-desc">管理系统文档和资源文件</p>
    </div>

    <div class="card">
      <div class="card-body">
        <p class="placeholder-text">文档列表将在这里显示</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.documents-page {
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
