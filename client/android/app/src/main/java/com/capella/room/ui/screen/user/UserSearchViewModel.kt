package com.capella.room.ui.screen.user

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.capella.room.data.remote.api.UserApi
import com.capella.room.data.remote.dto.UserDto
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.launch
import javax.inject.Inject

/**
 * 用户搜索 ViewModel
 */
@HiltViewModel
class UserSearchViewModel @Inject constructor(
    private val userApi: UserApi
) : ViewModel() {

    /**
     * 搜索用户
     */
    suspend fun searchUsers(query: String): List<UserDto> {
        return try {
            val response = userApi.searchUsers(
                query = query,
                limit = 20
            )
            if (response.isSuccessful && response.body()?.success == true) {
                response.body()?.data ?: emptyList()
            } else {
                emptyList()
            }
        } catch (e: Exception) {
            emptyList()
        }
    }

    /**
     * 获取用户详情
     */
    suspend fun getUserById(userId: String): UserDto? {
        return try {
            val response = userApi.getUserById(userId)
            if (response.isSuccessful && response.body()?.success == true) {
                response.body()?.data
            } else {
                null
            }
        } catch (e: Exception) {
            null
        }
    }

    /**
     * 获取好友列表
     */
    suspend fun getFriends(): List<UserDto> {
        return try {
            val response = userApi.getFriends()
            if (response.isSuccessful && response.body()?.success == true) {
                response.body()?.data ?: emptyList()
            } else {
                emptyList()
            }
        } catch (e: Exception) {
            emptyList()
        }
    }

    /**
     * 发送好友请求
     */
    fun sendFriendRequest(userId: String, onResult: (Boolean) -> Unit) {
        viewModelScope.launch {
            try {
                val response = userApi.sendFriendRequest(
                    mapOf("user_id" to userId)
                )
                onResult(response.isSuccessful && response.body()?.success == true)
            } catch (e: Exception) {
                onResult(false)
            }
        }
    }
}
