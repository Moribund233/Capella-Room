<script setup lang="ts">
import { computed } from 'vue'
import { MessageSquare, Clock, Users, TrendingUp } from 'lucide-vue-next'

const props = defineProps<{
  title: string
  value: number | string | undefined
  icon: 'MessageSquare' | 'Clock' | 'Users' | 'TrendingUp' | string
  color: string
}>()

type IconComponent = typeof MessageSquare

const iconComponent = computed<IconComponent>(() => {
  const icons: Record<string, IconComponent> = {
    MessageSquare,
    Clock,
    Users,
    TrendingUp
  }
  return icons[props.icon] || MessageSquare
})
</script>

<template>
  <div class="stat-card" :style="{ '--stat-color': color }">
    <div class="stat-card__icon-wrapper">
      <component :is="iconComponent" class="stat-card__icon" />
    </div>
    <div class="stat-card__content">
      <span class="stat-card__value">{{ value ?? '-' }}</span>
      <span class="stat-card__title">{{ title }}</span>
    </div>
  </div>
</template>

<style scoped>
.stat-card {
  display: flex;
  align-items: center;
  gap: var(--space-md);
  padding: var(--space-lg);
  background: var(--color-white);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-sm);
  border: 1px solid var(--color-border-light);
  transition: all 0.2s ease;
}

.stat-card:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow-md);
}

.stat-card__icon-wrapper {
  width: 48px;
  height: 48px;
  border-radius: var(--radius-md);
  background: var(--stat-color);
  opacity: 0.1;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.stat-card__icon {
  width: 24px;
  height: 24px;
  color: var(--stat-color);
}

.stat-card__content {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.stat-card__value {
  font-size: 24px;
  font-weight: 700;
  color: var(--color-text-primary);
  line-height: 1.2;
}

.stat-card__title {
  font-size: var(--font-size-small);
  color: var(--color-text-secondary);
}

/* 响应式 */
@media (max-width: 768px) {
  .stat-card {
    padding: var(--space-md);
  }

  .stat-card__icon-wrapper {
    width: 40px;
    height: 40px;
  }

  .stat-card__icon {
    width: 20px;
    height: 20px;
  }

  .stat-card__value {
    font-size: 20px;
  }
}
</style>
