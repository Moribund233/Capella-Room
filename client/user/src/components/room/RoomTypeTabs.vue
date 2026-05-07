<script setup lang="ts">
import { RoomType } from '@/types/room'

const props = defineProps<{
  modelValue: RoomType
}>()

const emit = defineEmits<{
  'update:modelValue': [value: RoomType]
}>()

const tabs = [
  { value: RoomType.Group, label: '群聊', icon: '👥' },
  { value: RoomType.Direct, label: '私聊', icon: '💬' },
]

function handleTabClick(value: RoomType) {
  if (value !== props.modelValue) {
    emit('update:modelValue', value)
  }
}
</script>

<template>
  <div class="room-type-tabs">
    <button
      v-for="tab in tabs"
      :key="tab.value"
      class="room-type-tabs__item"
      :class="{ 'room-type-tabs__item--active': modelValue === tab.value }"
      @click="handleTabClick(tab.value)"
    >
      <span class="room-type-tabs__icon">{{ tab.icon }}</span>
      <span class="room-type-tabs__label">{{ tab.label }}</span>
    </button>
  </div>
</template>

<style scoped>
.room-type-tabs {
  display: flex;
  gap: 4px;
  padding: 8px 12px;
  background: var(--color-background-soft, #f8f9fa);
  border-bottom: 1px solid var(--color-border, #eee);
}

.room-type-tabs__item {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 8px 12px;
  border: none;
  border-radius: var(--radius-md, 8px);
  background: transparent;
  color: var(--color-text-secondary, #666);
  font-size: var(--font-size-body, 14px);
  font-weight: 500;
  cursor: pointer;
  transition: all var(--duration-fast, 0.15s);
}

.room-type-tabs__item:hover {
  background: var(--color-background, #f0f0f0);
  color: var(--color-text-primary, #333);
}

.room-type-tabs__item--active {
  background: var(--color-white, #fff);
  color: var(--color-primary, #2080f0);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.room-type-tabs__icon {
  font-size: 14px;
}

.room-type-tabs__label {
  font-size: 13px;
}
</style>
