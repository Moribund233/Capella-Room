<script setup lang="ts">
import { ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { User, Lock, Message, ArrowLeft } from '@element-plus/icons-vue'
import { Globe, CodeSquare } from '@lucide/vue'
import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const route = useRoute()
const { t } = useI18n()
const authStore = useAuthStore()
const activeTab = ref<'login' | 'register'>('login')
const loading = ref(false)

// 登录表单
const loginForm = ref({
  email: '',
  password: '',
})

// 注册表单
const registerForm = ref({
  email: '',
  username: '',
  password: '',
  agreeTerms: false,
})

/**
 * 处理登录
 */
async function handleLogin() {
  if (!loginForm.value.email) {
    ElMessage.warning(t('auth.validation.emailRequired'))
    return
  }
  if (!loginForm.value.password) {
    ElMessage.warning(t('auth.validation.passwordRequired'))
    return
  }

  loading.value = true
  try {
    const result = await authStore.login({
      email: loginForm.value.email,
      password: loginForm.value.password,
    })

    if (result.data) {
      ElMessage.success(t('auth.loginSuccess'))
      // 跳转到之前的页面或应用首页
      const redirect = route.query.redirect as string
      router.push(redirect || '/app')
    } else {
      ElMessage.error(result.message || t('auth.loginFailed'))
    }
  } catch (error: unknown) {
    // 显示后端返回的具体错误消息
    console.error('[Login] Error:', error)
    const err = error as { response?: { data?: { message?: string; error?: string } }; message?: string }
    const errorMessage = err?.response?.data?.message
      || err?.response?.data?.error
      || err?.message
      || t('auth.loginFailed')
    ElMessage.error({
      message: errorMessage,
      duration: 3000,
      showClose: true,
    })
  } finally {
    loading.value = false
  }
}

/**
 * 处理注册
 */
async function handleRegister() {
  if (!registerForm.value.email) {
    ElMessage.warning(t('auth.validation.emailRequired'))
    return
  }
  if (!registerForm.value.username) {
    ElMessage.warning(t('auth.validation.usernameRequired'))
    return
  }
  if (!registerForm.value.password) {
    ElMessage.warning(t('auth.validation.passwordRequired'))
    return
  }
  if (!registerForm.value.agreeTerms) {
    ElMessage.warning(t('auth.validation.termsRequired'))
    return
  }

  loading.value = true
  try {
    const result = await authStore.register({
      email: registerForm.value.email,
      username: registerForm.value.username,
      password: registerForm.value.password,
    })

    if (result.data) {
      ElMessage.success(t('auth.registerSuccess'))
      // 注册成功后切换到登录页
      activeTab.value = 'login'
      loginForm.value.email = registerForm.value.email
    } else {
      ElMessage.error(result.message || t('auth.registerFailed'))
    }
  } catch (error: unknown) {
    // 显示后端返回的具体错误消息
    console.error('[Register] Error:', error)
    const err = error as { response?: { data?: { message?: string; error?: string } }; message?: string }
    const errorMessage = err?.response?.data?.message
      || err?.response?.data?.error
      || err?.message
      || t('auth.registerFailed')
    ElMessage.error({
      message: errorMessage,
      duration: 3000,
      showClose: true,
    })
  } finally {
    loading.value = false
  }
}

/**
 * 切换标签页
 * @param tab - 目标标签页
 */
function switchTab(tab: 'login' | 'register') {
  activeTab.value = tab
}
</script>

<template>
  <div class="login-page">
    <div class="auth-card">
      <!-- 返回链接 -->
      <a href="/" class="back-link">
        <el-icon><ArrowLeft /></el-icon>
        {{ t('common.back') }}
      </a>

      <!-- Logo -->
      <div class="auth-logo">
        <img src="/favicon.svg" alt="Logo" class="auth-logo-img" />
        <span>{{ t('common.appName') }}</span>
      </div>

      <!-- 标签页 -->
      <div class="auth-tabs">
        <button
          class="auth-tab"
          :class="{ active: activeTab === 'login' }"
          @click="switchTab('login')"
        >
          {{ t('auth.signIn') }}
        </button>
        <button
          class="auth-tab"
          :class="{ active: activeTab === 'register' }"
          @click="switchTab('register')"
        >
          {{ t('auth.createAccount') }}
        </button>
      </div>

      <!-- 登录表单 -->
      <div v-show="activeTab === 'login'" class="tab-content">
        <div class="form-field">
          <label>{{ t('auth.email') }}</label>
          <el-input
            v-model="loginForm.email"
            placeholder="your@example.com"
            :prefix-icon="Message"
            size="large"
            @keyup.enter="handleLogin"
          />
        </div>
        <div class="form-field">
          <div class="field-header">
            <label>{{ t('auth.password') }}</label>
            <a href="#" class="forgot-link">{{ t('auth.forgotPassword') }}</a>
          </div>
          <el-input
            v-model="loginForm.password"
            type="password"
            :placeholder="t('auth.passwordPlaceholder')"
            :prefix-icon="Lock"
            size="large"
            show-password
            @keyup.enter="handleLogin"
          />
        </div>
        <el-button
          type="primary"
          size="large"
          class="submit-btn"
          :loading="loading"
          @click="handleLogin"
        >
          {{ t('auth.signIn') }}
        </el-button>

        <el-divider>{{ t('auth.orContinueWith') }}</el-divider>

        <div class="social-row">
          <el-button size="large" class="social-btn" disabled>
            <el-icon class="social-icon"><Globe /></el-icon>
            {{ t('auth.google') }}
          </el-button>
          <el-button size="large" class="social-btn" disabled>
            <el-icon class="social-icon"><CodeSquare /></el-icon>
            {{ t('auth.github') }}
          </el-button>
        </div>

        <div class="auth-footer">
          {{ t('auth.noAccount') }} <a href="#" @click.prevent="switchTab('register')">{{ t('auth.createAccount') }}</a>
        </div>
      </div>

      <!-- 注册表单 -->
      <div v-show="activeTab === 'register'" class="tab-content">
        <div class="form-field">
          <label>{{ t('auth.email') }}</label>
          <el-input
            v-model="registerForm.email"
            placeholder="your@example.com"
            :prefix-icon="Message"
            size="large"
          />
        </div>
        <div class="form-field">
          <label>{{ t('auth.username') }}</label>
          <el-input
            v-model="registerForm.username"
            :placeholder="t('auth.usernamePlaceholder')"
            :prefix-icon="User"
            size="large"
          />
        </div>
        <div class="form-field">
          <label>{{ t('auth.password') }}</label>
          <el-input
            v-model="registerForm.password"
            type="password"
            :placeholder="t('auth.passwordMinLength')"
            :prefix-icon="Lock"
            size="large"
            show-password
          />
        </div>
        <div class="checkbox-field">
          <el-checkbox v-model="registerForm.agreeTerms">
            {{ t('auth.agreeTerms') }}
          </el-checkbox>
        </div>
        <el-button
          type="primary"
          size="large"
          class="submit-btn"
          :loading="loading"
          @click="handleRegister"
        >
          {{ t('auth.createAccount') }}
        </el-button>

        <div class="auth-footer">
          {{ t('auth.hasAccount') }} <a href="#" @click.prevent="switchTab('login')">{{ t('auth.signIn') }}</a>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.login-page {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg);
  padding: 20px;
}

.auth-card {
  width: 100%;
  max-width: 420px;
  background: var(--surface);
  border-radius: var(--radius-lg);
  padding: 32px;
  border: 1px solid var(--border);
}

.back-link {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  color: var(--muted);
  text-decoration: none;
  font-size: 14px;
  margin-bottom: 24px;
}

.back-link:hover {
  color: var(--accent);
}

.auth-logo {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  margin-bottom: 32px;
}

.auth-logo-img {
  width: 40px;
  height: 40px;
  border-radius: 10px;
  object-fit: contain;
  filter: var(--logo-filter);
}

.auth-logo span {
  font-size: 24px;
  font-weight: 600;
  color: var(--fg);
}

.auth-tabs {
  display: flex;
  gap: 8px;
  margin-bottom: 24px;
  background: var(--bg);
  padding: 4px;
  border-radius: var(--radius);
  border: 1px solid var(--border);
}

.auth-tab {
  flex: 1;
  padding: 10px;
  border: none;
  background: transparent;
  color: var(--muted);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  border-radius: 6px;
  transition: all 0.2s;
}

.auth-tab.active {
  background: var(--surface);
  color: var(--fg);
}

.tab-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.form-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-field label {
  font-size: 14px;
  font-weight: 500;
  color: var(--fg);
}

.field-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.forgot-link {
  font-size: 13px;
  color: var(--accent);
  text-decoration: none;
}

.forgot-link:hover {
  text-decoration: underline;
}

.submit-btn {
  width: 100%;
  margin-top: 8px;
}

.checkbox-field {
  margin-top: 4px;
}

.social-row {
  display: flex;
  gap: 12px;
}

.social-btn {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.social-icon {
  width: 18px;
  height: 18px;
}

.auth-footer {
  text-align: center;
  font-size: 14px;
  color: var(--muted);
  margin-top: 16px;
}

.auth-footer a {
  color: var(--accent);
  text-decoration: none;
}

.auth-footer a:hover {
  text-decoration: underline;
}
</style>
