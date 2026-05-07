<script setup lang="ts">
import { ref } from 'vue'
import { X, LogIn } from 'lucide-vue-next'
import { useInvitationStore } from '@/stores/invitation'

defineProps<{
  show: boolean
}>()

const emit = defineEmits<{
  close: []
  joined: [roomId: string]
}>()

const invitationStore = useInvitationStore()

const code = ref('')
const submitting = ref(false)
const error = ref('')

async function handleJoin() {
  const trimmed = code.value.trim()
  if (!trimmed) {
    error.value = '请输入邀请码'
    return
  }

  submitting.value = true
  error.value = ''

  const result = await invitationStore.joinByInviteCode(trimmed)
  if (result) {
    emit('joined', result.room_id)
    handleClose()
  } else {
    error.value = invitationStore.error || '加入房间失败，请检查邀请码'
  }

  submitting.value = false
}

function handleClose() {
  code.value = ''
  error.value = ''
  emit('close')
}
</script>

<template>
  <Teleport to="body">
    <div v-if="show" class="modal-overlay" @click.self="handleClose">
      <div class="modal-container">
        <div class="modal-header">
          <h3 class="modal-title">
            <LogIn :size="20" />
            <span>通过邀请码加入</span>
          </h3>
          <button class="modal-close" @click="handleClose">
            <X :size="18" />
          </button>
        </div>

        <div class="modal-body">
          <p class="modal-desc">输入邀请码加入一个私有房间</p>
          <div class="code-input-group">
            <input
              v-model="code"
              type="text"
              class="code-input"
              placeholder="输入邀请码"
              maxlength="20"
              @keyup.enter="handleJoin"
            />
          </div>
          <p v-if="error" class="form-error">{{ error }}</p>
        </div>

        <div class="modal-footer">
          <button class="btn btn--secondary" @click="handleClose">取消</button>
          <button
            class="btn btn--primary"
            :disabled="submitting || !code.trim()"
            @click="handleJoin"
          >
            {{ submitting ? '加入中...' : '加入房间' }}
          </button>
        </div>
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
  background: var(--color-mask);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-container {
  background: var(--color-white);
  border-radius: 12px;
  width: 400px;
  max-width: 90vw;
  box-shadow: 0 8px 32px var(--color-shadow-dark);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--color-border, #eee);
}

.modal-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 16px;
  font-weight: 600;
  margin: 0;
  color: var(--color-text, #333);
}

.modal-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--color-text-secondary, #666);
  cursor: pointer;
  transition: background var(--duration-fast, 0.15s);
}

.modal-close:hover {
  background: var(--color-background, #f5f5f5);
}

.modal-body {
  padding: 20px;
}

.modal-desc {
  font-size: 13px;
  color: var(--color-text-secondary, #666);
  margin: 0 0 12px;
}

.code-input-group {
  display: flex;
  gap: 8px;
}

.code-input {
  width: 100%;
  padding: 12px 16px;
  border: 2px solid var(--color-border, #d9d9d9);
  border-radius: 10px;
  font-size: 20px;
  font-weight: 700;
  text-align: center;
  letter-spacing: 4px;
  text-transform: uppercase;
  outline: none;
  transition: border-color var(--duration-fast, 0.15s);
  box-sizing: border-box;
  font-family: 'SF Mono', 'Fira Code', monospace;
}

.code-input:focus {
  border-color: var(--color-primary, #2080f0);
}

.form-error {
  color: var(--color-error, #f5222d);
  font-size: 13px;
  margin-top: 8px;
  text-align: center;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding: 16px 20px;
  border-top: 1px solid var(--color-border, #eee);
}

.btn {
  padding: 8px 16px;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--duration-fast, 0.15s);
  border: none;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn--secondary {
  background: var(--color-background, #f5f5f5);
  color: var(--color-text-secondary, #666);
}

.btn--secondary:hover:not(:disabled) {
  background: var(--color-background-hover, #e8e8e8);
}

.btn--primary {
  background: var(--color-primary, #2080f0);
  color: white;
}

.btn--primary:hover:not(:disabled) {
  opacity: 0.9;
}
</style>
