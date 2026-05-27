<script setup lang="ts">
import { ref, reactive } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, ElForm, ElFormItem, ElInput, ElButton } from 'element-plus'
import type { FormInstance, FormRules } from 'element-plus'
import { User, Message, Lock } from '@element-plus/icons-vue'
import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const authStore = useAuthStore()

const formRef = ref<FormInstance>()
const loading = ref(false)

const form = reactive({
  username: '',
  email: '',
  password: '',
  confirmPassword: '',
})

const rules: FormRules = {
  username: [
    { required: true, message: '请输入用户名', trigger: 'blur' },
    { min: 2, max: 20, message: '用户名长度 2-20 个字符', trigger: ['blur', 'change'] },
  ],
  email: [
    { required: true, message: '请输入邮箱', trigger: 'blur' },
    { type: 'email', message: '请输入有效的邮箱地址', trigger: ['blur', 'change'] },
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

async function handleRegister() {
  if (!formRef.value) return

  const valid = await formRef.value.validate().catch(() => false)
  if (!valid) return

  loading.value = true
  try {
    const success = await authStore.register({
      username: form.username,
      email: form.email,
      password: form.password,
    })
    if (success) {
      ElMessage.success('注册成功')
      router.push('/login')
    } else {
      ElMessage.error(authStore.error || '注册失败')
    }
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
        <ElFormItem prop="username">
          <ElInput
            v-model="form.username"
            placeholder="用户名"
            size="large"
            :prefix-icon="User"
          />
        </ElFormItem>

        <ElFormItem prop="email">
          <ElInput
            v-model="form.email"
            placeholder="邮箱"
            size="large"
            :prefix-icon="Message"
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
  background: var(--el-bg-color-page);
}

.register-container {
  width: 100%;
  max-width: 400px;
  padding: 40px;
  background: var(--el-bg-color);
  border-radius: 12px;
  box-shadow: var(--el-box-shadow-light);
}

.title {
  text-align: center;
  margin-bottom: 8px;
  font-size: 28px;
  font-weight: 600;
  color: var(--el-text-color-primary);
}

.subtitle {
  text-align: center;
  margin-bottom: 32px;
  color: var(--el-text-color-secondary);
}

.register-form {
  margin-bottom: 24px;
}

.submit-btn {
  width: 100%;
  margin-top: 8px;
}

.footer {
  text-align: center;
  color: var(--el-text-color-secondary);
}
</style>
