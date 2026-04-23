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

  <!-- 注册成功提示弹窗 -->
  <n-modal
    v-model:show="showSuccessModal"
    :mask-closable="false"
    preset="card"
    title="🎉 注册成功"
    style="width: 400px"
  >
    <n-card :bordered="false">
      <p>恭喜您，账号注册成功！</p>
      <p style="margin-top: 8px; color: #666;">
        现在可以使用您的邮箱 <strong>{{ formData.email }}</strong> 登录了。
      </p>
    </n-card>
    <template #footer>
      <n-space justify="end">
        <n-button type="primary" @click="handleModalConfirm">
          去登录
        </n-button>
      </n-space>
    </template>
  </n-modal>
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
  NModal,
  NCard,
  type FormInst,
  type FormRules,
} from 'naive-ui'
import { useAuthStore } from '@/store'

const emit = defineEmits<{
  'switch-to-login': []
}>()

const authStore = useAuthStore()

const formRef = ref<FormInst | null>(null)
const showSuccessModal = ref(false)

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
    showSuccessModal.value = true
  }
}

/**
 * 处理弹窗确认，跳转到登录页
 */
function handleModalConfirm() {
  showSuccessModal.value = false
  emit('switch-to-login')
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
