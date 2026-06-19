<script setup lang="ts">
defineProps<{
  variant?: 'primary' | 'success' | 'warning' | 'danger' | 'info'
  size?: 'sm' | 'md'
  dot?: boolean
}>()
</script>

<template>
  <span :class="['badge', `badge-${variant || 'primary'}`, `badge-${size || 'md'}`, { 'badge--dot': dot }]">
    <slot />
  </span>
</template>

<style scoped lang="scss">
.badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  border-radius: var(--radius-full);
  white-space: nowrap;
}

// 尺寸
.badge-sm {
  min-width: 16px;
  height: 16px;
  padding: 0 5px;
  font-size: 10px;
}

.badge-md {
  min-width: 20px;
  height: 20px;
  padding: 0 7px;
  font-size: 11px;
}

// 变体
.badge-primary {
  background: var(--accent);
  color: #fff;
}

.badge-success {
  background: var(--accent-green);
  color: #fff;
}

.badge-warning {
  background: var(--accent-orange);
  color: #fff;
}

.badge-danger {
  background: var(--accent-pink);
  color: #fff;
}

.badge-info {
  background: var(--accent-blue);
  color: #fff;
}

// 点状徽章
.badge--dot {
  min-width: auto;
  width: 8px;
  height: 8px;
  padding: 0;
}
</style>
