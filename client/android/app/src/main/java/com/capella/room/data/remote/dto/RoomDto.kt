package com.capella.room.data.remote.dto

import com.squareup.moshi.Json
import com.squareup.moshi.JsonClass

@JsonClass(generateAdapter = false)
data class RoomDto(
    @Json(name = "id") val id: String,
    @Json(name = "name") val name: String,
    @Json(name = "description") val description: String? = null,
    @Json(name = "owner") val owner: UserInfo? = null,
    @Json(name = "is_private") val isPrivate: Boolean = false,
    @Json(name = "max_members") val maxMembers: Int = 100,
    @Json(name = "member_count") val memberCount: Int = 0,
    @Json(name = "unread_count") val unreadCount: Int = 0,
    @Json(name = "last_message") val lastMessage: LastMessageDto? = null,
    @Json(name = "created_at") val createdAt: String = "",
    @Json(name = "updated_at") val updatedAt: String? = null
)

@JsonClass(generateAdapter = false)
data class LastMessageDto(
    @Json(name = "id") val id: String,
    @Json(name = "content") val content: String,
    @Json(name = "sender_name") val senderName: String,
    @Json(name = "created_at") val createdAt: String
)

@JsonClass(generateAdapter = true)
data class MessageListDto(
    @Json(name = "messages") val messages: List<MessageDto>,
    @Json(name = "total") val total: Int = 0,
    @Json(name = "has_more") val hasMore: Boolean = false
)

@JsonClass(generateAdapter = true)
data class MessageDto(
    @Json(name = "id") val id: String,
    @Json(name = "room_id") val roomId: String,
    @Json(name = "sender") val sender: UserInfo,
    @Json(name = "content") val content: String,
    @Json(name = "message_type") val messageType: String = "text",
    @Json(name = "reply_to") val replyTo: String? = null,
    @Json(name = "reply_to_message") val replyToMessage: Any? = null,
    @Json(name = "is_deleted") val isDeleted: Boolean = false,
    @Json(name = "created_at") val createdAt: String,
    @Json(name = "edit_count") val editCount: Int = 0,
    @Json(name = "edited_at") val editedAt: String? = null
)

@JsonClass(generateAdapter = true)
data class CreateRoomRequest(
    @Json(name = "name") val name: String,
    @Json(name = "description") val description: String? = null,
    @Json(name = "is_private") val isPrivate: Boolean,
    @Json(name = "max_members") val maxMembers: Int? = null
)

// ── Direct / DM room ──

@JsonClass(generateAdapter = true)
data class DirectRoomRequest(
    @Json(name = "target_user_id") val targetUserId: String
)

@JsonClass(generateAdapter = false)
data class DirectRoomDto(
    @Json(name = "id") val id: String,
    @Json(name = "name") val name: String,
    @Json(name = "target_user") val targetUser: UserInfo,
    @Json(name = "created_at") val createdAt: String = ""
)

// ── Room members ──

@JsonClass(generateAdapter = false)
data class RoomMemberDto(
    @Json(name = "room_id") val roomId: String,
    @Json(name = "user_id") val userId: String,
    @Json(name = "role") val role: String,
    @Json(name = "joined_at") val joinedAt: String,
    @Json(name = "username") val username: String,
    @Json(name = "email") val email: String? = null,
    @Json(name = "avatar_url") val avatarUrl: String? = null,
    @Json(name = "user_status") val userStatus: String = "offline"
)

@JsonClass(generateAdapter = true)
data class UpdateRoleRequest(
    @Json(name = "role") val role: String
)

// ── Invitations ──

@JsonClass(generateAdapter = true)
data class JoinByInviteRequest(
    @Json(name = "invite_code") val inviteCode: String
)

@JsonClass(generateAdapter = true)
data class CreateInvitationRequest(
    @Json(name = "expires_in_hours") val expiresInHours: Int? = null,
    @Json(name = "max_uses") val maxUses: Int? = null
)

@JsonClass(generateAdapter = false)
data class InvitationDto(
    @Json(name = "id") val id: String,
    @Json(name = "room_id") val roomId: String,
    @Json(name = "inviter") val inviter: UserInfo,
    @Json(name = "invite_code") val inviteCode: String,
    @Json(name = "expires_at") val expiresAt: String? = null,
    @Json(name = "max_uses") val maxUses: Int? = null,
    @Json(name = "used_count") val usedCount: Int = 0,
    @Json(name = "is_active") val isActive: Boolean = true,
    @Json(name = "created_at") val createdAt: String = ""
)

@JsonClass(generateAdapter = false)
data class InviteValidationDto(
    @Json(name = "valid") val valid: Boolean,
    @Json(name = "room_id") val roomId: String? = null,
    @Json(name = "expires_at") val expiresAt: String? = null,
    @Json(name = "max_uses") val maxUses: Int? = null,
    @Json(name = "used_count") val usedCount: Int? = null
)
