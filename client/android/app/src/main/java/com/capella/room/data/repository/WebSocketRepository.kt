package com.capella.room.data.repository

import android.util.Log
import com.capella.room.data.remote.dto.MessageDto
import com.capella.room.data.remote.dto.UserInfo
import com.capella.room.data.remote.websocket.EditMessagePayload
import com.capella.room.data.remote.websocket.MessageDeletedPayload
import com.capella.room.data.remote.websocket.MessageEditedPayload
import com.capella.room.data.remote.websocket.NewMessagePayload
import com.capella.room.data.remote.websocket.OnlineUserInfo
import com.capella.room.data.remote.websocket.OnlineUsersPayload
import com.capella.room.data.remote.websocket.ReplyToMessageInfo
import com.capella.room.data.remote.websocket.RoomJoinedPayload
import com.capella.room.data.remote.websocket.RoomLeftPayload
import com.capella.room.data.remote.websocket.RoomMessageSummaryPayload
import com.capella.room.data.remote.websocket.UserJoinedPayload
import com.capella.room.data.remote.websocket.UserLeftPayload
import com.capella.room.data.remote.websocket.UserStopTypingPayload
import com.capella.room.data.remote.websocket.UserTypingPayload
import com.capella.room.data.remote.websocket.WebSocketClient
import com.capella.room.data.remote.websocket.WebSocketConnectionState
import com.capella.room.data.remote.websocket.WebSocketMessage
import com.capella.room.data.remote.websocket.WebSocketMessageType
import com.squareup.moshi.Moshi
import com.squareup.moshi.kotlin.reflect.KotlinJsonAdapterFactory
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.launch
import javax.inject.Inject
import javax.inject.Singleton

/**
 * WebSocket Repository
 * 封装 WebSocket 业务逻辑，为 ViewModel 提供高层 API
 */
@Singleton
class WebSocketRepository @Inject constructor(
    private val webSocketClient: WebSocketClient
) {
    companion object {
        private const val TAG = "WebSocketRepository"
    }

    private val moshi: Moshi = Moshi.Builder()
        .add(KotlinJsonAdapterFactory())
        .build()

    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)

    // 连接状态
    val connectionState: StateFlow<WebSocketConnectionState> = webSocketClient.connectionState

    // ==================== 事件流 ====================

    // 新消息流
    private val _newMessages = MutableSharedFlow<NewMessagePayload>(extraBufferCapacity = 64)
    val newMessages: Flow<NewMessagePayload> = _newMessages.asSharedFlow()

    // 消息编辑流
    private val _messageEdited = MutableSharedFlow<MessageEditedPayload>(extraBufferCapacity = 32)
    val messageEdited: Flow<MessageEditedPayload> = _messageEdited.asSharedFlow()

    // 消息删除流
    private val _messageDeleted = MutableSharedFlow<MessageDeletedPayload>(extraBufferCapacity = 32)
    val messageDeleted: Flow<MessageDeletedPayload> = _messageDeleted.asSharedFlow()

    // 用户正在输入流
    private val _userTyping = MutableSharedFlow<UserTypingPayload>(extraBufferCapacity = 32)
    val userTyping: Flow<UserTypingPayload> = _userTyping.asSharedFlow()

    // 用户停止输入流
    private val _userStopTyping = MutableSharedFlow<UserStopTypingPayload>(extraBufferCapacity = 32)
    val userStopTyping: Flow<UserStopTypingPayload> = _userStopTyping.asSharedFlow()

    // 用户加入房间流
    private val _userJoined = MutableSharedFlow<UserJoinedPayload>(extraBufferCapacity = 32)
    val userJoined: Flow<UserJoinedPayload> = _userJoined.asSharedFlow()

    // 用户离开房间流
    private val _userLeft = MutableSharedFlow<UserLeftPayload>(extraBufferCapacity = 32)
    val userLeft: Flow<UserLeftPayload> = _userLeft.asSharedFlow()

    // 在线用户列表流
    private val _onlineUsers = MutableSharedFlow<OnlineUsersPayload>(extraBufferCapacity = 16)
    val onlineUsers: Flow<OnlineUsersPayload> = _onlineUsers.asSharedFlow()

    // 房间消息摘要流（用于房间列表更新）
    private val _roomMessageSummary = MutableSharedFlow<RoomMessageSummaryPayload>(extraBufferCapacity = 64)
    val roomMessageSummary: Flow<RoomMessageSummaryPayload> = _roomMessageSummary.asSharedFlow()

    // 当前房间 ID
    private val _currentRoomId = MutableStateFlow<String?>(null)
    val currentRoomId: StateFlow<String?> = _currentRoomId.asStateFlow()

    // 已加入的房间集合
    private val joinedRooms = mutableSetOf<String>()

    init {
        // 启动消息监听
        startMessageListener()
    }

    // ==================== 连接管理 ====================

    /**
     * 连接到 WebSocket 服务器
     */
    fun connect() {
        webSocketClient.connect()
    }

    /**
     * 断开连接
     */
    fun disconnect() {
        joinedRooms.clear()
        _currentRoomId.value = null
        webSocketClient.disconnect()
    }

    /**
     * 是否已连接并认证
     */
    fun isConnected(): Boolean {
        return connectionState.value == WebSocketConnectionState.Authenticated
    }

    // ==================== 房间管理 ====================

    /**
     * 加入房间
     */
    fun joinRoom(roomId: String): Boolean {
        return if (webSocketClient.joinRoom(roomId)) {
            joinedRooms.add(roomId)
            _currentRoomId.value = roomId
            Log.d(TAG, "Joined room: $roomId")
            true
        } else {
            Log.w(TAG, "Failed to join room: $roomId")
            false
        }
    }

    /**
     * 离开房间
     */
    fun leaveRoom(roomId: String): Boolean {
        return if (webSocketClient.leaveRoom(roomId)) {
            joinedRooms.remove(roomId)
            if (_currentRoomId.value == roomId) {
                _currentRoomId.value = null
            }
            Log.d(TAG, "Left room: $roomId")
            true
        } else {
            Log.w(TAG, "Failed to leave room: $roomId")
            false
        }
    }

    /**
     * 离开当前房间
     */
    fun leaveCurrentRoom() {
        _currentRoomId.value?.let { roomId ->
            leaveRoom(roomId)
        }
    }

    /**
     * 获取当前房间 ID
     */
    fun getCurrentRoomId(): String? = _currentRoomId.value

    /**
     * 设置当前房间（不发送加入请求，用于切换房间时）
     */
    fun setCurrentRoom(roomId: String?) {
        _currentRoomId.value = roomId
    }

    // ==================== 消息操作 ====================

    /**
     * 发送聊天消息
     * @return 发送是否成功（仅表示消息已发送到服务器，不代表对方已收到）
     */
    fun sendMessage(roomId: String, content: String, replyTo: String? = null): Boolean {
        return webSocketClient.sendChatMessage(roomId, content, replyTo)
    }

    /**
     * 发送正在输入状态
     */
    fun sendTyping(roomId: String): Boolean {
        return webSocketClient.sendTyping(roomId)
    }

    /**
     * 发送停止输入状态
     */
    fun sendStopTyping(roomId: String): Boolean {
        return webSocketClient.sendStopTyping(roomId)
    }

    /**
     * 标记消息已读
     */
    fun markMessageRead(messageId: String): Boolean {
        return webSocketClient.markMessageRead(messageId)
    }

    /**
     * 编辑消息
     */
    fun editMessage(messageId: String, newContent: String): Boolean {
        return webSocketClient.editMessage(messageId, newContent)
    }

    /**
     * 删除消息
     */
    fun deleteMessage(messageId: String): Boolean {
        return webSocketClient.deleteMessage(messageId)
    }

    /**
     * 获取离线消息
     */
    fun getMissedMessages(roomId: String, lastMessageId: String?): Boolean {
        return webSocketClient.getMissedMessages(roomId, lastMessageId)
    }

    // ==================== 用户状态 ====================

    /**
     * 更新用户状态
     */
    fun updateStatus(status: String): Boolean {
        return webSocketClient.updateStatus(status)
    }

    // ==================== 消息监听 ====================

    /**
     * 启动消息监听器
     */
    private fun startMessageListener() {
        scope.launch {
            webSocketClient.incomingMessages.collect { message ->
                handleIncomingMessage(message)
            }
        }
    }

    /**
     * 处理收到的消息
     */
    private fun handleIncomingMessage(message: WebSocketMessage) {
        val type = WebSocketMessageType.fromTypeName(message.type) ?: run {
            Log.w(TAG, "Unknown message type: ${message.type}")
            return
        }

        try {
            when (type) {
                WebSocketMessageType.NewMessage -> {
                    val payload = parsePayload(message.payload, NewMessagePayload::class.java)
                    payload?.let { _newMessages.tryEmit(it) }
                }
                WebSocketMessageType.MessageEdited -> {
                    val payload = parsePayload(message.payload, MessageEditedPayload::class.java)
                    payload?.let { _messageEdited.tryEmit(it) }
                }
                WebSocketMessageType.MessageDeleted -> {
                    val payload = parsePayload(message.payload, MessageDeletedPayload::class.java)
                    payload?.let { _messageDeleted.tryEmit(it) }
                }
                WebSocketMessageType.UserTyping -> {
                    val payload = parsePayload(message.payload, UserTypingPayload::class.java)
                    payload?.let { _userTyping.tryEmit(it) }
                }
                WebSocketMessageType.UserStopTyping -> {
                    val payload = parsePayload(message.payload, UserStopTypingPayload::class.java)
                    payload?.let { _userStopTyping.tryEmit(it) }
                }
                WebSocketMessageType.UserJoined -> {
                    val payload = parsePayload(message.payload, UserJoinedPayload::class.java)
                    payload?.let { _userJoined.tryEmit(it) }
                }
                WebSocketMessageType.UserLeft -> {
                    val payload = parsePayload(message.payload, UserLeftPayload::class.java)
                    payload?.let { _userLeft.tryEmit(it) }
                }
                WebSocketMessageType.OnlineUsers -> {
                    val payload = parsePayload(message.payload, OnlineUsersPayload::class.java)
                    payload?.let { _onlineUsers.tryEmit(it) }
                }
                WebSocketMessageType.RoomMessageSummary -> {
                    val payload = parsePayload(message.payload, RoomMessageSummaryPayload::class.java)
                    payload?.let { _roomMessageSummary.tryEmit(it) }
                }
                WebSocketMessageType.RoomJoined -> {
                    val payload = parsePayload(message.payload, RoomJoinedPayload::class.java)
                    Log.d(TAG, "Room joined: ${payload?.roomId}")
                }
                WebSocketMessageType.RoomLeft -> {
                    val payload = parsePayload(message.payload, RoomLeftPayload::class.java)
                    Log.d(TAG, "Room left: ${payload?.roomId}")
                }
                else -> {
                    // 其他消息类型暂不处理
                    Log.d(TAG, "Received message type: ${type.typeName}")
                }
            }
        } catch (e: Exception) {
            Log.e(TAG, "Failed to handle message of type ${type.typeName}", e)
        }
    }

    /**
     * 解析消息 payload
     */
    private fun <T> parsePayload(payload: Any?, clazz: Class<T>): T? {
        return try {
            if (payload == null) return null
            val json = moshi.adapter(Any::class.java).toJson(payload)
            moshi.adapter(clazz).fromJson(json)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to parse payload for ${clazz.simpleName}", e)
            null
        }
    }

    // ==================== 辅助方法 ====================

    /**
     * 将 NewMessagePayload 转换为 MessageDto
     */
    fun convertToMessageDto(payload: NewMessagePayload): MessageDto {
        return MessageDto(
            id = payload.messageId,
            roomId = payload.roomId,
            sender = UserInfo(
                id = payload.senderId,
                username = payload.senderName,
                avatarUrl = null
            ),
            content = payload.content,
            messageType = "text",
            replyTo = payload.replyTo,
            replyToMessage = payload.replyToMessage?.let { convertReplyToMessage(it) },
            isDeleted = false,
            createdAt = payload.createdAt,
            editCount = 0,
            editedAt = null
        )
    }

    /**
     * 转换回复消息信息
     */
    private fun convertReplyToMessage(replyInfo: ReplyToMessageInfo): Map<String, Any> {
        return mapOf(
            "id" to replyInfo.id,
            "sender_id" to replyInfo.senderId,
            "sender_name" to replyInfo.senderName,
            "content" to replyInfo.content,
            "created_at" to replyInfo.createdAt
        )
    }

    /**
     * 获取指定房间的新消息流
     */
    fun getRoomNewMessages(roomId: String): Flow<NewMessagePayload> {
        return newMessages.filter { it.roomId == roomId }
    }

    /**
     * 获取指定房间的用户输入状态流
     */
    fun getRoomTypingStatus(roomId: String): Flow<UserTypingPayload> {
        return userTyping.filter { it.roomId == roomId }
    }

    /**
     * 获取指定房间的用户停止输入状态流
     */
    fun getRoomStopTypingStatus(roomId: String): Flow<UserStopTypingPayload> {
        return userStopTyping.filter { it.roomId == roomId }
    }
}
