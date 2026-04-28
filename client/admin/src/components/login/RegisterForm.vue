<template>
  <n-form
    ref="formRef"
    :model="formData"
    :rules="formRules"
    class="register-form"
    @submit.prevent="handleRegister"
  >
    <n-form-item path="username">
      <n-input
        v-model:value="formData.username"
        placeholder="请输入用户名"
        size="large"
        :input-props="{ autocomplete: 'username' }"
      >
        <template #prefix>
          <User :size="18" />
        </template>
      </n-input>
    </n-form-item>

    <n-form-item path="password">
      <n-input
        v-model:value="formData.password"
        type="password"
        placeholder="请输入密码"
        size="large"
        show-password-on="click"
        :input-props="{ autocomplete: 'new-password' }"
      >
        <template #prefix>
          <Lock :size="18" />
        </template>
      </n-input>
    </n-form-item>

    <n-form-item path="confirmPassword">
      <n-input
        v-model:value="formData.confirmPassword"
        type="password"
        placeholder="请再次输入密码"
        size="large"
        show-password-on="click"
        :input-props="{ autocomplete: 'new-password' }"
      >
        <template #prefix>
          <Lock :size="18" />
        </template>
      </n-input>
    </n-form-item>

    <n-form-item>
      <n-button
        type="primary"
        size="large"
        block
        :loading="isLoading"
        @click="handleRegister"
      >
        注 册
      </n-button>
    </n-form-item>
  </n-form>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { NForm, NFormItem, NInput, NButton, useMessage } from 'naive-ui'
import { User, Lock } from 'lucide-vue-next'
import { authApi } from '@/api/auth'
import type { FormInst, FormRules, FormItemRule } from 'naive-ui'

const emit = defineEmits<{
  switchToLogin: []
}>()

const formRef = ref<FormInst | null>(null)
const message = useMessage()

const formData = ref({
  username: '',
  password: '',
  confirmPassword: '',
})

const isLoading = ref(false)

/**
 * 表单验证规则
 */
const formRules: FormRules = {
  username: [
    { required: true, message: '请输入用户名', trigger: 'blur' },
    { min: 3, message: '用户名至少3个字符', trigger: 'blur' },
    { max: 20, message: '用户名最多20个字符', trigger: 'blur' },
  ],
  password: [
    { required: true, message: '请输入密码', trigger: 'blur' },
    { min: 6, message: '密码至少6个字符', trigger: 'blur' },
    { max: 32, message: '密码最多32个字符', trigger: 'blur' },
  ],
  confirmPassword: [
    { required: true, message: '请再次输入密码', trigger: 'blur' },
    {
      validator: (_rule: FormItemRule, value: string) => {
        return value === formData.value.password
      },
      message: '两次输入的密码不一致',
      trigger: 'blur',
    },
  ],
}

/**
 * 处理注册
 */
const handleRegister = async () => {
  if (!formRef.value) return

  try {
    await formRef.value.validate()
  } catch {
    return
  }

  isLoading.value = true

  try {
    const result = await authApi.register({
      username: formData.value.username,
      email: '',
      password: formData.value.password,
    })

    if (result.success) {
      message.success('注册成功，正在跳转到登录页...')
      formData.value = { username: '', password: '', confirmPassword: '' }

      setTimeout(() => {
        emit('switchToLogin')
      }, 1500)
    } else {
      message.error(result.message || '注册失败，请检查输入信息')
    }
  } catch (error: unknown) {
    const err = error as { response?: { data?: { message?: string } }; message?: string }
    const errorMsg = err?.response?.data?.message || err?.message || '网络错误，请检查网络连接后重试'
    message.error(`注册失败：${errorMsg}`)
  } finally {
    isLoading.value = false
  }
}
</script>

<style scoped>
.register-form {
  padding: 24px;
}
</style>
