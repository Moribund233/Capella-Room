<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  typingUsers: Array<{ id: string; username: string }>
}>()

const typingText = computed(() => {
  const users = props.typingUsers
  if (users.length === 0) return ''
  const firstUser = users[0]!
  if (users.length === 1) {
    return `${firstUser.username} 正在输入...`
  }
  if (users.length === 2) {
    const secondUser = users[1]!
    return `${firstUser.username} 和 ${secondUser.username} 正在输入...`
  }
  return `${firstUser.username} 和其他 ${users.length - 1} 人正在输入...`
})
</script>

<template>
  <div v-if="typingUsers.length > 0" class="typing-indicator">
    <div class="typing-indicator__dots">
      <span class="typing-indicator__dot"></span>
      <span class="typing-indicator__dot"></span>
      <span class="typing-indicator__dot"></span>
    </div>
    <span class="typing-indicator__text">{{ typingText }}</span>
  </div>
</template>

<style scoped>
.typing-indicator {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  padding: var(--space-sm) var(--space-md);
  background: var(--color-background);
  border-top: 1px solid var(--color-border);
}

.typing-indicator__dots {
  display: flex;
  gap: 4px;
  align-items: center;
}

.typing-indicator__dot {
  width: 6px;
  height: 6px;
  background: var(--color-primary);
  border-radius: 50%;
  animation: typing-bounce 1.4s infinite ease-in-out both;
}

.typing-indicator__dot:nth-child(1) {
  animation-delay: -0.32s;
}

.typing-indicator__dot:nth-child(2) {
  animation-delay: -0.16s;
}

.typing-indicator__text {
  font-size: var(--font-size-small);
  color: var(--color-text-secondary);
}

@keyframes typing-bounce {
  0%,
  80%,
  100% {
    transform: scale(0.6);
    opacity: 0.4;
  }
  40% {
    transform: scale(1);
    opacity: 1;
  }
}
</style>
