<script setup lang="ts">
import { ref, reactive } from 'vue'
import { useRoomStore } from '@/stores/room'
import type { CreateRoomData } from '@/types/room'

defineProps<{
  show: boolean
}>()

const emit = defineEmits<{
  close: []
  created: [roomId: string]
}>()

const roomStore = useRoomStore()

const form = reactive<CreateRoomData>({
  name: '',
  description: '',
  is_private: false,
  max_members: 100,
})

const submitting = ref(false)
const formError = ref('')

const nameValid = ref(true)
const nameErrorMsg = ref('')

function validateName() {
  const n = form.name.trim()
  if (!n) {
    nameValid.value = false
    nameErrorMsg.value = '请输入聊天室名称'
    return false
  }
  if (n.length > 50) {
    nameValid.value = false
    nameErrorMsg.value = '名称不能超过50个字符'
    return false
  }
  nameValid.value = true
  nameErrorMsg.value = ''
  return true
}

function resetForm() {
  form.name = ''
  form.description = ''
  form.is_private = false
  form.max_members = 100
  formError.value = ''
  nameValid.value = true
  nameErrorMsg.value = ''
}

async function handleSubmit() {
  if (!validateName()) return
  submitting.value = true
  formError.value = ''

  try {
    const room = await roomStore.createRoom({
      name: form.name.trim(),
      description: form.description?.trim() || undefined,
      is_private: form.is_private,
      max_members: form.max_members || undefined,
    })

    if (room) {
      resetForm()
      emit('created', room.id)
    } else {
      formError.value = roomStore.error || '创建失败'
    }
  } catch {
    formError.value = '创建失败，请稍后重试'
  } finally {
    submitting.value = false
  }
}

function handleClose() {
  if (submitting.value) return
  resetForm()
  emit('close')
}
</script>

<template>
  <Teleport to="body">
    <div v-if="show" class="modal-overlay" @click.self="handleClose">
      <div class="modal-container">
        <div class="modal-header">
          <h3 class="modal-title">创建聊天室</h3>
          <button class="modal-close" @click="handleClose">✕</button>
        </div>

        <form class="modal-body" @submit.prevent="handleSubmit">
          <div class="form-group">
            <label class="form-label">名称 <span class="required">*</span></label>
            <input
              v-model="form.name"
              type="text"
              class="form-input"
              :class="{ 'form-input--error': !nameValid }"
              placeholder="输入聊天室名称"
              maxlength="50"
              @input="validateName"
            />
            <p v-if="!nameValid" class="form-error">{{ nameErrorMsg }}</p>
          </div>

          <div class="form-group">
            <label class="form-label">描述</label>
            <textarea
              v-model="form.description"
              class="form-input form-textarea"
              placeholder="输入聊天室描述（可选）"
              maxlength="200"
              rows="3"
            />
          </div>

          <div class="form-group">
            <label class="form-checkbox">
              <input v-model="form.is_private" type="checkbox" />
              <span>私密聊天室</span>
            </label>
            <p class="form-hint">私密聊天室需要邀请才能加入</p>
          </div>

          <div class="form-group">
            <label class="form-label">最大成员数</label>
            <input
              v-model.number="form.max_members"
              type="number"
              class="form-input"
              min="2"
              max="1000"
            />
          </div>

          <p v-if="formError" class="form-error form-error--global">{{ formError }}</p>

          <div class="modal-footer">
            <button
              type="button"
              class="btn btn--secondary"
              :disabled="submitting"
              @click="handleClose"
            >
              取消
            </button>
            <button
              type="submit"
              class="btn btn--primary"
              :disabled="submitting || !form.name.trim()"
            >
              {{ submitting ? '创建中...' : '创建' }}
            </button>
          </div>
        </form>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.45);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-container {
  background: #fff;
  border-radius: 12px;
  width: 440px;
  max-width: 90vw;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.15);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--color-border, #eee);
}

.modal-title {
  font-size: 16px;
  font-weight: 600;
  margin: 0;
}

.modal-close {
  background: none;
  border: none;
  font-size: 18px;
  cursor: pointer;
  color: var(--color-text-tertiary, #999);
  padding: 4px;
  line-height: 1;
}

.modal-body {
  padding: 20px;
}

.form-group {
  margin-bottom: 16px;
}

.form-label {
  display: block;
  font-size: 13px;
  font-weight: 500;
  margin-bottom: 6px;
  color: var(--color-text, #333);
}

.required {
  color: var(--color-error, #d03050);
}

.form-input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid var(--color-border, #d9d9d9);
  border-radius: 6px;
  font-size: 14px;
  outline: none;
  box-sizing: border-box;
  transition: border-color 0.2s;
}

.form-input:focus {
  border-color: var(--color-primary, #2080f0);
}

.form-input--error {
  border-color: var(--color-error, #d03050);
}

.form-textarea {
  resize: vertical;
  min-height: 60px;
}

.form-checkbox {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  cursor: pointer;
}

.form-hint {
  font-size: 12px;
  color: var(--color-text-tertiary, #999);
  margin: 4px 0 0 0;
}

.form-error {
  font-size: 12px;
  color: var(--color-error, #d03050);
  margin: 4px 0 0 0;
}

.form-error--global {
  text-align: center;
  margin-bottom: 0;
  padding: 8px;
  background: rgba(208, 48, 80, 0.06);
  border-radius: 6px;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  margin-top: 20px;
  padding-top: 16px;
  border-top: 1px solid var(--color-border, #eee);
}

.btn {
  padding: 8px 20px;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  cursor: pointer;
  transition: opacity 0.2s;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn--primary {
  background: var(--color-primary, #2080f0);
  color: #fff;
}

.btn--primary:hover:not(:disabled) {
  opacity: 0.9;
}

.btn--secondary {
  background: var(--color-background, #f5f5f5);
  color: var(--color-text, #333);
  border: 1px solid var(--color-border, #d9d9d9);
}

.btn--secondary:hover:not(:disabled) {
  background: #e8e8e8;
}
</style>
