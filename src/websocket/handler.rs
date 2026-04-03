use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::{interval, timeout};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    state::AppState,
    utils::logging::{PerformanceTimer, StructuredLogger},
    websocket::protocol::{MissedMessage, UserInfo, UserStatus, WebSocketMessage},
};

/// WebSocket 消息通道缓冲区大小
/// 每个连接最多缓冲 100 条待发送消息，超过后会触发背压机制
const WS_MESSAGE_BUFFER_SIZE: usize = 100;

/// WebSocket升级处理器
/// 处理WebSocket连接升级请求
pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// 处理 WebSocket 连接
/// 管理连接的整个生命周期
async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let _timer = PerformanceTimer::new("websocket_connection");
    info!("New WebSocket connection established");

    // 分割 socket 为发送和接收部分
    let (mut sender, mut receiver) = socket.split();

    // 创建有界消息通道用于从其他任务发送消息到 WebSocket
    // 使用有界通道（而非 unbounded_channel）实现背压机制
    // 当缓冲区满时，发送方会被阻塞，防止内存溢出
    let (tx, mut rx) = mpsc::channel::<String>(WS_MESSAGE_BUFFER_SIZE);

    // 等待认证或重连消息
    let (user_id, username, rooms_to_rejoin, is_reconnect) =
        match wait_for_auth(&mut receiver, &state).await {
            Ok(AuthResult::NewConnection { user_id, username }) => {
                info!(
                    "WebSocket authenticated for user: {} ({})",
                    username, user_id
                );
                StructuredLogger::websocket_connect(user_id, &username, None);
                // 发送认证成功消息
                let auth_success = WebSocketMessage::auth_success();
                if let Ok(json) = auth_success.to_json() {
                    let _ = sender.send(Message::Text(json)).await;
                }
                (user_id, username, Vec::new(), false)
            }
            Ok(AuthResult::Reconnection {
                user_id,
                username,
                rooms_to_rejoin,
            }) => {
                info!(
                    "WebSocket reconnected for user: {} ({}), rooms: {:?}",
                    username, user_id, rooms_to_rejoin
                );
                // 发送重连成功消息
                let reconnect_success =
                    WebSocketMessage::reconnect_success(rooms_to_rejoin.clone());
                if let Ok(json) = reconnect_success.to_json() {
                    let _ = sender.send(Message::Text(json)).await;
                }
                (user_id, username, rooms_to_rejoin, true)
            }
            Err(e) => {
                warn!("WebSocket authentication failed: {}", e);
                // 发送认证失败消息
                let auth_fail = WebSocketMessage::auth_failed(&e.to_string());
                if let Ok(json) = auth_fail.to_json() {
                    let _ = sender.send(Message::Text(json)).await;
                }
                // 关闭连接
                let _ = sender.close().await;
                return;
            }
        };

    // 如果是重连，先断开旧连接
    if is_reconnect && state.ws_manager().is_user_connected(user_id) {
        info!("Disconnecting old connection for user: {}", user_id);
        state.ws_manager().disconnect(user_id);
    }

    // 注册连接到管理器
    state
        .ws_manager()
        .connect(user_id, username.clone(), tx.clone());

    // 如果是重连，自动重新加入之前的房间
    if is_reconnect {
        let mut restored_rooms = 0;
        for room_id in &rooms_to_rejoin {
            // 检查用户是否仍在房间中（数据库层面）
            match state
                .room_service()
                .is_user_in_room(*room_id, user_id)
                .await
            {
                Ok(true) => {
                    state.ws_manager().join_room(*room_id, user_id);
                    restored_rooms += 1;
                    info!("User {} rejoined room {}", user_id, room_id);

                    // 通知房间内其他用户
                    let user_joined_msg = WebSocketMessage::UserJoined {
                        room_id: *room_id,
                        user_id,
                        username: username.clone(),
                    };
                    if let Ok(json) = user_joined_msg.to_json() {
                        state
                            .ws_manager()
                            .broadcast_to_room(*room_id, json, Some(user_id))
                            .await;
                    }
                }
                Ok(false) => {
                    debug!(
                        "User {} is no longer in room {}, skipping",
                        user_id, room_id
                    );
                }
                Err(e) => {
                    warn!("Failed to check room membership: {}", e);
                }
            }
        }

        // 发送会话恢复完成消息
        let restored_msg = WebSocketMessage::session_restored(restored_rooms, 0);
        if let Ok(json) = restored_msg.to_json() {
            let _ = tx.send(json).await;
        }

        info!(
            "Session restored for user: {}, restored {} rooms",
            user_id, restored_rooms
        );
    }

    // 创建心跳状态
    let last_pong = Arc::new(std::sync::Mutex::new(Instant::now()));
    let last_pong_clone = Arc::clone(&last_pong);

    // 启动消息发送任务
    let mut send_task = tokio::spawn(async move {
        // 发送心跳间隔
        let mut heartbeat_interval = interval(Duration::from_secs(30));
        // 心跳超时时间（90秒 = 3次心跳未响应）
        let heartbeat_timeout = Duration::from_secs(90);

        loop {
            tokio::select! {
                // 从通道接收消息并发送给客户端
                Some(message) = rx.recv() => {
                    if sender.send(Message::Text(message)).await.is_err() {
                        break;
                    }
                }
                // 定时发送心跳并检查超时
                _ = heartbeat_interval.tick() => {
                    // 检查上次收到 Pong 的时间是否超时
                    let should_disconnect = if let Ok(last) = last_pong.lock() {
                        last.elapsed() > heartbeat_timeout
                    } else {
                        false
                    };

                    if should_disconnect {
                        warn!("Heartbeat timeout for user: {}, disconnecting", user_id);
                        // 发送超时错误消息
                        let timeout_msg = WebSocketMessage::error("HEARTBEAT_TIMEOUT", "Connection closed due to heartbeat timeout");
                        if let Ok(json) = timeout_msg.to_json() {
                            let _ = sender.send(Message::Text(json)).await;
                        }
                        break;
                    }

                    // 发送 Ping
                    let ping = WebSocketMessage::Ping.to_json().unwrap_or_default();
                    if sender.send(Message::Text(ping)).await.is_err() {
                        break;
                    }
                }
            }
        }
    });

    // 创建状态克隆用于接收任务
    let state_clone = Arc::clone(&state);
    let tx_clone = tx.clone();
    let username_for_recv = username.clone();

    // 启动消息接收任务
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(message)) = receiver.next().await {
            match message {
                Message::Text(text) => match WebSocketMessage::from_json(&text) {
                    Ok(ws_msg) => {
                        if let Err(e) = handle_message(
                            ws_msg,
                            user_id,
                            &username_for_recv,
                            &state_clone,
                            &tx_clone,
                            &last_pong_clone,
                        )
                        .await
                        {
                            warn!("Error handling message: {}", e);
                        }
                    }
                    Err(e) => {
                        warn!("Failed to parse WebSocket message: {}", e);
                        let error_msg = WebSocketMessage::error(
                            "INVALID_MESSAGE",
                            &format!("Failed to parse message: {}", e),
                        );
                        if let Ok(json) = error_msg.to_json() {
                            let _ = tx_clone.send(json).await;
                        }
                    }
                },
                Message::Close(_) => {
                    info!("WebSocket connection closed by client");
                    break;
                }
                Message::Ping(_data) => {
                    // 自动回复 pong
                    debug!("Received ping, sending pong");
                }
                Message::Pong(_) => {
                    // 更新最后 pong 时间
                    if let Ok(mut last) = last_pong_clone.lock() {
                        *last = Instant::now();
                    }
                }
                _ => {}
            }
        }
    });

    // 等待任一任务结束
    tokio::select! {
        _ = &mut send_task => {
            recv_task.abort();
        }
        _ = &mut recv_task => {
            send_task.abort();
        }
    }

    // 断开连接清理
    info!("WebSocket connection closed for user: {}", user_id);
    StructuredLogger::websocket_disconnect(user_id, &username, "connection_closed");
    state.ws_manager().disconnect(user_id);
}

/// 认证结果类型
enum AuthResult {
    /// 新连接
    NewConnection { user_id: Uuid, username: String },
    /// 重连（携带需要恢复的房间列表）
    Reconnection {
        user_id: Uuid,
        username: String,
        rooms_to_rejoin: Vec<Uuid>,
    },
}

/// 等待客户端发送认证或重连消息
async fn wait_for_auth(
    receiver: &mut futures::stream::SplitStream<WebSocket>,
    state: &AppState,
) -> anyhow::Result<AuthResult> {
    // 设置认证超时（30 秒，给予客户端更充足的时间）
    let auth_timeout = Duration::from_secs(30);

    let result = timeout(auth_timeout, async {
        // 等待第一个消息（认证或重连）
        match receiver.next().await {
            Some(Ok(Message::Text(text))) => {
                match WebSocketMessage::from_json(&text) {
                    // 普通认证（新连接）
                    Ok(WebSocketMessage::Auth { token }) => {
                        info!("Received authentication request");
                        authenticate_token(&token, state)
                            .await
                            .map(|(user_id, username)| AuthResult::NewConnection {
                                user_id,
                                username,
                            })
                    }
                    // 重连请求
                    Ok(WebSocketMessage::Reconnect {
                        token,
                        last_disconnect_at: _,
                    }) => {
                        info!("Received reconnection request");
                        handle_reconnect(&token, state).await
                    }
                    Ok(msg) => {
                        warn!(
                            "First message must be authentication or reconnect, got: {:?}",
                            msg
                        );
                        Err(anyhow::anyhow!(
                            "First message must be authentication or reconnect"
                        ))
                    }
                    Err(e) => {
                        warn!("Invalid message format: {}", e);
                        Err(anyhow::anyhow!("Invalid message format: {}", e))
                    }
                }
            }
            Some(Ok(_)) => {
                warn!("Received non-text message before authentication");
                Err(anyhow::anyhow!("First message must be text"))
            }
            Some(Err(e)) => {
                error!("WebSocket error before authentication: {}", e);
                Err(anyhow::anyhow!("WebSocket error: {}", e))
            }
            None => Err(anyhow::anyhow!("Connection closed before authentication")),
        }
    })
    .await;

    match result {
        Ok(Ok(auth_result)) => Ok(auth_result),
        Ok(Err(e)) => {
            error!("Authentication failed: {}", e);
            Err(e)
        }
        Err(_) => {
            error!(
                "Authentication timeout after {} seconds",
                auth_timeout.as_secs()
            );
            Err(anyhow::anyhow!("Authentication timeout"))
        }
    }
}

/// 验证 Token 并返回用户信息
async fn authenticate_token(token: &str, state: &AppState) -> anyhow::Result<(Uuid, String)> {
    debug!("Authenticating token");

    match state.auth_service().verify_access_token(token) {
        Ok(claims) => {
            debug!("Token verified successfully");
            let user_id = state.auth_service().extract_user_id(&claims).map_err(|e| {
                error!("Failed to extract user ID from claims: {}", e);
                anyhow::anyhow!("Invalid user ID: {}", e)
            })?;

            // 获取用户信息
            match state.user_service().get_user_by_id(user_id).await {
                Ok(Some(user)) => {
                    debug!("User authenticated: {} ({})", user.username, user_id);
                    Ok((user_id, user.username))
                }
                Ok(None) => {
                    error!("User not found: {}", user_id);
                    Err(anyhow::anyhow!("User not found"))
                }
                Err(e) => {
                    error!("Failed to get user info: {}", e);
                    Err(anyhow::anyhow!("Failed to get user info: {}", e))
                }
            }
        }
        Err(e) => {
            error!("Token verification failed: {}", e);
            Err(anyhow::anyhow!("Invalid token: {}", e))
        }
    }
}

/// 处理重连请求
async fn handle_reconnect(token: &str, state: &AppState) -> anyhow::Result<AuthResult> {
    debug!("Handling reconnection request");

    // 验证 token
    let (user_id, username) = authenticate_token(token, state).await?;

    // 检查用户是否已有活跃连接
    let rooms_to_rejoin = if state.ws_manager().is_user_connected(user_id) {
        // 如果用户已有连接，获取其加入的房间列表以便恢复
        let rooms = state.ws_manager().get_user_rooms(user_id);
        debug!("User has existing connection, rooms to rejoin: {:?}", rooms);
        rooms
    } else {
        debug!("No existing connection found for user");
        Vec::new()
    };

    info!(
        "User {} ({}) reconnecting, rooms to rejoin: {:?}",
        username, user_id, rooms_to_rejoin
    );

    Ok(AuthResult::Reconnection {
        user_id,
        username,
        rooms_to_rejoin,
    })
}

/// 处理 WebSocket 消息
async fn handle_message(
    msg: WebSocketMessage,
    user_id: Uuid,
    username: &str,
    state: &AppState,
    tx: &mpsc::Sender<String>,
    last_pong: &Arc<std::sync::Mutex<Instant>>,
) -> anyhow::Result<()> {
    match msg {
        // 心跳响应
        WebSocketMessage::Pong => {
            if let Ok(mut last) = last_pong.lock() {
                *last = Instant::now();
            }
            debug!("Received pong from user: {}", user_id);
        }

        // 加入房间
        WebSocketMessage::JoinRoom { room_id } => {
            handle_join_room(room_id, user_id, username, state, tx).await?;
        }

        // 离开房间
        WebSocketMessage::LeaveRoom { room_id } => {
            handle_leave_room(room_id, user_id, username, state, tx).await?;
        }

        // 发送聊天消息
        WebSocketMessage::ChatMessage {
            room_id,
            content,
            reply_to,
        } => {
            handle_chat_message(room_id, user_id, username, content, reply_to, state, tx).await?;
        }

        // 正在输入
        WebSocketMessage::Typing { room_id } => {
            handle_typing(room_id, user_id, username, state).await?;
        }

        // 停止输入
        WebSocketMessage::StopTyping { room_id } => {
            handle_stop_typing(room_id, user_id, username, state).await?;
        }

        // 消息已读
        WebSocketMessage::MessageRead { message_id } => {
            handle_message_read(message_id, user_id, state, tx).await?;
        }

        // 编辑消息
        WebSocketMessage::EditMessage {
            message_id,
            new_content,
        } => {
            handle_edit_message(message_id, user_id, new_content, state, tx).await?;
        }

        // 删除消息
        WebSocketMessage::DeleteMessage { message_id } => {
            handle_delete_message(message_id, user_id, state, tx).await?;
        }

        // 获取离线消息
        WebSocketMessage::GetMissedMessages {
            room_id,
            last_message_id,
        } => {
            handle_get_missed_messages(room_id, user_id, last_message_id, state, tx).await?;
        }

        // 更新用户状态
        WebSocketMessage::UpdateStatus { status } => {
            handle_update_status(user_id, username, status, state, tx).await?;
        }

        // 获取全局在线用户列表
        WebSocketMessage::GetOnlineUsers => {
            handle_get_online_users(state, tx).await?;
        }

        // 其他消息类型
        _ => {
            warn!("Unhandled message type from user {}: {:?}", user_id, msg);
        }
    }

    Ok(())
}

/// 处理加入房间
async fn handle_join_room(
    room_id: Uuid,
    user_id: Uuid,
    username: &str,
    state: &AppState,
    tx: &mpsc::Sender<String>,
) -> anyhow::Result<()> {
    debug!("User {} joining room {}", user_id, room_id);

    // 检查房间是否存在
    match state.room_service().get_room_by_id(room_id).await {
        Ok(Some(_room)) => {
            // 检查用户是否是房间成员
            match state.room_service().get_room_member(room_id, user_id).await {
                Ok(Some(_member)) => {
                    // 加入房间
                    state.ws_manager().join_room(room_id, user_id);

                    // 发送加入成功消息
                    let joined_msg = WebSocketMessage::RoomJoined {
                        room_id,
                        user_id,
                        username: username.to_string(),
                    };
                    if let Ok(json) = joined_msg.to_json() {
                        let _ = tx.send(json).await;
                    }

                    // 广播用户加入消息给其他成员
                    let user_joined = WebSocketMessage::UserJoined {
                        room_id,
                        user_id,
                        username: username.to_string(),
                    };
                    if let Ok(json) = user_joined.to_json() {
                        state
                            .ws_manager()
                            .broadcast_to_room(room_id, json, Some(user_id))
                            .await;
                    }

                    // 发送在线用户列表
                    let online_users: Vec<UserInfo> = state
                        .ws_manager()
                        .get_room_users(room_id)
                        .into_iter()
                        .map(|(id, name)| UserInfo {
                            id,
                            username: name,
                            avatar_url: None,
                            status: UserStatus::Online,
                        })
                        .collect();

                    let online_users_msg = WebSocketMessage::OnlineUsers {
                        room_id,
                        users: online_users,
                    };
                    if let Ok(json) = online_users_msg.to_json() {
                        let _ = tx.send(json).await;
                    }

                    StructuredLogger::room_join(user_id, username, room_id);
                }
                Ok(None) => {
                    let error_msg =
                        WebSocketMessage::error("NOT_MEMBER", "You are not a member of this room");
                    if let Ok(json) = error_msg.to_json() {
                        let _ = tx.send(json).await;
                    }
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("Failed to check membership: {}", e));
                }
            }
        }
        Ok(None) => {
            let error_msg = WebSocketMessage::error("ROOM_NOT_FOUND", "Room not found");
            if let Ok(json) = error_msg.to_json() {
                let _ = tx.send(json).await;
            }
        }
        Err(e) => {
            return Err(anyhow::anyhow!("Failed to get room: {}", e));
        }
    }

    Ok(())
}

/// 处理离开房间
async fn handle_leave_room(
    room_id: Uuid,
    user_id: Uuid,
    username: &str,
    state: &AppState,
    tx: &mpsc::Sender<String>,
) -> anyhow::Result<()> {
    debug!("User {} leaving room {}", user_id, room_id);

    // 离开房间
    state.ws_manager().leave_room(room_id, user_id);

    // 发送离开成功消息
    let left_msg = WebSocketMessage::RoomLeft {
        room_id,
        user_id,
        username: username.to_string(),
    };
    if let Ok(json) = left_msg.to_json() {
        let _ = tx.send(json).await;
    }

    // 广播用户离开消息
    let user_left = WebSocketMessage::UserLeft {
        room_id,
        user_id,
        username: username.to_string(),
    };
    if let Ok(json) = user_left.to_json() {
        state
            .ws_manager()
            .broadcast_to_room(room_id, json, None)
            .await;
    }

    StructuredLogger::room_leave(user_id, username, room_id);
    Ok(())
}

/// 处理聊天消息
async fn handle_chat_message(
    room_id: Uuid,
    user_id: Uuid,
    username: &str,
    content: String,
    reply_to: Option<Uuid>,
    state: &AppState,
    tx: &mpsc::Sender<String>,
) -> anyhow::Result<()> {
    let _timer = PerformanceTimer::new("handle_chat_message");
    debug!("User {} sending message to room {}", user_id, room_id);

    // 1. 先验证消息内容（格式、长度等）
    if let Err(e) = crate::utils::validation::validate_message_content(&content) {
        let error_msg = WebSocketMessage::error("INVALID_CONTENT", &e.to_string());
        if let Ok(json) = error_msg.to_json() {
            let _ = tx.send(json).await;
        }
        return Ok(());
    }

    // 2. 检查用户是否在房间中
    if !state.ws_manager().is_user_in_room(room_id, user_id) {
        let error_msg = WebSocketMessage::error("NOT_IN_ROOM", "You are not in this room");
        if let Ok(json) = error_msg.to_json() {
            let _ = tx.send(json).await;
        }
        return Ok(());
    }

    // 3. 保存消息到数据库
    match state
        .message_service()
        .create_text_message(room_id, user_id, &content, reply_to)
        .await
    {
        Ok(message) => {
            // 广播消息给房间所有成员
            let new_message = WebSocketMessage::NewMessage {
                message_id: message.id,
                room_id,
                sender_id: user_id,
                sender_name: username.to_string(),
                content: message.content,
                reply_to: message.reply_to,
                created_at: message.created_at,
            };

            if let Ok(json) = new_message.to_json() {
                state
                    .ws_manager()
                    .broadcast_to_room_all(room_id, json)
                    .await;
            }

            StructuredLogger::message_sent(
                message.id,
                room_id,
                user_id,
                username,
                content.len(),
                _timer.finish().as_millis(),
            );
        }
        Err(e) => {
            error!("Failed to save message: {}", e);
            let error_msg = WebSocketMessage::error("SAVE_FAILED", "Failed to save message");
            if let Ok(json) = error_msg.to_json() {
                let _ = tx.send(json).await;
            }
        }
    }

    Ok(())
}

/// 处理正在输入
async fn handle_typing(
    room_id: Uuid,
    user_id: Uuid,
    username: &str,
    state: &AppState,
) -> anyhow::Result<()> {
    // 广播输入状态给其他成员
    let typing_msg = serde_json::json!({
        "type": "UserTyping",
        "room_id": room_id,
        "user_id": user_id,
        "username": username,
    });

    if let Ok(json) = serde_json::to_string(&typing_msg) {
        state
            .ws_manager()
            .broadcast_to_room(room_id, json, Some(user_id))
            .await;
    }

    Ok(())
}

/// 处理停止输入
async fn handle_stop_typing(
    room_id: Uuid,
    user_id: Uuid,
    username: &str,
    state: &AppState,
) -> anyhow::Result<()> {
    // 广播停止输入状态给其他成员
    let stop_typing_msg = serde_json::json!({
        "type": "UserStopTyping",
        "room_id": room_id,
        "user_id": user_id,
        "username": username,
    });

    if let Ok(json) = serde_json::to_string(&stop_typing_msg) {
        state
            .ws_manager()
            .broadcast_to_room(room_id, json, Some(user_id))
            .await;
    }

    Ok(())
}

/// 处理消息已读
async fn handle_message_read(
    message_id: Uuid,
    user_id: Uuid,
    _state: &AppState,
    tx: &mpsc::Sender<String>,
) -> anyhow::Result<()> {
    // 这里可以实现消息已读逻辑
    // 例如：更新数据库中的已读状态，然后广播已读回执

    let receipt = WebSocketMessage::MessageReadReceipt {
        message_id,
        user_id,
    };

    if let Ok(json) = receipt.to_json() {
        let _ = tx.send(json).await;
    }

    Ok(())
}

/// 处理编辑消息
async fn handle_edit_message(
    message_id: Uuid,
    _user_id: Uuid,
    new_content: String,
    _state: &AppState,
    tx: &mpsc::Sender<String>,
) -> anyhow::Result<()> {
    // 这里可以实现消息编辑逻辑
    // 需要检查用户是否有权限编辑该消息

    let edited_msg = WebSocketMessage::MessageEdited {
        message_id,
        new_content,
        edited_at: chrono::Utc::now(),
    };

    if let Ok(json) = edited_msg.to_json() {
        let _ = tx.send(json).await;
    }

    Ok(())
}

/// 处理删除消息
async fn handle_delete_message(
    message_id: Uuid,
    _user_id: Uuid,
    _state: &AppState,
    tx: &mpsc::Sender<String>,
) -> anyhow::Result<()> {
    // 这里可以实现消息删除逻辑
    // 需要检查用户是否有权限删除该消息

    let deleted_msg = WebSocketMessage::MessageDeleted { message_id };

    if let Ok(json) = deleted_msg.to_json() {
        let _ = tx.send(json).await;
    }

    Ok(())
}

/// 处理获取离线消息
async fn handle_get_missed_messages(
    room_id: Uuid,
    user_id: Uuid,
    last_message_id: Option<Uuid>,
    state: &AppState,
    tx: &mpsc::Sender<String>,
) -> anyhow::Result<()> {
    // 检查用户是否在房间中
    if !state.ws_manager().is_user_in_room(room_id, user_id) {
        let error_msg = WebSocketMessage::error("NOT_IN_ROOM", "You are not in this room");
        if let Ok(json) = error_msg.to_json() {
            let _ = tx.send(json).await;
        }
        return Ok(());
    }

    // 获取离线消息
    let messages = match state
        .message_service()
        .get_missed_messages(room_id, last_message_id, 50)
        .await
    {
        Ok(msgs) => msgs,
        Err(e) => {
            warn!("Failed to get missed messages: {}", e);
            let error_msg =
                WebSocketMessage::error("FETCH_FAILED", "Failed to fetch missed messages");
            if let Ok(json) = error_msg.to_json() {
                let _ = tx.send(json).await;
            }
            return Ok(());
        }
    };

    // 转换为 MissedMessage
    let ws_messages: Vec<MissedMessage> = messages
        .into_iter()
        .map(|msg| MissedMessage {
            message_id: msg.id,
            room_id: msg.room_id,
            sender_id: msg.sender.id,
            sender_name: msg.sender.username.clone(),
            content: msg.content,
            reply_to: msg.reply_to,
            created_at: msg.created_at,
        })
        .collect();

    let msg_count = ws_messages.len();
    let has_more = msg_count >= 50;

    // 发送离线消息列表
    let missed_msgs = WebSocketMessage::MissedMessages {
        room_id,
        messages: ws_messages,
        has_more,
    };

    if let Ok(json) = missed_msgs.to_json() {
        let _ = tx.send(json).await;
    }

    info!(
        "Sent {} missed messages to user {} for room {}",
        msg_count, user_id, room_id
    );

    Ok(())
}

/// 处理更新用户状态
async fn handle_update_status(
    user_id: Uuid,
    username: &str,
    status: crate::websocket::protocol::UserStatus,
    state: &AppState,
    tx: &mpsc::Sender<String>,
) -> anyhow::Result<()> {
    debug!("User {} updating status to {:?}", user_id, status);

    // 转换协议中的UserStatus为模型中的UserStatus
    let db_status = match status {
        crate::websocket::protocol::UserStatus::Online => crate::models::user::UserStatus::Online,
        crate::websocket::protocol::UserStatus::Away => crate::models::user::UserStatus::Away,
        crate::websocket::protocol::UserStatus::Busy => crate::models::user::UserStatus::Away, // Busy映射为Away
        crate::websocket::protocol::UserStatus::Offline => crate::models::user::UserStatus::Offline,
    };

    // 更新数据库中的用户状态
    match state
        .user_service()
        .update_user_status(user_id, db_status)
        .await
    {
        Ok(_) => {
            // 广播状态变更给用户的所有房间
            let user_rooms = state.ws_manager().get_user_rooms(user_id);
            let status_changed_msg = WebSocketMessage::UserStatusChanged {
                user_id,
                username: username.to_string(),
                status: status.clone(),
            };

            if let Ok(json) = status_changed_msg.to_json() {
                for room_id in user_rooms {
                    state
                        .ws_manager()
                        .broadcast_to_room(room_id, json.clone(), Some(user_id))
                        .await;
                }
            }

            info!("User {} status updated to {:?}", user_id, status);
        }
        Err(e) => {
            warn!("Failed to update user status: {}", e);
            let error_msg =
                WebSocketMessage::error("STATUS_UPDATE_FAILED", "Failed to update status");
            if let Ok(json) = error_msg.to_json() {
                let _ = tx.send(json).await;
            }
        }
    }

    Ok(())
}

/// 处理获取全局在线用户列表
async fn handle_get_online_users(
    state: &AppState,
    tx: &mpsc::Sender<String>,
) -> anyhow::Result<()> {
    debug!("Getting global online users");

    // 获取在线用户（限制100个）
    match state.user_service().get_online_users(100, 0).await {
        Ok(users) => {
            let user_infos: Vec<UserInfo> = users
                .into_iter()
                .map(|u| UserInfo {
                    id: u.id,
                    username: u.username,
                    avatar_url: u.avatar_url,
                    status: crate::websocket::protocol::UserStatus::Online,
                })
                .collect();

            let total = user_infos.len();
            let online_users_msg = WebSocketMessage::GlobalOnlineUsers {
                users: user_infos,
                total,
            };

            if let Ok(json) = online_users_msg.to_json() {
                let _ = tx.send(json).await;
            }

            debug!("Sent {} online users", total);
        }
        Err(e) => {
            warn!("Failed to get online users: {}", e);
            let error_msg = WebSocketMessage::error("FETCH_FAILED", "Failed to fetch online users");
            if let Ok(json) = error_msg.to_json() {
                let _ = tx.send(json).await;
            }
        }
    }

    Ok(())
}
