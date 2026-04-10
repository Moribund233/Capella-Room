<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { TestTube, User, Lock, Mail, ArrowRight, CheckCircle } from 'lucide-vue-next'
import type { TabsInst } from 'naive-ui'

const router = useRouter()
const authStore = useAuthStore()

// 当前激活的 Tab（'login' 或 'register'）
const activeTab = ref('login')

// 登录表单数据
const loginForm = ref({
  email: '',
  password: '',
})

// 注册表单数据
const registerForm = ref({
  username: '',
  email: '',
  password: '',
  confirmPassword: '',
})

// 表单验证错误
const loginErrors = ref({
  email: '',
  password: '',
})

const registerErrors = ref({
  username: '',
  email: '',
  password: '',
  confirmPassword: '',
})

// 登录成功提示
const showSuccessMessage = ref(false)
const successMessage = ref('')

// 验证登录表单
const validateLoginForm = (): boolean => {
  let isValid = true
  loginErrors.value = { email: '', password: '' }

  if (!loginForm.value.email.trim()) {
    loginErrors.value.email = '请输入邮箱'
    isValid = false
  } else if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(loginForm.value.email)) {
    loginErrors.value.email = '请输入有效的邮箱地址'
    isValid = false
  }

  if (!loginForm.value.password) {
    loginErrors.value.password = '请输入密码'
    isValid = false
  } else if (loginForm.value.password.length < 6) {
    loginErrors.value.password = '密码至少6个字符'
    isValid = false
  }

  return isValid
}

// 验证注册表单
const validateRegisterForm = (): boolean => {
  let isValid = true
  registerErrors.value = { username: '', email: '', password: '', confirmPassword: '' }

  if (!registerForm.value.username.trim()) {
    registerErrors.value.username = '请输入用户名'
    isValid = false
  } else if (registerForm.value.username.length < 3) {
    registerErrors.value.username = '用户名至少3个字符'
    isValid = false
  }

  if (!registerForm.value.email.trim()) {
    registerErrors.value.email = '请输入邮箱'
    isValid = false
  } else if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(registerForm.value.email)) {
    registerErrors.value.email = '请输入有效的邮箱地址'
    isValid = false
  }

  if (!registerForm.value.password) {
    registerErrors.value.password = '请输入密码'
    isValid = false
  } else if (registerForm.value.password.length < 6) {
    registerErrors.value.password = '密码至少6个字符'
    isValid = false
  }

  if (registerForm.value.password !== registerForm.value.confirmPassword) {
    registerErrors.value.confirmPassword = '两次输入的密码不一致'
    isValid = false
  }

  return isValid
}

// 处理登录
const handleLogin = async () => {
  if (!validateLoginForm()) return

  const success = await authStore.login({
    email: loginForm.value.email,
    password: loginForm.value.password,
  })

  if (success && authStore.user) {
    successMessage.value = `欢迎回来，${authStore.user.username}（${authStore.roleText}）`
    showSuccessMessage.value = true
    
    // 延迟跳转到首页
    setTimeout(() => {
      router.push('/')
    }, 1500)
  }
}

// 处理注册
const handleRegister = async () => {
  if (!validateRegisterForm()) return

  const success = await authStore.register({
    username: registerForm.value.username,
    email: registerForm.value.email,
    password: registerForm.value.password,
  })

  if (success && authStore.user) {
    successMessage.value = `注册成功，欢迎 ${authStore.user.username}（${authStore.roleText}）`
    showSuccessMessage.value = true
    
    // 延迟跳转到首页
    setTimeout(() => {
      router.push('/')
    }, 1500)
  }
}

// 处理 Tab 切换
const handleTabChange = (tab: string) => {
  activeTab.value = tab
  authStore.clearError()
}
</script>

<template>
  <div class="login-page">
    <!-- 成功提示 -->
    <n-modal
      v-model:show="showSuccessMessage"
      preset="dialog"
      title="登录成功"
      type="success"
      :closable="false"
      :mask-closable="false"
    >
      <div class="success-content">
        <CheckCircle class="success-icon" />
        <p class="success-message">{{ successMessage }}</p>
        <p class="success-hint">正在跳转到首页...</p>
      </div>
    </n-modal>

    <div class="login-container">
      <!-- 品牌 Logo -->
      <div class="brand-header">
        <div class="brand-logo">
          <TestTube class="icon-xl" />
        </div>
        <h1 class="brand-title">Seredeli Room</h1>
        <p class="brand-subtitle">调试客户端</p>
      </div>

      <!-- 登录/注册表单卡片 -->
      <n-card class="login-card" :bordered="false">
        <!-- Tab 切换 -->
        <n-tabs
          v-model:value="activeTab"
          type="line"
          animated
          @update:value="handleTabChange"
          class="login-tabs"
        >
          <n-tab-pane name="login" tab="登录">
            <!-- 错误提示 -->
            <n-alert
              v-if="authStore.error && activeTab === 'login'"
              type="error"
              closable
              @close="authStore.clearError"
              style="margin-bottom: var(--space-md)"
            >
              {{ authStore.error }}
            </n-alert>

            <form @submit.prevent="handleLogin">
              <!-- 邮箱 -->
              <div class="form-item">
                <n-input
                  v-model:value="loginForm.email"
                  placeholder="邮箱"
                  size="large"
                  :status="loginErrors.email ? 'error' : undefined"
                >
                  <template #prefix>
                    <Mail class="icon-sm" />
                  </template>
                </n-input>
                <span v-if="loginErrors.email" class="form-error">{{ loginErrors.email }}</span>
              </div>

              <!-- 密码 -->
              <div class="form-item">
                <n-input
                  v-model:value="loginForm.password"
                  type="password"
                  placeholder="密码"
                  size="large"
                  show-password-on="click"
                  :status="loginErrors.password ? 'error' : undefined"
                >
                  <template #prefix>
                    <Lock class="icon-sm" />
                  </template>
                </n-input>
                <span v-if="loginErrors.password" class="form-error">{{ loginErrors.password }}</span>
              </div>

              <!-- 登录按钮 -->
              <n-button
                type="primary"
                size="large"
                block
                :loading="authStore.loading"
                @click="handleLogin"
              >
                {{ authStore.loading ? '登录中...' : '登录' }}
                <template #icon>
                  <ArrowRight class="icon-sm" />
                </template>
              </n-button>
            </form>
          </n-tab-pane>

          <n-tab-pane name="register" tab="注册">
            <!-- 错误提示 -->
            <n-alert
              v-if="authStore.error && activeTab === 'register'"
              type="error"
              closable
              @close="authStore.clearError"
              style="margin-bottom: var(--space-md)"
            >
              {{ authStore.error }}
            </n-alert>

            <form @submit.prevent="handleRegister">
              <!-- 用户名 -->
              <div class="form-item">
                <n-input
                  v-model:value="registerForm.username"
                  placeholder="用户名"
                  size="large"
                  :status="registerErrors.username ? 'error' : undefined"
                >
                  <template #prefix>
                    <User class="icon-sm" />
                  </template>
                </n-input>
                <span v-if="registerErrors.username" class="form-error">{{ registerErrors.username }}</span>
              </div>

              <!-- 邮箱 -->
              <div class="form-item">
                <n-input
                  v-model:value="registerForm.email"
                  placeholder="邮箱"
                  size="large"
                  :status="registerErrors.email ? 'error' : undefined"
                >
                  <template #prefix>
                    <Mail class="icon-sm" />
                  </template>
                </n-input>
                <span v-if="registerErrors.email" class="form-error">{{ registerErrors.email }}</span>
              </div>

              <!-- 密码 -->
              <div class="form-item">
                <n-input
                  v-model:value="registerForm.password"
                  type="password"
                  placeholder="密码"
                  size="large"
                  show-password-on="click"
                  :status="registerErrors.password ? 'error' : undefined"
                >
                  <template #prefix>
                    <Lock class="icon-sm" />
                  </template>
                </n-input>
                <span v-if="registerErrors.password" class="form-error">{{ registerErrors.password }}</span>
              </div>

              <!-- 确认密码 -->
              <div class="form-item">
                <n-input
                  v-model:value="registerForm.confirmPassword"
                  type="password"
                  placeholder="确认密码"
                  size="large"
                  show-password-on="click"
                  :status="registerErrors.confirmPassword ? 'error' : undefined"
                >
                  <template #prefix>
                    <Lock class="icon-sm" />
                  </template>
                </n-input>
                <span v-if="registerErrors.confirmPassword" class="form-error">{{ registerErrors.confirmPassword }}</span>
              </div>

              <!-- 注册按钮 -->
              <n-button
                type="primary"
                size="large"
                block
                :loading="authStore.loading"
                @click="handleRegister"
              >
                {{ authStore.loading ? '注册中...' : '注册' }}
                <template #icon>
                  <ArrowRight class="icon-sm" />
                </template>
              </n-button>
            </form>
          </n-tab-pane>
        </n-tabs>
      </n-card>

      <!-- 底部信息 -->
      <p class="footer-text">
        Seredeli Room 调试客户端 · 实时聊天室后端服务调试工具
      </p>
    </div>
  </div>
</template>

<style scoped>
.login-page {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, var(--bg-primary) 0%, var(--bg-secondary) 100%);
  padding: var(--space-lg);
}

.login-container {
  width: 100%;
  max-width: 420px;
  display: flex;
  flex-direction: column;
  align-items: center;
}

/* 品牌头部 */
.brand-header {
  text-align: center;
  margin-bottom: var(--space-xl);
}

.brand-logo {
  width: 72px;
  height: 72px;
  background: var(--gradient-primary);
  border-radius: var(--radius-lg);
  display: flex;
  align-items: center;
  justify-content: center;
  margin: 0 auto var(--space-md);
  color: var(--text-white);
  box-shadow: var(--shadow-md);
}

.brand-title {
  font-size: 28px;
  font-weight: 700;
  color: var(--primary);
  margin-bottom: var(--space-xs);
}

.brand-subtitle {
  font-size: 16px;
  color: var(--text-secondary);
}

/* 登录卡片 */
.login-card {
  width: 100%;
  background: var(--bg-white);
  border-radius: var(--radius-xl);
  box-shadow: var(--shadow-lg);
  padding: var(--space-lg);
}

.login-tabs :deep(.n-tabs-nav) {
  margin-bottom: var(--space-lg);
}

.login-tabs :deep(.n-tab-pane) {
  padding: 0;
}

/* 表单样式 */
.form-item {
  margin-bottom: var(--space-md);
}

.form-error {
  display: block;
  font-size: 12px;
  color: var(--error);
  margin-top: var(--space-xs);
}

/* 成功提示 */
.success-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: var(--space-md);
}

.success-icon {
  width: 64px;
  height: 64px;
  color: var(--success);
  margin-bottom: var(--space-md);
}

.success-message {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: var(--space-sm);
  text-align: center;
}

.success-hint {
  font-size: 14px;
  color: var(--text-secondary);
}

/* 底部信息 */
.footer-text {
  margin-top: var(--space-xl);
  font-size: 13px;
  color: var(--text-muted);
  text-align: center;
}

/* 响应式 */
@media (max-width: 480px) {
  .login-page {
    padding: var(--space-md);
  }

  .brand-logo {
    width: 60px;
    height: 60px;
  }

  .brand-title {
    font-size: 24px;
  }

  .login-card {
    padding: var(--space-md);
  }
}
</style>
