<template>
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
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { NForm, NFormItem, NInput, NButton } from 'naive-ui'
import { User, Lock } from 'lucide-vue-next'
import { authApi } from '@/api/auth'
import { useAuthStore, useUIStore } from '@/store'
import type { FormInst, FormRules } from 'naive-ui'

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

const router = useRouter()
const route = useRoute()
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

      const redirect = (route.query.redirect as string) || '/home'
      router.push(redirect)
    } else {
      window.$message?.error?.(result.message || '登录失败，请重试')
    }
  } catch {
    window.$message?.error?.('网络错误，请检查网络连接后重试')
  } finally {
    isLoading.value = false
  }
}
</script>

<style scoped>
.login-form {
  padding: 24px;
}
</style>
