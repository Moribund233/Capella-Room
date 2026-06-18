<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage, ElButton, ElCard, ElResult } from 'element-plus'
import { Loading } from '@element-plus/icons-vue'
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
    ElMessage.success('成功加入房间')
    router.push(`/app`)
  } else {
    errorMessage.value = invitationStore.error || '加入房间失败'
  }
  joining.value = false
}

function goToLogin() {
  router.push(`/login?redirect=/invite/${code}`)
}

function goToHome() {
  router.push('/')
}

onMounted(() => {
  validate()
})
</script>

<template>
  <div class="invite-view">
    <div class="invite-container">
      <!-- 验证中 -->
      <ElCard v-if="validating" class="invite-card">
        <div class="loading-state">
          <ElIcon class="is-loading" :size="48">
            <Loading />
          </ElIcon>
          <p>正在验证邀请码...</p>
        </div>
      </ElCard>

      <!-- 验证成功 -->
      <ElCard v-else-if="valid" class="invite-card">
        <ElResult
          icon="success"
          title="邀请码有效"
          :sub-title="`您被邀请加入房间：${roomName}`"
        >
          <template #extra>
            <ElButton
              v-if="authStore.isAuthenticated"
              type="primary"
              :loading="joining"
              @click="handleJoin"
            >
              加入房间
            </ElButton>
            <ElButton v-else type="primary" @click="goToLogin">
              登录后加入
            </ElButton>
            <ElButton @click="goToHome">返回首页</ElButton>
          </template>
        </ElResult>
      </ElCard>

      <!-- 验证失败 -->
      <ElCard v-else class="invite-card">
        <ElResult icon="error" title="邀请码无效" :sub-title="errorMessage">
          <template #extra>
            <ElButton type="primary" @click="goToHome">返回首页</ElButton>
          </template>
        </ElResult>
      </ElCard>
    </div>
  </div>
</template>

<style scoped>
.invite-view {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg);
  padding: 20px;
}

.invite-container {
  width: 100%;
  max-width: 480px;
}

.invite-card {
  text-align: center;
}

.loading-state {
  padding: 40px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  color: var(--muted);
}

.is-loading {
  animation: rotating 2s linear infinite;
}

@keyframes rotating {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}
</style>
