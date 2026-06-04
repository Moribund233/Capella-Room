package com.capella.room.data.local.entity

import androidx.room.ColumnInfo
import androidx.room.Entity
import androidx.room.Index
import androidx.room.PrimaryKey

/**
 * 消息实体类
 * 用于本地数据库存储聊天消息
 */
@Entity(
    tableName = "messages",
    indices = [
        Index(value = ["room_id", "created_at"]),
        Index(value = ["sync_status"])
    ]
)
data class MessageEntity(
    @PrimaryKey
    @ColumnInfo(name = "id")
    val id: String,

    @ColumnInfo(name = "room_id")
    val roomId: String,

    @ColumnInfo(name = "sender_id")
    val senderId: String,

    @ColumnInfo(name = "sender_name")
    val senderName: String,

    @ColumnInfo(name = "sender_avatar_url")
    val senderAvatarUrl: String? = null,

    @ColumnInfo(name = "content")
    val content: String,

    @ColumnInfo(name = "message_type")
    val messageType: String = "text",

    @ColumnInfo(name = "reply_to")
    val replyTo: String? = null,

    @ColumnInfo(name = "reply_to_message")
    val replyToMessage: String? = null, // JSON 序列化的回复消息

    @ColumnInfo(name = "is_deleted")
    val isDeleted: Boolean = false,

    @ColumnInfo(name = "created_at")
    val createdAt: String,

    @ColumnInfo(name = "edit_count")
    val editCount: Int = 0,

    @ColumnInfo(name = "edited_at")
    val editedAt: String? = null,

    // 本地状态字段
    @ColumnInfo(name = "sync_status")
    val syncStatus: SyncStatus = SyncStatus.SYNCED,

    @ColumnInfo(name = "local_created_at")
    val localCreatedAt: Long = System.currentTimeMillis(),

    @ColumnInfo(name = "pending_content")
    val pendingContent: String? = null // 待同步的编辑内容
)

/**
 * 同步状态枚举
 */
enum class SyncStatus {
    SYNCED,      // 已同步
    PENDING,     // 待发送（新消息）
    SENDING,     // 发送中
    FAILED,      // 发送失败
    EDIT_PENDING,// 待同步编辑
    DELETE_PENDING // 待同步删除
}
