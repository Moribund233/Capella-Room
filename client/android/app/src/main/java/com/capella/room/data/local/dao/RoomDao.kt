package com.capella.room.data.local.dao

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import androidx.room.Transaction
import androidx.room.Update
import com.capella.room.data.local.entity.RoomEntity
import com.capella.room.data.local.entity.RoomMemberEntity
import kotlinx.coroutines.flow.Flow

/**
 * 房间数据访问对象
 */
@Dao
interface RoomDao {

    /**
     * 根据ID获取房间
     */
    @Query("SELECT * FROM rooms WHERE id = :roomId")
    suspend fun getRoomById(roomId: String): RoomEntity?

    /**
     * 获取所有已加入的房间
     */
    @Query("SELECT * FROM rooms WHERE is_joined = 1 ORDER BY last_message_time DESC, updated_at DESC")
    suspend fun getJoinedRooms(): List<RoomEntity>

    /**
     * 获取所有已加入的房间（Flow）
     */
    @Query("SELECT * FROM rooms WHERE is_joined = 1 ORDER BY last_message_time DESC, updated_at DESC")
    fun getJoinedRoomsFlow(): Flow<List<RoomEntity>>

    /**
     * 获取所有房间（包括未加入的）
     */
    @Query("SELECT * FROM rooms ORDER BY updated_at DESC")
    suspend fun getAllRooms(): List<RoomEntity>

    /**
     * 获取所有房间（Flow）
     */
    @Query("SELECT * FROM rooms ORDER BY updated_at DESC")
    fun getAllRoomsFlow(): Flow<List<RoomEntity>>

    /**
     * 插入房间
     */
    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertRoom(room: RoomEntity)

    /**
     * 批量插入房间
     */
    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertRooms(rooms: List<RoomEntity>)

    /**
     * 更新房间
     */
    @Update
    suspend fun updateRoom(room: RoomEntity)

    /**
     * 删除房间
     */
    @Query("DELETE FROM rooms WHERE id = :roomId")
    suspend fun deleteRoom(roomId: String)

    /**
     * 标记房间为已加入
     */
    @Query("UPDATE rooms SET is_joined = 1, joined_at = :joinedAt WHERE id = :roomId")
    suspend fun markAsJoined(roomId: String, joinedAt: Long = System.currentTimeMillis())

    /**
     * 标记房间为已离开
     */
    @Query("UPDATE rooms SET is_joined = 0, joined_at = NULL WHERE id = :roomId")
    suspend fun markAsLeft(roomId: String)

    /**
     * 更新未读消息数
     */
    @Query("UPDATE rooms SET unread_count = :count WHERE id = :roomId")
    suspend fun updateUnreadCount(roomId: String, count: Int)

    /**
     * 增加未读消息数
     */
    @Query("UPDATE rooms SET unread_count = unread_count + 1 WHERE id = :roomId")
    suspend fun incrementUnreadCount(roomId: String)

    /**
     * 清除未读消息数
     */
    @Query("UPDATE rooms SET unread_count = 0, last_read_message_id = :lastMessageId WHERE id = :roomId")
    suspend fun clearUnreadCount(roomId: String, lastMessageId: String? = null)

    /**
     * 更新最后一条消息
     */
    @Query("""
        UPDATE rooms SET 
            last_message_id = :messageId,
            last_message_content = :content,
            last_message_sender_name = :senderName,
            last_message_time = :time
        WHERE id = :roomId
    """)
    suspend fun updateLastMessage(
        roomId: String,
        messageId: String,
        content: String,
        senderName: String,
        time: String
    )

    /**
     * 搜索房间
     */
    @Query("SELECT * FROM rooms WHERE name LIKE '%' || :query || '%' OR description LIKE '%' || :query || '%' ORDER BY name LIMIT :limit")
    suspend fun searchRooms(query: String, limit: Int = 20): List<RoomEntity>

    /**
     * 获取收藏的房间
     */
    @Query("SELECT * FROM rooms WHERE is_favorite = 1 ORDER BY name")
    suspend fun getFavoriteRooms(): List<RoomEntity>

    /**
     * 切换收藏状态
     */
    @Query("UPDATE rooms SET is_favorite = NOT is_favorite WHERE id = :roomId")
    suspend fun toggleFavorite(roomId: String)

    /**
     * 清除所有房间
     */
    @Query("DELETE FROM rooms")
    suspend fun clearAllRooms()

    // ==================== 房间成员操作 ====================

    /**
     * 获取房间的所有成员
     */
    @Query("SELECT * FROM room_members WHERE room_id = :roomId")
    suspend fun getRoomMembers(roomId: String): List<RoomMemberEntity>

    /**
     * 插入房间成员
     */
    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertRoomMember(member: RoomMemberEntity)

    /**
     * 批量插入房间成员
     */
    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertRoomMembers(members: List<RoomMemberEntity>)

    /**
     * 删除房间成员
     */
    @Query("DELETE FROM room_members WHERE room_id = :roomId AND user_id = :userId")
    suspend fun deleteRoomMember(roomId: String, userId: String)

    /**
     * 删除房间的所有成员
     */
    @Query("DELETE FROM room_members WHERE room_id = :roomId")
    suspend fun deleteRoomMembers(roomId: String)

    /**
     * 事务：插入房间及其成员
     */
    @Transaction
    suspend fun insertRoomWithMembers(room: RoomEntity, members: List<RoomMemberEntity>) {
        insertRoom(room)
        deleteRoomMembers(room.id)
        insertRoomMembers(members)
    }
}
