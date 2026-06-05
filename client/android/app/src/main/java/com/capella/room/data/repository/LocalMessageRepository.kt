package com.capella.room.data.repository

import com.capella.room.data.local.dao.MessageDao
import com.capella.room.data.local.entity.MessageEntity
import com.capella.room.data.local.entity.SyncStatus
import com.capella.room.data.remote.dto.MessageDto
import com.capella.room.data.remote.dto.UserInfo
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import javax.inject.Inject
import javax.inject.Singleton

/**
 * 本地消息仓库
 * 负责消息的本地存储和离线支持
 */
@Singleton
class LocalMessageRepository @Inject constructor(
    private val messageDao: MessageDao
) {

    /**
     * 根据ID获取消息
     */
    suspend fun getMessageById(messageId: String): MessageEntity? {
        return messageDao.getMessageById(messageId)
    }

    /**
     * 获取房间的消息流
     */
    fun getMessagesFlow(roomId: String): Flow<List<MessageEntity>> {
        return messageDao.getMessagesByRoomFlow(roomId)
    }

    /**
     * 获取房间的最近消息
     */
    suspend fun getRecentMessages(roomId: String, limit: Int = 50): List<MessageEntity> {
        return messageDao.getRecentMessages(roomId, limit)
    }

    /**
     * 分页获取消息
     */
    suspend fun getMessagesPaged(roomId: String, page: Int, pageSize: Int): List<MessageEntity> {
        val offset = page * pageSize
        return messageDao.getMessagesByRoom(roomId, pageSize, offset)
    }

    /**
     * 保存消息（来自服务器）
     */
    suspend fun saveMessage(messageDto: MessageDto) {
        val entity = messageDto.toEntity(SyncStatus.SYNCED)
        messageDao.insertOrUpdate(entity)
    }

    /**
     * 批量保存消息
     */
    suspend fun saveMessages(messages: List<MessageDto>) {
        val entities = messages.map { it.toEntity(SyncStatus.SYNCED) }
        messageDao.insertMessages(entities)
    }

    /**
     * 创建待发送的本地消息
     * @return 创建的临时消息ID
     */
    suspend fun createPendingMessage(
        roomId: String,
        senderId: String,
        senderName: String,
        content: String,
        messageType: String = "text",
        replyTo: String? = null
    ): MessageEntity {
        val tempId = "temp_${System.currentTimeMillis()}_${(0..9999).random()}"
        val entity = MessageEntity(
            id = tempId,
            roomId = roomId,
            senderId = senderId,
            senderName = senderName,
            content = content,
            messageType = messageType,
            replyTo = replyTo,
            createdAt = java.time.Instant.now().toString(),
            syncStatus = SyncStatus.PENDING
        )
        messageDao.insertMessage(entity)
        return entity
    }

    /**
     * 标记消息为发送中
     */
    suspend fun markAsSending(messageId: String) {
        messageDao.updateSyncStatus(messageId, SyncStatus.SENDING)
    }

    /**
     * 标记消息为发送失败
     */
    suspend fun markAsFailed(messageId: String) {
        messageDao.updateSyncStatus(messageId, SyncStatus.FAILED)
    }

    /**
     * 消息发送成功，更新为服务器返回的ID
     */
    suspend fun confirmMessageSent(
        tempId: String,
        serverMessage: MessageDto
    ) {
        // 删除临时消息
        messageDao.deleteMessage(tempId)
        // 插入服务器确认的消息
        val entity = serverMessage.toEntity(SyncStatus.SYNCED)
        messageDao.insertMessage(entity)
    }

    /**
     * 更新消息内容（编辑）
     */
    suspend fun updateMessageContent(messageId: String, newContent: String) {
        val editedAt = java.time.Instant.now().toString()
        messageDao.updateMessageContent(messageId, newContent, editedAt, SyncStatus.EDIT_PENDING)
    }

    /**
     * 确认消息编辑已同步
     */
    suspend fun confirmMessageEdited(messageId: String, editedAt: String) {
        messageDao.confirmMessageEdited(messageId, editedAt)
    }

    /**
     * 软删除消息
     */
    suspend fun deleteMessage(messageId: String) {
        messageDao.markAsDeleted(messageId, SyncStatus.DELETE_PENDING)
    }

    /**
     * 确认消息删除已同步
     */
    suspend fun confirmMessageDeleted(messageId: String) {
        messageDao.deleteMessage(messageId)
    }

    /**
     * 获取待同步的消息
     */
    suspend fun getPendingMessages(): List<MessageEntity> {
        return messageDao.getPendingMessages()
    }

    /**
     * 获取待同步的消息流
     */
    fun getPendingMessagesFlow(): Flow<List<MessageEntity>> {
        return messageDao.getPendingMessagesFlow()
    }

    /**
     * 搜索消息
     */
    suspend fun searchMessages(roomId: String, query: String): List<MessageEntity> {
        return messageDao.searchMessages(roomId, query)
    }

    /**
     * 清除房间的所有消息
     */
    suspend fun clearRoomMessages(roomId: String) {
        messageDao.deleteMessagesByRoom(roomId)
    }

    /**
     * 清除所有消息
     */
    suspend fun clearAllMessages() {
        messageDao.clearAllMessages()
    }

    // ==================== 转换方法 ====================

    /**
     * 标记消息为已读
     */
    suspend fun markAsRead(messageId: String) {
        messageDao.markAsRead(messageId)
    }

    /**
     * 更新消息已读回执
     */
    suspend fun updateReadReceipt(messageId: String, readCount: Int, readBy: List<String>) {
        val readByJson = readBy.joinToString(",", prefix = "[", postfix = "]") { "\"$it\"" }
        messageDao.updateReadReceipt(messageId, readCount, readByJson)
    }

    /**
     * 将 DTO 转换为实体
     */
    private fun MessageDto.toEntity(syncStatus: SyncStatus = SyncStatus.SYNCED): MessageEntity {
        return MessageEntity(
            id = this.id,
            roomId = this.roomId,
            senderId = this.sender.id,
            senderName = this.sender.username,
            senderAvatarUrl = this.sender.avatarUrl,
            content = this.content,
            messageType = this.messageType,
            replyTo = this.replyTo,
            replyToMessage = this.replyToMessage?.toString(),
            isDeleted = this.isDeleted,
            createdAt = this.createdAt,
            editCount = this.editCount,
            editedAt = this.editedAt,
            syncStatus = syncStatus
        )
    }

    /**
     * 将实体转换为 DTO
     */
    fun toDto(entity: MessageEntity): MessageDto {
        return MessageDto(
            id = entity.id,
            roomId = entity.roomId,
            sender = UserInfo(
                id = entity.senderId,
                username = entity.senderName,
                avatarUrl = entity.senderAvatarUrl
            ),
            content = entity.content,
            messageType = entity.messageType,
            replyTo = entity.replyTo,
            replyToMessage = entity.replyToMessage,
            isDeleted = entity.isDeleted,
            createdAt = entity.createdAt,
            editCount = entity.editCount,
            editedAt = entity.editedAt
        )
    }

    /**
     * 将实体列表转换为 DTO 列表
     */
    fun toDtoList(entities: List<MessageEntity>): List<MessageDto> {
        return entities.map { toDto(it) }
    }
}
