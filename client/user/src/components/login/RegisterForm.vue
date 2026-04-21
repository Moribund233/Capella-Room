<template>
  <n-form
    ref="formRef"
    :model="formData"
    :rules="rules"
    label-placement="left"
    label-width="80"
    @submit.prevent="handleRegister"
  >
    <n-form-item label="用户名" path="username">
      <n-input
        v-model:value="formData.username"
        placeholder="请输入用户名"
        @keyup.enter="handleRegister"
      />
    </n-form-item>
    <n-form-item label="邮箱" path="email">
      <n-input
        v-model:value="formData.email"
        placeholder="请输入邮箱"
        @keyup.enter="handleRegister"
      />
    </n-form-item>
    <n-form-item label="密码" path="password">
      <n-input
        v-model:value="formData.password"
        type="password"
        placeholder="请输入密码"
        show-password-on="click"
        @keyup.enter="handleRegister"
      />
    </n-form-item>
  </n-form>

  <n-space vertical class="action-area">
    <n-button
      type="primary"
      block
      :loading="authStore.loading"
      @click="handleRegister"
    >
      注册
    </n-button>
    <n-button block @click="emit('switch-to-login')">
      已有账号？去登录
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
import { useAuthStore } from '@/store'

const emit = defineEmits<{
  'switch-to-login': []
}>()

const authStore = useAuthStore()

const formRef = ref<FormInst | null>(null)

const formData = reactive({
  username: '',
  email: '',
  password: '',
})

const rules: FormRules = {
  username: [
    { required: true, message: '请输入用户名', trigger: 'blur' },
    { min: 3, max: 20, message: '用户名长度3-20位', trigger: 'blur' },
  ],
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
 * 处理注册
 */
async function handleRegister() {
  if (!formRef.value) return

  try {
    await formRef.value.validate()
  } catch {
    return
  }

  authStore.clearError()
  const success = await authStore.register({
    username: formData.username,
    email: formData.email,
    password: formData.password,
  })

  if (success) {
    emit('switch-to-login')
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
