<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { User, Lock, Message, Key, ArrowLeft } from '@element-plus/icons-vue'
import { Globe, CodeSquare } from '@lucide/vue'
import { useAuthStore } from '@/stores/auth'
import { ROUTE_PATHS } from '@/constants'

const router = useRouter()
const route = useRoute()
const { t } = useI18n()
const authStore = useAuthStore()
const loading = ref(false)
const codeSending = ref(false)
const codeSent = ref(false)
const countdown = ref(0)
let timer: ReturnType<typeof setInterval> | null = null

type LoginMode = 'password' | 'code'
const loginMode = ref<LoginMode>('password')

const loginForm = ref({
  email: '',
  password: '',
  code: '',
})

const codeBtnText = computed(() => {
  if (codeSending.value) return '发送中...'
  if (countdown.value > 0) return `${countdown.value}s`
  return '发送验证码'
})

function startCountdown(sec: number) {
  countdown.value = sec
  timer = setInterval(() => {
    countdown.value--
    if (countdown.value <= 0) {
      if (timer) clearInterval(timer)
      timer = null
    }
  }, 1000)
}

async function handleSendCode() {
  if (!loginForm.value.email) {
    ElMessage.warning('请先输入邮箱')
    return
  }
  codeSending.value = true
  try {
    await authStore.loginSendCode(loginForm.value.email)
    codeSent.value = true
    ElMessage.success('验证码已发送到邮箱')
    startCountdown(60)
  } catch {
    ElMessage.error('发送验证码失败')
  } finally {
    codeSending.value = false
  }
}

async function handleLogin() {
  if (!loginForm.value.email) {
    ElMessage.warning(t('auth.validation.emailRequired'))
    return
  }

  if (loginMode.value === 'password') {
    if (!loginForm.value.password) {
      ElMessage.warning(t('auth.validation.passwordRequired'))
      return
    }
  } else {
    if (!loginForm.value.code) {
      ElMessage.warning('请输入验证码')
      return
    }
  }

  loading.value = true
  try {
    let result
    if (loginMode.value === 'password') {
      result = await authStore.login({
        email: loginForm.value.email,
        password: loginForm.value.password,
      })
    } else {
      result = await authStore.loginWithCode(loginForm.value.email, loginForm.value.code)
    }

    if (result.data) {
      ElMessage.success(t('auth.loginSuccess'))
      const redirect = route.query.redirect as string
      router.push(redirect || '/app')
    } else {
      ElMessage.error(result.message || t('auth.loginFailed'))
    }
  } catch (error: unknown) {
    console.error('[Login] Error:', error)
    const err = error as { response?: { data?: { message?: string; error?: string } }; message?: string }
    const errorMessage = err?.response?.data?.message
      || err?.response?.data?.error
      || err?.message
      || t('auth.loginFailed')
    ElMessage.error({ message: errorMessage, duration: 3000, showClose: true })
  } finally {
    loading.value = false
  }
}

function goToRegister() {
  router.push(ROUTE_PATHS.REGISTER)
}
</script>

<template>
  <div class="login-page">
    <div class="auth-card">
      <a href="/" class="back-link">
        <el-icon><ArrowLeft /></el-icon>
        {{ t('common.back') }}
      </a>

      <div class="auth-logo">
        <img src="/favicon.svg" alt="Logo" class="auth-logo-img" />
        <span>{{ t('common.appName') }}</span>
      </div>

      <!-- 登录表单 -->
      <div class="tab-content">
        <!-- 登录方式切换 -->
        <div class="mode-toggle">
          <button
            :class="['mode-btn', { active: loginMode === 'password' }]"
            @click="loginMode = 'password'"
          >
            <el-icon><Lock /></el-icon> 密码登录
          </button>
          <button
            :class="['mode-btn', { active: loginMode === 'code' }]"
            @click="loginMode = 'code'"
          >
            <el-icon><Key /></el-icon> 验证码登录
          </button>
        </div>

        <!-- 邮箱 -->
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

        <!-- 验证码登录模式 -->
        <template v-if="loginMode === 'code'">
          <div class="form-field">
            <label>验证码</label>
            <div class="code-row">
              <el-input
                v-model="loginForm.code"
                placeholder="输入 6 位验证码"
                size="large"
                maxlength="6"
                :prefix-icon="Key"
              />
              <el-button
                type="primary"
                size="large"
                :loading="codeSending"
                :disabled="countdown > 0 || !loginForm.email"
                class="code-btn"
                @click="handleSendCode"
              >
                {{ codeBtnText }}
              </el-button>
            </div>
            <p v-if="codeSent" class="spam-tip">若未收到邮件，请检查垃圾箱</p>
          </div>
        </template>

        <!-- 密码登录模式 -->
        <template v-if="loginMode === 'password'">
          <div class="form-field">
            <div class="field-header">
              <label>{{ t('auth.password') }}</label>
              <RouterLink to="/forgot-password" class="forgot-link">{{ t('auth.forgotPassword') }}</RouterLink>
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
        </template>

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
          {{ t('auth.noAccount') }}
          <a href="#" @click.prevent="goToRegister">{{ t('auth.createAccount') }}</a>
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

.mode-toggle {
  display: flex;
  gap: 8px;
  margin-bottom: 16px;
  background: var(--bg);
  padding: 3px;
  border-radius: var(--radius);
}

.mode-btn {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 8px;
  border: none;
  background: transparent;
  color: var(--muted);
  font-size: 13px;
  cursor: pointer;
  border-radius: 6px;
  transition: all 0.2s;
}

.mode-btn.active {
  background: var(--surface);
  color: var(--fg);
  box-shadow: 0 1px 3px rgba(0,0,0,0.1);
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

.code-row {
  display: flex;
  width: 100%;
  gap: 8px;
}

.code-row .el-input {
  flex: 1;
}

.code-btn {
  flex-shrink: 0;
}

.spam-tip {
  margin: 8px 0 0;
  font-size: 12px;
  color: var(--el-color-warning);
  line-height: 1.4;
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
