package com.capella.room.data.remote.dto

import com.squareup.moshi.Json
import com.squareup.moshi.JsonClass

// ── Auth request ──

@JsonClass(generateAdapter = true)
data class LoginRequest(
    @Json(name = "email") val email: String,
    @Json(name = "password") val password: String
)

@JsonClass(generateAdapter = true)
data class RegisterRequest(
    @Json(name = "username") val username: String,
    @Json(name = "email") val email: String,
    @Json(name = "password") val password: String
)

@JsonClass(generateAdapter = true)
data class RefreshRequest(
    @Json(name = "refresh_token") val refreshToken: String
)

// ── Auth response ──

@JsonClass(generateAdapter = false)
data class LoginData(
    @Json(name = "access_token") val accessToken: String,
    @Json(name = "refresh_token") val refreshToken: String,
    @Json(name = "expires_in") val expiresIn: Long,
    @Json(name = "token_type") val tokenType: String,
    @Json(name = "user") val user: UserDto
)

@JsonClass(generateAdapter = false)
data class TokenData(
    @Json(name = "access_token") val accessToken: String,
    @Json(name = "refresh_token") val refreshToken: String,
    @Json(name = "expires_in") val expiresIn: Long,
    @Json(name = "token_type") val tokenType: String
)

@JsonClass(generateAdapter = false)
data class UserDto(
    @Json(name = "id") val id: String,
    @Json(name = "username") val username: String,
    @Json(name = "email") val email: String,
    @Json(name = "avatar_url") val avatarUrl: String? = null,
    @Json(name = "status") val status: String = "offline",
    @Json(name = "role") val role: String = "user",
    @Json(name = "created_at") val createdAt: String = "",
    @Json(name = "updated_at") val updatedAt: String? = null
)

@JsonClass(generateAdapter = false)
data class UserInfo(
    @Json(name = "id") val id: String,
    @Json(name = "username") val username: String,
    @Json(name = "avatar_url") val avatarUrl: String? = null
)

// ── Profile stats ──
// 对应后端 UserStats: { joined_rooms, total_messages, online_hours }
@JsonClass(generateAdapter = false)
data class UserStatsDto(
    @Json(name = "joined_rooms") val joinedRooms: Long = 0,
    @Json(name = "total_messages") val totalMessages: Long = 0,
    @Json(name = "online_hours") val onlineHours: Long = 0
)

// ── User settings ──
// 后端 settings 是平铺的 Map<String, Any>，定义一个强类型壳方便 UI 使用
// 所有字段 nullable，缺失时使用默认值
@JsonClass(generateAdapter = false)
data class UserSettingsDto(
    @Json(name = "theme") val theme: String? = null,                  // "light" | "dark" | "system"
    @Json(name = "language") val language: String? = null,            // "zh-CN" | "en-US"
    @Json(name = "notification_enabled") val notificationEnabled: Boolean? = null,
    @Json(name = "sound_enabled") val soundEnabled: Boolean? = null,
    @Json(name = "desktop_notification") val desktopNotification: Boolean? = null,
    @Json(name = "email_notification") val emailNotification: Boolean? = null,
    @Json(name = "mention_notification") val mentionNotification: Boolean? = null,
    @Json(name = "show_online_status") val showOnlineStatus: Boolean? = null,
    @Json(name = "read_receipts_enabled") val readReceiptsEnabled: Boolean? = null,
    @Json(name = "typing_indicator_enabled") val typingIndicatorEnabled: Boolean? = null,
    @Json(name = "single_device_login") val singleDeviceLogin: Boolean? = null,
    @Json(name = "enter_to_send") val enterToSend: Boolean? = null
) {
    companion object {
        val DEFAULT = UserSettingsDto(
            theme = "dark",
            language = "zh-CN",
            notificationEnabled = true,
            soundEnabled = true,
            desktopNotification = true,
            emailNotification = false,
            mentionNotification = true,
            showOnlineStatus = true,
            readReceiptsEnabled = true,
            typingIndicatorEnabled = true,
            singleDeviceLogin = false,
            enterToSend = true
        )
    }
}

// ── Password change ──
@JsonClass(generateAdapter = true)
data class ChangePasswordRequest(
    @Json(name = "current_password") val currentPassword: String,
    @Json(name = "new_password") val newPassword: String
)

// ── Update profile ──
@JsonClass(generateAdapter = true)
data class UpdateProfileRequest(
    @Json(name = "username") val username: String? = null,
    @Json(name = "email") val email: String? = null,
    @Json(name = "avatar_url") val avatarUrl: String? = null,
    @Json(name = "bio") val bio: String? = null
)
