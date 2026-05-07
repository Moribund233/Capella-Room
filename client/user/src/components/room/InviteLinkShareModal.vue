<script setup lang="ts">
import { ref, computed } from 'vue'
import { X, Copy, Check, Link } from 'lucide-vue-next'
import type { RoomInvitation } from '@/types/invitation'

const props = defineProps<{
  show: boolean
  invitation: RoomInvitation | null
}>()

const emit = defineEmits<{
  close: []
}>()

const copied = ref(false)

const inviteLink = computed(() => {
  if (!props.invitation) return ''
  const base = window.location.origin
  return `${base}/invite/${props.invitation.invite_code}`
})

async function copyLink() {
  try {
    await navigator.clipboard.writeText(inviteLink.value)
    copied.value = true
    setTimeout(() => { copied.value = false }, 2000)
  } catch {
    // fallback
    const ta = document.createElement('textarea')
    ta.value = inviteLink.value
    document.body.appendChild(ta)
    ta.select()
    document.execCommand('copy')
    document.body.removeChild(ta)
    copied.value = true
    setTimeout(() => { copied.value = false }, 2000)
  }
}

async function copyCode() {
  if (!props.invitation) return
  try {
    await navigator.clipboard.writeText(props.invitation.invite_code)
    copied.value = true
    setTimeout(() => { copied.value = false }, 2000)
  } catch {
    const ta = document.createElement('textarea')
    ta.value = props.invitation.invite_code
    document.body.appendChild(ta)
    ta.select()
    document.execCommand('copy')
    document.body.removeChild(ta)
    copied.value = true
    setTimeout(() => { copied.value = false }, 2000)
  }
}

function handleClose() {
  copied.value = false
  emit('close')
}
</script>

<template>
  <Teleport to="body">
    <div v-if="show" class="modal-overlay" @click.self="handleClose">
      <div class="modal-container">
        <div class="modal-header">
          <h3 class="modal-title">
            <Link :size="20" />
            <span>分享邀请</span>
          </h3>
          <button class="modal-close" @click="handleClose">
            <X :size="18" />
          </button>
        </div>

        <div class="modal-body">
          <div v-if="invitation" class="share-content">
            <div class="share-code">
              <span class="share-code__label">邀请码</span>
              <div class="share-code__value">
                <code>{{ invitation.invite_code }}</code>
                <button class="copy-btn" @click="copyCode">
                  <Check v-if="copied" :size="16" />
                  <Copy v-else :size="16" />
                </button>
              </div>
            </div>

            <div class="share-link">
              <span class="share-link__label">邀请链接</span>
              <div class="share-link__value">
                <span class="share-link__url">{{ inviteLink }}</span>
                <button class="copy-btn" @click="copyLink">
                  <Check v-if="copied" :size="16" />
                  <Copy v-else :size="16" />
                </button>
              </div>
            </div>
          </div>
          <p v-else class="share-empty">暂无邀请信息</p>
        </div>

        <div class="modal-footer">
          <button class="btn btn--primary" @click="handleClose">完成</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: var(--color-mask);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-container {
  background: var(--color-white);
  border-radius: 12px;
  width: 460px;
  max-width: 90vw;
  box-shadow: 0 8px 32px var(--color-shadow-dark);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--color-border, #eee);
}

.modal-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 16px;
  font-weight: 600;
  margin: 0;
  color: var(--color-text, #333);
}

.modal-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--color-text-secondary, #666);
  cursor: pointer;
  transition: background var(--duration-fast, 0.15s);
}

.modal-close:hover {
  background: var(--color-background, #f5f5f5);
}

.modal-body {
  padding: 20px;
}

.share-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.share-code__label,
.share-link__label {
  display: block;
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-secondary, #666);
  margin-bottom: 6px;
}

.share-code__value,
.share-link__value {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  background: var(--color-background, #f5f5f5);
  border-radius: 8px;
  border: 1px solid var(--color-border, #eee);
}

.share-code__value code {
  flex: 1;
  font-size: 18px;
  font-weight: 700;
  letter-spacing: 2px;
  font-family: 'SF Mono', 'Fira Code', monospace;
  color: var(--color-text, #333);
}

.share-link__url {
  flex: 1;
  font-size: 13px;
  color: var(--color-text-secondary, #666);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.copy-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: 1px solid var(--color-border, #d9d9d9);
  border-radius: 6px;
  background: var(--color-white, #fff);
  color: var(--color-text-secondary, #666);
  cursor: pointer;
  transition: all var(--duration-fast, 0.15s);
  flex-shrink: 0;
}

.copy-btn:hover {
  border-color: var(--color-primary, #2080f0);
  color: var(--color-primary, #2080f0);
}

.share-empty {
  text-align: center;
  color: var(--color-text-tertiary, #999);
  font-size: 13px;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  padding: 16px 20px;
  border-top: 1px solid var(--color-border, #eee);
}

.btn {
  padding: 8px 16px;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  border: none;
}

.btn--primary {
  background: var(--color-primary, #2080f0);
  color: white;
}
</style>
