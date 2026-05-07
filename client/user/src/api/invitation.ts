import httpClient from '@/services/http'
import type { ApiResponse } from '@/types/api'
import type {
  RoomInvitation,
  CreateInvitationData,
  InviteValidationResult,
} from '@/types/invitation'

export const invitationApi = {
  /** 创建邀请 */
  createInvitation(roomId: string, data: CreateInvitationData): Promise<ApiResponse<RoomInvitation>> {
    return httpClient.post(`/rooms/${roomId}/invitations`, data)
  },

  /** 获取房间的邀请列表 */
  getRoomInvitations(roomId: string): Promise<ApiResponse<RoomInvitation[]>> {
    return httpClient.get(`/rooms/${roomId}/invitations`)
  },

  /** 撤销邀请 */
  revokeInvitation(roomId: string, invitationId: string): Promise<ApiResponse<unknown>> {
    return httpClient.delete(`/rooms/${roomId}/invitations/${invitationId}`)
  },

  /** 通过邀请码加入房间 */
  joinByInviteCode(code: string): Promise<ApiResponse<{ room_id: string; room_name: string }>> {
    return httpClient.post('/rooms/join-by-invite', { invite_code: code })
  },

  /** 验证邀请码 */
  validateInviteCode(code: string): Promise<ApiResponse<InviteValidationResult>> {
    return httpClient.get('/rooms/validate-invite', { params: { invite_code: code } })
  },
}
