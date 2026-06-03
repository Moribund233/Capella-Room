package com.capella.room.ui.screen.auth

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.capella.room.data.repository.AuthRepository
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.launch
import javax.inject.Inject

data class LoginUiState(
    val isLoginTab: Boolean = true,
    val isLoading: Boolean = false,
    val errorMessage: String? = null,
    val loginEmail: String = "",
    val loginPassword: String = "",
    val loginPasswordVisible: Boolean = false,
    val rememberMe: Boolean = false,
    val regUsername: String = "",
    val regEmail: String = "",
    val regPassword: String = "",
    val regPasswordVisible: Boolean = false,
    val agreeTerms: Boolean = false
)

@HiltViewModel
class LoginViewModel @Inject constructor(
    private val authRepository: AuthRepository
) : ViewModel() {

    var uiState by mutableStateOf(LoginUiState())
        private set

    fun clearError() {
        uiState = uiState.copy(errorMessage = null)
    }

    fun switchToLogin() {
        uiState = uiState.copy(isLoginTab = true, errorMessage = null)
    }

    fun switchToRegister() {
        uiState = uiState.copy(isLoginTab = false, errorMessage = null)
    }

    fun updateLoginEmail(value: String) {
        uiState = uiState.copy(loginEmail = value)
    }

    fun updateLoginPassword(value: String) {
        uiState = uiState.copy(loginPassword = value)
    }

    fun toggleLoginPasswordVisibility() {
        uiState = uiState.copy(loginPasswordVisible = !uiState.loginPasswordVisible)
    }

    fun updateRememberMe(value: Boolean) {
        uiState = uiState.copy(rememberMe = value)
    }

    fun updateRegUsername(value: String) {
        uiState = uiState.copy(regUsername = value)
    }

    fun updateRegEmail(value: String) {
        uiState = uiState.copy(regEmail = value)
    }

    fun updateRegPassword(value: String) {
        uiState = uiState.copy(regPassword = value)
    }

    fun toggleRegPasswordVisibility() {
        uiState = uiState.copy(regPasswordVisible = !uiState.regPasswordVisible)
    }

    fun updateAgreeTerms(value: Boolean) {
        uiState = uiState.copy(agreeTerms = value)
    }

    fun login(onSuccess: () -> Unit) {
        val state = uiState
        if (state.loginEmail.isBlank()) {
            uiState = state.copy(errorMessage = "请输入邮箱")
            return
        }
        if (state.loginPassword.isBlank()) {
            uiState = state.copy(errorMessage = "请输入密码")
            return
        }

        uiState = state.copy(isLoading = true, errorMessage = null)

        viewModelScope.launch {
            val result = authRepository.login(state.loginEmail, state.loginPassword)
            result.fold(
                onSuccess = {
                    uiState = uiState.copy(isLoading = false)
                    onSuccess()
                },
                onFailure = { e ->
                    uiState = uiState.copy(
                        isLoading = false,
                        errorMessage = e.message ?: "登录失败"
                    )
                }
            )
        }
    }

    fun register(onSuccess: () -> Unit) {
        val state = uiState
        if (state.regUsername.isBlank()) {
            uiState = state.copy(errorMessage = "请输入用户名")
            return
        }
        if (state.regEmail.isBlank()) {
            uiState = state.copy(errorMessage = "请输入邮箱")
            return
        }
        if (state.regPassword.isBlank()) {
            uiState = state.copy(errorMessage = "请输入密码")
            return
        }
        if (state.regPassword.length < 8) {
            uiState = state.copy(errorMessage = "密码至少8位")
            return
        }
        if (!state.agreeTerms) {
            uiState = state.copy(errorMessage = "请同意服务条款和隐私政策")
            return
        }

        uiState = state.copy(isLoading = true, errorMessage = null)

        viewModelScope.launch {
            val regResult = authRepository.register(state.regUsername, state.regEmail, state.regPassword)
            regResult.fold(
                onSuccess = {
                    // Auto-login after registration
                    val loginResult = authRepository.login(state.regEmail, state.regPassword)
                    loginResult.fold(
                        onSuccess = {
                            uiState = uiState.copy(isLoading = false)
                            onSuccess()
                        },
                        onFailure = { e ->
                            uiState = uiState.copy(
                                isLoading = false,
                                errorMessage = "注册成功但登录失败：${e.message}"
                            )
                        }
                    )
                },
                onFailure = { e ->
                    uiState = uiState.copy(
                        isLoading = false,
                        errorMessage = e.message ?: "注册失败"
                    )
                }
            )
        }
    }
}
