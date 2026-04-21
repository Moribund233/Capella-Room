<template>
  <div class="login-modal-content">
    <div class="login-header">
      <h3 class="login-title">用户登录</h3>
      <p class="login-subtitle">请输入您的邮箱和密码</p>
    </div>

    <n-form ref="formRef" :model="formData" :rules="formRules" class="login-form" @submit.prevent="handleLogin">
      <n-form-item path="email">
        <n-input v-model:value="formData.email" placeholder="请输入邮箱" size="large"
          :input-props="{ autocomplete: 'email' }">
          <template #prefix>
            <User :size="18" />
          </template>
        </n-input>
      </n-form-item>

      <n-form-item path="password">
        <n-input v-model:value="formData.password" type="password" placeholder="请输入密码" size="large"
          show-password-on="click" :input-props="{ autocomplete: 'current-password' }">
          <template #prefix>
            <Lock :size="18" />
          </template>
        </n-input>
      </n-form-item>

      <n-form-item>
        <n-button type="primary" size="large" block :loading="isLoading" @click="handleLogin">
          登 录
        </n-button>
      </n-form-item>
    </n-form>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { NForm, NFormItem, NInput, NButton } from 'naive-ui'
import { User, Lock } from 'lucide-vue-next'
import { useAuthStore } from '@/store'
import type { FormInst, FormRules } from 'naive-ui'

/**
 * 组件事件定义
 */
const emit = defineEmits<{
  /** 登录成功 */
  (e: 'success'): void
  /** 登录失败 */
  (e: 'error', message: string): void
}>()

/**
 * 表单引用
 */
const formRef = ref<FormInst | null>(null)

/**
 * 表单数据
 */
const formData = ref({
  email: '',
  password: '',
})

/**
 * 加载状态
 */
const isLoading = ref(false)

const authStore = useAuthStore()

/**
 * 表单验证规则
 */
const formRules: FormRules = {
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
const handleLogin = async () => {
  if (!formRef.value) return

  try {
    await formRef.value.validate()
  } catch {
    return
  }

  isLoading.value = true

  try {
    const success = await authStore.login({
      email: formData.value.email,
      password: formData.value.password,
    })

    if (success) {
      emit('success')
    } else {
      const message = authStore.error || '登录失败，请重试'
      window.$message?.error?.(message)
      emit('error', message)
    }
  } catch {
    const message = '网络错误，请检查网络连接后重试'
    window.$message?.error?.(message)
    emit('error', message)
  } finally {
    isLoading.value = false
  }
}
</script>

<style scoped>
.login-modal-content {
  padding: 8px 0;
}

.login-header {
  text-align: center;
  margin-bottom: 24px;
}

.login-title {
  font-size: 20px;
  font-weight: 600;
  color: var(--text-color);
  margin: 0 0 8px;
}

.login-subtitle {
  font-size: 14px;
  color: var(--text-color-secondary);
  margin: 0;
}
</style>
