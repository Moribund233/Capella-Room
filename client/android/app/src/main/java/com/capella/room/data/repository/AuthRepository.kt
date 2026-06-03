package com.capella.room.data.repository

import com.capella.room.data.local.TokenManager
import com.capella.room.data.remote.api.AuthApi
import com.capella.room.data.remote.dto.*
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class AuthRepository @Inject constructor(
    private val authApi: AuthApi,
    private val tokenManager: TokenManager
) {

    suspend fun login(email: String, password: String): Result<LoginData> {
        return try {
            val response = authApi.login(LoginRequest(email, password))
            if (response.isSuccessful) {
                val body = response.body()
                if (body?.success == true && body.data != null) {
                    val data = body.data
                    tokenManager.saveTokens(data.accessToken, data.refreshToken)
                    tokenManager.saveUserInfo(data.user.id, data.user.username)
                    Result.success(data)
                } else {
                    Result.failure(AuthException(body?.message ?: "登录失败"))
                }
            } else {
                val errorBody = response.errorBody()?.string()
                Result.failure(AuthException(errorBody ?: "登录失败：${response.code()}"))
            }
        } catch (e: Exception) {
            Result.failure(AuthException("网络错误：${e.localizedMessage ?: "未知错误"}"))
        }
    }

    suspend fun register(username: String, email: String, password: String): Result<UserDto> {
        return try {
            val response = authApi.register(RegisterRequest(username, email, password))
            if (response.isSuccessful) {
                val body = response.body()
                if (body?.success == true && body.data != null) {
                    Result.success(body.data)
                } else {
                    Result.failure(AuthException(body?.message ?: "注册失败"))
                }
            } else {
                val errorBody = response.errorBody()?.string()
                Result.failure(AuthException(errorBody ?: "注册失败：${response.code()}"))
            }
        } catch (e: Exception) {
            Result.failure(AuthException("网络错误：${e.localizedMessage ?: "未知错误"}"))
        }
    }

    suspend fun refreshToken(): Result<TokenData> {
        val refreshToken = tokenManager.getRefreshToken() ?: return Result.failure(AuthException("未登录"))
        return try {
            val response = authApi.refresh(RefreshRequest(refreshToken))
            if (response.isSuccessful) {
                val body = response.body()
                if (body?.success == true && body.data != null) {
                    val data = body.data
                    tokenManager.saveTokens(data.accessToken, data.refreshToken)
                    Result.success(data)
                } else {
                    tokenManager.clear()
                    Result.failure(AuthException(body?.message ?: "刷新失败"))
                }
            } else {
                tokenManager.clear()
                Result.failure(AuthException("刷新失败：${response.code()}"))
            }
        } catch (e: Exception) {
            tokenManager.clear()
            Result.failure(AuthException("网络错误：${e.localizedMessage}"))
        }
    }

    suspend fun logout() {
        try {
            authApi.logout()
        } catch (_: Exception) { }
        tokenManager.clear()
    }
}

class AuthException(message: String) : Exception(message)
