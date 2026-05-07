<script setup lang="ts">
import { ref, computed } from 'vue'
import { useDirectRoomStore } from '@/stores/directRoom'
import type { User } from '@/types/user'

const props = defineProps<{
  show: boolean
  targetUser: User | null
}>()

const emit = defineEmits<{
  close: []
  started: [roomId: string]
}>()

const directRoomStore = useDirectRoomStore()

const submitting = ref(false)
const error = ref('')

const targetUserName = computed(() => props.targetUser?.username ?? '')
const initial = computed(() =>
  targetUserName.value.charAt(0).toUpperCase(),
)

async function handleConfirm() {
  if (!props.targetUser) return

  submitting.value = true
  error.value = ''

  try {
    const room = await directRoomStore.createOrGetDirectRoom(props.targetUser.id)

    if (room) {
      emit('started', room.id)
      emit('close')
    } else {
      error.value = directRoomStore.error || '创建私聊失败'
    }
  } catch {
    error.value = '创建私聊失败，请稍后重试'
  } finally {
    submitting.value = false
  }
}

function handleClose() {
  if (submitting.value) return
  error.value = ''
  emit('close')
}
</script>

<template>
  <Teleport to="body">
    <div v-if="show" class="modal-overlay" @click.self="handleClose">
      <div class="modal-container">
        <div class="modal-header">
          <h3 class="modal-title">发起私聊</h3>
          <button class="modal-close" @click="handleClose">✕</button>
        </div>

        <div class="modal-body">
          <div class="target-user">
            <div class="target-user__avatar">
              <img
                v-if="targetUser?.avatar_url"
                :src="targetUser.avatar_url"
                :alt="targetUser.username"
                class="target-user__avatar-img"
              />
              <span v-else class="target-user__avatar-text">{{ initial }}</span>
            </div>
            <div class="target-user__info">
              <span class="target-user__name">{{ targetUserName }}</span>
              <span class="target-user__hint">即将开始与该用户的私聊</span>
            </div>
          </div>

          <p v-if="error" class="form-error">{{ error }}</p>
        </div>

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
            type="button"
            class="btn btn--primary"
            :disabled="submitting || !targetUser"
            @click="handleConfirm"
          >
            {{ submitting ? '创建中...' : '开始私聊' }}
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
  font-size: 18px;
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0;
}

.modal-close {
  background: none;
  border: none;
  font-size: 20px;
  color: var(--color-text-secondary);
  cursor: pointer;
  padding: 4px;
  line-height: 1;
  border-radius: 4px;
  transition: all 0.15s;
}

.modal-close:hover {
  background: var(--color-background-soft);
  color: var(--color-text-primary);
}

.modal-body {
  padding: 24px 20px;
}

.target-user {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px;
  background: var(--color-background-soft, #f8f9fa);
  border-radius: 12px;
}

.target-user__avatar {
  width: 56px;
  height: 56px;
  border-radius: 50%;
  background: var(--color-primary, #2080f0);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  overflow: hidden;
}

.target-user__avatar-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.target-user__avatar-text {
  color: var(--color-white);
  font-weight: 600;
  font-size: 20px;
}

.target-user__info {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}

.target-user__name {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text-primary, #333);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.target-user__hint {
  font-size: 13px;
  color: var(--color-text-tertiary, #999);
}

.form-error {
  color: var(--color-error, #d03050);
  font-size: 13px;
  margin-top: 12px;
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
  transition: all 0.15s;
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
  color: var(--color-white);
}

.btn--primary:hover:not(:disabled) {
  background: var(--color-primary-hover, #4098f0);
}
</style>
