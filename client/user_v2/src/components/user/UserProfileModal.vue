<script setup lang="ts">
import { computed, reactive, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { UserFilled, Message, Plus } from '@element-plus/icons-vue'
import { userApi } from '@/api/user'
import { friendApi } from '@/api/friend'
import { useAuthStore } from '@/stores/auth'
import { useDirectRoomStore } from '@/stores/directRoom'
import type { User } from '@/types/user'

const props = defineProps<{
  userId: string
}>()

const emit = defineEmits<{
  close: []
}>()

const { t } = useI18n()
const authStore = useAuthStore()
const directRoomStore = useDirectRoomStore()

const state = reactive({
  loading: true,
  error: '',
  user: null as User | null,
  sendingRequest: false,
})

const isSelf = computed(() => authStore.user?.id === props.userId)

onMounted(async () => {
  try {
    const res = await userApi.getUser(props.userId)
    state.user = res.data ?? null
  } catch {
    state.error = t('common.error')
  } finally {
    state.loading = false
  }
})

async function handleStartDirectChat() {
  if (!state.user) return
  try {
    await directRoomStore.createOrGetDirectRoom(state.user.id)
    emit('close')
  } catch {
    ElMessage.error(t('common.error'))
  }
}

async function handleAddFriend() {
  if (!state.user) return
  state.sendingRequest = true
  try {
    await friendApi.sendFriendRequest({ target_user_id: state.user.id })
    ElMessage.success(t('friend.requestSent'))
  } catch {
    ElMessage.error(t('common.error'))
  } finally {
    state.sendingRequest = false
  }
}

function getStatusClass(status: string): string {
  return `status-${status}`
}
</script>

<template>
  <div class="user-profile">
    <div v-if="state.loading" class="user-profile__loading">
      {{ t('common.loading') }}
    </div>

    <div v-else-if="state.error" class="user-profile__error">
      {{ state.error }}
    </div>

    <div v-else-if="state.user" class="user-profile__content">
      <div class="user-profile__header">
        <div class="user-profile__avatar">
          <img
            v-if="state.user.avatar_url"
            :src="state.user.avatar_url"
            :alt="state.user.username"
          />
          <el-icon v-else :size="40"><UserFilled /></el-icon>
          <span class="user-profile__status" :class="getStatusClass(state.user.status)" />
        </div>
        <div class="user-profile__info">
          <h3 class="user-profile__name">{{ state.user.username }}</h3>
          <span class="user-profile__email">{{ state.user.email }}</span>
        </div>
      </div>

      <div v-if="!isSelf" class="user-profile__actions">
        <button class="action-btn action-btn--primary" @click="handleStartDirectChat">
          <el-icon :size="16"><Message /></el-icon>
          {{ t('chat.sendMessage') }}
        </button>
        <button
          class="action-btn"
          :disabled="state.sendingRequest"
          @click="handleAddFriend"
        >
          <el-icon :size="16"><Plus /></el-icon>
          {{ t('friend.addFriend') }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">
.user-profile {
  min-height: 200px;

  &__loading,
  &__error {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 48px;
    color: var(--muted);
    font-size: 14px;
  }

  &__header {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 24px;
  }

  &__avatar {
    position: relative;
    width: 72px;
    height: 72px;
    border-radius: 50%;
    background: var(--surface);
    display: grid;
    place-items: center;
    flex-shrink: 0;
    overflow: hidden;

    img {
      width: 100%;
      height: 100%;
      object-fit: cover;
    }

    .el-icon {
      color: var(--muted);
    }
  }

  &__status {
    position: absolute;
    bottom: 2px;
    right: 2px;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    border: 2px solid var(--bg);

    &.status-online {
      background: var(--accent-green);
    }
    &.status-away {
      background: var(--accent-orange);
    }
    &.status-busy {
      background: var(--accent-pink);
    }
    &.status-offline {
      background: var(--muted);
    }
  }

  &__info {
    min-width: 0;
  }

  &__name {
    font-size: 18px;
    font-weight: 600;
    margin: 0 0 4px;
    color: var(--fg);
  }

  &__email {
    font-size: 13px;
    color: var(--muted);
  }

  &__actions {
    display: flex;
    gap: 8px;
    padding: 0 24px 24px;
  }
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  border-radius: 8px;
  border: 1px solid var(--border);
  background: var(--surface);
  color: var(--fg);
  font-size: 13px;
  cursor: pointer;
  transition: background 0.15s, border-color 0.15s;

  &:hover:not(:disabled) {
    background: var(--message-hover);
    border-color: var(--accent);
  }

  &:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  &--primary {
    background: var(--accent);
    color: #fff;
    border-color: var(--accent);

    &:hover:not(:disabled) {
      filter: brightness(1.1);
      border-color: var(--accent);
    }
  }
}
</style>
