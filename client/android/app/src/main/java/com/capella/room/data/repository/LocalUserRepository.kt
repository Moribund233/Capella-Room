package com.capella.room.data.repository

import com.capella.room.data.local.dao.UserDao
import com.capella.room.data.local.entity.CurrentUserEntity
import com.capella.room.data.local.entity.UserEntity
import com.capella.room.data.remote.dto.UserInfo
import kotlinx.coroutines.flow.Flow
import javax.inject.Inject
import javax.inject.Singleton

/**
 * 本地用户仓库
 * 负责用户信息的本地存储
 */
@Singleton
class LocalUserRepository @Inject constructor(
    private val userDao: UserDao
) {

    /**
     * 获取当前登录用户流
     */
    fun getCurrentUserFlow(): Flow<CurrentUserEntity?> {
        return userDao.getCurrentUserFlow()
    }

    /**
     * 获取当前登录用户
     */
    suspend fun getCurrentUser(): CurrentUserEntity? {
        return userDao.getCurrentUser()
    }

    /**
     * 保存当前登录用户
     */
    suspend fun saveCurrentUser(
        userId: String,
        username: String,
        accessToken: String,
        refreshToken: String,
        avatarUrl: String? = null
    ) {
        val entity = CurrentUserEntity(
            userId = userId,
            username = username,
            accessToken = accessToken,
            refreshToken = refreshToken,
            avatarUrl = avatarUrl
        )
        userDao.saveCurrentUser(entity)
    }

    /**
     * 清除当前登录用户
     */
    suspend fun clearCurrentUser() {
        userDao.clearCurrentUser()
    }

    /**
     * 检查是否有登录用户
     */
    suspend fun hasCurrentUser(): Boolean {
        return userDao.hasCurrentUser() > 0
    }

    /**
     * 获取用户流
     */
    fun getAllUsersFlow(): Flow<List<UserEntity>> {
        return userDao.getAllUsersFlow()
    }

    /**
     * 根据ID获取用户
     */
    suspend fun getUserById(userId: String): UserEntity? {
        return userDao.getUserById(userId)
    }

    /**
     * 根据用户名获取用户
     */
    suspend fun getUserByUsername(username: String): UserEntity? {
        return userDao.getUserByUsername(username)
    }

    /**
     * 保存用户
     */
    suspend fun saveUser(userInfo: UserInfo, isCurrentUser: Boolean = false) {
        val entity = userInfo.toEntity(isCurrentUser)
        userDao.insertUser(entity)
    }

    /**
     * 批量保存用户
     */
    suspend fun saveUsers(users: List<UserInfo>) {
        val entities = users.map { it.toEntity(false) }
        userDao.insertUsers(entities)
    }

    /**
     * 搜索用户
     */
    suspend fun searchUsers(query: String): List<UserEntity> {
        return userDao.searchUsers(query)
    }

    /**
     * 删除用户
     */
    suspend fun deleteUser(userId: String) {
        userDao.deleteUser(userId)
    }

    /**
     * 清除所有用户
     */
    suspend fun clearAllUsers() {
        userDao.clearAllUsers()
    }

    // ==================== 转换方法 ====================

    /**
     * 将 UserInfo DTO 转换为实体
     */
    private fun UserInfo.toEntity(isCurrentUser: Boolean = false): UserEntity {
        return UserEntity(
            id = this.id,
            username = this.username,
            avatarUrl = this.avatarUrl,
            isCurrentUser = isCurrentUser
        )
    }

    /**
     * 将实体转换为 UserInfo DTO
     */
    fun UserEntity.toDto(): UserInfo {
        return UserInfo(
            id = this.id,
            username = this.username,
            avatarUrl = this.avatarUrl
        )
    }

    /**
     * 将实体列表转换为 DTO 列表
     */
    fun List<UserEntity>.toDtoList(): List<UserInfo> {
        return this.map { it.toDto() }
    }
}
