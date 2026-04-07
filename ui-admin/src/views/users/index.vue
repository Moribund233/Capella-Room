<script setup lang="ts">
/**
 * 用户管理页面
 * 用户列表和管理功能
 */
</script>

<template>
  <div class="users-page">
    <div class="page-header">
      <h2 class="page-title">用户管理</h2>
      <p class="page-desc">管理系统用户账号和权限</p>
    </div>

    <div class="card">
      <div class="card-body">
        <p class="placeholder-text">用户列表将在这里显示</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.users-page {
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
