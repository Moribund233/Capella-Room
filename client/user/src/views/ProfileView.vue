<script setup lang="ts">
import { storeToRefs } from 'pinia'
import { useWebSocketStore } from '@/stores/websocket'

const wsStore = useWebSocketStore()
const { connectionState, isConnected } = storeToRefs(wsStore)
</script>

<template>
  <div class="profile-view">
    <h1>个人中心</h1>

    <!-- WebSocket 连接状态指示器（调试用） -->
    <div class="ws-debug-panel">
      <h3>WebSocket 调试</h3>
      <div class="ws-status">
        <span class="ws-status-label">连接状态:</span>
        <span
          class="ws-status-value"
          :class="{
            'ws-status--connected': isConnected,
            'ws-status--disconnected': !isConnected,
            'ws-status--connecting': connectionState === 'connecting',
            'ws-status--reconnecting': connectionState === 'reconnecting',
          }"
        >
          {{ connectionState }}
        </span>
      </div>
      <div class="ws-indicator">
        <div
          class="ws-dot"
          :class="{
            'ws-dot--connected': isConnected,
            'ws-dot--disconnected': !isConnected,
            'ws-dot--connecting': connectionState === 'connecting',
            'ws-dot--reconnecting': connectionState === 'reconnecting',
          }"
        />
        <span class="ws-indicator-text">
          {{ isConnected ? '已连接' : connectionState === 'connecting' ? '连接中...' : connectionState === 'reconnecting' ? '重连中...' : '已断开' }}
        </span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.profile-view {
  padding: var(--space-2xl);
}

/* WebSocket 调试面板 */
.ws-debug-panel {
  margin-top: var(--space-xl);
  padding: var(--space-lg);
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  max-width: 400px;
}

.ws-debug-panel h3 {
  margin: 0 0 var(--space-md) 0;
  font-size: var(--font-size-base);
  color: var(--color-text);
}

.ws-status {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  margin-bottom: var(--space-sm);
}

.ws-status-label {
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
}

.ws-status-value {
  font-size: var(--font-size-sm);
  font-weight: 600;
  text-transform: capitalize;
}

.ws-status--connected {
  color: var(--color-success);
}

.ws-status--disconnected {
  color: var(--color-error);
}

.ws-status--connecting,
.ws-status--reconnecting {
  color: var(--color-warning);
}

.ws-indicator {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

.ws-dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  transition: all 0.3s ease;
}

.ws-dot--connected {
  background: var(--color-success);
  box-shadow: 0 0 8px var(--color-success);
}

.ws-dot--disconnected {
  background: var(--color-error);
}

.ws-dot--connecting,
.ws-dot--reconnecting {
  background: var(--color-warning);
  animation: pulse 1s infinite;
}

@keyframes pulse {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
}

.ws-indicator-text {
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
}
</style>
