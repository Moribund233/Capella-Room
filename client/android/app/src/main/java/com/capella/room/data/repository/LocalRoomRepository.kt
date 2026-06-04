package com.capella.room.data.repository

import com.capella.room.data.local.dao.RoomDao
import com.capella.room.data.local.entity.RoomEntity
import com.capella.room.data.local.entity.RoomMemberEntity
import com.capella.room.data.remote.dto.LastMessageDto
import com.capella.room.data.remote.dto.RoomDto
import com.capella.room.data.remote.dto.RoomMemberDto
import com.capella.room.data.remote.dto.UserInfo
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.first
import javax.inject.Inject
import javax.inject.Singleton

/**
 * 本地房间仓库
 * 负责房间信息的本地存储和离线支持
 */
@Singleton
class LocalRoomRepository @Inject constructor(
    private val roomDao: RoomDao
) {

    /**
     * 获取已加入的房间流
     */
    fun getJoinedRoomsFlow(): Flow<List<RoomEntity>> {
        return roomDao.getJoinedRoomsFlow()
    }

    /**
     * 获取已加入的房间列表
     */
    suspend fun getJoinedRooms(): List<RoomEntity> {
        return roomDao.getJoinedRooms()
    }

    /**
     * 获取所有房间
     */
    suspend fun getAllRooms(): List<RoomEntity> {
        return roomDao.getAllRooms()
    }

    /**
     * 根据ID获取房间
     */
    suspend fun getRoomById(roomId: String): RoomEntity? {
        return roomDao.getRoomById(roomId)
    }

    /**
     * 保存房间（来自服务器）
     */
    suspend fun saveRoom(roomDto: RoomDto, isJoined: Boolean = false) {
        val entity = roomDto.toEntity(isJoined)
        roomDao.insertRoom(entity)
    }

    /**
     * 批量保存房间
     */
    suspend fun saveRooms(rooms: List<RoomDto>, isJoined: Boolean = false) {
        val entities = rooms.map { it.toEntity(isJoined) }
        roomDao.insertRooms(entities)
    }

    /**
     * 标记房间为已加入
     */
    suspend fun joinRoom(roomId: String) {
        roomDao.markAsJoined(roomId)
    }

    /**
     * 标记房间为已离开
     */
    suspend fun leaveRoom(roomId: String) {
        roomDao.markAsLeft(roomId)
    }

    /**
     * 更新未读消息数
     */
    suspend fun updateUnreadCount(roomId: String, count: Int) {
        roomDao.updateUnreadCount(roomId, count)
    }

    /**
     * 增加未读消息数
     */
    suspend fun incrementUnreadCount(roomId: String) {
        roomDao.incrementUnreadCount(roomId)
    }

    /**
     * 清除未读消息数
     */
    suspend fun clearUnreadCount(roomId: String, lastMessageId: String? = null) {
        roomDao.clearUnreadCount(roomId, lastMessageId)
    }

    /**
     * 更新房间最后一条消息
     */
    suspend fun updateLastMessage(
        roomId: String,
        messageId: String,
        content: String,
        senderName: String,
        time: String
    ) {
        roomDao.updateLastMessage(roomId, messageId, content, senderName, time)
    }

    /**
     * 保存房间成员
     */
    suspend fun saveRoomMembers(roomId: String, members: List<RoomMemberDto>) {
        val entities = members.map { it.toEntity(roomId) }
        roomDao.deleteRoomMembers(roomId)
        roomDao.insertRoomMembers(entities)
    }

    /**
     * 获取房间成员
     */
    suspend fun getRoomMembers(roomId: String): List<RoomMemberEntity> {
        return roomDao.getRoomMembers(roomId)
    }

    /**
     * 搜索房间
     */
    suspend fun searchRooms(query: String): List<RoomEntity> {
        return roomDao.searchRooms(query)
    }

    /**
     * 切换收藏状态
     */
    suspend fun toggleFavorite(roomId: String) {
        roomDao.toggleFavorite(roomId)
    }

    /**
     * 删除房间
     */
    suspend fun deleteRoom(roomId: String) {
        roomDao.deleteRoom(roomId)
    }

    /**
     * 清除所有房间数据
     */
    suspend fun clearAllRooms() {
        roomDao.clearAllRooms()
    }

    // ==================== 转换方法 ====================

    /**
     * 将 DTO 转换为实体
     */
    private fun RoomDto.toEntity(isJoined: Boolean = false): RoomEntity {
        return RoomEntity(
            id = this.id,
            name = this.name,
            description = this.description,
            ownerId = this.owner?.id,
            ownerName = this.owner?.username,
            isPrivate = this.isPrivate,
            maxMembers = this.maxMembers,
            memberCount = this.memberCount,
            unreadCount = this.unreadCount,
            lastMessageId = this.lastMessage?.id,
            lastMessageContent = this.lastMessage?.content,
            lastMessageSenderName = this.lastMessage?.senderName,
            lastMessageTime = this.lastMessage?.createdAt,
            createdAt = this.createdAt,
            updatedAt = this.updatedAt,
            isJoined = isJoined
        )
    }

    /**
     * 将实体转换为 DTO
     */
    fun toDto(entity: RoomEntity): RoomDto {
        return RoomDto(
            id = entity.id,
            name = entity.name,
            description = entity.description,
            owner = if (entity.ownerId != null) {
                UserInfo(
                    id = entity.ownerId,
                    username = entity.ownerName ?: "",
                    avatarUrl = null
                )
            } else null,
            isPrivate = entity.isPrivate,
            maxMembers = entity.maxMembers,
            memberCount = entity.memberCount,
            unreadCount = entity.unreadCount,
            lastMessage = if (entity.lastMessageId != null) {
                LastMessageDto(
                    id = entity.lastMessageId,
                    content = entity.lastMessageContent ?: "",
                    senderName = entity.lastMessageSenderName ?: "",
                    createdAt = entity.lastMessageTime ?: ""
                )
            } else null,
            createdAt = entity.createdAt,
            updatedAt = entity.updatedAt
        )
    }

    /**
     * 将成员 DTO 转换为实体
     */
    private fun RoomMemberDto.toEntity(roomId: String): RoomMemberEntity {
        return RoomMemberEntity(
            roomId = roomId,
            userId = this.userId,
            username = this.username,
            role = this.role,
            joinedAt = this.joinedAt
        )
    }

    /**
     * 将实体列表转换为 DTO 列表
     */
    fun toDtoList(entities: List<RoomEntity>): List<RoomDto> {
        return entities.map { toDto(it) }
    }
}
