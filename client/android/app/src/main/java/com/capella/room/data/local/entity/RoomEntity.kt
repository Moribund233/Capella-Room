package com.capella.room.data.local.entity

import androidx.room.ColumnInfo
import androidx.room.Entity
import androidx.room.Index
import androidx.room.PrimaryKey

/**
 * 聊天室实体类
 * 用于本地数据库存储房间信息
 */
@Entity(
    tableName = "rooms",
    indices = [
        Index(value = ["updated_at"]),
        Index(value = ["is_joined"])
    ]
)
data class RoomEntity(
    @PrimaryKey
    @ColumnInfo(name = "id")
    val id: String,

    @ColumnInfo(name = "name")
    val name: String,

    @ColumnInfo(name = "description")
    val description: String? = null,

    @ColumnInfo(name = "owner_id")
    val ownerId: String? = null,

    @ColumnInfo(name = "owner_name")
    val ownerName: String? = null,

    @ColumnInfo(name = "is_private")
    val isPrivate: Boolean = false,

    @ColumnInfo(name = "max_members")
    val maxMembers: Int = 100,

    @ColumnInfo(name = "member_count")
    val memberCount: Int = 0,

    @ColumnInfo(name = "unread_count")
    val unreadCount: Int = 0,

    @ColumnInfo(name = "last_message_id")
    val lastMessageId: String? = null,

    @ColumnInfo(name = "last_message_content")
    val lastMessageContent: String? = null,

    @ColumnInfo(name = "last_message_sender_name")
    val lastMessageSenderName: String? = null,

    @ColumnInfo(name = "last_message_time")
    val lastMessageTime: String? = null,

    @ColumnInfo(name = "created_at")
    val createdAt: String = "",

    @ColumnInfo(name = "updated_at")
    val updatedAt: String? = null,

    // 本地状态字段
    @ColumnInfo(name = "is_joined")
    val isJoined: Boolean = false,

    @ColumnInfo(name = "joined_at")
    val joinedAt: Long? = null,

    @ColumnInfo(name = "last_read_message_id")
    val lastReadMessageId: String? = null,

    @ColumnInfo(name = "is_favorite")
    val isFavorite: Boolean = false,

    @ColumnInfo(name = "local_updated_at")
    val localUpdatedAt: Long = System.currentTimeMillis()
)

/**
 * 房间成员实体类
 */
@Entity(
    tableName = "room_members",
    primaryKeys = ["room_id", "user_id"],
    indices = [
        Index(value = ["room_id"]),
        Index(value = ["user_id"])
    ]
)
data class RoomMemberEntity(
    @ColumnInfo(name = "room_id")
    val roomId: String,

    @ColumnInfo(name = "user_id")
    val userId: String,

    @ColumnInfo(name = "username")
    val username: String,

    @ColumnInfo(name = "avatar_url")
    val avatarUrl: String? = null,

    @ColumnInfo(name = "role")
    val role: String = "member", // owner, admin, member

    @ColumnInfo(name = "joined_at")
    val joinedAt: String? = null,

    @ColumnInfo(name = "local_updated_at")
    val localUpdatedAt: Long = System.currentTimeMillis()
)
