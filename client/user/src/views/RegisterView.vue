<script setup lang="ts">
import { ref, reactive } from 'vue'
import { useRouter } from 'vue-router'
import { NButton, NInput, NForm, NFormItem, useMessage } from 'naive-ui'
import type { FormRules } from 'naive-ui'
import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const message = useMessage()
const authStore = useAuthStore()

const form = reactive({
  username: '',
  email: '',
  password: '',
  confirmPassword: '',
})

const rules: FormRules = {
  username: [
    { required: true, message: '请输入用户名', trigger: 'blur' },
    { min: 2, max: 20, message: '用户名长度 2-20 个字符', trigger: ['blur', 'input'] },
  ],
  email: [
    { required: true, message: '请输入邮箱', trigger: 'blur' },
    { type: 'email', message: '请输入有效的邮箱地址', trigger: ['blur', 'input'] },
  ],
  password: [
    { required: true, message: '请输入密码', trigger: 'blur' },
    { min: 6, message: '密码至少 6 个字符', trigger: ['blur', 'input'] },
  ],
  confirmPassword: [
    { required: true, message: '请确认密码', trigger: 'blur' },
    {
      validator: (_rule, value: string) => value === form.password,
      message: '两次输入的密码不一致',
      trigger: ['blur', 'input'],
    },
  ],
}

const formRef = ref<InstanceType<typeof NForm> | null>(null)
const loading = ref(false)

async function handleRegister() {
  const validated = await formRef.value?.validate().catch(() => false)
  if (validated === false) return

  loading.value = true
  try {
    await authStore.register({
      username: form.username,
      email: form.email,
      password: form.password,
    })
    message.success('注册成功，请登录')
    router.push('/login')
  } catch (err: unknown) {
    const msg =
      err && typeof err === 'object' && 'response' in err
        ? ((err as { response?: { data?: { message?: string } } }).response?.data?.message ?? '注册失败，请稍后重试')
        : '网络错误，请稍后重试'
    message.error(msg)
  } finally {
    loading.value = false
  }
}

function goLogin() {
  router.push('/login')
}
</script>

<template>
  <div class="register-view">
    <h2 class="register-view__title">创建账号</h2>
    <p class="register-view__subtitle">注册一个新账号开始聊天</p>

    <NForm ref="formRef" :model="form" :rules="rules" class="register-view__form" @submit.prevent="handleRegister">
      <NFormItem label="用户名" path="username">
        <NInput
          v-model:value="form.username"
          placeholder="请输入用户名"
          size="large"
        />
      </NFormItem>

      <NFormItem label="邮箱" path="email">
        <NInput
          v-model:value="form.email"
          placeholder="请输入邮箱"
          :input-props="{ type: 'email' }"
          size="large"
        />
      </NFormItem>

      <NFormItem label="密码" path="password">
        <NInput
          v-model:value="form.password"
          placeholder="请输入密码"
          type="password"
          size="large"
          show-password-on="click"
        />
      </NFormItem>

      <NFormItem label="确认密码" path="confirmPassword">
        <NInput
          v-model:value="form.confirmPassword"
          placeholder="请再次输入密码"
          type="password"
          size="large"
          show-password-on="click"
        />
      </NFormItem>

      <NButton
        type="primary"
        size="large"
        block
        :loading="loading"
        @click="handleRegister"
      >
        注册
      </NButton>
    </NForm>

    <p class="register-view__footer">
      已有账号？
      <NButton text type="primary" @click="goLogin">立即登录</NButton>
    </p>
  </div>
</template>

<style scoped>
.register-view {
  width: 100%;
  max-width: 320px;
}

.register-view__title {
  font-size: var(--font-size-h2);
  font-weight: 600;
  margin-bottom: var(--space-xs);
}

.register-view__subtitle {
  color: var(--color-text-tertiary);
  margin-bottom: var(--space-2xl);
}

.register-view__form {
  display: flex;
  flex-direction: column;
  gap: var(--space-lg);
}

.register-view__footer {
  margin-top: var(--space-2xl);
  text-align: center;
  color: var(--color-text-tertiary);
  font-size: var(--font-size-small);
}
</style>
