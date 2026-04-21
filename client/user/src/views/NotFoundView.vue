<template>
  <div class="not-found">
    <div class="not-found-content">
      <h1 class="error-code">404</h1>
      <h2 class="error-title">页面未找到</h2>
      <p class="error-message">抱歉，您访问的页面不存在或已被移除</p>
      <button class="back-btn" @click="goHome">返回首页</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'

const router = useRouter()

const goHome = () => {
  router.push({ name: 'Home' })
}
</script>

<style scoped>
.not-found {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 100vh;
  background: var(--bg-base);
}

.not-found-content {
  text-align: center;
  padding: 48px;
}

.error-code {
  font-size: 120px;
  font-weight: 700;
  color: var(--color-primary);
  margin: 0;
  line-height: 1;
}

.error-title {
  font-size: 24px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 16px 0 8px;
}

.error-message {
  font-size: 16px;
  color: var(--text-secondary);
  margin: 0 0 32px;
}

.back-btn {
  padding: 12px 32px;
  background: var(--color-primary);
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 16px;
  cursor: pointer;
  transition: var(--transition-fast);
}

.back-btn:hover {
  background: var(--color-primary-hover);
}
</style>
