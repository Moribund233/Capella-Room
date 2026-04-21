<template>
  <div class="login-modal-content">
    <div class="login-header">
      <h3 class="login-title">用户登录</h3>
      <p class="login-subtitle">请输入您的账号和密码</p>
    </div>

    <n-form ref="formRef" :model="formData" :rules="formRules" class="login-form" @submit.prevent="handleLogin">
      <n-form-item path="username">
        <n-input v-model:value="formData.username" placeholder="请输入用户名" size="large"
          :input-props="{ autocomplete: 'username' }">
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
import { authApi } from '@/api/auth'
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
  username: '',
  password: '',
})

/**
 * 加载状态
 */
const isLoading = ref(false)

const authStore = useAuthStore()
const uiStore = useUIStore()

/**
 * 表单验证规则
 */
const formRules: FormRules = {
  username: [
    { required: true, message: '请输入用户名', trigger: 'blur' },
  ],
  password: [
    { required: true, message: '请输入密码', trigger: 'blur' },
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
    const result = await authApi.login(formData.value)

    if (result.success && result.data) {
      authStore.setToken(result.data.token)
      authStore.setUserInfo(result.data.userInfo)

      // 登录成功后加载云端 UI 配置
      await uiStore.initAfterLogin()

      // 触发成功事件，由父组件处理后续操作（如关闭弹窗）
      emit('success')
    } else {
      const message = result.message || '登录失败，请重试'
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

.login-form {
  :deep(.n-form-item) {
    margin-bottom: 16px;

    &:last-child {
      margin-bottom: 0;
      margin-top: 24px;
    }
  }
}
</style>
