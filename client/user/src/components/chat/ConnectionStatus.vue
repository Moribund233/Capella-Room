<script setup lang="ts">
import { computed } from 'vue'
import { useWebSocketStore } from '@/stores/websocket'
import { useConnectivityStore } from '@/stores/connectivity'
import { storeToRefs } from 'pinia'

const wsStore = useWebSocketStore()
const connectivityStore = useConnectivityStore()
const { connectionState } = storeToRefs(wsStore)
const { isOffline } = storeToRefs(connectivityStore)

const statusConfig = computed(() => {
  if (isOffline.value) {
    return {
      label: '离线模式',
      icon: '⚠',
      class: 'connection-status--offline',
    }
  }

  switch (connectionState.value) {
    case 'connected':
      return {
        label: '已连接',
        icon: '●',
        class: 'connection-status--connected',
      }
    case 'connecting':
      return {
        label: '连接中...',
        icon: '◌',
        class: 'connection-status--connecting',
      }
    case 'reconnecting':
      return {
        label: '重连中...',
        icon: '⟳',
        class: 'connection-status--reconnecting',
      }
    case 'disconnected':
    default:
      return {
        label: '未连接',
        icon: '○',
        class: 'connection-status--disconnected',
      }
  }
})
</script>

<template>
  <div class="connection-status" :class="statusConfig.class">
    <span class="connection-status__dot">{{ statusConfig.icon }}</span>
    <span class="connection-status__label">{{ statusConfig.label }}</span>
    <div v-if="connectionState === 'reconnecting'" class="connection-status__spinner" />
  </div>
</template>

<style scoped>
.connection-status {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 12px;
  border-radius: var(--radius-sm, 6px);
  font-size: var(--font-size-small, 12px);
  transition: background var(--duration-fast, 0.2s);
}

.connection-status__dot {
  font-size: 10px;
  line-height: 1;
}

.connection-status__label {
  white-space: nowrap;
}

.connection-status--connected {
  color: var(--color-success, #18a058);
}

.connection-status--connected .connection-status__dot {
  animation: none;
}

.connection-status--connecting {
  color: var(--color-warning, #f0a020);
}

.connection-status--connecting .connection-status__dot {
  animation: pulse 1.5s ease-in-out infinite;
}

.connection-status--reconnecting {
  color: var(--color-warning, #f0a020);
}

.connection-status--reconnecting .connection-status__dot {
  animation: pulse 1s ease-in-out infinite;
}

.connection-status--disconnected {
  color: var(--color-error, #d03050);
}

.connection-status--offline {
  color: var(--color-error, #d03050);
  background: rgba(208, 48, 80, 0.08);
}

.connection-status--offline .connection-status__dot {
  animation: none;
}

@keyframes pulse {
  0%, 100% { opacity: 0.4; }
  50% { opacity: 1; }
}
</style>
