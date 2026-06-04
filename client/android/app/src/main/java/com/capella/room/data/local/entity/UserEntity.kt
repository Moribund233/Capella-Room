package com.capella.room.data.local.entity

import androidx.room.ColumnInfo
import androidx.room.Entity
import androidx.room.Index
import androidx.room.PrimaryKey

/**
 * 用户实体类
 * 用于本地数据库存储用户信息
 */
@Entity(
    tableName = "users",
    indices = [
        Index(value = ["username"]),
        Index(value = ["is_current_user"])
    ]
)
data class UserEntity(
    @PrimaryKey
    @ColumnInfo(name = "id")
    val id: String,

    @ColumnInfo(name = "username")
    val username: String,

    @ColumnInfo(name = "avatar_url")
    val avatarUrl: String? = null,

    @ColumnInfo(name = "bio")
    val bio: String? = null,

    @ColumnInfo(name = "is_current_user")
    val isCurrentUser: Boolean = false,

    @ColumnInfo(name = "last_seen_at")
    val lastSeenAt: String? = null,

    @ColumnInfo(name = "local_updated_at")
    val localUpdatedAt: Long = System.currentTimeMillis()
)

/**
 * 当前登录用户信息实体
 * 单例表，只存储一条记录
 */
@Entity(tableName = "current_user")
data class CurrentUserEntity(
    @PrimaryKey
    @ColumnInfo(name = "id")
    val id: String = "current_user",

    @ColumnInfo(name = "user_id")
    val userId: String,

    @ColumnInfo(name = "username")
    val username: String,

    @ColumnInfo(name = "access_token")
    val accessToken: String,

    @ColumnInfo(name = "refresh_token")
    val refreshToken: String,

    @ColumnInfo(name = "avatar_url")
    val avatarUrl: String? = null,

    @ColumnInfo(name = "login_at")
    val loginAt: Long = System.currentTimeMillis()
)
