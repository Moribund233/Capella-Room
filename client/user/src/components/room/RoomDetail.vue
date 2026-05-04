<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { storeToRefs } from 'pinia'
import { useRoomStore } from '@/stores/room'
import { useAuthStore } from '@/stores/auth'

const props = defineProps<{
  roomId: string
  visible: boolean
}>()

const emit = defineEmits<{
  close: []
  leave: [roomId: string]
}>()

const roomStore = useRoomStore()
const authStore = useAuthStore()
const { currentRoom, members } = storeToRefs(roomStore)

const isOwner = ref(false)
const isMember = ref(false)

onMounted(() => {
  loadDetail()
})

async function loadDetail() {
  await Promise.all([
    roomStore.fetchRoomDetail(props.roomId),
    roomStore.fetchMembers(props.roomId),
  ])
  if (currentRoom.value && authStore.user) {
    isOwner.value = currentRoom.value.owner.id === authStore.user.id
    isMember.value = members.value.some((m) => m.user_id === authStore.user?.id)
  }
}

function statusText(status: string) {
  switch (status) {
    case 'online': return '在线'
    case 'away': return '离开'
    case 'busy': return '忙碌'
    default: return '离线'
  }
}

function roleLabel(role: string) {
  switch (role) {
    case 'owner': return '创建者'
    case 'admin': return '管理员'
    default: return '成员'
  }
}
</script>

<template>
  <div v-if="visible" class="room-detail">
    <div class="room-detail__header">
      <h3 class="room-detail__title">房间详情</h3>
      <button class="room-detail__close" @click="emit('close')">✕</button>
    </div>

    <div v-if="currentRoom" class="room-detail__body">
      <!-- 基本信息 -->
      <section class="room-detail__section">
        <h4 class="room-detail__section-title">基本信息</h4>
        <div class="room-detail__info-row">
          <span class="room-detail__label">名称</span>
          <span class="room-detail__value">{{ currentRoom.name }}</span>
        </div>
        <div v-if="currentRoom.description" class="room-detail__info-row">
          <span class="room-detail__label">描述</span>
          <span class="room-detail__value">{{ currentRoom.description }}</span>
        </div>
        <div class="room-detail__info-row">
          <span class="room-detail__label">类型</span>
          <span class="room-detail__value">{{ currentRoom.is_private ? '私密' : '公开' }}</span>
        </div>
        <div class="room-detail__info-row">
          <span class="room-detail__label">成员</span>
          <span class="room-detail__value">{{ currentRoom.member_count }} / {{ currentRoom.max_members }}</span>
        </div>
        <div class="room-detail__info-row">
          <span class="room-detail__label">创建者</span>
          <span class="room-detail__value">{{ currentRoom.owner.username }}</span>
        </div>
      </section>

      <!-- 成员列表 -->
      <section class="room-detail__section">
        <h4 class="room-detail__section-title">
          成员列表
          <span class="room-detail__count">{{ members.length }}</span>
        </h4>
        <div class="room-detail__members">
          <div v-for="m in members" :key="m.user_id" class="room-detail__member">
            <div class="room-detail__member-avatar">
              {{ m.username.charAt(0).toUpperCase() }}
            </div>
            <div class="room-detail__member-info">
              <span class="room-detail__member-name">{{ m.username }}</span>
              <span class="room-detail__member-role">{{ roleLabel(m.role) }}</span>
            </div>
            <span class="room-detail__member-status" :class="`status--${m.user_status}`">
              {{ statusText(m.user_status) }}
            </span>
          </div>
          <div v-if="members.length === 0" class="room-detail__members-empty">
            暂无成员
          </div>
        </div>
      </section>

      <!-- 操作 -->
      <section class="room-detail__section">
        <button
          class="room-detail__leave-btn"
          @click="emit('leave', roomId)"
        >
          离开聊天室
        </button>
      </section>
    </div>

    <div v-else class="room-detail__loading">
      加载中...
    </div>
  </div>
</template>

<style scoped>
.room-detail {
  width: 280px;
  border-left: 1px solid var(--color-border, #e0e0e0);
  background: var(--color-white, #fff);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  overflow: hidden;
}

.room-detail__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 16px;
  border-bottom: 1px solid var(--color-border, #e0e0e0);
  flex-shrink: 0;
}

.room-detail__title {
  font-size: 15px;
  font-weight: 600;
  margin: 0;
}

.room-detail__close {
  background: none;
  border: none;
  font-size: 16px;
  cursor: pointer;
  color: var(--color-text-tertiary, #999);
  padding: 2px;
}

.room-detail__body {
  flex: 1;
  overflow-y: auto;
}

.room-detail__section {
  padding: 14px 16px;
  border-bottom: 1px solid var(--color-border, #f0f0f0);
}

.room-detail__section-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-tertiary, #999);
  text-transform: uppercase;
  margin: 0 0 10px;
  display: flex;
  align-items: center;
  gap: 6px;
}

.room-detail__count {
  font-size: 11px;
  background: var(--color-background, #f5f5f5);
  padding: 1px 6px;
  border-radius: 8px;
  color: var(--color-text-secondary, #666);
}

.room-detail__info-row {
  display: flex;
  justify-content: space-between;
  padding: 6px 0;
  font-size: 13px;
}

.room-detail__label {
  color: var(--color-text-secondary, #666);
}

.room-detail__value {
  color: var(--color-text, #333);
  text-align: right;
  max-width: 60%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.room-detail__members {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.room-detail__member {
  display: flex;
  align-items: center;
  gap: 10px;
}

.room-detail__member-avatar {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: var(--color-primary, #2080f0);
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 13px;
  font-weight: 600;
  flex-shrink: 0;
}

.room-detail__member-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
}

.room-detail__member-name {
  font-size: 13px;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.room-detail__member-role {
  font-size: 11px;
  color: var(--color-text-tertiary, #999);
}

.room-detail__member-status {
  font-size: 11px;
  flex-shrink: 0;
}

.status--online { color: var(--color-success, #18a058); }
.status--away { color: var(--color-warning, #f0a020); }
.status--busy { color: var(--color-error, #d03050); }
.status--offline { color: var(--color-text-tertiary, #999); }

.room-detail__members-empty {
  text-align: center;
  color: var(--color-text-tertiary, #999);
  font-size: 12px;
  padding: 12px;
}

.room-detail__loading {
  display: flex;
  justify-content: center;
  padding: 32px;
  color: var(--color-text-tertiary, #999);
  font-size: 13px;
}

.room-detail__leave-btn {
  width: 100%;
  padding: 8px;
  border: 1px solid var(--color-error, #d03050);
  border-radius: 6px;
  background: transparent;
  color: var(--color-error, #d03050);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.15s;
}

.room-detail__leave-btn:hover {
  background: var(--color-error, #d03050);
  color: #fff;
}
</style>
