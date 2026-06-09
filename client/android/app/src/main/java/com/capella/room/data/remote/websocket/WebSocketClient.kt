package com.capella.room.data.remote.websocket

import android.util.Log
import com.capella.room.data.local.TokenManager
import com.squareup.moshi.Moshi
import com.squareup.moshi.kotlin.reflect.KotlinJsonAdapterFactory
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import org.java_websocket.client.WebSocketClient
import org.java_websocket.handshake.ServerHandshake
import java.net.URI
import java.time.Instant
import java.util.concurrent.atomic.AtomicBoolean
import javax.inject.Inject
import javax.inject.Singleton

/**
 * WebSocket 客户端管理器
 * 负责连接管理、认证、心跳、重连等核心功能
 */
@Singleton
class WebSocketClient @Inject constructor(
    private val tokenManager: TokenManager
) {
    companion object {
        private const val TAG = "WebSocketClient"
        // 生产环境 WebSocket (Cloudflare Tunnel - 使用 wss)
        private const val WS_URL = "wss://chat.moribund.top/ws"
        // 本地开发环境
        // private const val WS_URL = "ws://10.0.2.2:3000/ws"
        private const val AUTH_TIMEOUT_MS = 30000L
        private const val HEARTBEAT_INTERVAL_MS = 30000L
        private const val HEARTBEAT_TIMEOUT_MS = 90000L
        private const val RECONNECT_BASE_DELAY_MS = 1000L
        private const val RECONNECT_MAX_DELAY_MS = 30000L
        private const val MAX_RECONNECT_ATTEMPTS = 10
    }

    private val moshi: Moshi = Moshi.Builder()
        .add(KotlinJsonAdapterFactory())
        .build()

    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)

    // WebSocket 实例
    private var webSocket: WebSocketClient? = null

    // 连接状态
    private val _connectionState = MutableStateFlow<WebSocketConnectionState>(WebSocketConnectionState.Disconnected)
    val connectionState: StateFlow<WebSocketConnectionState> = _connectionState.asStateFlow()

    // 收到的消息流
    private val _incomingMessages = MutableSharedFlow<WebSocketMessage>(extraBufferCapacity = 64)
    val incomingMessages: SharedFlow<WebSocketMessage> = _incomingMessages.asSharedFlow()

    // 重连相关
    private var reconnectAttempts = 0
    private var isReconnecting = AtomicBoolean(false)
    private var lastDisconnectTime: Instant? = null

    // 心跳相关
    private var heartbeatJob: Job? = null
    private var lastPongTime: Long = 0

    // 认证状态
    private var isAuthenticated = AtomicBoolean(false)

    /**
     * 连接到 WebSocket 服务器
     */
    fun connect() {
        if (_connectionState.value == WebSocketConnectionState.Connecting ||
            _connectionState.value == WebSocketConnectionState.Connected ||
            _connectionState.value == WebSocketConnectionState.Authenticated
        ) {
            Log.d(TAG, "Already connected or connecting, skipping")
            return
        }

        _connectionState.value = WebSocketConnectionState.Connecting

        try {
            val uri = URI(WS_URL)
            webSocket = object : WebSocketClient(uri) {
                override fun onOpen(handshakedata: ServerHandshake?) {
                    Log.d(TAG, "WebSocket connected")
                    _connectionState.value = WebSocketConnectionState.Connected
                    reconnectAttempts = 0
                    authenticate()
                }

                override fun onMessage(message: String?) {
                    message?.let { handleMessage(it) }
                }

                override fun onClose(code: Int, reason: String?, remote: Boolean) {
                    Log.d(TAG, "WebSocket closed: code=$code, reason=$reason, remote=$remote")
                    handleDisconnect()
                }

                override fun onError(ex: Exception?) {
                    Log.e(TAG, "WebSocket error", ex)
                    _connectionState.value = WebSocketConnectionState.Error(ex?.message ?: "Unknown error")
                }
            }

            webSocket?.connect()
        } catch (e: Exception) {
            Log.e(TAG, "Failed to connect", e)
            _connectionState.value = WebSocketConnectionState.Error(e.message ?: "Connection failed")
            scheduleReconnect()
        }
    }

    /**
     * 断开连接
     */
    fun disconnect() {
        Log.d(TAG, "Disconnecting WebSocket")
        heartbeatJob?.cancel()
        isAuthenticated.set(false)
        webSocket?.close()
        webSocket = null
        _connectionState.value = WebSocketConnectionState.Disconnected
    }

    /**
     * 发送消息
     */
    fun sendMessage(type: WebSocketMessageType, payload: Any? = null): Boolean {
        val ws = webSocket ?: run {
            Log.w(TAG, "Cannot send message: WebSocket not connected")
            return false
        }

        if (ws.readyState != org.java_websocket.enums.ReadyState.OPEN) {
            Log.w(TAG, "Cannot send message: WebSocket not open")
            return false
        }

        return try {
            val message = if (payload != null) {
                WebSocketMessage(type.typeName, payload)
            } else {
                WebSocketMessage(type.typeName, null)
            }

            val json = moshi.adapter(WebSocketMessage::class.java).toJson(message)
            Log.d(TAG, "Sending: $json")
            ws.send(json)
            true
        } catch (e: Exception) {
            Log.e(TAG, "Failed to send message", e)
            false
        }
    }

    /**
     * 加入房间
     */
    fun joinRoom(roomId: String): Boolean {
        return sendMessage(WebSocketMessageType.JoinRoom, JoinRoomPayload(roomId))
    }

    /**
     * 离开房间
     */
    fun leaveRoom(roomId: String): Boolean {
        return sendMessage(WebSocketMessageType.LeaveRoom, LeaveRoomPayload(roomId))
    }

    /**
     * 发送聊天消息
     */
    fun sendChatMessage(roomId: String, content: String, replyTo: String? = null): Boolean {
        return sendMessage(
            WebSocketMessageType.ChatMessage,
            ChatMessagePayload(roomId, content, replyTo)
        )
    }

    /**
     * 发送正在输入状态
     */
    fun sendTyping(roomId: String): Boolean {
        return sendMessage(WebSocketMessageType.Typing, TypingPayload(roomId))
    }

    /**
     * 发送停止输入状态
     */
    fun sendStopTyping(roomId: String): Boolean {
        return sendMessage(WebSocketMessageType.StopTyping, StopTypingPayload(roomId))
    }

    /**
     * 标记消息已读
     */
    fun markMessageRead(messageId: String): Boolean {
        return sendMessage(WebSocketMessageType.MessageRead, MessageReadPayload(messageId))
    }

    /**
     * 编辑消息
     */
    fun editMessage(messageId: String, newContent: String): Boolean {
        return sendMessage(
            WebSocketMessageType.EditMessage,
            EditMessagePayload(messageId, newContent)
        )
    }

    /**
     * 删除消息
     */
    fun deleteMessage(messageId: String): Boolean {
        return sendMessage(WebSocketMessageType.DeleteMessage, DeleteMessagePayload(messageId))
    }

    /**
     * 获取离线消息
     */
    fun getMissedMessages(roomId: String, lastMessageId: String?): Boolean {
        return sendMessage(
            WebSocketMessageType.GetMissedMessages,
            GetMissedMessagesPayload(roomId, lastMessageId)
        )
    }

    /**
     * 更新用户状态
     */
    fun updateStatus(status: String): Boolean {
        return sendMessage(WebSocketMessageType.UpdateStatus, UpdateStatusPayload(status))
    }

    // ==================== 私有方法 ====================

    /**
     * 认证连接
     */
    private fun authenticate() {
        _connectionState.value = WebSocketConnectionState.Authenticating

        scope.launch {
            try {
                val token = tokenManager.getAccessToken()
                if (token == null) {
                    Log.e(TAG, "No token available for authentication")
                    _connectionState.value = WebSocketConnectionState.Error("No authentication token")
                    disconnect()
                    return@launch
                }

                // 发送认证消息
                sendMessage(WebSocketMessageType.Auth, AuthPayload(token))

                // 设置认证超时
                delay(AUTH_TIMEOUT_MS)
                if (!isAuthenticated.get() && _connectionState.value == WebSocketConnectionState.Authenticating) {
                    Log.e(TAG, "Authentication timeout")
                    _connectionState.value = WebSocketConnectionState.Error("Authentication timeout")
                    disconnect()
                }
            } catch (e: Exception) {
                Log.e(TAG, "Authentication failed", e)
                _connectionState.value = WebSocketConnectionState.Error(e.message ?: "Authentication failed")
                disconnect()
            }
        }
    }

    /**
     * 处理收到的消息
     */
    private fun handleMessage(message: String) {
        Log.d(TAG, "Received: $message")

        try {
            val adapter = moshi.adapter(WebSocketMessage::class.java)
            val wsMessage = adapter.fromJson(message) ?: return

            // 处理特殊消息类型
            when (WebSocketMessageType.fromTypeName(wsMessage.type)) {
                WebSocketMessageType.Ping -> handlePing()
                WebSocketMessageType.AuthResult -> handleAuthResult(wsMessage)
                WebSocketMessageType.ReconnectResult -> handleReconnectResult(wsMessage)
                WebSocketMessageType.SessionRestored -> handleSessionRestored(wsMessage)
                else -> {
                    // 其他消息通过 Flow 分发
                    _incomingMessages.tryEmit(wsMessage)
                }
            }
        } catch (e: Exception) {
            Log.e(TAG, "Failed to parse message", e)
        }
    }

    /**
     * 处理 Ping 消息（回复 Pong）
     */
    private fun handlePing() {
        Log.d(TAG, "Received Ping, sending Pong")
        sendMessage(WebSocketMessageType.Pong)
        lastPongTime = System.currentTimeMillis()
    }

    /**
     * 处理认证结果
     */
    private fun handleAuthResult(message: WebSocketMessage) {
        try {
            val payloadJson = moshi.adapter(Any::class.java).toJson(message.payload)
            val authResult = moshi.adapter(AuthResultPayload::class.java).fromJson(payloadJson)

            if (authResult?.success == true) {
                Log.d(TAG, "Authentication successful")
                isAuthenticated.set(true)
                _connectionState.value = WebSocketConnectionState.Authenticated
                startHeartbeat()
            } else {
                Log.e(TAG, "Authentication failed: ${authResult?.message}")
                _connectionState.value = WebSocketConnectionState.Error(
                    authResult?.message ?: "Authentication failed"
                )
                disconnect()
            }
        } catch (e: Exception) {
            Log.e(TAG, "Failed to parse auth result", e)
            _connectionState.value = WebSocketConnectionState.Error("Failed to parse auth result")
            disconnect()
        }
    }

    /**
     * 处理重连结果
     */
    private fun handleReconnectResult(message: WebSocketMessage) {
        try {
            val payloadJson = moshi.adapter(Any::class.java).toJson(message.payload)
            val reconnectResult = moshi.adapter(ReconnectResultPayload::class.java).fromJson(payloadJson)

            if (reconnectResult?.success == true) {
                Log.d(TAG, "Reconnection successful")
                isAuthenticated.set(true)
                _connectionState.value = WebSocketConnectionState.Authenticated
                startHeartbeat()

                // 重新加入房间
                reconnectResult.roomsToRejoin?.forEach { roomId ->
                    joinRoom(roomId)
                }
            } else {
                Log.e(TAG, "Reconnection failed: ${reconnectResult?.message}")
                // 重连失败，尝试普通认证
                authenticate()
            }
        } catch (e: Exception) {
            Log.e(TAG, "Failed to parse reconnect result", e)
            authenticate()
        }
    }

    /**
     * 处理会话恢复完成
     */
    private fun handleSessionRestored(message: WebSocketMessage) {
        try {
            val payloadJson = moshi.adapter(Any::class.java).toJson(message.payload)
            val restored = moshi.adapter(SessionRestoredPayload::class.java).fromJson(payloadJson)
            Log.d(TAG, "Session restored: ${restored?.roomsRestored} rooms, ${restored?.totalUnread} unread messages")
        } catch (e: Exception) {
            Log.e(TAG, "Failed to parse session restored", e)
        }
    }

    /**
     * 处理断开连接
     */
    private fun handleDisconnect() {
        heartbeatJob?.cancel()
        isAuthenticated.set(false)
        lastDisconnectTime = Instant.now()

        if (_connectionState.value != WebSocketConnectionState.Disconnected) {
            _connectionState.value = WebSocketConnectionState.Disconnected
            scheduleReconnect()
        }
    }

    /**
     * 启动心跳检测
     */
    private fun startHeartbeat() {
        heartbeatJob?.cancel()
        lastPongTime = System.currentTimeMillis()

        heartbeatJob = scope.launch {
            while (true) {
                delay(HEARTBEAT_INTERVAL_MS)

                // 检查心跳超时
                val timeSinceLastPong = System.currentTimeMillis() - lastPongTime
                if (timeSinceLastPong > HEARTBEAT_TIMEOUT_MS) {
                    Log.e(TAG, "Heartbeat timeout")
                    disconnect()
                    scheduleReconnect()
                    break
                }
            }
        }
    }

    /**
     * 调度重连
     */
    private fun scheduleReconnect() {
        if (isReconnecting.get()) return
        isReconnecting.set(true)

        scope.launch {
            if (reconnectAttempts >= MAX_RECONNECT_ATTEMPTS) {
                Log.e(TAG, "Max reconnection attempts reached")
                _connectionState.value = WebSocketConnectionState.Error("Max reconnection attempts reached")
                isReconnecting.set(false)
                return@launch
            }

            reconnectAttempts++
            val delayMs = calculateReconnectDelay()
            Log.d(TAG, "Scheduling reconnect attempt $reconnectAttempts in ${delayMs}ms")

            delay(delayMs)

            isReconnecting.set(false)

            // 尝试重连
            tryReconnect()
        }
    }

    /**
     * 尝试重连
     */
    private fun tryReconnect() {
        scope.launch {
            val token = tokenManager.getAccessToken()
            if (token == null) {
                Log.e(TAG, "No token for reconnection, using normal connect")
                connect()
                return@launch
            }

            _connectionState.value = WebSocketConnectionState.Connecting

            try {
                val uri = URI(WS_URL)
                webSocket = object : WebSocketClient(uri) {
                    override fun onOpen(handshakedata: ServerHandshake?) {
                        Log.d(TAG, "WebSocket reconnected")
                        _connectionState.value = WebSocketConnectionState.Connected
                        reconnectAttempts = 0

                        // 发送重连消息
                        sendReconnect(token)
                    }

                    override fun onMessage(message: String?) {
                        message?.let { handleMessage(it) }
                    }

                    override fun onClose(code: Int, reason: String?, remote: Boolean) {
                        Log.d(TAG, "WebSocket closed during reconnect: code=$code, reason=$reason")
                        handleDisconnect()
                    }

                    override fun onError(ex: Exception?) {
                        Log.e(TAG, "WebSocket error during reconnect", ex)
                        _connectionState.value = WebSocketConnectionState.Error(ex?.message ?: "Reconnection error")
                    }
                }

                webSocket?.connect()
            } catch (e: Exception) {
                Log.e(TAG, "Failed to reconnect", e)
                scheduleReconnect()
            }
        }
    }

    /**
     * 发送重连消息
     */
    private fun sendReconnect(token: String) {
        val payload = ReconnectPayload(
            token = token,
            lastDisconnectAt = lastDisconnectTime?.toString()
        )
        sendMessage(WebSocketMessageType.Reconnect, payload)
    }

    /**
     * 计算重连延迟（指数退避）
     */
    private fun calculateReconnectDelay(): Long {
        val delay = RECONNECT_BASE_DELAY_MS * (1 shl (reconnectAttempts - 1))
        return minOf(delay, RECONNECT_MAX_DELAY_MS)
    }
}
