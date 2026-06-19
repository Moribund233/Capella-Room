<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { useWebSocketStore } from '@/stores/websocket'
import { useResponsive } from '@/composables/useResponsive'
import { Search, Close } from '@element-plus/icons-vue'

defineProps<{
  modelValue: string
  filter: 'all' | 'groups' | 'dms'
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
  'update:filter': [value: 'all' | 'groups' | 'dms']
  close: []
}>()

const { t } = useI18n()
const wsStore = useWebSocketStore()
const { isMobile, sidebarCollapsed } = useResponsive()

const filterOptions = [
  { value: 'all', label: 'chat.filterAll' },
  { value: 'groups', label: 'chat.filterGroups' },
  { value: 'dms', label: 'chat.filterDMs' },
]
</script>

<template>
  <div class="sidebar-header">
    <div class="sidebar-header__top">
      <div class="sidebar-header__title">
        <span class="sidebar-header__dot" :class="`sidebar-header__dot--${wsStore.connectionState}`" />
        <span class="sidebar-header__app-name">{{ t('common.appName') }}</span>
      </div>
      <button
        v-if="isMobile"
        class="sidebar-header__close"
        @click="sidebarCollapsed = true"
      >
        <el-icon :size="20"><Close /></el-icon>
      </button>
    </div>
    <div class="sidebar-header__controls">
      <el-input
        :model-value="modelValue"
        :placeholder="t('chat.findRoom')"
        size="small"
        clearable
        class="sidebar-header__search"
        @input="emit('update:modelValue', $event)"
      >
        <template #prefix>
          <el-icon><Search /></el-icon>
        </template>
      </el-input>
      <el-select
        :model-value="filter"
        size="small"
        class="sidebar-header__filter"
        @change="emit('update:filter', $event)"
      >
        <el-option
          v-for="option in filterOptions"
          :key="option.value"
          :value="option.value"
          :label="t(option.label)"
        />
      </el-select>
    </div>
  </div>
</template>

<style scoped lang="scss">
.sidebar-header {
  flex-shrink: 0;
  padding: 12px;
  border-bottom: 1px solid var(--border);

  &__top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 10px;
  }

  &__title {
    display: flex;
    align-items: center;
    gap: 8px;
    font-family: var(--font-display);
    font-size: 15px;
    font-weight: 700;
    color: var(--fg);
  }

  &__close {
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

    &:hover {
      background: var(--message-hover);
      color: var(--fg);
    }
  }

  &__dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    flex-shrink: 0;
    transition: background 0.3s ease, box-shadow 0.3s ease;

    &--connected {
      background: #22c55e;
      animation: breathe-green 2.5s ease-in-out infinite;
    }

    &--connecting,
    &--reconnecting {
      background: #f59e0b;
      animation: breathe-amber 1.2s ease-in-out infinite;
    }

    &--disconnected {
      background: #ef4444;
      animation: breathe-red 2.5s ease-in-out infinite;
    }
  }

  &__controls {
    display: flex;
    gap: 8px;
  }

  &__search {
    flex: 1;
  }

  &__filter {
    width: 100px;
  }

  :deep(.el-input) {
    .el-input__wrapper {
      background: var(--bg);
      box-shadow: none;
      border-radius: var(--radius);
      padding: 4px 8px;

      &.is-focus {
        outline: 1px solid var(--accent);
        box-shadow: none;
      }
    }

    .el-input__inner {
      font-size: 13px;
      color: var(--fg);

      &::placeholder {
        color: var(--muted);
      }
    }

    .el-input__prefix {
      color: var(--muted);
    }
  }
}

@keyframes breathe-green {
  0%, 100% { box-shadow: 0 0 3px 0 rgba(34, 197, 94, 0.4); }
  50% { box-shadow: 0 0 6px 2px rgba(34, 197, 94, 0.7); }
}

@keyframes breathe-amber {
  0%, 100% { box-shadow: 0 0 3px 0 rgba(245, 158, 11, 0.4); }
  50% { box-shadow: 0 0 6px 2px rgba(245, 158, 11, 0.7); }
}

@keyframes breathe-red {
  0%, 100% { box-shadow: 0 0 3px 0 rgba(239, 68, 68, 0.4); }
  50% { box-shadow: 0 0 6px 2px rgba(239, 68, 68, 0.7); }
}
</style>
