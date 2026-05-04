<script setup lang="ts">
import { ref, computed } from 'vue'
import ConnectionStatus from '@/components/chat/ConnectionStatus.vue'
import { useConnectivityStore } from '@/stores/connectivity'
import { useWebSocket } from '@/composables/useWebSocket'

const connectivityStore = useConnectivityStore()
const { isConnected, isConnecting, send } = useWebSocket()
const isOffline = computed(() => connectivityStore.isOffline)

const testMessage = ref('')
const receivedMessages = ref<Array<{ type: string; payload: unknown; time: string }>>([])
const showTestPanel = ref(false)

function handleSendTest() {
  if (!testMessage.value.trim() || !isConnected.value) return
  send('ChatMessage', {
    room_id: '00000000-0000-0000-0000-000000000000',
    content: testMessage.value,
  })
  testMessage.value = ''
}

function handleSendPing() {
  send('Ping')
}
</script>

<template>
  <div class="chat-view">
    <!-- 离线模式横幅 -->
    <div v-if="isOffline" class="chat-view__offline-banner">
      <span class="chat-view__offline-icon">⚠</span>
      <span>服务器无法连接，应用处于离线模式。</span>
      <span>请刷新页面重试。</span>
    </div>

    <div class="chat-view__header">
      <ConnectionStatus />
      <button
        class="chat-view__test-btn"
        @click="showTestPanel = !showTestPanel"
      >
        {{ showTestPanel ? '关闭测试' : '连接测试' }}
      </button>
    </div>

    <div v-if="showTestPanel" class="chat-view__test-panel">
      <div class="chat-view__test-info">
        <span>状态: </span>
        <span v-if="isConnected" class="chat-view__test-info--ok">已连接</span>
        <span v-else-if="isConnecting" class="chat-view__test-info--pending">连接中...</span>
        <span v-else class="chat-view__test-info--err">未连接</span>
      </div>

      <div class="chat-view__test-controls">
        <input
          v-model="testMessage"
          type="text"
          class="chat-view__test-input"
          placeholder="输入测试消息..."
          @keyup.enter="handleSendTest"
          :disabled="!isConnected"
        />
        <button
          class="chat-view__test-send"
          @click="handleSendTest"
          :disabled="!isConnected || !testMessage.trim()"
        >
          发送
        </button>
        <button
          class="chat-view__test-ping"
          @click="handleSendPing"
          :disabled="!isConnected"
        >
          Ping
        </button>
      </div>
    </div>

    <div class="chat-view__empty">
      <div class="chat-view__empty-icon">💬</div>
      <p class="chat-view__empty-text">选择一个聊天室开始聊天</p>
    </div>
  </div>
</template>

<style scoped>
.chat-view {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.chat-view__offline-banner {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 10px var(--space-lg);
  background: var(--color-error, #d03050);
  color: #fff;
  font-size: var(--font-size-small, 13px);
  flex-shrink: 0;
}

.chat-view__offline-icon {
  font-size: 14px;
}

.chat-view__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-sm) var(--space-lg);
  border-bottom: 1px solid var(--color-border);
  flex-shrink: 0;
}

.chat-view__test-btn {
  padding: 4px 12px;
  font-size: var(--font-size-small, 12px);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm, 6px);
  background: var(--color-white);
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all var(--duration-fast, 0.2s);
}

.chat-view__test-btn:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.chat-view__test-panel {
  padding: var(--space-md) var(--space-lg);
  border-bottom: 1px solid var(--color-border);
  background: var(--color-background);
  flex-shrink: 0;
}

.chat-view__test-info {
  font-size: var(--font-size-small, 12px);
  margin-bottom: var(--space-sm);
}

.chat-view__test-info--ok {
  color: var(--color-success, #18a058);
}

.chat-view__test-info--pending {
  color: var(--color-warning, #f0a020);
}

.chat-view__test-info--err {
  color: var(--color-error, #d03050);
}

.chat-view__test-controls {
  display: flex;
  gap: var(--space-sm);
}

.chat-view__test-input {
  flex: 1;
  padding: 6px 12px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm, 6px);
  font-size: var(--font-size-body, 14px);
  outline: none;
  transition: border-color var(--duration-fast, 0.2s);
}

.chat-view__test-input:focus {
  border-color: var(--color-primary);
}

.chat-view__test-send,
.chat-view__test-ping {
  padding: 6px 16px;
  border: none;
  border-radius: var(--radius-sm, 6px);
  font-size: var(--font-size-body, 14px);
  cursor: pointer;
  transition: all var(--duration-fast, 0.2s);
}

.chat-view__test-send {
  background: var(--color-primary);
  color: #fff;
}

.chat-view__test-send:hover:not(:disabled) {
  opacity: 0.9;
}

.chat-view__test-ping {
  background: var(--color-background);
  border: 1px solid var(--color-border);
}

.chat-view__test-ping:hover:not(:disabled) {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.chat-view__test-send:disabled,
.chat-view__test-ping:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.chat-view__empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--color-text-tertiary);
}

.chat-view__empty-icon {
  font-size: 48px;
  margin-bottom: var(--space-lg);
  opacity: 0.5;
}

.chat-view__empty-text {
  font-size: var(--font-size-body);
}
</style>
