<script setup lang="ts">
import { useRouter } from 'vue-router'

/**
 * 404错误页面
 * 页面不存在时显示
 */

const router = useRouter()

/**
 * 返回首页
 */
const goHome = () => {
  router.push('/')
}

/**
 * 返回上一页
 */
const goBack = () => {
  router.back()
}
</script>

<template>
  <div class="error-page">
    <div class="error-content">
      <div class="error-code">404</div>
      <h1 class="error-title">页面不存在</h1>
      <p class="error-desc">抱歉，您访问的页面已经丢失在虚拟空间中</p>
      <div class="error-actions">
        <button class="btn btn-primary" @click="goHome">返回首页</button>
        <button class="btn btn-outline" @click="goBack">返回上一页</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.error-page {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: var(--bg-primary);
  padding: var(--spacing-6);
}

.error-content {
  text-align: center;
  max-width: 500px;
}

.error-code {
  font-size: 120px;
  font-weight: var(--font-weight-bold);
  color: var(--primary);
  line-height: 1;
  margin-bottom: var(--spacing-4);
  opacity: 0.8;
}

.error-title {
  font-size: var(--font-size-2xl);
  font-weight: var(--font-weight-bold);
  color: var(--text-primary);
  margin-bottom: var(--spacing-3);
}

.error-desc {
  font-size: var(--font-size-base);
  color: var(--text-secondary);
  margin-bottom: var(--spacing-8);
}

.error-actions {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--spacing-4);
}

.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: var(--spacing-3) var(--spacing-6);
  font-size: var(--font-size-base);
  font-weight: var(--font-weight-medium);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-primary {
  background-color: var(--primary);
  color: var(--primary-contrast);
  border: none;
}

.btn-primary:hover {
  background-color: var(--primary-hover);
}

.btn-outline {
  background-color: transparent;
  color: var(--text-primary);
  border: 1px solid var(--border-primary);
}

.btn-outline:hover {
  background-color: var(--bg-secondary);
}

@media (max-width: 480px) {
  .error-code {
    font-size: 80px;
  }

  .error-actions {
    flex-direction: column;
  }

  .btn {
    width: 100%;
  }
}
</style>
