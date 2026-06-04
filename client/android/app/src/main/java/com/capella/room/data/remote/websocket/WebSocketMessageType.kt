package com.capella.room.data.remote.websocket

/**
 * WebSocket 消息类型枚举
 * 对应后端 WebSocket 协议定义的所有消息类型
 */
sealed class WebSocketMessageType(val typeName: String) {

    // ==================== 连接管理 ====================
    object Auth : WebSocketMessageType("Auth")
    object AuthResult : WebSocketMessageType("AuthResult")
    object Ping : WebSocketMessageType("Ping")
    object Pong : WebSocketMessageType("Pong")
    object Reconnect : WebSocketMessageType("Reconnect")
    object ReconnectResult : WebSocketMessageType("ReconnectResult")
    object SessionRestored : WebSocketMessageType("SessionRestored")
    object Error : WebSocketMessageType("Error")

    // ==================== 房间管理 ====================
    object JoinRoom : WebSocketMessageType("JoinRoom")
    object LeaveRoom : WebSocketMessageType("LeaveRoom")
    object RoomJoined : WebSocketMessageType("RoomJoined")
    object RoomLeft : WebSocketMessageType("RoomLeft")
    object UserJoined : WebSocketMessageType("UserJoined")
    object UserLeft : WebSocketMessageType("UserLeft")
    object OnlineUsers : WebSocketMessageType("OnlineUsers")
    object RoomMessageSummary : WebSocketMessageType("RoomMessageSummary")

    // ==================== 消息通信 ====================
    object ChatMessage : WebSocketMessageType("ChatMessage")
    object NewMessage : WebSocketMessageType("NewMessage")
    object Typing : WebSocketMessageType("Typing")
    object StopTyping : WebSocketMessageType("StopTyping")
    object UserTyping : WebSocketMessageType("UserTyping")
    object UserStopTyping : WebSocketMessageType("UserStopTyping")
    object MessageRead : WebSocketMessageType("MessageRead")
    object MessageReadReceipt : WebSocketMessageType("MessageReadReceipt")
    object EditMessage : WebSocketMessageType("EditMessage")
    object MessageEdited : WebSocketMessageType("MessageEdited")
    object DeleteMessage : WebSocketMessageType("DeleteMessage")
    object MessageDeleted : WebSocketMessageType("MessageDeleted")
    object GetMissedMessages : WebSocketMessageType("GetMissedMessages")
    object MissedMessages : WebSocketMessageType("MissedMessages")

    // ==================== 用户状态 ====================
    object UpdateStatus : WebSocketMessageType("UpdateStatus")
    object UserStatusChanged : WebSocketMessageType("UserStatusChanged")
    object GetOnlineUsers : WebSocketMessageType("GetOnlineUsers")
    object GlobalOnlineUsers : WebSocketMessageType("GlobalOnlineUsers")

    // ==================== 通知系统 ====================
    object PrivateMessage : WebSocketMessageType("PrivateMessage")
    object Mentioned : WebSocketMessageType("Mentioned")
    object RoomInvitation : WebSocketMessageType("RoomInvitation")
    object SystemNotification : WebSocketMessageType("SystemNotification")
    object FileUploadComplete : WebSocketMessageType("FileUploadComplete")

    companion object {
        /**
         * 根据类型名称查找对应的枚举值
         */
        fun fromTypeName(typeName: String): WebSocketMessageType? {
            return when (typeName) {
                "Auth" -> Auth
                "AuthResult" -> AuthResult
                "Ping" -> Ping
                "Pong" -> Pong
                "Reconnect" -> Reconnect
                "ReconnectResult" -> ReconnectResult
                "SessionRestored" -> SessionRestored
                "Error" -> Error
                "JoinRoom" -> JoinRoom
                "LeaveRoom" -> LeaveRoom
                "RoomJoined" -> RoomJoined
                "RoomLeft" -> RoomLeft
                "UserJoined" -> UserJoined
                "UserLeft" -> UserLeft
                "OnlineUsers" -> OnlineUsers
                "RoomMessageSummary" -> RoomMessageSummary
                "ChatMessage" -> ChatMessage
                "NewMessage" -> NewMessage
                "Typing" -> Typing
                "StopTyping" -> StopTyping
                "UserTyping" -> UserTyping
                "UserStopTyping" -> UserStopTyping
                "MessageRead" -> MessageRead
                "MessageReadReceipt" -> MessageReadReceipt
                "EditMessage" -> EditMessage
                "MessageEdited" -> MessageEdited
                "DeleteMessage" -> DeleteMessage
                "MessageDeleted" -> MessageDeleted
                "GetMissedMessages" -> GetMissedMessages
                "MissedMessages" -> MissedMessages
                "UpdateStatus" -> UpdateStatus
                "UserStatusChanged" -> UserStatusChanged
                "GetOnlineUsers" -> GetOnlineUsers
                "GlobalOnlineUsers" -> GlobalOnlineUsers
                "PrivateMessage" -> PrivateMessage
                "Mentioned" -> Mentioned
                "RoomInvitation" -> RoomInvitation
                "SystemNotification" -> SystemNotification
                "FileUploadComplete" -> FileUploadComplete
                else -> null
            }
        }
    }
}

/**
 * WebSocket 连接状态
 */
sealed class WebSocketConnectionState {
    object Disconnected : WebSocketConnectionState()
    object Connecting : WebSocketConnectionState()
    object Connected : WebSocketConnectionState()
    object Authenticating : WebSocketConnectionState()
    object Authenticated : WebSocketConnectionState()
    data class Error(val message: String) : WebSocketConnectionState()
}

/**
 * WebSocket 错误码
 */
sealed class WebSocketErrorCode(val code: String) {
    object AuthRequired : WebSocketErrorCode("AUTH_REQUIRED")
    object AuthFailed : WebSocketErrorCode("AUTH_FAILED")
    object TokenExpired : WebSocketErrorCode("TOKEN_EXPIRED")
    object InvalidMessage : WebSocketErrorCode("INVALID_MESSAGE")
    object NotInRoom : WebSocketErrorCode("NOT_IN_ROOM")
    object RoomNotFound : WebSocketErrorCode("ROOM_NOT_FOUND")
    object RateLimited : WebSocketErrorCode("RATE_LIMITED")
    object HeartbeatTimeout : WebSocketErrorCode("HEARTBEAT_TIMEOUT")
    object IpBlocked : WebSocketErrorCode("IP_BLOCKED")
    object ServerError : WebSocketErrorCode("SERVER_ERROR")
    object InvalidContent : WebSocketErrorCode("INVALID_CONTENT")
    object NotMember : WebSocketErrorCode("NOT_MEMBER")
    object JoinFailed : WebSocketErrorCode("JOIN_FAILED")
    object InvalidReply : WebSocketErrorCode("INVALID_REPLY")
    object SaveFailed : WebSocketErrorCode("SAVE_FAILED")

    companion object {
        fun fromCode(code: String): WebSocketErrorCode? {
            return when (code) {
                "AUTH_REQUIRED" -> AuthRequired
                "AUTH_FAILED" -> AuthFailed
                "TOKEN_EXPIRED" -> TokenExpired
                "INVALID_MESSAGE" -> InvalidMessage
                "NOT_IN_ROOM" -> NotInRoom
                "ROOM_NOT_FOUND" -> RoomNotFound
                "RATE_LIMITED" -> RateLimited
                "HEARTBEAT_TIMEOUT" -> HeartbeatTimeout
                "IP_BLOCKED" -> IpBlocked
                "SERVER_ERROR" -> ServerError
                "INVALID_CONTENT" -> InvalidContent
                "NOT_MEMBER" -> NotMember
                "JOIN_FAILED" -> JoinFailed
                "INVALID_REPLY" -> InvalidReply
                "SAVE_FAILED" -> SaveFailed
                else -> null
            }
        }
    }
}
