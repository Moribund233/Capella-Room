<script setup lang="ts">
import { ref, watch } from 'vue'
import { X, User, Check } from 'lucide-vue-next'
import type { User as UserType } from '@/types/user'

const props = defineProps<{
  show: boolean
  user: UserType | null | undefined
}>()

const emit = defineEmits<{
  'update:show': [value: boolean]
  submit: [data: Partial<UserType>]
}>()

const username = ref('')
const isSubmitting = ref(false)
const error = ref('')

watch(() => props.show, (newVal) => {
  if (newVal && props.user) {
    username.value = props.user.username
    error.value = ''
  }
})

function handleClose() {
  emit('update:show', false)
}

function validateUsername(value: string): boolean {
  if (!value || value.length < 3) {
    error.value = '用户名至少需要3个字符'
    return false
  }
  if (value.length > 20) {
    error.value = '用户名不能超过20个字符'
    return false
  }
  if (!/^[a-zA-Z0-9_\u4e00-\u9fa5]+$/.test(value)) {
    error.value = '用户名只能包含字母、数字、下划线和中文'
    return false
  }
  error.value = ''
  return true
}

async function handleSubmit() {
  if (!validateUsername(username.value)) {
    return
  }

  if (username.value === props.user?.username) {
    handleClose()
    return
  }

  isSubmitting.value = true
  try {
    emit('submit', { username: username.value })
  } finally {
    isSubmitting.value = false
  }
}
</script>

<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="show" class="modal-overlay" @click="handleClose">
        <div class="modal-container" @click.stop>
          <div class="modal-header">
            <h3 class="modal-title">
              <User class="modal-title-icon" />
              编辑资料
            </h3>
            <button class="modal-close" @click="handleClose">
              <X class="modal-close-icon" />
            </button>
          </div>

          <div class="modal-body">
            <div class="form-group">
              <label class="form-label">用户名</label>
              <div class="form-input-wrapper">
                <input
                  v-model="username"
                  type="text"
                  class="form-input"
                  :class="{ 'form-input--error': error }"
                  placeholder="请输入用户名"
                  maxlength="20"
                  @input="error = ''"
                  @keydown.enter="handleSubmit"
                />
                <span class="form-input-count">{{ username.length }}/20</span>
              </div>
              <p v-if="error" class="form-error">{{ error }}</p>
              <p v-else class="form-hint">用户名只能包含字母、数字、下划线和中文</p>
            </div>

            <div class="form-group form-group--readonly">
              <label class="form-label">邮箱</label>
              <input
                type="email"
                class="form-input form-input--readonly"
                :value="user?.email"
                disabled
              />
              <p class="form-hint">邮箱地址不可修改</p>
            </div>
          </div>

          <div class="modal-footer">
            <button class="modal-btn modal-btn--secondary" @click="handleClose">
              取消
            </button>
            <button
              class="modal-btn modal-btn--primary"
              :disabled="isSubmitting"
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

.form-group--readonly {
  opacity: 0.7;
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
  padding: 10px 12px;
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

.form-input--readonly {
  background: var(--color-background);
  cursor: not-allowed;
}

.form-input-count {
  position: absolute;
  right: 12px;
  top: 50%;
  transform: translateY(-50%);
  font-size: var(--font-size-small);
  color: var(--color-text-quaternary);
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
