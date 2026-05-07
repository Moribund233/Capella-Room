<script setup lang="ts">
import { computed, type Component } from 'vue'
import { Link, CheckCircle, Clock, Ban } from 'lucide-vue-next'
import type { RoomInvitation } from '@/types/invitation'

const props = defineProps<{
  invitation: RoomInvitation
}>()

const emit = defineEmits<{
  revoke: [invitationId: string]
  share: [invitation: RoomInvitation]
}>()

// 根据 is_active、expires_at 和 used_count/max_uses 计算状态
const status = computed(() => {
  if (!props.invitation.is_active) return 'revoked'
  if (props.invitation.expires_at && new Date(props.invitation.expires_at) < new Date()) return 'expired'
  if (props.invitation.max_uses && props.invitation.used_count >= props.invitation.max_uses) return 'exhausted'
  return 'active'
})

const statusConfig = computed(() => {
  const map: Record<string, { label: string; icon: Component; class: string }> = {
    active: { label: '有效', icon: CheckCircle, class: 'invitation-card__status--active' },
    expired: { label: '已过期', icon: Clock, class: 'invitation-card__status--expired' },
    revoked: { label: '已撤销', icon: Ban, class: 'invitation-card__status--revoked' },
    exhausted: { label: '已用完', icon: Ban, class: 'invitation-card__status--exhausted' },
  }
  return map[status.value] ?? map.expired
})

const statusClass = computed(() => statusConfig.value?.class ?? '')
const statusIcon = computed(() => statusConfig.value?.icon ?? CheckCircle)
const statusLabel = computed(() => statusConfig.value?.label ?? '')

const isExpired = computed(() => {
  if (!props.invitation.expires_at) return false
  return new Date(props.invitation.expires_at) < new Date()
})

const usageText = computed(() => {
  const max = props.invitation.max_uses
  const used = props.invitation.used_count
  return max ? `${used}/${max}` : `${used}/∞`
})

const expiredText = computed(() => {
  if (!props.invitation.expires_at) return '永不过期'
  return `过期: ${new Date(props.invitation.expires_at).toLocaleDateString()}`
})
</script>

<template>
  <div class="invitation-card" :class="`invitation-card--${status}`">
    <div class="invitation-card__main">
      <div class="invitation-card__code">
        <Link :size="14" />
        <code>{{ invitation.invite_code }}</code>
      </div>
      <div class="invitation-card__meta">
        <span>{{ usageText }} 次使用</span>
        <span>·</span>
        <span>{{ expiredText }}</span>
      </div>
      <div class="invitation-card__creator">
        由 {{ invitation.inviter.username }} 创建
      </div>
    </div>
    <div class="invitation-card__side">
      <span
        class="invitation-card__status"
        :class="statusClass"
      >
        <component :is="statusIcon" :size="14" />
        <span>{{ statusLabel }}</span>
      </span>
      <div v-if="status === 'active' && !isExpired" class="invitation-card__actions">
        <button
          class="invitation-card__action-btn"
          title="分享"
          @click="emit('share', invitation)"
        >
          分享
        </button>
        <button
          class="invitation-card__action-btn invitation-card__action-btn--danger"
          title="撤销"
          @click="emit('revoke', invitation.id)"
        >
          撤销
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.invitation-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 14px 16px;
  border: 1px solid var(--color-border, #eee);
  border-radius: 10px;
  margin-bottom: 8px;
  background: var(--color-white, #fff);
  transition: all var(--duration-fast, 0.15s);
}

.invitation-card--expired,
.invitation-card--revoked,
.invitation-card--exhausted {
  opacity: 0.6;
  background: var(--color-background, #f9f9f9);
}

.invitation-card__main {
  flex: 1;
  min-width: 0;
}

.invitation-card__code {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.invitation-card__code code {
  font-size: 16px;
  font-weight: 700;
  letter-spacing: 1px;
  color: var(--color-text, #333);
  font-family: 'SF Mono', 'Fira Code', monospace;
}

.invitation-card__meta {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--color-text-tertiary, #999);
  margin-bottom: 4px;
}

.invitation-card__creator {
  font-size: 11px;
  color: var(--color-text-tertiary, #999);
}

.invitation-card__side {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 8px;
  flex-shrink: 0;
}

.invitation-card__status {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  font-weight: 500;
  padding: 3px 8px;
  border-radius: 12px;
}

.invitation-card__status--active {
  background: rgba(82, 196, 26, 0.1);
  color: var(--color-success, #52c41a);
}

.invitation-card__status--expired {
  background: rgba(250, 173, 20, 0.1);
  color: var(--color-warning, #faad14);
}

.invitation-card__status--revoked,
.invitation-card__status--exhausted {
  background: rgba(153, 153, 153, 0.1);
  color: var(--color-text-tertiary, #999);
}

.invitation-card__actions {
  display: flex;
  gap: 6px;
}

.invitation-card__action-btn {
  padding: 4px 10px;
  border: 1px solid var(--color-border, #d9d9d9);
  border-radius: 6px;
  background: var(--color-white, #fff);
  color: var(--color-text-secondary, #666);
  font-size: 12px;
  cursor: pointer;
  transition: all var(--duration-fast, 0.15s);
}

.invitation-card__action-btn:hover {
  border-color: var(--color-primary, #2080f0);
  color: var(--color-primary, #2080f0);
}

.invitation-card__action-btn--danger:hover {
  border-color: var(--color-error, #f5222d);
  color: var(--color-error, #f5222d);
}
</style>
