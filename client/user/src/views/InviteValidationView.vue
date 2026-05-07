<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { CheckCircle, XCircle, Loader2, LogIn } from 'lucide-vue-next'
import { useInvitationStore } from '@/stores/invitation'
import { useAuthStore } from '@/stores/auth'

const route = useRoute()
const router = useRouter()
const invitationStore = useInvitationStore()
const authStore = useAuthStore()

const code = route.params.code as string

const validating = ref(true)
const valid = ref(false)
const roomName = ref('')
const roomId = ref('')
const errorMessage = ref('')
const joining = ref(false)

async function validate() {
  validating.value = true
  const result = await invitationStore.validateInviteCode(code)
  if (result && result.valid) {
    valid.value = true
    roomName.value = result.room_name || ''
    roomId.value = result.room_id || ''
  } else {
    valid.value = false
    errorMessage.value = result?.message || '邀请码无效或已过期'
  }
  validating.value = false
}

async function handleJoin() {
  if (!authStore.isAuthenticated) {
    router.push(`/login?redirect=/invite/${code}`)
    return
  }

  joining.value = true
  const result = await invitationStore.joinByInviteCode(code)
  if (result) {
    router.push(`/room/${result.room_id}`)
  } else {
    errorMessage.value = invitationStore.error || '加入房间失败'
  }
  joining.value = false
}

onMounted(validate)
</script>

<template>
  <div class="invite-page">
    <div class="invite-card">
      <!-- 验证中 -->
      <template v-if="validating">
        <Loader2 :size="48" class="spinner" />
        <h2>验证邀请码...</h2>
      </template>

      <!-- 有效 -->
      <template v-else-if="valid">
        <div class="invite-icon invite-icon--success">
          <CheckCircle :size="48" />
        </div>
        <h2>邀请有效</h2>
        <p class="invite-room-name">
          你被邀请加入 <strong>{{ roomName }}</strong>
        </p>
        <button
          class="join-btn"
          :disabled="joining"
          @click="handleJoin"
        >
          <LogIn v-if="!joining" :size="20" />
          <Loader2 v-else :size="20" class="spinner" />
          <span>{{ joining ? '加入中...' : authStore.isAuthenticated ? '加入房间' : '登录后加入' }}</span>
        </button>
      </template>

      <!-- 无效 -->
      <template v-else>
        <div class="invite-icon invite-icon--error">
          <XCircle :size="48" />
        </div>
        <h2>邀请无效</h2>
        <p class="invite-error-msg">{{ errorMessage }}</p>
      </template>
    </div>
  </div>
</template>

<style scoped>
.invite-page {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  background: var(--color-background, #f5f5f5);
}

.invite-card {
  background: var(--color-white, #fff);
  border-radius: 16px;
  padding: 48px 40px;
  text-align: center;
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.08);
  max-width: 400px;
  width: 90%;
}

.invite-card h2 {
  font-size: 22px;
  font-weight: 600;
  color: var(--color-text, #333);
  margin: 20px 0 12px;
}

.spinner {
  animation: spin 1s linear infinite;
  color: var(--color-primary, #2080f0);
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.invite-icon {
  margin-bottom: 8px;
}

.invite-icon--success {
  color: var(--color-success, #52c41a);
}

.invite-icon--error {
  color: var(--color-error, #f5222d);
}

.invite-room-name {
  font-size: 15px;
  color: var(--color-text-secondary, #666);
  margin: 0 0 24px;
}

.invite-error-msg {
  font-size: 14px;
  color: var(--color-text-tertiary, #999);
  margin: 0;
}

.join-btn {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 12px 28px;
  background: var(--color-primary, #2080f0);
  color: white;
  border: none;
  border-radius: 10px;
  font-size: 15px;
  font-weight: 500;
  cursor: pointer;
  transition: opacity var(--duration-fast, 0.15s);
}

.join-btn:hover:not(:disabled) {
  opacity: 0.9;
}

.join-btn:disabled {
  opacity: 0.7;
  cursor: not-allowed;
}
</style>
