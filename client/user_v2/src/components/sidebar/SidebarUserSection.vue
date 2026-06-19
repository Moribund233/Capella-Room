<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useAuthStore } from '@/stores/auth'
import { getAvatarGradient, getAvatarShadow } from '@/utils/avatar'
import { Setting, SwitchButton } from '@element-plus/icons-vue'

const { t } = useI18n()
const authStore = useAuthStore()

const avatarStyle = computed(() => ({
  background: getAvatarGradient(authStore.user?.username || 'user'),
  boxShadow: getAvatarShadow('sm'),
}))

defineProps<{
  isMobile?: boolean
}>()

defineEmits<{
  openProfile: []
  openSettings: []
  logout: []
}>()
</script>

<template>
  <div
    class="user-section"
    :class="{ 'user-section--mobile': isMobile }"
  >
    <div class="user-section__left" @click="isMobile && $emit('openProfile')">
      <div class="user-section__avatar" :style="avatarStyle">
        {{ authStore.user?.username?.charAt(0).toUpperCase() || '?' }}
        <span class="user-section__status-dot" />
      </div>
      <div class="user-section__info">
        <div class="user-section__name">{{ authStore.user?.username || 'User' }}</div>
        <div class="user-section__status">{{ t('chat.online') }}</div>
      </div>
    </div>
    <div class="user-section__actions">
      <button :title="t('profile.settings.basicInfo.title')" @click="$emit('openSettings')">
        <el-icon :size="18"><Setting /></el-icon>
      </button>
      <button v-if="isMobile" :title="t('chat.logout')" @click="$emit('logout')">
        <el-icon :size="18"><SwitchButton /></el-icon>
      </button>
    </div>
  </div>
</template>

<style scoped lang="scss">
.user-section {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  border-top: 1px solid var(--border);

  &__left {
    display: flex;
    align-items: center;
    gap: 10px;
    flex: 1;
    min-width: 0;
  }

  &__avatar {
    width: 34px;
    height: 34px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    font-size: 14px;
    font-weight: 600;
    color: #fff;
    flex-shrink: 0;
    position: relative;
  }

  &__status-dot {
    position: absolute;
    bottom: 0;
    right: 0;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--accent-green);
    border: 2px solid var(--sidebar-bg);
  }

  &__info {
    flex: 1;
    min-width: 0;
  }

  &__name {
    font-size: 14px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    color: var(--fg);
  }

  &__status {
    font-size: 12px;
    color: var(--muted);
  }

  &__actions {
    display: flex;
    gap: 4px;

    button {
      width: 32px;
      height: 32px;
      display: flex;
      align-items: center;
      justify-content: center;
      background: none;
      border: none;
      border-radius: 6px;
      color: var(--muted);
      cursor: pointer;
      transition: background 0.15s, color 0.15s;

      &:hover {
        color: var(--fg);
        background: var(--message-hover);
      }
    }
  }
}

.user-section--mobile {
  padding: 12px 12px;
  border-top: none;
  border-bottom: 1px solid var(--border);

  .user-section__left {
    cursor: pointer;
    border-radius: 8px;
    padding: 4px 4px 4px 0;

    &:active {
      background: var(--message-hover);
    }
  }

  .user-section__avatar {
    width: 40px;
    height: 40px;
    font-size: 16px;
  }
}
</style>
