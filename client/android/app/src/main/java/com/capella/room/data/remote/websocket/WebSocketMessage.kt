package com.capella.room.data.remote.websocket

import com.squareup.moshi.Json
import com.squareup.moshi.JsonClass

/**
 * WebSocket 消息基类
 * 所有 WebSocket 消息都包含 type 和 payload 字段
 */
@JsonClass(generateAdapter = false)
data class WebSocketMessage(
    @Json(name = "type") val type: String,
    @Json(name = "payload") val payload: Any? = null
)

// ==================== 连接管理消息 ====================

/**
 * 认证请求
 */
@JsonClass(generateAdapter = true)
data class AuthPayload(
    @Json(name = "token") val token: String
)

/**
 * 认证结果响应
 */
@JsonClass(generateAdapter = true)
data class AuthResultPayload(
    @Json(name = "success") val success: Boolean,
    @Json(name = "message") val message: String
)

/**
 * 重连请求
 */
@JsonClass(generateAdapter = true)
data class ReconnectPayload(
    @Json(name = "token") val token: String,
    @Json(name = "last_disconnect_at") val lastDisconnectAt: String? = null
)

/**
 * 重连结果响应
 */
@JsonClass(generateAdapter = true)
data class ReconnectResultPayload(
    @Json(name = "success") val success: Boolean,
    @Json(name = "message") val message: String,
    @Json(name = "rooms_to_rejoin") val roomsToRejoin: List<String>? = null
)

/**
 * 会话恢复完成
 */
@JsonClass(generateAdapter = true)
data class SessionRestoredPayload(
    @Json(name = "restored_at") val restoredAt: String,
    @Json(name = "rooms_restored") val roomsRestored: Int,
    @Json(name = "total_unread") val totalUnread: Int
)

/**
 * 错误消息
 */
@JsonClass(generateAdapter = true)
data class ErrorPayload(
    @Json(name = "code") val code: String,
    @Json(name = "message") val message: String
)

// ==================== 房间管理消息 ====================

/**
 * 加入房间请求
 */
@JsonClass(generateAdapter = true)
data class JoinRoomPayload(
    @Json(name = "room_id") val roomId: String
)

/**
 * 离开房间请求
 */
@JsonClass(generateAdapter = true)
data class LeaveRoomPayload(
    @Json(name = "room_id") val roomId: String
)

/**
 * 加入房间成功响应
 */
@JsonClass(generateAdapter = true)
data class RoomJoinedPayload(
    @Json(name = "room_id") val roomId: String,
    @Json(name = "user_id") val userId: String,
    @Json(name = "username") val username: String
)

/**
 * 离开房间成功响应
 */
@JsonClass(generateAdapter = true)
data class RoomLeftPayload(
    @Json(name = "room_id") val roomId: String,
    @Json(name = "user_id") val userId: String,
    @Json(name = "username") val username: String
)

/**
 * 用户加入房间通知
 */
@JsonClass(generateAdapter = true)
data class UserJoinedPayload(
    @Json(name = "room_id") val roomId: String,
    @Json(name = "user_id") val userId: String,
    @Json(name = "username") val username: String
)

/**
 * 用户离开房间通知
 */
@JsonClass(generateAdapter = true)
data class UserLeftPayload(
    @Json(name = "room_id") val roomId: String,
    @Json(name = "user_id") val userId: String,
    @Json(name = "username") val username: String
)

/**
 * 在线用户信息
 */
@JsonClass(generateAdapter = true)
data class OnlineUserInfo(
    @Json(name = "id") val id: String,
    @Json(name = "username") val username: String,
    @Json(name = "avatar_url") val avatarUrl: String?,
    @Json(name = "status") val status: String
)

/**
 * 在线用户列表响应
 */
@JsonClass(generateAdapter = true)
data class OnlineUsersPayload(
    @Json(name = "room_id") val roomId: String,
    @Json(name = "users") val users: List<OnlineUserInfo>
)

/**
 * 房间消息摘要（用于房间列表实时更新）
 */
@JsonClass(generateAdapter = true)
data class RoomMessageSummaryPayload(
    @Json(name = "room_id") val roomId: String,
    @Json(name = "last_message") val lastMessage: LastMessageSummary?,
    @Json(name = "unread_count") val unreadCount: Int
)

/**
 * 最后一条消息摘要
 */
@JsonClass(generateAdapter = true)
data class LastMessageSummary(
    @Json(name = "id") val id: String,
    @Json(name = "content") val content: String,
    @Json(name = "sender_name") val senderName: String,
    @Json(name = "created_at") val createdAt: String
)

// ==================== 消息通信 ====================

/**
 * 发送聊天消息请求
 */
@JsonClass(generateAdapter = true)
data class ChatMessagePayload(
    @Json(name = "room_id") val roomId: String,
    @Json(name = "content") val content: String,
    @Json(name = "reply_to") val replyTo: String? = null
)

/**
 * 被回复消息信息
 */
@JsonClass(generateAdapter = true)
data class ReplyToMessageInfo(
    @Json(name = "id") val id: String,
    @Json(name = "sender_id") val senderId: String,
    @Json(name = "sender_name") val senderName: String,
    @Json(name = "content") val content: String,
    @Json(name = "created_at") val createdAt: String
)

/**
 * 新消息通知
 */
@JsonClass(generateAdapter = true)
data class NewMessagePayload(
    @Json(name = "message_id") val messageId: String,
    @Json(name = "room_id") val roomId: String,
    @Json(name = "sender_id") val senderId: String,
    @Json(name = "sender_name") val senderName: String,
    @Json(name = "content") val content: String,
    @Json(name = "reply_to") val replyTo: String?,
    @Json(name = "reply_to_message") val replyToMessage: ReplyToMessageInfo?,
    @Json(name = "created_at") val createdAt: String
)

/**
 * 正在输入状态请求
 */
@JsonClass(generateAdapter = true)
data class TypingPayload(
    @Json(name = "room_id") val roomId: String
)

/**
 * 停止输入状态请求
 */
@JsonClass(generateAdapter = true)
data class StopTypingPayload(
    @Json(name = "room_id") val roomId: String
)

/**
 * 用户正在输入通知
 */
@JsonClass(generateAdapter = true)
data class UserTypingPayload(
    @Json(name = "room_id") val roomId: String,
    @Json(name = "user_id") val userId: String,
    @Json(name = "username") val username: String
)

/**
 * 用户停止输入通知
 */
@JsonClass(generateAdapter = true)
data class UserStopTypingPayload(
    @Json(name = "room_id") val roomId: String,
    @Json(name = "user_id") val userId: String,
    @Json(name = "username") val username: String
)

/**
 * 消息已读确认请求
 */
@JsonClass(generateAdapter = true)
data class MessageReadPayload(
    @Json(name = "message_id") val messageId: String
)

/**
 * 消息已读回执
 */
@JsonClass(generateAdapter = true)
data class MessageReadReceiptPayload(
    @Json(name = "message_id") val messageId: String,
    @Json(name = "user_id") val userId: String
)

/**
 * 编辑消息请求
 */
@JsonClass(generateAdapter = true)
data class EditMessagePayload(
    @Json(name = "message_id") val messageId: String,
    @Json(name = "new_content") val newContent: String
)

/**
 * 消息已编辑通知
 */
@JsonClass(generateAdapter = true)
data class MessageEditedPayload(
    @Json(name = "message_id") val messageId: String,
    @Json(name = "new_content") val newContent: String,
    @Json(name = "edited_at") val editedAt: String
)

/**
 * 删除消息请求
 */
@JsonClass(generateAdapter = true)
data class DeleteMessagePayload(
    @Json(name = "message_id") val messageId: String
)

/**
 * 消息已删除通知
 */
@JsonClass(generateAdapter = true)
data class MessageDeletedPayload(
    @Json(name = "message_id") val messageId: String
)

/**
 * 获取离线消息请求
 */
@JsonClass(generateAdapter = true)
data class GetMissedMessagesPayload(
    @Json(name = "room_id") val roomId: String,
    @Json(name = "last_message_id") val lastMessageId: String?
)

/**
 * 离线消息响应
 */
@JsonClass(generateAdapter = true)
data class MissedMessagesPayload(
    @Json(name = "room_id") val roomId: String,
    @Json(name = "messages") val messages: List<NewMessagePayload>,
    @Json(name = "has_more") val hasMore: Boolean
)

// ==================== 用户状态 ====================

/**
 * 更新用户状态请求
 */
@JsonClass(generateAdapter = true)
data class UpdateStatusPayload(
    @Json(name = "status") val status: String
)

/**
 * 用户状态变更通知
 */
@JsonClass(generateAdapter = true)
data class UserStatusChangedPayload(
    @Json(name = "user_id") val userId: String,
    @Json(name = "username") val username: String,
    @Json(name = "status") val status: String
)

// ==================== 通知系统 ====================

/**
 * 私信通知
 */
@JsonClass(generateAdapter = true)
data class PrivateMessagePayload(
    @Json(name = "message_id") val messageId: String,
    @Json(name = "sender_id") val senderId: String,
    @Json(name = "sender_name") val senderName: String,
    @Json(name = "content") val content: String,
    @Json(name = "created_at") val createdAt: String
)

/**
 * @提及通知
 */
@JsonClass(generateAdapter = true)
data class MentionedPayload(
    @Json(name = "message_id") val messageId: String,
    @Json(name = "room_id") val roomId: String,
    @Json(name = "mentioned_by") val mentionedBy: String,
    @Json(name = "mentioned_by_name") val mentionedByName: String,
    @Json(name = "content_preview") val contentPreview: String,
    @Json(name = "created_at") val createdAt: String
)

/**
 * 房间邀请通知
 */
@JsonClass(generateAdapter = true)
data class RoomInvitationPayload(
    @Json(name = "invitation_id") val invitationId: String,
    @Json(name = "room_id") val roomId: String,
    @Json(name = "room_name") val roomName: String,
    @Json(name = "invited_by") val invitedBy: String,
    @Json(name = "invited_by_name") val invitedByName: String,
    @Json(name = "created_at") val createdAt: String
)

/**
 * 系统通知
 */
@JsonClass(generateAdapter = true)
data class SystemNotificationPayload(
    @Json(name = "notification_type") val notificationType: String,
    @Json(name = "title") val title: String?,
    @Json(name = "content") val content: String,
    @Json(name = "data") val data: Map<String, Any>?,
    @Json(name = "created_at") val createdAt: String
)

/**
 * 文件上传完成通知
 */
@JsonClass(generateAdapter = true)
data class FileUploadCompletePayload(
    @Json(name = "file_id") val fileId: String,
    @Json(name = "file_name") val fileName: String,
    @Json(name = "file_url") val fileUrl: String,
    @Json(name = "file_size") val fileSize: Long,
    @Json(name = "uploaded_at") val uploadedAt: String
)
