<script setup lang="ts">
import { ref, onErrorCaptured } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const error = ref<Error | null>(null)
const errorInfo = ref<string>('')

onErrorCaptured((err: Error, _instance, info: string) => {
  error.value = err
  errorInfo.value = info
  return false
})

function handleRetry() {
  error.value = null
  errorInfo.value = ''
}
</script>

<template>
  <div v-if="error" class="error-boundary">
    <div class="error-boundary__icon">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="48" height="48">
        <circle cx="12" cy="12" r="10" />
        <line x1="12" y1="8" x2="12" y2="12" />
        <line x1="12" y1="16" x2="12.01" y2="16" />
      </svg>
    </div>
    <h3 class="error-boundary__title">{{ t('error.boundaryTitle') }}</h3>
    <p class="error-boundary__message">{{ error.message }}</p>
    <button class="error-boundary__retry" @click="handleRetry">
      {{ t('error.boundaryRetry') }}
    </button>
  </div>
  <slot v-else />
</template>

<style scoped lang="scss">
.error-boundary {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px 24px;
  text-align: center;
  min-height: 200px;

  &__icon {
    color: var(--accent-pink);
    margin-bottom: 16px;
  }

  &__title {
    font-size: 18px;
    font-weight: 600;
    color: var(--fg);
    margin: 0 0 8px;
  }

  &__message {
    font-size: 14px;
    color: var(--muted);
    margin: 0 0 24px;
    max-width: 400px;
    word-break: break-word;
  }

  &__retry {
    padding: 8px 24px;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--fg);
    font-size: 14px;
    cursor: pointer;
    transition: background 0.15s, border-color 0.15s;

    &:hover {
      background: var(--message-hover);
      border-color: var(--accent);
    }
  }
}
</style>
