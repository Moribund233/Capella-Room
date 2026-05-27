import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invitationApi } from '@/api/invitation'
import { useAuthStore } from '@/stores/auth'
import type {
  RoomInvitation,
  CreateInvitationData,
  InviteValidationResult,
} from '@/types/invitation'

export const useInvitationStore = defineStore('invitation', () => {
  const invitations = ref<RoomInvitation[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  const getAuthStore = () => useAuthStore()

  /** 获取房间的邀请列表 */
  async function fetchInvitations(roomId: string) {
    const authStore = getAuthStore()
    if (!authStore.isAuthenticated) return

    loading.value = true
    error.value = null
    try {
      const res = await invitationApi.getRoomInvitations(roomId)
      if (res.success && res.data) {
        invitations.value = res.data
      }
    } catch (err) {
      error.value = '获取邀请列表失败'
      console.error('[InvitationStore] fetchInvitations error:', err)
    } finally {
      loading.value = false
    }
  }

  /** 创建邀请 */
  async function createInvitation(roomId: string, data: CreateInvitationData): Promise<RoomInvitation | null> {
    const authStore = getAuthStore()
    if (!authStore.isAuthenticated) return null

    error.value = null
    try {
      const res = await invitationApi.createInvitation(roomId, data)
      if (res.success && res.data) {
        invitations.value.unshift(res.data)
        return res.data
      }
      error.value = res.message || '创建邀请失败'
      return null
    } catch (err) {
      error.value = '创建邀请失败'
      console.error('[InvitationStore] createInvitation error:', err)
      return null
    }
  }

  /** 撤销邀请 */
  async function revokeInvitation(roomId: string, invitationId: string): Promise<boolean> {
    error.value = null
    try {
      const res = await invitationApi.revokeInvitation(roomId, invitationId)
      if (res.success) {
        const idx = invitations.value.findIndex((i) => i.id === invitationId)
        if (idx !== -1 && invitations.value[idx]) {
          invitations.value[idx].is_active = false
        }
        return true
      }
      error.value = res.message || '撤销邀请失败'
      return false
    } catch (err) {
      error.value = '撤销邀请失败'
      console.error('[InvitationStore] revokeInvitation error:', err)
      return false
    }
  }

  /** 通过邀请码加入 */
  async function joinByInviteCode(code: string): Promise<{ room_id: string; room_name: string } | null> {
    error.value = null
    try {
      const res = await invitationApi.joinByInviteCode(code)
      if (res.success && res.data) {
        return res.data
      }
      error.value = res.message || '加入房间失败'
      return null
    } catch (err) {
      error.value = '加入房间失败'
      console.error('[InvitationStore] joinByInviteCode error:', err)
      return null
    }
  }

  /** 验证邀请码 */
  async function validateInviteCode(code: string): Promise<InviteValidationResult | null> {
    error.value = null
    try {
      const res = await invitationApi.validateInviteCode(code)
      if (res.success && res.data) {
        return res.data
      }
      return null
    } catch (err) {
      error.value = '验证邀请码失败'
      console.error('[InvitationStore] validateInviteCode error:', err)
      return null
    }
  }

  function $reset() {
    invitations.value = []
    loading.value = false
    error.value = null
  }

  return {
    invitations,
    loading,
    error,
    fetchInvitations,
    createInvitation,
    revokeInvitation,
    joinByInviteCode,
    validateInviteCode,
    $reset,
  }
})
