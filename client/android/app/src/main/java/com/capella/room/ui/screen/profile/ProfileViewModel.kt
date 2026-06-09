package com.capella.room.ui.screen.profile

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.capella.room.data.remote.dto.UpdateProfileRequest
import com.capella.room.data.remote.dto.UserDto
import com.capella.room.data.remote.dto.UserSettingsDto
import com.capella.room.data.remote.dto.UserStatsDto
import com.capella.room.data.repository.AuthRepository
import com.capella.room.data.repository.LocalUserRepository
import com.capella.room.data.repository.ProfileException
import com.capella.room.data.repository.ProfileRepository
import com.capella.room.data.repository.ProfileUnauthorizedException
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import javax.inject.Inject

/**
 * Profile 页面状态机
 *
 * 加载阶段：`Init -> LoadingProfile -> RefreshingSettings -> Ready`
 * 错误阶段：任意状态可转入 `Error(message)`，UI 展示重试入口。
 */
sealed interface ProfileUiState {
    data object Loading : ProfileUiState
    data class Error(val message: String) : ProfileUiState
    data class Ready(
        val profile: UserDto,
        val stats: UserStatsDto,
        val settings: UserSettingsDto
    ) : ProfileUiState
}

/**
 * 一次性事件（Snackbar / Toast / 导航）
 */
sealed interface ProfileEvent {
    data class Toast(val message: String) : ProfileEvent
    data object Logout : ProfileEvent
    data class Error(val message: String) : ProfileEvent
}

@HiltViewModel
class ProfileViewModel @Inject constructor(
    private val profileRepository: ProfileRepository,
    private val authRepository: AuthRepository,
    private val localUserRepository: LocalUserRepository
) : ViewModel() {

    private val _uiState = MutableStateFlow<ProfileUiState>(ProfileUiState.Loading)
    val uiState: StateFlow<ProfileUiState> = _uiState.asStateFlow()

    private val _events = MutableSharedFlow<ProfileEvent>(extraBufferCapacity = 16)
    val events: SharedFlow<ProfileEvent> = _events.asSharedFlow()

    // ── 初始化 / 刷新 ──

    init {
        load()
    }

    fun load() {
        _uiState.value = ProfileUiState.Loading
        viewModelScope.launch {
            // 并行拉取：profile + stats + settings；任一失败不阻塞其他结果
            val profileResult = profileRepository.getProfile()
            val statsResult = profileRepository.getStats()
            val settingsResult = profileRepository.getSettings()

            val profile = profileResult.getOrNull()
            val stats = statsResult.getOrElse { UserStatsDto() }
            val settings = settingsResult.getOrElse { UserSettingsDto.DEFAULT }

            if (profile == null) {
                val err = profileResult.exceptionOrNull()
                // 401 未授权，触发登出
                if (err is ProfileUnauthorizedException) {
                    authRepository.logout()
                    _events.tryEmit(ProfileEvent.Logout)
                    return@launch
                }
                _uiState.value = ProfileUiState.Error(
                    err?.localizedMessage ?: "无法加载个人资料"
                )
                return@launch
            }

            _uiState.value = ProfileUiState.Ready(profile, stats, settings)
        }
    }

    // ── 修改资料 ──

    fun updateProfile(
        username: String?,
        email: String?,
        bio: String?,
        onSuccess: () -> Unit = {}
    ) {
        viewModelScope.launch {
            val current = (uiState.value as? ProfileUiState.Ready)?.profile ?: return@launch
            val request = UpdateProfileRequest(
                username = username?.takeIf { it != current.username },
                email = email?.takeIf { it != current.email },
                bio = bio
            )
            // 至少有一个字段变更
            if (request.username == null && request.email == null && request.bio == null) {
                _events.emit(ProfileEvent.Toast("未做任何修改"))
                onSuccess()
                return@launch
            }

            profileRepository.updateProfile(request)
                .onSuccess { updated ->
                    _uiState.update { state ->
                        if (state is ProfileUiState.Ready) {
                            state.copy(profile = updated)
                        } else state
                    }
                    // 同步到本地用户表
                    localUserRepository.saveUser(
                        com.capella.room.data.remote.dto.UserInfo(
                            id = updated.id,
                            username = updated.username,
                            avatarUrl = updated.avatarUrl
                        ),
                        isCurrentUser = true
                    )
                    _events.emit(ProfileEvent.Toast("资料已更新"))
                    onSuccess()
                }
                .onFailure { e ->
                    _events.emit(ProfileEvent.Error(e.localizedMessage ?: "更新资料失败"))
                }
        }
    }

    // ── 切换设置（乐观更新 + 失败回滚） ──

    fun toggleSetting(transform: (UserSettingsDto) -> UserSettingsDto, key: String) {
        val current = (uiState.value as? ProfileUiState.Ready) ?: return
        val newSettings = transform(current.settings)

        // 1. 乐观更新
        _uiState.value = current.copy(settings = newSettings)

        // 2. 异步持久化
        viewModelScope.launch {
            profileRepository.updateSettings(newSettings)
                .onSuccess { synced ->
                    _uiState.update { state ->
                        if (state is ProfileUiState.Ready) state.copy(settings = synced)
                        else state
                    }
                }
                .onFailure { e ->
                    // 回滚
                    _uiState.value = current
                    _events.emit(
                        ProfileEvent.Error(e.localizedMessage ?: "设置保存失败：$key")
                    )
                }
        }
    }

    fun setTheme(theme: String) = toggleSetting(
        transform = { it.copy(theme = theme) },
        key = "theme"
    )

    fun setLanguage(language: String) = toggleSetting(
        transform = { it.copy(language = language) },
        key = "language"
    )

    // 偏好开关快捷入口
    fun toggleNotification() = toggleSetting(
        { it.copy(notificationEnabled = !(it.notificationEnabled ?: true)) },
        "notification_enabled"
    )

    fun toggleSound() = toggleSetting(
        { it.copy(soundEnabled = !(it.soundEnabled ?: true)) },
        "sound_enabled"
    )

    fun toggleReadReceipts() = toggleSetting(
        { it.copy(readReceiptsEnabled = !(it.readReceiptsEnabled ?: true)) },
        "read_receipts_enabled"
    )

    fun toggleSingleDeviceLogin() = toggleSetting(
        { it.copy(singleDeviceLogin = !(it.singleDeviceLogin ?: false)) },
        "single_device_login"
    )

    fun toggleShowOnlineStatus() = toggleSetting(
        { it.copy(showOnlineStatus = !(it.showOnlineStatus ?: true)) },
        "show_online_status"
    )

    fun toggleEnterToSend() = toggleSetting(
        { it.copy(enterToSend = !(it.enterToSend ?: true)) },
        "enter_to_send"
    )

    // ── 修改密码 ──

    fun changePassword(current: String, new: String, onSuccess: () -> Unit) {
        viewModelScope.launch {
            if (current.isBlank() || new.isBlank()) {
                _events.emit(ProfileEvent.Error("密码不能为空"))
                return@launch
            }
            if (new.length < 8) {
                _events.emit(ProfileEvent.Error("新密码至少 8 位"))
                return@launch
            }
            if (new == current) {
                _events.emit(ProfileEvent.Error("新密码不能与当前密码相同"))
                return@launch
            }

            profileRepository.changePassword(current, new)
                .onSuccess {
                    _events.emit(ProfileEvent.Toast("密码已修改"))
                    onSuccess()
                }
                .onFailure { e ->
                    _events.emit(ProfileEvent.Error(e.localizedMessage ?: "修改密码失败"))
                }
        }
    }

    // ── 退出登录 ──

    fun logout() {
        viewModelScope.launch {
            // 1. 调用后端 logout（best-effort）
            authRepository.logout()
            // 2. 清理本地状态
            localUserRepository.clearCurrentUser()
            // 3. 通知 UI
            _events.emit(ProfileEvent.Logout)
        }
    }
}
