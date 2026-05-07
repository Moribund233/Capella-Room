<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Plus } from 'lucide-vue-next'
import { storeToRefs } from 'pinia'
import { useInvitationStore } from '@/stores/invitation'
import InvitationCard from './InvitationCard.vue'
import CreateInvitationModal from './CreateInvitationModal.vue'
import InviteLinkShareModal from './InviteLinkShareModal.vue'
import type { RoomInvitation } from '@/types/invitation'

const props = defineProps<{
  roomId: string
}>()

const invitationStore = useInvitationStore()
const { invitations, loading } = storeToRefs(invitationStore)

const showCreateModal = ref(false)
const showShareModal = ref(false)
const sharingInvitation = ref<RoomInvitation | null>(null)

async function handleRevoke(invitationId: string) {
  if (!confirm('确定要撤销此邀请码？撤销后该邀请码将失效。')) return
  await invitationStore.revokeInvitation(props.roomId, invitationId)
}

function handleShare(invitation: RoomInvitation) {
  sharingInvitation.value = invitation
  showShareModal.value = true
}

function handleCreated(invitation: RoomInvitation) {
  // 自动弹出分享
  sharingInvitation.value = invitation
  showShareModal.value = true
}

onMounted(() => {
  invitationStore.fetchInvitations(props.roomId)
})
</script>

<template>
  <div class="room-invitation-manager">
    <div class="room-invitation-manager__header">
      <h4>邀请管理</h4>
      <button class="create-btn" @click="showCreateModal = true">
        <Plus :size="16" />
        <span>创建邀请</span>
      </button>
    </div>

    <div v-if="loading" class="room-invitation-manager__loading">加载中...</div>

    <div v-else-if="invitations.length === 0" class="room-invitation-manager__empty">
      暂无邀请码，点击上方按钮创建
    </div>

    <div v-else class="room-invitation-manager__list">
      <InvitationCard
        v-for="invitation in invitations"
        :key="invitation.id"
        :invitation="invitation"
        @revoke="handleRevoke"
        @share="handleShare"
      />
    </div>

    <CreateInvitationModal
      :show="showCreateModal"
      :room-id="roomId"
      @close="showCreateModal = false"
      @created="handleCreated"
    />

    <InviteLinkShareModal
      :show="showShareModal"
      :invitation="sharingInvitation"
      @close="showShareModal = false"
    />
  </div>
</template>

<style scoped>
.room-invitation-manager {
  padding: 16px;
}

.room-invitation-manager__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
}

.room-invitation-manager__header h4 {
  font-size: 15px;
  font-weight: 600;
  margin: 0;
  color: var(--color-text, #333);
}

.create-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 12px;
  border: 1px solid var(--color-primary, #2080f0);
  border-radius: 6px;
  background: transparent;
  color: var(--color-primary, #2080f0);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--duration-fast, 0.15s);
}

.create-btn:hover {
  background: var(--color-primary, #2080f0);
  color: white;
}

.room-invitation-manager__loading,
.room-invitation-manager__empty {
  text-align: center;
  padding: 24px;
  color: var(--color-text-tertiary, #999);
  font-size: 13px;
}

.room-invitation-manager__list {
  display: flex;
  flex-direction: column;
}
</style>
