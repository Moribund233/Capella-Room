<script setup lang="ts">
import { ref, reactive, computed } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, ElForm, ElFormItem, ElInput, ElButton } from 'element-plus'
import type { FormInstance, FormRules } from 'element-plus'
import { User, Message, Lock, Key } from '@element-plus/icons-vue'
import { useAuthStore } from '@/stores/auth'
import { ROUTE_PATHS } from '@/constants'

const router = useRouter()
const authStore = useAuthStore()

const formRef = ref<FormInstance>()
const loading = ref(false)
const codeSending = ref(false)
const codeSent = ref(false)
const countdown = ref(0)
let timer: ReturnType<typeof setInterval> | null = null

const form = reactive({
  email: '',
  code: '',
  username: '',
  password: '',
  confirmPassword: '',
})

const codeBtnText = computed(() => {
  if (codeSending.value) return '发送中...'
  if (countdown.value > 0) return `${countdown.value}s`
  if (codeSent.value) return '重新发送'
  return '发送验证码'
})

const rules: FormRules = {
  email: [
    { required: true, message: '请输入邮箱', trigger: 'blur' },
    { type: 'email', message: '请输入有效的邮箱地址', trigger: ['blur', 'change'] },
  ],
  code: [
    { required: true, message: '请输入验证码', trigger: 'blur' },
    { len: 6, message: '验证码为 6 位数字', trigger: ['blur', 'change'] },
  ],
  username: [
    { required: true, message: '请输入用户名', trigger: 'blur' },
    { min: 2, max: 20, message: '用户名长度 2-20 个字符', trigger: ['blur', 'change'] },
  ],
  password: [
    { required: true, message: '请输入密码', trigger: 'blur' },
    { min: 6, message: '密码至少 6 个字符', trigger: ['blur', 'change'] },
  ],
  confirmPassword: [
    { required: true, message: '请确认密码', trigger: 'blur' },
    {
      validator: (_rule, value: string, callback) => {
        if (value !== form.password) {
          callback(new Error('两次输入的密码不一致'))
        } else {
          callback()
        }
      },
      trigger: ['blur', 'change'],
    },
  ],
}

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
    ElMessage.warning('请先输入邮箱')
    return
  }

  codeSending.value = true
  try {
    await authStore.registerSendCode(form.email)
    codeSent.value = true
    ElMessage.success('验证码已发送到邮箱')
    startCountdown(60)
  } catch {
    ElMessage.error(authStore.error || '发送验证码失败')
  } finally {
    codeSending.value = false
  }
}

async function handleRegister() {
  if (!formRef.value) return

  const valid = await formRef.value.validate().catch(() => false)
  if (!valid) return

  loading.value = true
  try {
    await authStore.register({
      username: form.username,
      email: form.email,
      password: form.password,
      code: form.code,
    })
    ElMessage.success('注册成功')
    router.push(ROUTE_PATHS.CHAT)
  } catch {
    ElMessage.error(authStore.error || '注册失败')
  } finally {
    loading.value = false
  }
}

function goToLogin() {
  router.push('/login')
}
</script>

<template>
  <div class="register-view">
    <div class="register-container">
      <h1 class="title">创建账号</h1>
      <p class="subtitle">加入 CapellaRoom 开始聊天</p>

      <ElForm
        ref="formRef"
        :model="form"
        :rules="rules"
        class="register-form"
        @keyup.enter="handleRegister"
      >
        <ElFormItem prop="email">
          <ElInput
            v-model="form.email"
            placeholder="邮箱"
            size="large"
            :prefix-icon="Message"
            :disabled="codeSent"
          />
        </ElFormItem>

        <ElFormItem prop="username">
          <ElInput
            v-model="form.username"
            placeholder="用户名"
            size="large"
            :prefix-icon="User"
          />
        </ElFormItem>

        <ElFormItem prop="password">
          <ElInput
            v-model="form.password"
            type="password"
            placeholder="密码"
            size="large"
            show-password
            :prefix-icon="Lock"
          />
        </ElFormItem>

        <ElFormItem prop="confirmPassword">
          <ElInput
            v-model="form.confirmPassword"
            type="password"
            placeholder="确认密码"
            size="large"
            show-password
            :prefix-icon="Lock"
          />
        </ElFormItem>

        <ElFormItem prop="code">
          <div class="code-row">
            <ElInput
              v-model="form.code"
              placeholder="验证码"
              size="large"
              maxlength="6"
              :prefix-icon="Key"
            />
            <ElButton
              type="primary"
              size="large"
              :loading="codeSending"
              :disabled="countdown > 0 || !form.email"
              class="code-btn"
              @click="handleSendCode"
            >
              {{ codeBtnText }}
            </ElButton>
          </div>
          <p v-if="codeSent" class="spam-tip">若未收到邮件，请检查垃圾箱</p>
        </ElFormItem>

        <ElButton
          type="primary"
          size="large"
          class="submit-btn"
          :loading="loading"
          @click="handleRegister"
        >
          注册
        </ElButton>
      </ElForm>

      <div class="footer">
        <span>已有账号？</span>
        <ElButton link type="primary" @click="goToLogin">立即登录</ElButton>
      </div>
    </div>
  </div>
</template>

<style scoped>
.register-view {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg);
}

.register-container {
  width: 100%;
  max-width: 400px;
  padding: 40px;
  background: var(--surface);
  border-radius: var(--radius-lg);
  border: 1px solid var(--border);
}

.title {
  text-align: center;
  margin-bottom: 8px;
  font-size: 28px;
  font-weight: 600;
  color: var(--fg);
}

.subtitle {
  text-align: center;
  margin-bottom: 32px;
  color: var(--muted);
}

.register-form {
  margin-bottom: 24px;
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

.submit-btn {
  width: 100%;
  margin-top: 8px;
}

.footer {
  text-align: center;
  color: var(--muted);
}
</style>
