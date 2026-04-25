<template>
  <div class="quick-actions">
    <n-button
      v-for="action in actions"
      :key="action.key"
      :type="action.type"
      size="large"
      class="action-button"
      @click="action.handler"
    >
      <template #icon>
        <n-icon :component="action.icon" />
      </template>
      <div class="action-content">
        <div class="action-label">{{ action.label }}</div>
        <div class="action-desc">{{ action.description }}</div>
      </div>
    </n-button>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'
import { NButton, NIcon } from 'naive-ui'
import { MessageSquare, User, Settings, PlusCircle } from 'lucide-vue-next'
import type { Component } from 'vue'

interface Action {
  key: string
  label: string
  description: string
  icon: Component
  type: 'primary' | 'default' | 'info'
  handler: () => void
}

const router = useRouter()

const actions: Action[] = [
  {
    key: 'rooms',
    label: '进入房间',
    description: '浏览并加入聊天房间',
    icon: MessageSquare,
    type: 'primary',
    handler: () => router.push('/rooms'),
  },
  {
    key: 'profile',
    label: '个人中心',
    description: '查看和编辑个人资料',
    icon: User,
    type: 'default',
    handler: () => router.push('/profile'),
  },
  {
    key: 'settings',
    label: '系统设置',
    description: '自定义应用外观和行为',
    icon: Settings,
    type: 'default',
    handler: () => router.push('/settings'),
  },
]
</script>

<style scoped>
.quick-actions {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 16px;
}

.action-button {
  height: auto;
  padding: 20px 24px;
  justify-content: flex-start;
  text-align: left;
  border-radius: 12px;
}

.action-button :deep(.n-button__content) {
  display: flex;
  align-items: center;
  gap: 16px;
}

.action-content {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.action-label {
  font-size: 16px;
  font-weight: 500;
}

.action-desc {
  font-size: 12px;
  opacity: 0.7;
}

@media (max-width: 640px) {
  .quick-actions {
    grid-template-columns: 1fr;
  }
}
</style>
