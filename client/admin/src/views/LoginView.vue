<template>
  <div class="login-view">
    <div class="login-container">
      <!-- Login Card -->
      <n-card class="login-card" :bordered="false">
        <!-- Logo Section -->
        <div class="logo-section">
          <img src="/favicon.svg" alt="Seredeli Logo" class="logo-image" />
          <h1 class="logo-title">Seredeli Admin</h1>
        </div>

        <n-tabs v-model:value="activeTab" type="line" animated class="login-tabs">
          <n-tab-pane name="login" tab="登录">
            <LoginForm />
          </n-tab-pane>
          <n-tab-pane name="register" tab="注册">
            <RegisterForm @switch-to-login="activeTab = 'login'" />
          </n-tab-pane>
        </n-tabs>
      </n-card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { NCard, NTabs, NTabPane } from 'naive-ui'
import LoginForm from '@/components/login/LoginForm.vue'
import RegisterForm from '@/components/login/RegisterForm.vue'

const activeTab = ref('login')
</script>

<style scoped>
.login-view {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 100vh;
  background: var(--bg-base);
  padding: 20px;
}

.login-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 100%;
  max-width: 480px;
}

/* Login Card */
.login-card {
  width: 100%;
  border-radius: 12px;
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.1);
}

/* Logo Section */
.logo-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 32px 24px 16px;
  background: linear-gradient(135deg, var(--primary-color) 0%, var(--primary-color-hover) 100%);
  margin: -24px -24px 16px;
  border-radius: 12px 12px 0 0;
}

.logo-image {
  width: 64px;
  height: 64px;
  margin-bottom: 12px;
}

.logo-title {
  font-size: 24px;
  font-weight: 700;
  margin: 0 0 4px 0;
  letter-spacing: 1px;
  color: #ffffff;
}

.logo-subtitle {
  font-size: 14px;
  opacity: 0.9;
  margin: 0;
  color: #ffffff;
}

.login-tabs :deep(.n-tabs-nav) {
  padding: 0 24px;
}

.login-tabs :deep(.n-tab-pane) {
  padding: 0;
}

/* 响应式调整 */
@media (max-width: 480px) {
  .login-view {
    padding: 16px;
  }

  .logo-section {
    padding: 24px 16px 12px;
    margin: -16px -16px 12px;
  }

  .logo-image {
    width: 48px;
    height: 48px;
  }

  .logo-title {
    font-size: 20px;
  }
}
</style>
