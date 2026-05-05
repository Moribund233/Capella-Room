<script setup lang="ts">
import { ref, reactive, computed } from 'vue'
import { X, Key, Eye, EyeOff, Check } from 'lucide-vue-next'

defineProps<{
  show: boolean
}>()

const emit = defineEmits<{
  'update:show': [value: boolean]
  submit: [data: { oldPassword: string; newPassword: string }]
}>()

const form = reactive({
  oldPassword: '',
  newPassword: '',
  confirmPassword: ''
})

const showPassword = reactive({
  old: false,
  new: false,
  confirm: false
})

const isSubmitting = ref(false)
const errors = reactive({
  oldPassword: '',
  newPassword: '',
  confirmPassword: ''
})

const isFormValid = computed(() => {
  return form.oldPassword &&
    form.newPassword &&
    form.confirmPassword &&
    form.newPassword === form.confirmPassword &&
    form.newPassword.length >= 8 &&
    !errors.oldPassword &&
    !errors.newPassword &&
    !errors.confirmPassword
})

function handleClose() {
  emit('update:show', false)
  // 重置表单
  form.oldPassword = ''
  form.newPassword = ''
  form.confirmPassword = ''
  errors.oldPassword = ''
  errors.newPassword = ''
  errors.confirmPassword = ''
}

function validateOldPassword() {
  if (!form.oldPassword) {
    errors.oldPassword = '请输入当前密码'
    return false
  }
  errors.oldPassword = ''
  return true
}

function validateNewPassword() {
  if (!form.newPassword) {
    errors.newPassword = '请输入新密码'
    return false
  }
  if (form.newPassword.length < 8) {
    errors.newPassword = '密码长度至少为8位'
    return false
  }
  if (!/(?=.*[a-z])(?=.*[A-Z])(?=.*\d)/.test(form.newPassword)) {
    errors.newPassword = '密码需包含大小写字母和数字'
    return false
  }
  errors.newPassword = ''
  return true
}

function validateConfirmPassword() {
  if (!form.confirmPassword) {
    errors.confirmPassword = '请确认新密码'
    return false
  }
  if (form.confirmPassword !== form.newPassword) {
    errors.confirmPassword = '两次输入的密码不一致'
    return false
  }
  errors.confirmPassword = ''
  return true
}

async function handleSubmit() {
  const isValid = validateOldPassword() && validateNewPassword() && validateConfirmPassword()
  if (!isValid) return

  isSubmitting.value = true
  try {
    emit('submit', {
      oldPassword: form.oldPassword,
      newPassword: form.newPassword
    })
  } finally {
    isSubmitting.value = false
  }
}

function getPasswordStrength(password: string): { level: number; text: string; color: string } {
  const defaultResult = { level: 0, text: '太短', color: '#ff4d4f' }
  if (!password) return defaultResult

  let score = 0
  if (password.length >= 8) score++
  if (password.length >= 12) score++
  if (/[a-z]/.test(password) && /[A-Z]/.test(password)) score++
  if (/\d/.test(password)) score++
  if (/[^a-zA-Z0-9]/.test(password)) score++

  const levels = [
    { level: 0, text: '太短', color: '#ff4d4f' },
    { level: 1, text: '弱', color: '#ff4d4f' },
    { level: 2, text: '一般', color: '#faad14' },
    { level: 3, text: '良好', color: '#1890ff' },
    { level: 4, text: '强', color: '#07c160' },
    { level: 5, text: '非常强', color: '#07c160' }
  ]

  return levels[Math.min(score, 5)] ?? defaultResult
}

const passwordStrength = computed(() => getPasswordStrength(form.newPassword))
</script>

<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="show" class="modal-overlay" @click="handleClose">
        <div class="modal-container" @click.stop>
          <div class="modal-header">
            <h3 class="modal-title">
              <Key class="modal-title-icon" />
              修改密码
            </h3>
            <button class="modal-close" @click="handleClose">
              <X class="modal-close-icon" />
            </button>
          </div>

          <div class="modal-body">
            <!-- 当前密码 -->
            <div class="form-group">
              <label class="form-label">当前密码</label>
              <div class="form-input-wrapper">
                <input
                  v-model="form.oldPassword"
                  :type="showPassword.old ? 'text' : 'password'"
                  class="form-input"
                  :class="{ 'form-input--error': errors.oldPassword }"
                  placeholder="请输入当前密码"
                  @blur="validateOldPassword"
                />
                <button
                  type="button"
                  class="form-input-suffix"
                  @click="showPassword.old = !showPassword.old"
                >
                  <Eye v-if="showPassword.old" class="form-input-icon" />
                  <EyeOff v-else class="form-input-icon" />
                </button>
              </div>
              <p v-if="errors.oldPassword" class="form-error">{{ errors.oldPassword }}</p>
            </div>

            <!-- 新密码 -->
            <div class="form-group">
              <label class="form-label">新密码</label>
              <div class="form-input-wrapper">
                <input
                  v-model="form.newPassword"
                  :type="showPassword.new ? 'text' : 'password'"
                  class="form-input"
                  :class="{ 'form-input--error': errors.newPassword }"
                  placeholder="请输入新密码"
                  @blur="validateNewPassword"
                />
                <button
                  type="button"
                  class="form-input-suffix"
                  @click="showPassword.new = !showPassword.new"
                >
                  <Eye v-if="showPassword.new" class="form-input-icon" />
                  <EyeOff v-else class="form-input-icon" />
                </button>
              </div>
              <!-- 密码强度指示器 -->
              <div v-if="form.newPassword" class="password-strength">
                <div class="password-strength-bar">
                  <div
                    class="password-strength-fill"
                    :style="{ width: `${(passwordStrength.level / 5) * 100}%`, backgroundColor: passwordStrength.color }"
                  />
                </div>
                <span class="password-strength-text" :style="{ color: passwordStrength.color }">
                  {{ passwordStrength.text }}
                </span>
              </div>
              <p v-if="errors.newPassword" class="form-error">{{ errors.newPassword }}</p>
              <p v-else class="form-hint">密码需至少8位，包含大小写字母和数字</p>
            </div>

            <!-- 确认新密码 -->
            <div class="form-group">
              <label class="form-label">确认新密码</label>
              <div class="form-input-wrapper">
                <input
                  v-model="form.confirmPassword"
                  :type="showPassword.confirm ? 'text' : 'password'"
                  class="form-input"
                  :class="{ 'form-input--error': errors.confirmPassword }"
                  placeholder="请再次输入新密码"
                  @blur="validateConfirmPassword"
                />
                <button
                  type="button"
                  class="form-input-suffix"
                  @click="showPassword.confirm = !showPassword.confirm"
                >
                  <Eye v-if="showPassword.confirm" class="form-input-icon" />
                  <EyeOff v-else class="form-input-icon" />
                </button>
              </div>
              <p v-if="errors.confirmPassword" class="form-error">{{ errors.confirmPassword }}</p>
            </div>
          </div>

          <div class="modal-footer">
            <button class="modal-btn modal-btn--secondary" @click="handleClose">
              取消
            </button>
            <button
              class="modal-btn modal-btn--primary"
              :disabled="!isFormValid || isSubmitting"
              @click="handleSubmit"
            >
              <Check v-if="!isSubmitting" class="modal-btn-icon" />
              <span v-else class="modal-btn-spinner" />
              {{ isSubmitting ? '保存中...' : '保存' }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.45);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: var(--space-lg);
}

.modal-container {
  background: var(--color-white);
  border-radius: var(--radius-xl);
  width: 100%;
  max-width: 420px;
  box-shadow: var(--shadow-xl);
  overflow: hidden;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-lg) var(--space-xl);
  border-bottom: 1px solid var(--color-border-light);
}

.modal-title {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  font-size: var(--font-size-h3);
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0;
}

.modal-title-icon {
  width: 20px;
  height: 20px;
  color: var(--color-primary);
}

.modal-close {
  width: 32px;
  height: 32px;
  border-radius: var(--radius-md);
  border: none;
  background: transparent;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
}

.modal-close:hover {
  background: var(--color-background);
}

.modal-close-icon {
  width: 18px;
  height: 18px;
  color: var(--color-text-secondary);
}

.modal-body {
  padding: var(--space-xl);
}

.form-group {
  margin-bottom: var(--space-lg);
}

.form-group:last-child {
  margin-bottom: 0;
}

.form-label {
  display: block;
  font-size: var(--font-size-body);
  font-weight: 500;
  color: var(--color-text-primary);
  margin-bottom: var(--space-sm);
}

.form-input-wrapper {
  position: relative;
}

.form-input {
  width: 100%;
  padding: 10px 40px 10px 12px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  font-size: var(--font-size-body);
  color: var(--color-text-primary);
  background: var(--color-white);
  transition: all 0.2s ease;
  outline: none;
}

.form-input:focus {
  border-color: var(--color-primary);
  box-shadow: 0 0 0 2px var(--color-primary-soft);
}

.form-input--error {
  border-color: var(--color-error);
}

.form-input--error:focus {
  box-shadow: 0 0 0 2px var(--color-error-light);
}

.form-input-suffix {
  position: absolute;
  right: 8px;
  top: 50%;
  transform: translateY(-50%);
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  transition: all 0.2s ease;
}

.form-input-suffix:hover {
  background: var(--color-background);
}

.form-input-icon {
  width: 18px;
  height: 18px;
  color: var(--color-text-tertiary);
}

.form-error {
  font-size: var(--font-size-small);
  color: var(--color-error);
  margin: var(--space-xs) 0 0 0;
}

.form-hint {
  font-size: var(--font-size-small);
  color: var(--color-text-tertiary);
  margin: var(--space-xs) 0 0 0;
}

/* 密码强度指示器 */
.password-strength {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  margin-top: var(--space-xs);
}

.password-strength-bar {
  flex: 1;
  height: 4px;
  background: var(--color-border-light);
  border-radius: var(--radius-full);
  overflow: hidden;
}

.password-strength-fill {
  height: 100%;
  border-radius: var(--radius-full);
  transition: all 0.3s ease;
}

.password-strength-text {
  font-size: var(--font-size-small);
  font-weight: 500;
  min-width: 48px;
  text-align: right;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-md);
  padding: var(--space-lg) var(--space-xl);
  border-top: 1px solid var(--color-border-light);
  background: var(--color-background-light);
}

.modal-btn {
  display: inline-flex;
  align-items: center;
  gap: var(--space-xs);
  padding: 10px 20px;
  border-radius: var(--radius-md);
  font-size: var(--font-size-body);
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  border: none;
  outline: none;
}

.modal-btn--secondary {
  background: var(--color-white);
  color: var(--color-text-primary);
  border: 1px solid var(--color-border);
}

.modal-btn--secondary:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.modal-btn--primary {
  background: var(--color-primary);
  color: var(--color-white);
}

.modal-btn--primary:hover:not(:disabled) {
  background: var(--color-primary-hover);
}

.modal-btn--primary:disabled {
  opacity: 0.7;
  cursor: not-allowed;
}

.modal-btn-icon {
  width: 16px;
  height: 16px;
}

.modal-btn-spinner {
  width: 16px;
  height: 16px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: var(--color-white);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

/* 动画 */
.modal-enter-active,
.modal-leave-active {
  transition: all 0.2s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-from .modal-container,
.modal-leave-to .modal-container {
  transform: scale(0.95);
  opacity: 0;
}
</style>
