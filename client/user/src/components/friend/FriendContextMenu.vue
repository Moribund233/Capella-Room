<script setup lang="ts">
import { ref, onMounted, onUnmounted, type Component } from 'vue'
import { MessageCircle, UserX, User } from 'lucide-vue-next'

interface MenuItem {
  key: string
  label: string
  icon: Component
  danger?: boolean
}

defineProps<{
  visible: boolean
  x: number
  y: number
}>()

const emit = defineEmits<{
  close: []
  sendMessage: []
  deleteFriend: []
  viewProfile: []
}>()

const menuRef = ref<HTMLElement>()

const items: MenuItem[] = [
  { key: 'sendMessage', label: '发送私信', icon: MessageCircle },
  { key: 'viewProfile', label: '查看资料', icon: User },
  { key: 'deleteFriend', label: '删除好友', icon: UserX, danger: true },
]

function handleItemClick(key: string) {
  if (key === 'sendMessage') emit('sendMessage')
  else if (key === 'deleteFriend') emit('deleteFriend')
  else if (key === 'viewProfile') emit('viewProfile')
  emit('close')
}

function handleClickOutside(e: MouseEvent) {
  if (menuRef.value && !menuRef.value.contains(e.target as HTMLElement)) {
    emit('close')
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>

<template>
  <Teleport to="body">
    <div
      v-if="visible"
      ref="menuRef"
      class="context-menu"
      :style="{ left: x + 'px', top: y + 'px' }"
    >
      <button
        v-for="item in items"
        :key="item.key"
        class="context-menu__item"
        :class="{ 'context-menu__item--danger': item.danger }"
        @click="handleItemClick(item.key)"
      >
        <component :is="item.icon" :size="16" />
        <span>{{ item.label }}</span>
      </button>
    </div>
  </Teleport>
</template>

<style scoped>
.context-menu {
  position: fixed;
  background: var(--color-white, #fff);
  border: 1px solid var(--color-border, #e8e8e8);
  border-radius: 8px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.12);
  padding: 4px;
  z-index: 2000;
  min-width: 160px;
}

.context-menu__item {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 8px 12px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--color-text, #333);
  font-size: 13px;
  cursor: pointer;
  transition: background var(--duration-fast, 0.15s);
  white-space: nowrap;
}

.context-menu__item:hover {
  background: var(--color-background, #f5f5f5);
}

.context-menu__item--danger {
  color: var(--color-error, #f5222d);
}

.context-menu__item--danger:hover {
  background: rgba(245, 34, 45, 0.06);
}
</style>
