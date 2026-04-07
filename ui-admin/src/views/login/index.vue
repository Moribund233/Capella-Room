<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { User, Lock } from '@element-plus/icons-vue'
import { useUserStore } from '@/stores/user'
import { useThemeStore } from '@/stores/theme'

/**
 * 管理员登录页面
 * 仅允许管理员角色(admin, super_admin)登录
 */

const router = useRouter()
const userStore = useUserStore()
const themeStore = useThemeStore()

/** 登录表单数据 */
const loginForm = ref({
  email: '',
  password: '',
  remember: false,
})

/**
 * 处理登录
 * 调用 UserStore 的 login 方法进行认证
 */
const handleLogin = async () => {
  // 表单验证
  if (!loginForm.value.email || !loginForm.value.password) {
    return
  }

  const success = await userStore.login(
    {
      email: loginForm.value.email,
      password: loginForm.value.password,
    },
    loginForm.value.remember
  )

  if (success) {
    // 登录成功，跳转到仪表盘
    router.push('/dashboard')
  }
}

/**
 * 页面挂载时初始化
 * - 初始化主题
 * - 初始化 UserStore
 * - 如果已登录则自动跳转到仪表盘
 */
onMounted(() => {
  // 初始化主题
  themeStore.initTheme()

  // 初始化用户状态
  userStore.initialize()

  // 如果已登录，直接跳转
  if (userStore.isLoggedIn) {
    router.push('/dashboard')
  }
})
</script>

<template>
  <div class="login-page">
    <div class="login-box">
      <!-- Logo -->
      <div class="login-header">
        <img src="/admin.svg" alt="Seredeli" class="logo-img" />
        <h1 class="title">Seredeli</h1>
        <p class="subtitle">管理后台登录</p>
      </div>

      <!-- 错误提示 -->
      <div v-if="userStore.error" class="error-message">
        {{ userStore.error }}
      </div>

      <!-- 登录表单 -->
      <form class="login-form" @submit.prevent="handleLogin">
        <div class="form-item">
          <div class="input-wrapper">
            <User class="input-icon" />
            <input
              v-model="loginForm.email"
              type="email"
              class="form-input"
              placeholder="管理员邮箱"
              required
              :disabled="userStore.isLoading"
              @input="userStore.clearError"
            />
          </div>
        </div>

        <div class="form-item">
          <div class="input-wrapper">
            <Lock class="input-icon" />
            <input
              v-model="loginForm.password"
              type="password"
              class="form-input"
              placeholder="密码"
              required
              :disabled="userStore.isLoading"
              @input="userStore.clearError"
            />
          </div>
        </div>

        <div class="form-options">
          <label class="remember">
            <input v-model="loginForm.remember" type="checkbox" :disabled="userStore.isLoading" />
            <span>记住我</span>
          </label>
        </div>

        <button type="submit" class="login-btn" :disabled="userStore.isLoading">
          <span v-if="userStore.isLoading" class="spinner"></span>
          <span v-else>登录</span>
        </button>
      </form>

      <!-- 提示信息 -->
      <div class="login-tips">
        <p>仅管理员可登录此系统</p>
      </div>
    </div>

    <!-- 页脚 -->
    <footer class="login-footer">
      <p>© {{ new Date().getFullYear() }} Seredeli. All rights reserved.</p>
    </footer>
  </div>
</template>

<style scoped>
.login-page {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  background-color: var(--bg-primary);
  padding: var(--spacing-6);
}

.login-box {
  width: 100%;
  max-width: 400px;
  padding: var(--spacing-10);
  background-color: var(--bg-card);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-xl);
  box-shadow: var(--shadow-lg);
}

/* 头部 */
.login-header {
  text-align: center;
  margin-bottom: var(--spacing-8);
}

.logo-img {
  width: 32px;
  height: 32px;
  margin: 0 auto var(--spacing-4);
  display: block;
  border-radius: var(--radius-lg);
  object-fit: contain;
  filter: var(--logo-filter);
  transition: filter var(--transition-fast);
}

.title {
  font-size: var(--font-size-2xl);
  font-weight: var(--font-weight-bold);
  color: var(--text-primary);
  margin-bottom: var(--spacing-2);
}

.subtitle {
  font-size: var(--font-size-base);
  color: var(--text-secondary);
}

/* 错误消息 */
.error-message {
  margin-bottom: var(--spacing-4);
  padding: var(--spacing-3) var(--spacing-4);
  background-color: var(--danger-alpha);
  color: var(--danger);
  border: 1px solid var(--danger);
  border-radius: var(--radius-md);
  font-size: var(--font-size-sm);
}

/* 表单 */
.login-form {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-4);
}

.form-item {
  width: 100%;
}

.input-wrapper {
  position: relative;
  display: flex;
  align-items: center;
}

.input-icon {
  position: absolute;
  left: var(--spacing-3);
  width: 20px;
  height: 20px;
  color: var(--text-tertiary);
}

.form-input {
  width: 100%;
  height: 44px;
  padding: 0 var(--spacing-3) 0 calc(var(--spacing-3) + 28px);
  font-size: var(--font-size-base);
  color: var(--text-primary);
  background-color: var(--bg-input);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-md);
  transition: border-color var(--transition-fast), box-shadow var(--transition-fast);
}

.form-input:focus {
  outline: none;
  border-color: var(--primary);
  box-shadow: 0 0 0 3px var(--primary-alpha);
}

.form-input::placeholder {
  color: var(--text-tertiary);
}

.form-input:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

/* 表单选项 */
.form-options {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: var(--font-size-sm);
}

.remember {
  display: flex;
  align-items: center;
  gap: var(--spacing-2);
  color: var(--text-secondary);
  cursor: pointer;
}

.remember input[type="checkbox"] {
  width: 16px;
  height: 16px;
  accent-color: var(--primary);
}

.remember input[type="checkbox"]:disabled {
  cursor: not-allowed;
}

/* 登录按钮 */
.login-btn {
  width: 100%;
  height: 44px;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: var(--primary);
  color: var(--primary-contrast);
  font-size: var(--font-size-base);
  font-weight: var(--font-weight-medium);
  border: none;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: background-color var(--transition-fast);
}

.login-btn:hover:not(:disabled) {
  background-color: var(--primary-hover);
}

.login-btn:disabled {
  opacity: 0.7;
  cursor: not-allowed;
}

/* 加载动画 */
.spinner {
  width: 20px;
  height: 20px;
  border: 2px solid var(--primary-contrast);
  border-top-color: transparent;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

/* 提示信息 */
.login-tips {
  margin-top: var(--spacing-6);
  text-align: center;
  font-size: var(--font-size-sm);
  color: var(--text-tertiary);
}

/* 页脚 */
.login-footer {
  margin-top: var(--spacing-8);
  text-align: center;
  font-size: var(--font-size-sm);
  color: var(--text-tertiary);
}

/* 响应式 */
@media (max-width: 480px) {
  .login-box {
    padding: var(--spacing-6);
  }

  .logo-img {
    width: 28px;
    height: 28px;
  }

  .title {
    font-size: var(--font-size-xl);
  }
}
</style>
