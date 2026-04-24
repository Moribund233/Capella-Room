<template>
  <div class="user-profile-modal">
    <!-- 头部信息 -->
    <div class="profile-header">
      <div class="avatar">{{ userAvatar }}</div>
      <div class="user-info">
        <span class="username">{{ user?.username }}</span>
        <span class="email">{{ user?.email }}</span>
        <n-tag :type="roleTagType" size="small">{{ roleText }}</n-tag>
      </div>
    </div>

    <!-- 可滚动内容区 -->
    <div class="profile-body">
      <n-form
        ref="formRef"
        :model="formData"
        :rules="formRules"
        label-placement="top"
        size="small"
        class="profile-form"
      >
        <n-form-item label="用户名" path="username">
          <n-input
            v-model:value="formData.username"
            placeholder="请输入用户名"
            :disabled="!isEditing"
            size="small"
          />
        </n-form-item>

        <n-form-item label="邮箱" path="email">
          <n-input
            v-model:value="formData.email"
            placeholder="请输入邮箱"
            :disabled="!isEditing"
            size="small"
          />
        </n-form-item>

        <div class="info-row">
          <span class="info-label">注册时间</span>
          <span class="info-value">{{ formatDate(user?.created_at) }}</span>
        </div>

        <div class="info-row">
          <span class="info-label">最后更新</span>
          <span class="info-value">{{ formatDate(user?.updated_at) }}</span>
        </div>
      </n-form>
    </div>

    <!-- 底部按钮 -->
    <div class="profile-footer">
      <template v-if="!isEditing">
        <n-button type="primary" size="small" block @click="startEditing">
          编辑资料
        </n-button>
      </template>
      <template v-else>
        <n-button size="small" @click="cancelEditing">取消</n-button>
        <n-button type="primary" size="small" :loading="isSaving" @click="saveProfile">
          保存
        </n-button>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { NForm, NFormItem, NInput, NButton, NTag } from 'naive-ui'
import { useAuthStore } from '@/store'
import type { FormInst, FormRules } from 'naive-ui'
import type { User } from '@/types/api'

/**
 * 用户资料弹窗组件
 * 显示当前用户信息并支持修改
 */

const authStore = useAuthStore()

/**
 * 表单引用
 */
const formRef = ref<FormInst | null>(null)

/**
 * 编辑状态
 */
const isEditing = ref(false)

/**
 * 保存状态
 */
const isSaving = ref(false)

/**
 * 当前用户信息
 */
const user = computed<User | null>(() => authStore.user)

/**
 * 用户头像文字
 */
const userAvatar = computed(() => {
  if (user.value?.username) {
    return user.value.username.charAt(0).toUpperCase()
  }
  return '?'
})

/**
 * 角色文本
 */
const roleText = computed(() => {
  const roleMap: Record<string, string> = {
    user: '普通用户',
    admin: '管理员',
    super_admin: '超级管理员',
  }
  return roleMap[user.value?.role || 'user'] || '用户'
})

/**
 * 角色标签类型
 */
const roleTagType = computed(() => {
  const typeMap: Record<string, 'default' | 'primary' | 'success' | 'warning' | 'error'> = {
    user: 'default',
    admin: 'warning',
    super_admin: 'error',
  }
  return typeMap[user.value?.role || 'user'] || 'default'
})

/**
 * 表单数据
 */
const formData = ref({
  username: '',
  email: '',
})

/**
 * 表单验证规则
 */
const formRules: FormRules = {
  username: [
    { required: true, message: '请输入用户名', trigger: 'blur' },
    { min: 2, max: 20, message: '用户名长度2-20个字符', trigger: 'blur' },
  ],
  email: [
    { required: true, message: '请输入邮箱', trigger: 'blur' },
    { type: 'email', message: '请输入有效的邮箱地址', trigger: 'blur' },
  ],
}

/**
 * 组件事件定义
 */
const emit = defineEmits<{
  /** 保存成功 */
  (e: 'success'): void
  /** 取消编辑 */
  (e: 'cancel'): void
}>()

/**
 * 格式化日期
 */
function formatDate(dateString?: string): string {
  if (!dateString) return '-'
  const date = new Date(dateString)
  return date.toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
  })
}

/**
 * 开始编辑
 */
function startEditing(): void {
  formData.value = {
    username: user.value?.username || '',
    email: user.value?.email || '',
  }
  isEditing.value = true
}

/**
 * 取消编辑
 */
function cancelEditing(): void {
  isEditing.value = false
  emit('cancel')
}

/**
 * 保存用户资料
 */
async function saveProfile(): Promise<void> {
  if (!formRef.value) return

  try {
    await formRef.value.validate()
  } catch {
    return
  }

  isSaving.value = true

  try {
    // TODO: 调用更新用户资料 API
    // await updateUserProfile(formData.value)

    // 更新本地用户信息
    if (user.value) {
      user.value.username = formData.value.username
      user.value.email = formData.value.email
    }

    isEditing.value = false
    emit('success')
  } catch (error) {
    console.error('保存用户资料失败:', error)
  } finally {
    isSaving.value = false
  }
}

/**
 * 组件挂载时初始化表单数据
 */
onMounted(() => {
  if (user.value) {
    formData.value = {
      username: user.value.username,
      email: user.value.email,
    }
  }
})
</script>

<style scoped>
.user-profile-modal {
  display: flex;
  flex-direction: column;
  height: 300px;
  width: 100%;
  overflow: hidden;
  box-sizing: border-box;
}

/* 头部区域 */
.profile-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  border-bottom: 1px solid var(--border-color-base);
  flex-shrink: 0;
}

.avatar {
  width: 48px;
  height: 48px;
  background: linear-gradient(135deg, var(--primary-color) 0%, var(--primary-color-hover) 100%);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 20px;
  font-weight: 600;
  color: #fff;
  flex-shrink: 0;
}

.user-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
  flex: 1;
}

.username {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-color);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.email {
  font-size: 11px;
  color: var(--text-color-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* 可滚动内容区 */
.profile-body {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 12px;
}

.profile-body::-webkit-scrollbar {
  width: 4px;
}

.profile-body::-webkit-scrollbar-track {
  background: transparent;
}

.profile-body::-webkit-scrollbar-thumb {
  background: var(--border-color-base);
  border-radius: 2px;
}

.profile-body::-webkit-scrollbar-thumb:hover {
  background: var(--text-color-secondary);
}

/* 表单样式 */
.profile-form :deep(.n-form-item) {
  margin-bottom: 12px;
}

.profile-form :deep(.n-form-item:last-child) {
  margin-bottom: 0;
}

.profile-form :deep(.n-form-item-label) {
  font-size: 12px;
  padding-bottom: 4px;
}

/* 信息行 */
.info-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 0;
  font-size: 12px;
  border-bottom: 1px solid var(--border-color-light);
}

.info-row:last-child {
  border-bottom: none;
}

.info-label {
  color: var(--text-color-secondary);
}

.info-value {
  color: var(--text-color);
}

/* 底部按钮 */
.profile-footer {
  display: flex;
  gap: 8px;
  padding: 12px;
  border-top: 1px solid var(--border-color-base);
  flex-shrink: 0;
}

.profile-footer .n-button {
  flex: 1;
}
</style>
