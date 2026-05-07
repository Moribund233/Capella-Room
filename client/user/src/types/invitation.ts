/** 邀请状态 */
export type InvitationStatus = 'active' | 'expired' | 'revoked' | 'exhausted'

/** 房间邀请 */
export interface RoomInvitation {
  id: string
  room_id: string
  inviter: {
    id: string
    username: string
    avatar_url?: string
  }
  invite_code: string
  expires_at: string | null
  max_uses: number | null
  used_count: number
  is_active: boolean
  created_at: string
}

/** 创建邀请参数 */
export interface CreateInvitationData {
  expires_in_hours?: number | null
  max_uses?: number | null
}

/** 通过邀请码加入参数 */
export interface JoinByInviteData {
  code: string
}

/** 邀请码验证结果 */
export interface InviteValidationResult {
  valid: boolean
  room_name?: string
  room_id?: string
  message?: string
}
