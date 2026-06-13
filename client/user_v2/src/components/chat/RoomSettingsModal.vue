<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoomStore } from '@/stores/room'
import { useInvitationStore } from '@/stores/invitation'
import { useAuthStore } from '@/stores/auth'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Close, CopyDocument } from '@element-plus/icons-vue'
import type { Room } from '@/types/room'

const { t } = useI18n()
const roomStore = useRoomStore()
const invitationStore = useInvitationStore()
const authStore = useAuthStore()

const props = defineProps<{
  room: Room
}>()

const emit = defineEmits<{
  close: []
}>()

const activeTab = ref<'general' | 'members' | 'invitations'>('general')

// ── General ──
const editName = ref(props.room.name)
const editDesc = ref(props.room.description || '')
const saving = ref(false)

// ── Members ──
const members = ref(roomStore.members)

// ── Invitations ──
const newInviteExpires = ref<number | null>(null)
const newInviteMaxUses = ref<number | null>(null)
const creatingInvite = ref(false)

onMounted(() => {
  invitationStore.fetchInvitations(props.room.id)
})

const isOwner = authStore.user?.id === props.room.owner.id
const isAdmin = isOwner || roomStore.members.some(
  (m) => m.user_id === authStore.user?.id && m.role === 'admin',
)

async function handleSave() {
  saving.value = true
  const ok = await roomStore.updateRoom(props.room.id, {
    name: editName.value.trim(),
    description: editDesc.value.trim() || undefined,
  })
  saving.value = false
  if (ok) {
    ElMessage.success('Room updated')
  } else {
    ElMessage.error('Failed to update room')
  }
}

async function handleDelete() {
  try {
    await ElMessageBox.confirm(
      'Are you sure you want to permanently delete this room? This action cannot be undone.',
      'Delete Room',
      { confirmButtonText: 'Delete', cancelButtonText: 'Cancel', type: 'warning' },
    )
    const ok = await roomStore.deleteRoom(props.room.id)
    if (ok) {
      ElMessage.success('Room deleted')
      emit('close')
    } else {
      ElMessage.error('Failed to delete room')
    }
  } catch { /* cancelled */ }
}

async function handleKickMember(userId: string, username: string) {
  try {
    await ElMessageBox.confirm(
      `Remove ${username} from this room?`,
      'Kick Member',
      { confirmButtonText: 'Remove', cancelButtonText: 'Cancel', type: 'warning' },
    )
    const ok = await roomStore.kickMember(props.room.id, userId)
    if (ok) {
      members.value = members.value.filter((m) => m.user_id !== userId)
      ElMessage.success(`${username} removed`)
    } else {
      ElMessage.error('Failed to remove member')
    }
  } catch { /* cancelled */ }
}

async function handleToggleAdmin(userId: string, currentRole: string) {
  const newRole = currentRole === 'admin' ? 'member' : 'admin'
  const ok = await roomStore.setMemberRole(props.room.id, userId, newRole)
  if (ok) {
    const m = members.value.find((m) => m.user_id === userId)
    if (m) m.role = newRole
    ElMessage.success(`Role updated to ${newRole}`)
  } else {
    ElMessage.error('Failed to update role')
  }
}

async function handleCreateInvitation() {
  creatingInvite.value = true
  const inv = await invitationStore.createInvitation(props.room.id, {
    expires_in_hours: newInviteExpires.value,
    max_uses: newInviteMaxUses.value,
  })
  creatingInvite.value = false
  if (inv) {
    ElMessage.success('Invitation created')
  } else {
    ElMessage.error(invitationStore.error || 'Failed to create invitation')
  }
}

async function handleRevokeInvitation(invitationId: string) {
  const ok = await invitationStore.revokeInvitation(props.room.id, invitationId)
  if (ok) {
    ElMessage.success('Invitation revoked')
  } else {
    ElMessage.error('Failed to revoke invitation')
  }
}

function copyInviteCode(code: string) {
  navigator.clipboard.writeText(`${window.location.origin}/invite/${code}`)
  ElMessage.success('Invite link copied')
}

function getDotColor(status: string): string {
  switch (status) {
    case 'online': return 'var(--accent-green)'
    case 'away': return 'var(--accent-orange)'
    case 'busy': return 'var(--accent-pink)'
    default: return 'var(--muted)'
  }
}
</script>

<template>
  <div class="settings-overlay" @click.self="emit('close')">
    <div class="settings-modal">
      <div class="settings-header">
        <h3>{{ room.name }} — Settings</h3>
        <button class="settings-close" @click="emit('close')">
          <el-icon :size="18"><Close /></el-icon>
        </button>
      </div>

      <div class="settings-tabs">
        <button :class="{ active: activeTab === 'general' }" @click="activeTab = 'general'">General</button>
        <button :class="{ active: activeTab === 'members' }" @click="activeTab = 'members'">Members ({{ members.length }})</button>
        <button :class="{ active: activeTab === 'invitations' }" @click="activeTab = 'invitations'">Invitations</button>
      </div>

      <div class="settings-body">
        <!-- ─── General ─── -->
        <template v-if="activeTab === 'general'">
          <div class="form-group">
            <label>Room Name</label>
            <input v-model="editName" class="form-input" :disabled="!isAdmin" />
          </div>
          <div class="form-group">
            <label>Description</label>
            <textarea v-model="editDesc" class="form-input form-textarea" rows="3" :disabled="!isAdmin" />
          </div>
          <div v-if="isAdmin" class="form-actions">
            <button class="btn btn-primary" :disabled="saving" @click="handleSave">
              {{ saving ? 'Saving…' : 'Save' }}
            </button>
          </div>

          <div v-if="isOwner" class="danger-zone">
            <h4>Danger Zone</h4>
            <p>Permanently delete this room and all its messages.</p>
            <button class="btn btn-danger" @click="handleDelete">Delete Room</button>
          </div>
        </template>

        <!-- ─── Members ─── -->
        <template v-if="activeTab === 'members'">
          <div v-for="m in members" :key="m.user_id" class="member-row">
            <span class="member-dot" :style="{ background: getDotColor(m.user_status) }" />
            <span class="member-name">{{ m.username }}</span>
            <span class="member-role">{{ m.role }}</span>
            <div v-if="isAdmin && m.user_id !== authStore.user?.id" class="member-actions">
              <button
                class="btn-small"
                @click="handleToggleAdmin(m.user_id, m.role)"
              >
                {{ m.role === 'admin' ? 'Demote' : 'Make admin' }}
              </button>
              <button
                class="btn-small btn-small--danger"
                @click="handleKickMember(m.user_id, m.username)"
              >
                Kick
              </button>
            </div>
          </div>
        </template>

        <!-- ─── Invitations ─── -->
        <template v-if="activeTab === 'invitations'">
          <div v-if="isAdmin" class="create-invite">
            <div class="invite-fields">
              <input v-model="newInviteExpires" class="form-input invite-input" placeholder="Expires in hours (empty = never)" type="number" min="1" />
              <input v-model="newInviteMaxUses" class="form-input invite-input" placeholder="Max uses (empty = unlimited)" type="number" min="1" />
            </div>
            <button class="btn btn-primary" :disabled="creatingInvite" @click="handleCreateInvitation">
              {{ creatingInvite ? 'Creating…' : 'Create Invitation' }}
            </button>
          </div>

          <div v-if="invitationStore.invitations.length === 0" class="empty-state">
            No invitations yet
          </div>

          <div v-for="inv in invitationStore.invitations" :key="inv.id" class="invite-card">
            <div class="invite-code">
              <code>{{ inv.invite_code }}</code>
              <button class="btn-icon" title="Copy link" @click="copyInviteCode(inv.invite_code)">
                <el-icon :size="16"><CopyDocument /></el-icon>
              </button>
            </div>
            <div class="invite-meta">
              {{ inv.used_count }}/{{ inv.max_uses || '∞' }} uses ·
              {{ inv.expires_at ? new Date(inv.expires_at).toLocaleDateString() : 'never expires' }}
            </div>
            <div v-if="inv.is_active" class="invite-actions">
              <button class="btn-small btn-small--danger" @click="handleRevokeInvitation(inv.id)">
                Revoke
              </button>
            </div>
            <div v-else class="invite-status">Revoked</div>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">
.settings-overlay {
  position: fixed;
  inset: 0;
  z-index: 500;
  background: rgba(0, 0, 0, 0.4);
  display: grid;
  place-items: center;
}

.settings-modal {
  width: min(560px, 92vw);
  max-height: 80vh;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.settings-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border);

  h3 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
  }
}

.settings-close {
  background: none;
  border: none;
  color: var(--muted);
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  display: grid;
  place-items: center;

  &:hover {
    color: var(--fg);
    background: var(--message-hover);
  }
}

.settings-tabs {
  display: flex;
  gap: 0;
  border-bottom: 1px solid var(--border);

  button {
    flex: 1;
    background: none;
    border: none;
    border-bottom: 2px solid transparent;
    color: var(--muted);
    font-size: 13px;
    font-weight: 500;
    padding: 10px 16px;
    cursor: pointer;
    transition: color 0.15s, border-color 0.15s;

    &.active {
      color: var(--accent);
      border-bottom-color: var(--accent);
    }

    &:hover:not(.active) {
      color: var(--fg);
    }
  }
}

.settings-body {
  flex: 1;
  overflow-y: auto;
  padding: 16px 20px;
}

.form-group {
  margin-bottom: 16px;

  label {
    display: block;
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--muted);
    margin-bottom: 6px;
  }
}

.form-input {
  width: 100%;
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  color: var(--fg);
  font-size: 14px;
  font-family: inherit;
  padding: 8px 12px;
  outline: none;

  &:focus {
    border-color: var(--accent);
  }

  &:disabled {
    opacity: 0.5;
  }

  &::placeholder {
    color: var(--muted);
  }
}

.form-textarea {
  resize: vertical;
  min-height: 60px;
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  margin-top: 16px;
}

.btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 18px;
  border-radius: var(--radius);
  font-size: 14px;
  font-weight: 500;
  border: 1px solid transparent;
  cursor: pointer;
  transition: background 0.15s;

  &:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
}

.btn-primary {
  background: var(--accent);
  color: #fff;

  &:hover:not(:disabled) {
    background: color-mix(in oklch, var(--accent) 88%, black);
  }
}

.btn-danger {
  background: transparent;
  color: var(--accent-pink);
  border-color: var(--accent-pink);

  &:hover:not(:disabled) {
    background: color-mix(in oklch, var(--accent-pink) 15%, transparent);
  }
}

.danger-zone {
  margin-top: 32px;
  padding-top: 20px;
  border-top: 1px solid var(--border);

  h4 {
    margin: 0 0 4px;
    font-size: 14px;
    color: var(--accent-pink);
  }

  p {
    margin: 0 0 12px;
    font-size: 13px;
    color: var(--muted);
  }
}

// ─── Members ───
.member-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 0;
  border-bottom: 1px solid var(--border);

  &:last-child {
    border-bottom: none;
  }
}

.member-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.member-name {
  flex: 1;
  font-size: 14px;
}

.member-role {
  font-size: 11px;
  color: var(--muted);
  text-transform: uppercase;
  font-weight: 600;
  min-width: 40px;
}

.member-actions {
  display: flex;
  gap: 6px;
}

.btn-small {
  padding: 4px 10px;
  font-size: 12px;
  border-radius: 4px;
  border: 1px solid var(--border);
  background: transparent;
  color: var(--muted);
  cursor: pointer;
  transition: color 0.15s, border-color 0.15s;

  &:hover {
    color: var(--fg);
    border-color: var(--fg);
  }

  &--danger:hover {
    color: var(--accent-pink);
    border-color: var(--accent-pink);
  }
}

// ─── Invitations ───
.create-invite {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-bottom: 20px;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--border);
}

.invite-fields {
  display: flex;
  gap: 8px;
}

.invite-input {
  flex: 1;
}

.empty-state {
  text-align: center;
  padding: 24px 0;
  font-size: 14px;
  color: var(--muted);
  font-style: italic;
}

.invite-card {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 12px 0;
  border-bottom: 1px solid var(--border);

  &:last-child {
    border-bottom: none;
  }
}

.invite-code {
  display: flex;
  align-items: center;
  gap: 8px;

  code {
    font-size: 15px;
    font-weight: 700;
    letter-spacing: 1px;
    color: var(--fg);
    font-family: var(--font-mono);
  }
}

.btn-icon {
  background: none;
  border: none;
  color: var(--muted);
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  display: grid;
  place-items: center;

  &:hover {
    color: var(--accent);
    background: var(--message-hover);
  }
}

.invite-meta {
  font-size: 12px;
  color: var(--muted);
}

.invite-actions {
  display: flex;
  gap: 6px;
}

.invite-status {
  font-size: 12px;
  color: var(--muted);
  font-style: italic;
}
</style>
