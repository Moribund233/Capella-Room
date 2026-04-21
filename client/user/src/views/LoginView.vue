<template>
  <div class="login-view">
    <div class="login-container">
      <!-- Logo Card -->
      <n-card class="logo-card" :bordered="false">
        <div class="logo-content">
          <div class="logo-icon">
            <MessageCircle :size="48" />
          </div>
          <h1 class="logo-title">Seredeli</h1>
          <p class="logo-subtitle">实时聊天室</p>
        </div>
      </n-card>

      <!-- Login Card -->
      <n-card class="login-card" :bordered="false">
        <n-tabs v-model:value="activeTab" type="line" animated class="login-tabs">
          <n-tab-pane name="login" tab="登录">
            <LoginForm @register="activeTab = 'register'" />
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
import { MessageCircle } from 'lucide-vue-next'
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
  gap: 24px;
  width: 100%;
  max-width: 480px;
}

/* Logo Card */
.logo-card {
  width: 100%;
  border-radius: 12px;
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.1);
  background: linear-gradient(135deg, var(--primary-color) 0%, var(--primary-color-hover) 100%);
}

.logo-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 32px 24px;
  color: #fff;
}

.logo-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 80px;
  height: 80px;
  border-radius: 20px;
  background: rgba(255, 255, 255, 0.2);
  margin-bottom: 16px;
  backdrop-filter: blur(10px);
}

.logo-title {
  font-size: 28px;
  font-weight: 700;
  margin: 0 0 8px 0;
  letter-spacing: 1px;
}

.logo-subtitle {
  font-size: 14px;
  opacity: 0.9;
  margin: 0;
}

/* Login Card */
.login-card {
  width: 100%;
  border-radius: 12px;
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.1);
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

  .login-container {
    gap: 16px;
  }

  .logo-content {
    padding: 24px 16px;
  }

  .logo-icon {
    width: 64px;
    height: 64px;
  }

  .logo-title {
    font-size: 24px;
  }
}
</style>
