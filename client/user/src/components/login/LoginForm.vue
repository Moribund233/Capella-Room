<template>
  <n-form
    ref="formRef"
    :model="formData"
    :rules="rules"
    label-placement="left"
    label-width="80"
    @submit.prevent="handleLogin"
  >
    <n-form-item label="邮箱" path="email">
      <n-input
        v-model:value="formData.email"
        placeholder="请输入邮箱"
        @keyup.enter="handleLogin"
      />
    </n-form-item>
    <n-form-item label="密码" path="password">
      <n-input
        v-model:value="formData.password"
        type="password"
        placeholder="请输入密码"
        show-password-on="click"
        @keyup.enter="handleLogin"
      />
    </n-form-item>
  </n-form>

  <n-space vertical class="action-area">
    <n-button
      type="primary"
      block
      :loading="authStore.loading"
      @click="handleLogin"
    >
      登录
    </n-button>
    <n-button block @click="emit('register')">
      注册账号
    </n-button>
  </n-space>

  <n-alert
    v-if="authStore.error"
    type="error"
    :show-icon="false"
    class="error-alert"
  >
    {{ authStore.error }}
  </n-alert>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import {
  NForm,
  NFormItem,
  NInput,
  NButton,
  NSpace,
  NAlert,
  type FormInst,
  type FormRules,
} from 'naive-ui'
import { useAuthStore, useUIStore } from '@/store'

const emit = defineEmits<{
  register: []
}>()

const router = useRouter()
const route = useRoute()
const authStore = useAuthStore()
const uiStore = useUIStore()

const formRef = ref<FormInst | null>(null)

const formData = reactive({
  email: '',
  password: '',
})

const rules: FormRules = {
  email: [
    { required: true, message: '请输入邮箱', trigger: 'blur' },
    { type: 'email', message: '请输入有效的邮箱地址', trigger: 'blur' },
  ],
  password: [
    { required: true, message: '请输入密码', trigger: 'blur' },
    { min: 6, message: '密码至少6位', trigger: 'blur' },
  ],
}

/**
 * 处理登录
 */
async function handleLogin() {
  if (!formRef.value) return

  try {
    await formRef.value.validate()
  } catch {
    return
  }

  authStore.clearError()
  const success = await authStore.login({
    email: formData.email,
    password: formData.password,
  })

  if (success) {
    // 登录成功后加载云端 UI 配置
    await uiStore.initAfterLogin()

    const redirect = (route.query.redirect as string) || '/'
    router.push(redirect)
  }
}
</script>

<style scoped>
.action-area {
  margin-top: 24px;
}

.error-alert {
  margin-top: 16px;
}
</style>
