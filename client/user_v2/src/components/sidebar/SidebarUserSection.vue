<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { useAuthStore } from '@/stores/auth'

const { t } = useI18n()
const authStore = useAuthStore()

defineEmits<{
  openSettings: []
  logout: []
}>()
</script>

<template>
  <div class="user-section">
    <div class="user-section__avatar">
      {{ authStore.user?.username?.charAt(0).toUpperCase() || '?' }}
      <span class="user-section__status-dot" />
    </div>
    <div class="user-section__info">
      <div class="user-section__name">{{ authStore.user?.username || 'User' }}</div>
      <div class="user-section__status">{{ t('chat.online') }}</div>
    </div>
    <div class="user-section__actions">
      <button :title="t('profile.settings.basicInfo.title')" @click="$emit('openSettings')">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="18" height="18">
          <circle cx="12" cy="12" r="3" />
          <path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-2 2 2 2 0 01-2-2v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 01-2-2 2 2 0 012-2h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 010-2.83 2 2 0 012.83 0l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 012-2 2 2 0 012 2v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 0 2 2 0 010 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 012 2 2 2 0 01-2 2h-.09a1.65 1.65 0 00-1.51 1z" />
        </svg>
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

  &__avatar {
    width: 34px;
    height: 34px;
    border-radius: 50%;
    background: linear-gradient(135deg, var(--accent), var(--accent-pink));
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
</style>
