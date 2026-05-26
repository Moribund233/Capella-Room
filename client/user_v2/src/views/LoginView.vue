<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { User, Lock, Message, ArrowLeft } from '@element-plus/icons-vue'

const router = useRouter()
const { t } = useI18n()
const activeTab = ref<'login' | 'register'>('login')

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
function handleLogin() {
  if (!loginForm.value.email) {
    ElMessage.warning(t('auth.validation.emailRequired'))
    return
  }
  if (!loginForm.value.password) {
    ElMessage.warning(t('auth.validation.passwordRequired'))
    return
  }
  // TODO: 调用登录API
  router.push('/app')
}

/**
 * 处理注册
 */
function handleRegister() {
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
  // TODO: 调用注册API
  router.push('/app')
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
        <div class="auth-logo-mark">W</div>
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
          />
        </div>
        <el-button type="primary" size="large" class="submit-btn" @click="handleLogin">
          {{ t('auth.signIn') }}
        </el-button>

        <el-divider>{{ t('auth.orContinueWith') }}</el-divider>

        <div class="social-row">
          <el-button size="large" class="social-btn">
            <svg class="social-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="2" y="3" width="20" height="14" rx="2"/>
              <path d="M22 5l-10 7L2 5"/>
            </svg>
            {{ t('auth.google') }}
          </el-button>
          <el-button size="large" class="social-btn">
            <svg class="social-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M16 8a6 6 0 016-6v6a6 6 0 01-6 6"/>
              <path d="M2 16a6 6 0 016-6v6a6 6 0 01-6 6"/>
              <circle cx="8" cy="8" r="2"/>
              <circle cx="8" cy="16" r="2"/>
              <circle cx="16" cy="8" r="2"/>
            </svg>
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
        <el-button type="primary" size="large" class="submit-btn" @click="handleRegister">
          {{ t('auth.createAccount') }}
        </el-button>

        <el-divider>{{ t('auth.orContinueWith') }}</el-divider>

        <div class="social-row">
          <el-button size="large" class="social-btn">
            <svg class="social-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="2" y="3" width="20" height="14" rx="2"/>
              <path d="M22 5l-10 7L2 5"/>
            </svg>
            {{ t('auth.google') }}
          </el-button>
          <el-button size="large" class="social-btn">
            <svg class="social-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M16 8a6 6 0 016-6v6a6 6 0 01-6 6"/>
              <path d="M2 16a6 6 0 016-6v6a6 6 0 01-6 6"/>
              <circle cx="8" cy="8" r="2"/>
              <circle cx="8" cy="16" r="2"/>
              <circle cx="16" cy="8" r="2"/>
            </svg>
            {{ t('auth.github') }}
          </el-button>
        </div>

        <div class="auth-footer">
          {{ t('auth.hasAccount') }} <a href="#" @click.prevent="switchTab('login')">{{ t('auth.signIn') }}</a>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">
.login-page {
  min-height: 100vh;
  display: grid;
  place-items: center;
  padding: 24px;
  background: var(--wave-bg);
}

.auth-card {
  width: 100%;
  max-width: 420px;
  background: var(--wave-surface);
  border: 1px solid var(--wave-border);
  border-radius: var(--wave-radius-lg);
  padding: 40px 36px;
  position: relative;
  overflow: hidden;

  &::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 3px;
    background: linear-gradient(90deg, var(--wave-accent), var(--wave-accent-pink));
  }
}

.back-link {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  color: var(--wave-muted);
  font-size: 13px;
  margin-bottom: 24px;
  text-decoration: none;

  &:hover {
    color: var(--wave-fg);
  }
}

.auth-logo {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  margin-bottom: 28px;
}

.auth-logo-mark {
  width: 36px;
  height: 36px;
  background: linear-gradient(135deg, var(--wave-accent), var(--wave-accent-pink));
  border-radius: 10px;
  display: grid;
  place-items: center;
  font-size: 17px;
  color: #fff;
  font-weight: 700;
}

.auth-logo span {
  font-family: var(--wave-font-display);
  font-size: 22px;
  font-weight: 600;
}

.auth-tabs {
  display: flex;
  background: var(--wave-bg);
  border-radius: var(--wave-radius);
  padding: 4px;
  margin-bottom: 28px;
}

.auth-tab {
  flex: 1;
  text-align: center;
  padding: 8px 0;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.15s, color 0.15s;
  border: none;
  background: none;
  color: var(--wave-muted);
  font-family: inherit;

  &.active {
    background: var(--wave-surface);
    color: var(--wave-fg);
  }

  &:hover:not(.active) {
    color: var(--wave-fg);
  }
}

.tab-content {
  animation: fadeIn 0.2s ease;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(4px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.form-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 18px;

  label {
    font-size: 13px;
    color: var(--wave-muted);
    font-weight: 500;
  }
}

.field-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.forgot-link {
  font-size: 13px;
  color: var(--wave-muted);
  text-decoration: none;

  &:hover {
    color: var(--wave-accent);
  }
}

.submit-btn {
  width: 100%;
  border-radius: var(--wave-radius-full);
  font-weight: 500;
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
  font-size: 13px;
  background: transparent;
  border-color: var(--wave-border);
  color: var(--wave-fg);

  &:hover {
    border-color: var(--wave-fg);
    color: var(--wave-fg);
  }
}

.social-icon {
  width: 18px;
  height: 18px;
}

.checkbox-field {
  margin-bottom: 18px;

  :deep(.el-checkbox__label) {
    font-size: 13px;
    color: var(--wave-muted);

    a {
      color: var(--wave-accent);
      text-decoration: none;

      &:hover {
        text-decoration: underline;
      }
    }
  }
}

.auth-footer {
  text-align: center;
  margin-top: 24px;
  font-size: 13px;
  color: var(--wave-muted);

  a {
    color: var(--wave-accent);
    text-decoration: none;

    &:hover {
      text-decoration: underline;
    }
  }
}

:deep(.el-divider__text) {
  background: var(--wave-surface);
  color: var(--wave-muted);
  font-size: 12px;
}

:deep(.el-input__wrapper) {
  background-color: var(--wave-bg);
}
</style>
