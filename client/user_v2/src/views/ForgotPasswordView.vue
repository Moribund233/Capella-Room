<script setup lang="ts">
import { ref, reactive, computed } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { Message, Key, Lock, ArrowLeft } from '@element-plus/icons-vue'
import { authApi } from '@/api/auth'
import { ROUTE_PATHS } from '@/constants'

const router = useRouter()
const loading = ref(false)
const codeSending = ref(false)
const codeSent = ref(false)
const countdown = ref(0)
const step = ref<'send' | 'reset'>('send')
let timer: ReturnType<typeof setInterval> | null = null

const form = reactive({
  email: '',
  code: '',
  password: '',
  confirmPassword: '',
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
  if (!form.email) {
    ElMessage.warning('请输入邮箱')
    return
  }
  codeSending.value = true
  try {
    await authApi.resetPasswordSendCode(form.email)
    codeSent.value = true
    ElMessage.success('验证码已发送到邮箱')
    startCountdown(60)
    step.value = 'reset'
  } catch (e: unknown) {
    const msg = (e as { response?: { data?: { message?: string } } })?.response?.data?.message || '发送失败'
    ElMessage.error(msg)
  } finally {
    codeSending.value = false
  }
}

async function handleReset() {
  if (!form.code || !form.password) {
    ElMessage.warning('请填写完整')
    return
  }
  if (form.password.length < 6) {
    ElMessage.warning('密码至少 6 个字符')
    return
  }
  if (form.password !== form.confirmPassword) {
    ElMessage.warning('两次密码不一致')
    return
  }

  loading.value = true
  try {
    await authApi.resetPassword(form.email, form.code, form.password)
    ElMessage.success('密码重置成功，请登录')
    router.push(ROUTE_PATHS.LOGIN)
  } catch (e: unknown) {
    const msg = (e as { response?: { data?: { message?: string } } })?.response?.data?.message || '重置失败'
    ElMessage.error(msg)
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="forgot-page">
    <div class="card">
      <RouterLink to="/login" class="back-link">
        <el-icon><ArrowLeft /></el-icon> 返回登录
      </RouterLink>

      <h1 class="title">找回密码</h1>
      <p class="subtitle">请输入您注册时使用的邮箱</p>

      <!-- 发送验证码 -->
      <div class="form-group">
        <label>邮箱</label>
        <div class="code-row">
          <el-input
            v-model="form.email"
            placeholder="your@example.com"
            :prefix-icon="Message"
            size="large"
            :disabled="step === 'reset'"
          />
          <el-button
            type="primary"
            size="large"
            :loading="codeSending"
            :disabled="countdown > 0 || !form.email"
            class="code-btn"
            @click="handleSendCode"
          >
            {{ codeBtnText }}
          </el-button>
        </div>
      </div>

      <!-- 重置密码 -->
      <template v-if="step === 'reset'">
        <div class="form-group">
          <label>验证码</label>
          <el-input
            v-model="form.code"
            placeholder="输入 6 位验证码"
            :prefix-icon="Key"
            size="large"
            maxlength="6"
          />
          <p v-if="codeSent" class="spam-tip">若未收到邮件，请检查垃圾箱</p>
        </div>

        <div class="form-group">
          <label>新密码</label>
          <el-input
            v-model="form.password"
            type="password"
            placeholder="至少 6 个字符"
            :prefix-icon="Lock"
            size="large"
            show-password
          />
        </div>

        <div class="form-group">
          <label>确认密码</label>
          <el-input
            v-model="form.confirmPassword"
            type="password"
            placeholder="再次输入新密码"
            :prefix-icon="Lock"
            size="large"
            show-password
            @keyup.enter="handleReset"
          />
        </div>

        <el-button
          type="primary"
          size="large"
          class="submit-btn"
          :loading="loading"
          @click="handleReset"
        >
          重置密码
        </el-button>
      </template>
    </div>
  </div>
</template>

<style scoped>
.forgot-page {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg);
  padding: 20px;
}

.card {
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

.title {
  font-size: 24px;
  font-weight: 600;
  color: var(--fg);
  margin-bottom: 8px;
}

.subtitle {
  color: var(--muted);
  margin-bottom: 24px;
  font-size: 14px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 16px;
}

.form-group label {
  font-size: 14px;
  font-weight: 500;
  color: var(--fg);
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
  margin: 4px 0 0;
  font-size: 12px;
  color: var(--el-color-warning);
  line-height: 1.4;
}

.submit-btn {
  width: 100%;
  margin-top: 8px;
}
</style>
