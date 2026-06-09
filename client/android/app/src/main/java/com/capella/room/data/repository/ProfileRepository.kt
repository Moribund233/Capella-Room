package com.capella.room.data.repository

import com.capella.room.data.remote.api.UserApi
import com.capella.room.data.remote.dto.UpdateProfileRequest
import com.capella.room.data.remote.dto.UserDto
import com.capella.room.data.remote.dto.UserSettingsDto
import com.capella.room.data.remote.dto.UserStatsDto
import javax.inject.Inject
import javax.inject.Singleton

/**
 * 用户资料 / 设置 仓库
 *
 * 封装对 `GET/PUT /api/v1/users/me`、
 * `GET /api/v1/users/me/stats`、
 * `GET/PATCH /api/v1/users/me/settings`、
 * `PUT /api/v1/users/me/password` 的访问。
 *
 * 任何网络/解析异常统一包装为 [ProfileException]，UI 层只需 catch 一种类型。
 */
@Singleton
class ProfileRepository @Inject constructor(
    private val userApi: UserApi
) {

    // ── Profile ──

    suspend fun getProfile(): Result<UserDto> = runCatching {
        val resp = userApi.getProfile()
        val body = resp.body()
        if (resp.isSuccessful && body?.success == true && body.data != null) {
            body.data!!
        } else {
            // 401 未授权，需要重新登录
            if (resp.code() == 401) {
                throw ProfileUnauthorizedException(body?.message ?: "登录已过期，请重新登录")
            }
            throw ProfileException(body?.message ?: "获取资料失败：${resp.code()}")
        }
    }.recoverCatching { throw it.asProfileException("获取资料失败") }

    suspend fun updateProfile(request: UpdateProfileRequest): Result<UserDto> = runCatching {
        val payload = buildMap<String, Any> {
            request.username?.let { put("username", it) }
            request.email?.let { put("email", it) }
            request.avatarUrl?.let { put("avatar_url", it) }
            request.bio?.let { put("bio", it) }
        }
        val resp = userApi.updateProfile(payload)
        val body = resp.body()
        if (resp.isSuccessful && body?.success == true && body.data != null) {
            body.data!!
        } else {
            throw ProfileException(body?.message ?: "更新资料失败：${resp.code()}")
        }
    }.recoverCatching { throw it.asProfileException("更新资料失败") }

    // ── Stats ──

    suspend fun getStats(): Result<UserStatsDto> = runCatching {
        val resp = userApi.getMyStats()
        val body = resp.body()
        if (resp.isSuccessful && body?.success == true) {
            body.data ?: UserStatsDto()
        } else {
            throw ProfileException(body?.message ?: "获取统计失败：${resp.code()}")
        }
    }.recoverCatching { throw it.asProfileException("获取统计失败") }

    // ── Settings ──

    suspend fun getSettings(): Result<UserSettingsDto> = runCatching {
        val resp = userApi.getSettings()
        val body = resp.body()
        if (resp.isSuccessful && body?.success == true) {
            // 后端返回 Map，强类型映射
            mapToSettings(body.data)
        } else {
            throw ProfileException(body?.message ?: "获取设置失败：${resp.code()}")
        }
    }.recoverCatching { throw it.asProfileException("获取设置失败") }

    suspend fun updateSettings(settings: UserSettingsDto): Result<UserSettingsDto> = runCatching {
        val payload = settingsToMap(settings)
        val resp = userApi.updateSettings(payload)
        val body = resp.body()
        if (resp.isSuccessful && body?.success == true) {
            mapToSettings(body.data)
        } else {
            throw ProfileException(body?.message ?: "更新设置失败：${resp.code()}")
        }
    }.recoverCatching { throw it.asProfileException("更新设置失败") }

    // ── Password ──

    suspend fun changePassword(current: String, new: String): Result<Unit> = runCatching {
        val resp = userApi.changePassword(
            mapOf("current_password" to current, "new_password" to new)
        )
        val body = resp.body()
        if (resp.isSuccessful && body?.success == true) {
            Unit
        } else {
            throw ProfileException(body?.message ?: "修改密码失败：${resp.code()}")
        }
    }.recoverCatching { throw it.asProfileException("修改密码失败") }

    // ── 映射 ──

    /**
     * 将后端松散的 Map 转为强类型 DTO。
     * 任何字段缺失都使用默认值（[UserSettingsDto.DEFAULT] 的对应项）。
     */
    private fun mapToSettings(map: Map<String, Any>?): UserSettingsDto {
        if (map.isNullOrEmpty()) return UserSettingsDto.DEFAULT
        val def = UserSettingsDto.DEFAULT
        return UserSettingsDto(
            theme = map["theme"]?.toString() ?: def.theme,
            language = map["language"]?.toString() ?: def.language,
            notificationEnabled = (map["notification_enabled"] as? Boolean) ?: def.notificationEnabled,
            soundEnabled = (map["sound_enabled"] as? Boolean) ?: def.soundEnabled,
            desktopNotification = (map["desktop_notification"] as? Boolean) ?: def.desktopNotification,
            emailNotification = (map["email_notification"] as? Boolean) ?: def.emailNotification,
            mentionNotification = (map["mention_notification"] as? Boolean) ?: def.mentionNotification,
            showOnlineStatus = (map["show_online_status"] as? Boolean) ?: def.showOnlineStatus,
            readReceiptsEnabled = (map["read_receipts_enabled"] as? Boolean) ?: def.readReceiptsEnabled,
            typingIndicatorEnabled = (map["typing_indicator_enabled"] as? Boolean) ?: def.typingIndicatorEnabled,
            singleDeviceLogin = (map["single_device_login"] as? Boolean) ?: def.singleDeviceLogin,
            enterToSend = (map["enter_to_send"] as? Boolean) ?: def.enterToSend
        )
    }

    /**
     * 将强类型 DTO 转为 Map 发送至 PATCH。
     */
    private fun settingsToMap(s: UserSettingsDto): Map<String, Any> = buildMap {
        s.theme?.let { put("theme", it) }
        s.language?.let { put("language", it) }
        s.notificationEnabled?.let { put("notification_enabled", it) }
        s.soundEnabled?.let { put("sound_enabled", it) }
        s.desktopNotification?.let { put("desktop_notification", it) }
        s.emailNotification?.let { put("email_notification", it) }
        s.mentionNotification?.let { put("mention_notification", it) }
        s.showOnlineStatus?.let { put("show_online_status", it) }
        s.readReceiptsEnabled?.let { put("read_receipts_enabled", it) }
        s.typingIndicatorEnabled?.let { put("typing_indicator_enabled", it) }
        s.singleDeviceLogin?.let { put("single_device_login", it) }
        s.enterToSend?.let { put("enter_to_send", it) }
    }

    private fun Throwable.asProfileException(defaultMessage: String): Throwable =
        when (this) {
            is ProfileUnauthorizedException -> this
            is ProfileException -> this
            else -> ProfileException("$defaultMessage：${localizedMessage ?: "未知错误"}")
        }
}

class ProfileException(message: String) : Exception(message)

/**
 * 401 未授权异常，需要重新登录
 */
class ProfileUnauthorizedException(message: String) : Exception(message)
