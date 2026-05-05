<script setup lang="ts">
import { ref, reactive } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { NButton, NInput, NForm, NFormItem, useMessage } from 'naive-ui'
import type { FormRules } from 'naive-ui'
import { useAuthStore } from '@/stores/auth'
import { useResponsive } from '@/composables/useResponsive'

const router = useRouter()
const route = useRoute()
const message = useMessage()
const authStore = useAuthStore()
const { isMobile } = useResponsive()

const form = reactive({
  email: '',
  password: '',
})

const rules: FormRules = {
  email: [
    { required: true, message: '请输入邮箱', trigger: 'blur' },
    { type: 'email', message: '请输入有效的邮箱地址', trigger: ['blur', 'input'] },
  ],
  password: [
    { required: true, message: '请输入密码', trigger: 'blur' },
    { min: 6, message: '密码至少 6 个字符', trigger: ['blur', 'input'] },
  ],
}

const formRef = ref<InstanceType<typeof NForm> | null>(null)
const loading = ref(false)

async function handleLogin() {
  const validated = await formRef.value?.validate().catch(() => false)
  if (validated === false) return

  loading.value = true
  try {
    await authStore.login({ email: form.email, password: form.password })
    message.success('登录成功')
    const redirect = (route.query.redirect as string) || '/'
    router.push(redirect)
  } catch (err: unknown) {
    const msg =
      err && typeof err === 'object' && 'response' in err
        ? ((err as { response?: { data?: { message?: string } } }).response?.data?.message ?? '登录失败，请检查邮箱和密码')
        : '网络错误，请稍后重试'
    message.error(msg)
  } finally {
    loading.value = false
  }
}

function goRegister() {
  router.push('/register')
}
</script>

<template>
  <div class="login-view">
    <div v-if="isMobile" class="login-view__header">
      <img src="/favicon.svg" alt="Seredeli Room" class="login-view__logo" />
      <span class="login-view__app-name">Seredeli Room</span>
    </div>
    <h2 class="login-view__title">欢迎回来</h2>
    <p class="login-view__subtitle">请登录你的账号</p>

    <NForm ref="formRef" :model="form" :rules="rules" class="login-view__form" @submit.prevent="handleLogin">
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

      <NButton
        type="primary"
        size="large"
        block
        :loading="loading"
        @click="handleLogin"
      >
        登录
      </NButton>
    </NForm>

    <p class="login-view__footer">
      还没有账号？
      <NButton text type="primary" @click="goRegister">立即注册</NButton>
    </p>
  </div>
</template>

<style scoped>
.login-view {
  width: 100%;
  max-width: 320px;
}

.login-view__header {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  margin-bottom: var(--space-xl);
}

.login-view__logo {
  width: 32px;
  height: 32px;
}

.login-view__app-name {
  font-size: var(--font-size-h4);
  font-weight: 600;
  color: var(--color-text-primary);
}

.login-view__title {
  font-size: var(--font-size-h2);
  font-weight: 600;
  margin-bottom: var(--space-xs);
}

.login-view__subtitle {
  color: var(--color-text-tertiary);
  margin-bottom: var(--space-2xl);
}

.login-view__form {
  display: flex;
  flex-direction: column;
  gap: var(--space-lg);
}

.login-view__footer {
  margin-top: var(--space-2xl);
  text-align: center;
  color: var(--color-text-tertiary);
  font-size: var(--font-size-small);
}
</style>
