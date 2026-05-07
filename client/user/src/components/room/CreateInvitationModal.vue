<script setup lang="ts">
import { ref } from 'vue'
import { X, Link } from 'lucide-vue-next'
import { useInvitationStore } from '@/stores/invitation'
import type { RoomInvitation } from '@/types/invitation'

const props = defineProps<{
  show: boolean
  roomId: string
}>()

const emit = defineEmits<{
  close: []
  created: [invitation: RoomInvitation]
}>()

const invitationStore = useInvitationStore()

const expiresDays = ref<number | null>(null)
const maxUses = ref<number | null>(null)
const submitting = ref(false)
const error = ref('')

async function handleCreate() {
  submitting.value = true
  error.value = ''

  // 将天数转换为小时数
  const expiresInHours = expiresDays.value
    ? expiresDays.value * 24
    : null

  const invitation = await invitationStore.createInvitation(props.roomId, {
    expires_in_hours: expiresInHours,
    max_uses: maxUses.value,
  })

  if (invitation) {
    emit('created', invitation)
    handleClose()
  } else {
    error.value = invitationStore.error || '创建邀请失败'
  }

  submitting.value = false
}

function handleClose() {
  expiresDays.value = null
  maxUses.value = null
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
            <Link :size="20" />
            <span>创建邀请</span>
          </h3>
          <button class="modal-close" @click="handleClose">
            <X :size="18" />
          </button>
        </div>

        <div class="modal-body">
          <div class="form-group">
            <label class="form-label">过期时间</label>
            <select v-model="expiresDays" class="form-select">
              <option :value="null">永不过期</option>
              <option :value="1">1 天</option>
              <option :value="3">3 天</option>
              <option :value="7">7 天</option>
              <option :value="30">30 天</option>
            </select>
          </div>

          <div class="form-group">
            <label class="form-label">最大使用次数</label>
            <select v-model="maxUses" class="form-select">
              <option :value="null">不限次数</option>
              <option :value="1">1 次</option>
              <option :value="5">5 次</option>
              <option :value="10">10 次</option>
              <option :value="50">50 次</option>
            </select>
          </div>

          <p v-if="error" class="form-error">{{ error }}</p>
        </div>

        <div class="modal-footer">
          <button class="btn btn--secondary" @click="handleClose">取消</button>
          <button
            class="btn btn--primary"
            :disabled="submitting"
            @click="handleCreate"
          >
            {{ submitting ? '创建中...' : '生成邀请码' }}
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

.form-group {
  margin-bottom: 16px;
}

.form-label {
  display: block;
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-secondary, #666);
  margin-bottom: 6px;
}

.form-select {
  width: 100%;
  padding: 9px 12px;
  border: 1px solid var(--color-border, #d9d9d9);
  border-radius: 8px;
  font-size: 14px;
  outline: none;
  background: var(--color-white, #fff);
  transition: border-color var(--duration-fast, 0.15s);
  box-sizing: border-box;
}

.form-select:focus {
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
