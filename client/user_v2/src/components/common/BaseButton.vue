<script setup lang="ts">
defineProps<{
  variant?: 'primary' | 'ghost' | 'danger' | 'danger-outline'
  size?: 'sm' | 'md' | 'lg'
  disabled?: boolean
}>()

defineEmits<{
  click: [event: MouseEvent]
}>()
</script>

<template>
  <button
    :class="['btn', `btn-${variant || 'primary'}`, `btn-${size || 'md'}`]"
    :disabled="disabled"
    @click="$emit('click', $event)"
  >
    <slot />
  </button>
</template>

<style scoped lang="scss">
.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  border-radius: var(--radius-full);
  border: 1px solid transparent;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
  white-space: nowrap;

  &:active:not(:disabled) {
    transform: translateY(1px);
  }

  &:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
}

// 尺寸
.btn-sm {
  padding: 4px 10px;
  font-size: 12px;
}

.btn-md {
  padding: 7px 16px;
  font-size: 13px;
}

.btn-lg {
  padding: 10px 20px;
  font-size: 14px;
}

// 变体
.btn-primary {
  background: var(--accent);
  color: #fff;
  border-color: var(--accent);

  &:hover:not(:disabled) {
    background: color-mix(in oklch, var(--accent) 85%, black);
  }
}

.btn-ghost {
  background: transparent;
  color: var(--muted);
  border-color: var(--border);

  &:hover:not(:disabled) {
    border-color: var(--fg);
    color: var(--fg);
  }
}

.btn-danger {
  background: var(--accent-pink);
  color: #fff;
  border-color: var(--accent-pink);

  &:hover:not(:disabled) {
    background: color-mix(in oklch, var(--accent-pink) 85%, black);
  }
}

.btn-danger-outline {
  background: transparent;
  color: var(--accent-pink);
  border-color: color-mix(in oklch, var(--accent-pink) 40%, transparent);

  &:hover:not(:disabled) {
    background: color-mix(in oklch, var(--accent-pink) 10%, transparent);
    border-color: var(--accent-pink);
  }
}
</style>
