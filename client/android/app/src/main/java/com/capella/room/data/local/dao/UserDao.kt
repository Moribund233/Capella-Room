package com.capella.room.data.local.dao

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import androidx.room.Update
import com.capella.room.data.local.entity.CurrentUserEntity
import com.capella.room.data.local.entity.UserEntity
import kotlinx.coroutines.flow.Flow

/**
 * 用户数据访问对象
 */
@Dao
interface UserDao {

    /**
     * 根据ID获取用户
     */
    @Query("SELECT * FROM users WHERE id = :userId")
    suspend fun getUserById(userId: String): UserEntity?

    /**
     * 根据用户名获取用户
     */
    @Query("SELECT * FROM users WHERE username = :username LIMIT 1")
    suspend fun getUserByUsername(username: String): UserEntity?

    /**
     * 获取所有用户
     */
    @Query("SELECT * FROM users ORDER BY username")
    suspend fun getAllUsers(): List<UserEntity>

    /**
     * 获取所有用户（Flow）
     */
    @Query("SELECT * FROM users ORDER BY username")
    fun getAllUsersFlow(): Flow<List<UserEntity>>

    /**
     * 插入用户
     */
    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertUser(user: UserEntity)

    /**
     * 批量插入用户
     */
    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertUsers(users: List<UserEntity>)

    /**
     * 更新用户
     */
    @Update
    suspend fun updateUser(user: UserEntity)

    /**
     * 删除用户
     */
    @Query("DELETE FROM users WHERE id = :userId")
    suspend fun deleteUser(userId: String)

    /**
     * 搜索用户
     */
    @Query("SELECT * FROM users WHERE username LIKE '%' || :query || '%' LIMIT :limit")
    suspend fun searchUsers(query: String, limit: Int = 20): List<UserEntity>

    /**
     * 清除所有用户
     */
    @Query("DELETE FROM users")
    suspend fun clearAllUsers()

    // ==================== 当前用户操作 ====================

    /**
     * 获取当前登录用户
     */
    @Query("SELECT * FROM current_user LIMIT 1")
    suspend fun getCurrentUser(): CurrentUserEntity?

    /**
     * 获取当前登录用户（Flow）
     */
    @Query("SELECT * FROM current_user LIMIT 1")
    fun getCurrentUserFlow(): Flow<CurrentUserEntity?>

    /**
     * 保存当前登录用户
     */
    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun saveCurrentUser(user: CurrentUserEntity)

    /**
     * 清除当前登录用户
     */
    @Query("DELETE FROM current_user")
    suspend fun clearCurrentUser()

    /**
     * 检查是否有登录用户
     */
    @Query("SELECT COUNT(*) FROM current_user")
    suspend fun hasCurrentUser(): Int
}
