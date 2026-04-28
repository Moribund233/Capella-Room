<template>
  <div class="login-modal-content">
    <div class="login-header">
      <h3 class="login-title">管理员登录</h3>
      <p class="login-subtitle">请输入您的邮箱和密码</p>
    </div>

    <n-form ref="formRef" :model="formData" :rules="formRules" class="login-form" @submit.prevent="handleLogin">
      <n-form-item path="email">
        <n-input v-model:value="formData.email" placeholder="请输入邮箱" size="large"
          :input-props="{ autocomplete: 'email' }">
          <template #prefix>
            <Mail :size="18" />
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

      <div class="login-tips">
        <n-text depth="3">仅管理员可登录</n-text>
      </div>
    </n-form>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { NForm, NFormItem, NInput, NButton, NText, useMessage } from 'naive-ui'
import { Mail, Lock } from 'lucide-vue-next'
import { useAuthStore, useUIStore } from '@/store'
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
const uiStore = useUIStore()
const message = useMessage()

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
    { min: 6, message: '密码至少6个字符', trigger: 'blur' },
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
    const result = await authStore.login(formData.value.email, formData.value.password)

    if (result.success) {
      message.success('登录成功')

      // 登录成功后加载云端 UI 配置
      await uiStore.initAfterLogin()

      // 触发成功事件，由父组件处理后续操作（如关闭弹窗）
      emit('success')
    } else {
      message.error(result.message)
      emit('error', result.message)
    }
  } catch (error: unknown) {
    const err = error as { response?: { data?: { message?: string } }; message?: string }
    const errorMsg = err?.response?.data?.message || err?.message || '网络错误，请检查网络连接后重试'
    message.error(`登录失败：${errorMsg}`)
    emit('error', errorMsg)
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
  color: var(--text-secondary);
  margin: 0;
}

.login-form {
  padding: 0 8px;
}

.login-tips {
  text-align: center;
  margin-top: 16px;
}
</style>
