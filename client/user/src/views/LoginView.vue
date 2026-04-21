<template>
  <div class="login-view">
    <n-card class="login-card" :bordered="false">
      <n-tabs v-model:value="activeTab" type="line" animated class="login-tabs">
        <n-tab-pane name="login" tab="登录">
          <LoginForm />
        </n-tab-pane>
        <n-tab-pane name="register" tab="注册">
          <RegisterForm @switch-to-login="handleSwitchToLogin" />
        </n-tab-pane>
      </n-tabs>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { NCard, NTabs, NTabPane } from 'naive-ui'
import LoginForm from '@/components/login/LoginForm.vue'
import RegisterForm from '@/components/login/RegisterForm.vue'

const activeTab = ref('login')

const handleSwitchToLogin = () => {
  activeTab.value = 'login'
}
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

.login-card {
  width: 100%;
  max-width: 480px;
  border-radius: 12px;
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.1);
}

.login-tabs :deep(.n-tabs-nav) {
  padding: 0 24px;
}

.login-tabs :deep(.n-tab-pane) {
  padding: 0;
}
</style>
