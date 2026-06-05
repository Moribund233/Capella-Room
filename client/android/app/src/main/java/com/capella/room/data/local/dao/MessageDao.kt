package com.capella.room.data.local.dao

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import androidx.room.Transaction
import androidx.room.Update
import com.capella.room.data.local.entity.MessageEntity
import com.capella.room.data.local.entity.SyncStatus
import kotlinx.coroutines.flow.Flow

/**
 * 消息数据访问对象
 */
@Dao
interface MessageDao {

    /**
     * 根据ID获取消息
     */
    @Query("SELECT * FROM messages WHERE id = :messageId")
    suspend fun getMessageById(messageId: String): MessageEntity?

    /**
     * 获取房间的所有消息，按时间倒序
     */
    @Query("SELECT * FROM messages WHERE room_id = :roomId ORDER BY created_at DESC LIMIT :limit OFFSET :offset")
    suspend fun getMessagesByRoom(roomId: String, limit: Int, offset: Int): List<MessageEntity>

    /**
     * 获取房间的所有消息（Flow）
     */
    @Query("SELECT * FROM messages WHERE room_id = :roomId ORDER BY created_at ASC")
    fun getMessagesByRoomFlow(roomId: String): Flow<List<MessageEntity>>

    /**
     * 获取房间的最近消息
     */
    @Query("SELECT * FROM messages WHERE room_id = :roomId ORDER BY created_at DESC LIMIT :limit")
    suspend fun getRecentMessages(roomId: String, limit: Int): List<MessageEntity>

    /**
     * 插入消息
     */
    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertMessage(message: MessageEntity)

    /**
     * 批量插入消息
     */
    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertMessages(messages: List<MessageEntity>)

    /**
     * 更新消息
     */
    @Update
    suspend fun updateMessage(message: MessageEntity)

    /**
     * 删除消息（软删除）
     */
    @Query("UPDATE messages SET is_deleted = 1, sync_status = :syncStatus WHERE id = :messageId")
    suspend fun markAsDeleted(messageId: String, syncStatus: SyncStatus = SyncStatus.DELETE_PENDING)

    /**
     * 物理删除消息
     */
    @Query("DELETE FROM messages WHERE id = :messageId")
    suspend fun deleteMessage(messageId: String)

    /**
     * 删除房间的所有消息
     */
    @Query("DELETE FROM messages WHERE room_id = :roomId")
    suspend fun deleteMessagesByRoom(roomId: String)

    /**
     * 获取待同步的消息
     */
    @Query("SELECT * FROM messages WHERE sync_status != 'SYNCED' ORDER BY local_created_at ASC")
    suspend fun getPendingMessages(): List<MessageEntity>

    /**
     * 获取待同步的消息（Flow）
     */
    @Query("SELECT * FROM messages WHERE sync_status != 'SYNCED' ORDER BY local_created_at ASC")
    fun getPendingMessagesFlow(): Flow<List<MessageEntity>>

    /**
     * 更新消息同步状态
     */
    @Query("UPDATE messages SET sync_status = :syncStatus WHERE id = :messageId")
    suspend fun updateSyncStatus(messageId: String, syncStatus: SyncStatus)

    /**
     * 更新消息内容（编辑）
     */
    @Query("UPDATE messages SET content = :content, edit_count = edit_count + 1, edited_at = :editedAt, sync_status = :syncStatus WHERE id = :messageId")
    suspend fun updateMessageContent(messageId: String, content: String, editedAt: String, syncStatus: SyncStatus = SyncStatus.EDIT_PENDING)

    /**
     * 确认消息编辑已同步（更新编辑时间和同步状态）
     */
    @Query("UPDATE messages SET edited_at = :editedAt, sync_status = 'SYNCED' WHERE id = :messageId")
    suspend fun confirmMessageEdited(messageId: String, editedAt: String)

    // ==================== 已读回执 ====================

    /**
     * 标记消息为已读
     */
    @Query("UPDATE messages SET is_read = 1 WHERE id = :messageId")
    suspend fun markAsRead(messageId: String)

    /**
     * 更新消息已读人数和已读用户列表
     */
    @Query("UPDATE messages SET read_count = :readCount, read_by = :readBy WHERE id = :messageId")
    suspend fun updateReadReceipt(messageId: String, readCount: Int, readBy: String)

    /**
     * 搜索消息
     */
    @Query("SELECT * FROM messages WHERE room_id = :roomId AND content LIKE '%' || :query || '%' ORDER BY created_at DESC LIMIT :limit")
    suspend fun searchMessages(roomId: String, query: String, limit: Int = 50): List<MessageEntity>

    /**
     * 获取房间消息数量
     */
    @Query("SELECT COUNT(*) FROM messages WHERE room_id = :roomId")
    suspend fun getMessageCount(roomId: String): Int

    /**
     * 清除所有消息
     */
    @Query("DELETE FROM messages")
    suspend fun clearAllMessages()

    /**
     * 获取指定时间之后的消息
     */
    @Query("SELECT * FROM messages WHERE room_id = :roomId AND created_at > :after ORDER BY created_at ASC")
    suspend fun getMessagesAfter(roomId: String, after: String): List<MessageEntity>

    /**
     * 事务：插入或更新消息
     */
    @Transaction
    suspend fun insertOrUpdate(message: MessageEntity) {
        val existing = getMessageById(message.id)
        if (existing == null) {
            insertMessage(message)
        } else {
            updateMessage(message.copy(syncStatus = existing.syncStatus))
        }
    }
}
