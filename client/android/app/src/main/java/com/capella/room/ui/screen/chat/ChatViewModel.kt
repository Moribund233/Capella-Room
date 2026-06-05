package com.capella.room.ui.screen.chat

import android.net.Uri
import android.util.Log
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.capella.room.data.local.entity.SyncStatus
import com.capella.room.data.remote.api.RoomApi
import com.capella.room.data.remote.dto.MessageDto
import com.capella.room.data.remote.dto.RoomDto
import com.capella.room.data.remote.websocket.WebSocketConnectionState
import com.capella.room.data.repository.FileRepository
import com.capella.room.data.repository.LocalMessageRepository
import com.capella.room.data.repository.LocalRoomRepository
import com.capella.room.data.repository.LocalUserRepository
import com.capella.room.data.repository.WebSocketRepository
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.launch
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneId
import java.time.format.DateTimeFormatter
import javax.inject.Inject

/**
 * 聊天界面状态
 */
data class ChatUiState(
    val roomInfo: RoomDto? = null,
    val messages: List<MessageDto> = emptyList(),
    val isLoading: Boolean = true,
    val isLoadingMore: Boolean = false,
    val hasMoreMessages: Boolean = true,
    val inputText: String = "",
    val showEmojiPanel: Boolean = false,
    val errorMessage: String? = null,
    val connectionState: WebSocketConnectionState = WebSocketConnectionState.Disconnected,
    val typingUsers: List<String> = emptyList(),
    val onlineUserCount: Int = 0,
    val isOfflineMode: Boolean = false, // 离线模式标志
    val pendingMessageCount: Int = 0, // 待发送消息数量
    val currentUserId: String? = null // 当前用户ID
)

/**
 * 聊天室 ViewModel
 * 支持离线模式，消息本地缓存和同步
 */
@HiltViewModel
class ChatViewModel @Inject constructor(
    savedStateHandle: SavedStateHandle,
    private val roomApi: RoomApi,
    private val webSocketRepository: WebSocketRepository,
    private val localMessageRepository: LocalMessageRepository,
    private val localRoomRepository: LocalRoomRepository,
    private val localUserRepository: LocalUserRepository,
    private val fileRepository: FileRepository
) : ViewModel() {

    companion object {
        private const val TAG = "ChatViewModel"
        private const val TYPING_DEBOUNCE_MS = 300L
        private const val STOP_TYPING_DELAY_MS = 3000L
        private const val SYNC_RETRY_DELAY_MS = 5000L
    }

    private val channelId: String = savedStateHandle["channelId"] ?: ""

    /** 当前房间ID */
    val currentRoomId: String get() = channelId

    var uiState by mutableStateOf(ChatUiState())
        private set

    // 输入防抖任务
    private var typingJob: Job? = null
    private var stopTypingJob: Job? = null
    private var hasSentTyping = false
    private var syncJob: Job? = null

    // 当前用户ID
    private var currentUserId: String? = null

    init {
        if (channelId.isNotBlank()) {
            loadCurrentUser()
            loadRoomAndMessages()
            observeLocalMessages()
            observePendingMessages()
            connectWebSocket()
        }
    }

    /**
     * 加载当前用户信息
     */
    private fun loadCurrentUser() {
        viewModelScope.launch {
            val currentUser = localUserRepository.getCurrentUser()
            currentUserId = currentUser?.userId
            uiState = uiState.copy(currentUserId = currentUser?.userId)
        }
    }

    /**
     * 观察本地消息变化
     */
    private fun observeLocalMessages() {
        viewModelScope.launch {
            localMessageRepository.getMessagesFlow(channelId)
                .collectLatest { entities ->
                    val messages = localMessageRepository.toDtoList(entities)
                    uiState = uiState.copy(
                        messages = messages,
                        isLoading = false
                    )
                }
        }
    }

    /**
     * 观察待同步消息
     */
    private fun observePendingMessages() {
        viewModelScope.launch {
            localMessageRepository.getPendingMessagesFlow()
                .collectLatest { pendingMessages ->
                    val roomPendingCount = pendingMessages.count { it.roomId == channelId }
                    uiState = uiState.copy(
                        pendingMessageCount = roomPendingCount
                    )
                }
        }
    }

    /**
     * 连接 WebSocket 并监听消息
     */
    private fun connectWebSocket() {
        // 先建立连接
        webSocketRepository.connect()

        viewModelScope.launch {
            // 监听连接状态
            launch {
                webSocketRepository.connectionState.collect { state ->
                    val wasOffline = uiState.isOfflineMode
                    val isOffline = state != WebSocketConnectionState.Authenticated

                    uiState = uiState.copy(
                        connectionState = state,
                        isOfflineMode = isOffline
                    )
                    Log.d(TAG, "WebSocket state: $state, offline: $isOffline")

                    // 从离线恢复时同步消息
                    if (wasOffline && !isOffline) {
                        syncPendingMessages()
                    }

                    // 认证成功后加入房间
                    if (state == WebSocketConnectionState.Authenticated) {
                        joinRoom()
                    }
                }
            }

            // 监听新消息
            launch {
                webSocketRepository.newMessages.collect { message ->
                    if (message.roomId == channelId) {
                        handleNewMessage(message)
                    }
                }
            }

            // 监听消息编辑
            launch {
                webSocketRepository.messageEdited.collect { edited ->
                    handleMessageEdited(edited.messageId, edited.newContent, edited.editedAt)
                }
            }

            // 监听消息删除
            launch {
                webSocketRepository.messageDeleted.collect { deleted ->
                    handleMessageDeleted(deleted.messageId)
                }
            }

            // 监听用户输入状态
            launch {
                webSocketRepository.userTyping.collect { typing ->
                    if (typing.roomId == channelId) {
                        addTypingUser(typing.username)
                    }
                }
            }

            // 监听用户停止输入
            launch {
                webSocketRepository.userStopTyping.collect { stopTyping ->
                    if (stopTyping.roomId == channelId) {
                        removeTypingUser(stopTyping.username)
                    }
                }
            }

            // 监听在线用户列表
            launch {
                webSocketRepository.onlineUsers.collect { onlineUsers ->
                    if (onlineUsers.roomId == channelId) {
                        uiState = uiState.copy(onlineUserCount = onlineUsers.users.size)
                    }
                }
            }

            // 监听消息已读回执
            launch {
                webSocketRepository.messageReadReceipt.collect { receipt ->
                    handleMessageReadReceipt(receipt.messageId, receipt.userId)
                }
            }

            // 连接 WebSocket
            webSocketRepository.connect()
        }
    }

    /**
     * 同步待发送的消息
     */
    private fun syncPendingMessages() {
        syncJob?.cancel()
        syncJob = viewModelScope.launch {
            val pendingMessages = localMessageRepository.getPendingMessages()
                .filter { it.syncStatus == SyncStatus.PENDING || it.syncStatus == SyncStatus.FAILED }

            pendingMessages.forEach { message ->
                if (message.roomId == channelId) {
                    localMessageRepository.markAsSending(message.id)
                    val success = webSocketRepository.sendMessage(
                        roomId = message.roomId,
                        content = message.content,
                        replyTo = message.replyTo
                    )
                    if (!success) {
                        localMessageRepository.markAsFailed(message.id)
                    }
                }
            }
        }
    }

    /**
     * 加入房间
     */
    private fun joinRoom() {
        if (webSocketRepository.joinRoom(channelId)) {
            Log.d(TAG, "Joined room: $channelId")
            // 标记房间为已加入
            viewModelScope.launch {
                localRoomRepository.joinRoom(channelId)
            }
            // 请求离线消息
            val lastMessageId = uiState.messages.lastOrNull()?.id
            webSocketRepository.getMissedMessages(channelId, lastMessageId)
        } else {
            Log.w(TAG, "Failed to join room: $channelId")
        }
    }

    /**
     * 加载房间信息和消息
     * 优先从本地加载，同时从网络获取最新数据
     */
    private fun loadRoomAndMessages() {
        viewModelScope.launch {
            // 1. 先从本地加载房间信息
            val localRoom = localRoomRepository.getRoomById(channelId)
            if (localRoom != null) {
                uiState = uiState.copy(roomInfo = localRoomRepository.toDto(localRoom))
            }

            // 2. 从网络获取最新房间信息
            try {
                val roomResponse = roomApi.getRoom(channelId)
                if (roomResponse.isSuccessful && roomResponse.body()?.success == true) {
                    val roomDto = roomResponse.body()?.data
                    roomDto?.let {
                        uiState = uiState.copy(roomInfo = it)
                        localRoomRepository.saveRoom(it, isJoined = true)
                    }
                }
            } catch (e: Exception) {
                Log.w(TAG, "Failed to fetch room info from network", e)
                // 网络失败，使用本地数据
                if (localRoom == null) {
                    uiState = uiState.copy(
                        isLoading = false,
                        errorMessage = "无法连接到服务器，请检查网络"
                    )
                }
            }

            // 3. 从网络加载消息
            loadMessagesFromNetwork()
        }
    }

    /**
     * 从网络加载消息
     */
    private suspend fun loadMessagesFromNetwork() {
        try {
            val response = roomApi.getMessages(channelId, limit = 50)
            if (response.isSuccessful && response.body()?.success == true) {
                val data = response.body()?.data
                val msgs = data?.messages?.reversed() ?: emptyList()

                // 保存到本地数据库
                localMessageRepository.saveMessages(msgs)

                uiState = uiState.copy(
                    isLoading = false,
                    hasMoreMessages = data?.hasMore ?: false
                )
            } else {
                uiState = uiState.copy(isLoading = false)
            }
        } catch (e: Exception) {
            Log.e(TAG, "Failed to load messages from network", e)
            uiState = uiState.copy(isLoading = false)
            // 本地数据已经通过 Flow 自动加载
        }
    }

    /**
     * 加载更多消息（历史消息）
     */
    fun loadMoreMessages() {
        if (uiState.isLoadingMore || !uiState.hasMoreMessages || uiState.messages.isEmpty()) return

        val oldestMessage = uiState.messages.firstOrNull() ?: return
        uiState = uiState.copy(isLoadingMore = true)

        viewModelScope.launch {
            try {
                val response = roomApi.getMessages(channelId, limit = 50, before = oldestMessage.id)
                if (response.isSuccessful && response.body()?.success == true) {
                    val data = response.body()?.data
                    val olderMsgs = data?.messages?.reversed() ?: emptyList()

                    // 保存到本地
                    localMessageRepository.saveMessages(olderMsgs)

                    uiState = uiState.copy(
                        isLoadingMore = false,
                        hasMoreMessages = data?.hasMore ?: false
                    )
                } else {
                    uiState = uiState.copy(isLoadingMore = false)
                }
            } catch (e: Exception) {
                Log.e(TAG, "Failed to load more messages", e)
                uiState = uiState.copy(isLoadingMore = false)
            }
        }
    }

    /**
     * 更新输入文本
     */
    fun updateInputText(text: String) {
        uiState = uiState.copy(inputText = text)
        handleTypingStatus()
    }

    /**
     * 处理输入状态（防抖）
     */
    private fun handleTypingStatus() {
        stopTypingJob?.cancel()

        typingJob?.cancel()
        typingJob = viewModelScope.launch {
            delay(TYPING_DEBOUNCE_MS)

            if (!hasSentTyping && uiState.inputText.isNotBlank() && !uiState.isOfflineMode) {
                webSocketRepository.sendTyping(channelId)
                hasSentTyping = true
            }

            stopTypingJob = launch {
                delay(STOP_TYPING_DELAY_MS)
                if (!uiState.isOfflineMode) {
                    webSocketRepository.sendStopTyping(channelId)
                }
                hasSentTyping = false
            }
        }
    }

    /**
     * 切换表情面板
     */
    fun toggleEmojiPanel() {
        uiState = uiState.copy(showEmojiPanel = !uiState.showEmojiPanel)
    }

    /**
     * 插入表情
     */
    fun insertEmoji(emoji: String) {
        uiState = uiState.copy(
            inputText = uiState.inputText + emoji,
            showEmojiPanel = false
        )
        handleTypingStatus()
    }

    /**
     * 发送消息
     * 支持离线模式，消息会先保存到本地，网络恢复后自动同步
     */
    fun sendMessage(content: String? = null, messageType: String = "text", replyTo: String? = null) {
        val text = content ?: uiState.inputText.trim()
        Log.d(TAG, "sendMessage called, text='$text', type='$messageType', isOfflineMode=${uiState.isOfflineMode}, connectionState=${uiState.connectionState}")
        if (text.isBlank()) {
            Log.d(TAG, "sendMessage: text is blank, returning")
            return
        }

        viewModelScope.launch {
            val currentUser = localUserRepository.getCurrentUser()
            Log.d(TAG, "sendMessage: currentUser=$currentUser")
            if (currentUser == null) {
                uiState = uiState.copy(errorMessage = "用户未登录")
                return@launch
            }

            // 创建本地待发送消息
            val pendingMessage = localMessageRepository.createPendingMessage(
                roomId = channelId,
                senderId = currentUser.userId,
                senderName = currentUser.username,
                content = text,
                messageType = messageType,
                replyTo = replyTo
            )
            Log.d(TAG, "sendMessage: created pending message ${pendingMessage.id}")

            // 清空输入框（仅当是文本输入时）
            if (content == null) {
                uiState = uiState.copy(inputText = "")
            }

            // 发送停止输入状态
            if (!uiState.isOfflineMode && content == null) {
                webSocketRepository.sendStopTyping(channelId)
            }
            hasSentTyping = false
            stopTypingJob?.cancel()

            // 如果在线，立即尝试发送
            if (!uiState.isOfflineMode) {
                Log.d(TAG, "sendMessage: sending via WebSocket")
                localMessageRepository.markAsSending(pendingMessage.id)
                val success = webSocketRepository.sendMessage(channelId, text, replyTo)
                Log.d(TAG, "sendMessage: WebSocket send result=$success")
                if (!success) {
                    localMessageRepository.markAsFailed(pendingMessage.id)
                    uiState = uiState.copy(errorMessage = "发送失败，消息已保存，将自动重试")
                }
            } else {
                // 离线状态，消息已保存到本地，稍后自动同步
                Log.d(TAG, "Message saved locally for offline sync: ${pendingMessage.id}")
            }
        }
    }

    /**
     * 处理新消息（来自 WebSocket）
     */
    private fun handleNewMessage(message: com.capella.room.data.remote.websocket.NewMessagePayload) {
        viewModelScope.launch {
            val messageDto = webSocketRepository.convertToMessageDto(message)

            // 检查是否是本地临时消息的确认
            val pendingMessages = localMessageRepository.getPendingMessages()
            val matchingPending = pendingMessages.find {
                it.roomId == message.roomId &&
                it.content == message.content &&
                it.senderId == message.senderId
            }

            if (matchingPending != null) {
                // 确认本地消息已发送成功
                localMessageRepository.confirmMessageSent(matchingPending.id, messageDto)
            } else {
                // 保存新消息到本地
                localMessageRepository.saveMessage(messageDto)
            }

            // 标记消息已读（如果是其他人发送的消息）
            if (message.senderId != currentUserId) {
                webSocketRepository.markMessageRead(message.messageId)
                // 更新未读数
                localRoomRepository.incrementUnreadCount(channelId)
            }

            // 更新房间最后消息
            localRoomRepository.updateLastMessage(
                roomId = channelId,
                messageId = message.messageId,
                content = message.content,
                senderName = message.senderName,
                time = message.createdAt
            )
        }
    }

    /**
     * 处理消息编辑（来自 WebSocket 的实时通知）
     */
    private fun handleMessageEdited(messageId: String, newContent: String, editedAt: String) {
        viewModelScope.launch {
            localMessageRepository.confirmMessageEdited(messageId, editedAt)
        }
    }

    /**
     * 处理消息删除
     */
    private fun handleMessageDeleted(messageId: String) {
        viewModelScope.launch {
            localMessageRepository.deleteMessage(messageId)
        }
    }

    /**
     * 处理消息已读回执
     */
    private fun handleMessageReadReceipt(messageId: String, userId: String) {
        viewModelScope.launch {
            // 更新本地消息的已读状态
            localMessageRepository.markAsRead(messageId)
            // TODO: 更新已读人数统计
        }
    }

    /**
     * 添加正在输入的用户
     */
    private fun addTypingUser(username: String) {
        if (!uiState.typingUsers.contains(username)) {
            uiState = uiState.copy(typingUsers = uiState.typingUsers + username)
        }
    }

    /**
     * 移除正在输入的用户
     */
    private fun removeTypingUser(username: String) {
        uiState = uiState.copy(typingUsers = uiState.typingUsers - username)
    }

    /**
     * 编辑消息
     */
    fun editMessage(messageId: String, newContent: String) {
        viewModelScope.launch {
            // 先更新本地
            localMessageRepository.updateMessageContent(messageId, newContent)

            // 如果在线，同步到服务器
            if (!uiState.isOfflineMode) {
                val success = webSocketRepository.editMessage(messageId, newContent)
                if (success) {
                    val editedAt = java.time.Instant.now().toString()
                    localMessageRepository.confirmMessageEdited(messageId, editedAt)
                }
            }
        }
    }

    /**
     * 删除消息
     */
    fun deleteMessage(messageId: String) {
        viewModelScope.launch {
            // 先软删除本地
            localMessageRepository.deleteMessage(messageId)

            // 如果在线，同步到服务器
            if (!uiState.isOfflineMode) {
                val success = webSocketRepository.deleteMessage(messageId)
                if (success) {
                    localMessageRepository.confirmMessageDeleted(messageId)
                }
            }
        }
    }

    /**
     * 重试发送失败的消息
     */
    fun retryFailedMessage(messageId: String) {
        if (uiState.isOfflineMode) {
            uiState = uiState.copy(errorMessage = "当前处于离线状态，请检查网络连接")
            return
        }

        viewModelScope.launch {
            val message = localMessageRepository.getMessageById(messageId)
            if (message != null && message.syncStatus == SyncStatus.FAILED) {
                localMessageRepository.markAsSending(messageId)
                val success = webSocketRepository.sendMessage(
                    roomId = message.roomId,
                    content = message.content,
                    replyTo = message.replyTo
                )
                if (!success) {
                    localMessageRepository.markAsFailed(messageId)
                }
            }
        }
    }

    /**
     * 清除错误消息
     */
    fun clearError() {
        uiState = uiState.copy(errorMessage = null)
    }

    // ==================== 消息搜索 ====================

    /**
     * 搜索本地缓存的消息
     */
    suspend fun searchLocalMessages(roomId: String, query: String): List<MessageDto> {
        return try {
            val entities = localMessageRepository.searchMessages(roomId, query)
            localMessageRepository.toDtoList(entities)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to search local messages", e)
            emptyList()
        }
    }

    /**
     * 搜索远程消息
     */
    suspend fun searchRemoteMessages(roomId: String, query: String): List<MessageDto> {
        return try {
            val response = roomApi.searchMessages(
                query = query,
                roomId = roomId,
                limit = 50
            )
            if (response.isSuccessful && response.body()?.success == true) {
                response.body()?.data ?: emptyList()
            } else {
                emptyList()
            }
        } catch (e: Exception) {
            Log.e(TAG, "Failed to search remote messages", e)
            emptyList()
        }
    }

    // ==================== 文件上传 ====================

    /**
     * 上传图片
     */
    fun uploadImage(uri: Uri) {
        if (uiState.isOfflineMode) {
            uiState = uiState.copy(errorMessage = "当前处于离线状态，无法上传文件")
            return
        }

        viewModelScope.launch {
            uiState = uiState.copy(isLoading = true)
            val result = fileRepository.uploadImage(uri)
            result.fold(
                onSuccess = { fileDto ->
                    // 发送图片消息
                    sendMessage("[图片] ${fileDto.url}", messageType = "image")
                    uiState = uiState.copy(isLoading = false)
                },
                onFailure = { error ->
                    uiState = uiState.copy(
                        isLoading = false,
                        errorMessage = "上传失败: ${error.message}"
                    )
                }
            )
        }
    }

    /**
     * 上传文件
     */
    fun uploadFile(uri: Uri) {
        if (uiState.isOfflineMode) {
            uiState = uiState.copy(errorMessage = "当前处于离线状态，无法上传文件")
            return
        }

        viewModelScope.launch {
            uiState = uiState.copy(isLoading = true)
            val result = fileRepository.uploadFile(uri)
            result.fold(
                onSuccess = { fileDto ->
                    // 发送文件消息
                    sendMessage("[文件] ${fileDto.originalName}\n${fileDto.url}", messageType = "file")
                    uiState = uiState.copy(isLoading = false)
                },
                onFailure = { error ->
                    uiState = uiState.copy(
                        isLoading = false,
                        errorMessage = "上传失败: ${error.message}"
                    )
                }
            )
        }
    }

    // ==================== 格式化工具方法 ====================

    fun formatMessageTime(isoString: String): String {
        return try {
            val instant = Instant.parse(isoString)
            val localDateTime = instant.atZone(ZoneId.systemDefault()).toLocalDateTime()
            val formatter = DateTimeFormatter.ofPattern("HH:mm")
            localDateTime.format(formatter)
        } catch (_: Exception) {
            isoString.take(5)
        }
    }

    fun formatDateLabel(isoString: String): String {
        return try {
            val instant = Instant.parse(isoString)
            val date = instant.atZone(ZoneId.systemDefault()).toLocalDate()
            val today = LocalDate.now()
            val yesterday = today.minusDays(1)
            when (date) {
                today -> "今天"
                yesterday -> "昨天"
                else -> date.format(DateTimeFormatter.ofPattern("M月d日"))
            }
        } catch (_: Exception) {
            ""
        }
    }

    fun shouldShowDateDivider(current: String, previous: String?): Boolean {
        if (previous == null) return true
        return try {
            val currDate = Instant.parse(current).atZone(ZoneId.systemDefault()).toLocalDate()
            val prevDate = Instant.parse(previous).atZone(ZoneId.systemDefault()).toLocalDate()
            currDate != prevDate
        } catch (_: Exception) {
            false
        }
    }

    /**
     * 获取正在输入提示文本
     */
    fun getTypingIndicatorText(): String? {
        return when (uiState.typingUsers.size) {
            0 -> null
            1 -> "${uiState.typingUsers[0]} 正在输入..."
            2 -> "${uiState.typingUsers[0]} 和 ${uiState.typingUsers[1]} 正在输入..."
            else -> "${uiState.typingUsers.size} 人正在输入..."
        }
    }

    override fun onCleared() {
        super.onCleared()
        webSocketRepository.leaveRoom(channelId)
        typingJob?.cancel()
        stopTypingJob?.cancel()
        syncJob?.cancel()
    }
}
