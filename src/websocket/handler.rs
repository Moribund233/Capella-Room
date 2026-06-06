use axum::{
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, State, WebSocketUpgrade,
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, mpsc};
use tokio::time::{interval, timeout};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    models::security::IpCheckResult,
    state::AppState,
    utils::logging::{PerformanceTimer, StructuredLogger},
    websocket::protocol::{MissedMessage, ReplyToInfo, UserInfo, UserStatus, WebSocketMessage},
};

/// 连接上下文
/// 封装 WebSocket 连接的相关信息，便于传递和扩展
/// 为后续 IP 白名单/黑名单检查等安全功能预留
#[derive(Debug, Clone)]
struct ConnectionContext {
    /// 客户端 IP 地址
    client_ip: SocketAddr,
    /// 用户代理（可选）
    user_agent: Option<String>,
}

impl ConnectionContext {
    fn new(client_ip: SocketAddr) -> Self {
        Self {
            client_ip,
            user_agent: None,
        }
    }
}

/// 连接状态机
/// 用于跟踪 WebSocket 连接的认证状态
#[derive(Debug, Clone, Copy, PartialEq)]
enum ConnectionState {
    /// 未认证状态 - 只允许 Auth/Reconnect 消息
    Unauthenticated,
    /// 已认证状态 - 允许所有业务消息
    Authenticated,
}

impl ConnectionState {
    /// 检查消息类型是否允许在当前状态下发送
    fn is_message_allowed(&self, msg: &WebSocketMessage) -> bool {
        match self {
            ConnectionState::Unauthenticated => matches!(
                msg,
                WebSocketMessage::Auth { .. } | WebSocketMessage::Reconnect { .. }
            ),
            ConnectionState::Authenticated => true,
        }
    }
}

/// WebSocket升级处理器
/// 处理WebSocket连接升级请求
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<AppState>>,
) -> Response {
    // 创建连接上下文，包含客户端 IP 等信息
    let ctx = ConnectionContext::new(addr);
    ws.on_upgrade(move |socket| handle_socket(socket, state, ctx))
}

/// 处理 WebSocket 连接
/// 管理连接的整个生命周期
async fn handle_socket(socket: WebSocket, state: Arc<AppState>, ctx: ConnectionContext) {
    info!(
        "New WebSocket connection established from {}",
        ctx.client_ip
    );

    // 执行 IP 安全检查
    match state
        .ip_security_service()
        .check_ip(ctx.client_ip.ip())
        .await
    {
        Ok(IpCheckResult::Allowed) => {
            debug!("IP {} passed security check", ctx.client_ip.ip());
        }
        Ok(result) => {
            warn!("IP {} blocked: {:?}", ctx.client_ip.ip(), result);
            state
                .ip_security_service()
                .log_ip_check(ctx.client_ip.ip(), &result, ctx.user_agent.as_deref())
                .await;

            // 发送拒绝连接消息并关闭
            let (mut sender, _receiver) = socket.split();
            let reject_msg = WebSocketMessage::Error {
                code: "IP_BLOCKED".to_string(),
                message: result
                    .rejection_reason()
                    .unwrap_or_else(|| "Access denied".to_string()),
            };
            if let Ok(json) = reject_msg.to_json() {
                let _ = sender.send(Message::Text(json)).await;
            }
            let _ = sender.close().await;
            return;
        }
        Err(e) => {
            error!("IP security check failed: {}", e);
            // 根据安全策略，检查失败时拒绝连接
            let (mut sender, _receiver) = socket.split();
            let error_msg = WebSocketMessage::Error {
                code: "SECURITY_CHECK_FAILED".to_string(),
                message: "Security check failed".to_string(),
            };
            if let Ok(json) = error_msg.to_json() {
                let _ = sender.send(Message::Text(json)).await;
            }
            let _ = sender.close().await;
            return;
        }
    }

    // IP 安全检查通过，继续处理
    let _conn_state = ConnectionState::Unauthenticated;

    // 分割 socket 为发送和接收部分
    let (mut sender, mut receiver) = socket.split();

    // 创建有界消息通道用于从其他任务发送消息到 WebSocket
    // 使用有界通道（而非 unbounded_channel）实现背压机制
    // 当缓冲区满时，发送方会被阻塞，防止内存溢出
    // 从配置读取缓冲区大小
    let buffer_size = state
        .config_manager
        .get_config()
        .await
        .websocket
        .message_buffer_size;
    let (tx, mut rx) = mpsc::channel::<String>(buffer_size);

    // 等待认证或重连消息
    let (user_id, username, rooms_to_rejoin, is_reconnect, token) =
        match wait_for_auth(&mut receiver, &state).await {
            Ok(auth_result) => match auth_result {
                AuthResult::NewConnection {
                    user_id,
                    username,
                    token,
                } => (user_id, username, Vec::new(), false, token),
                AuthResult::Reconnection {
                    user_id,
                    username,
                    rooms_to_rejoin,
                    token,
                } => (user_id, username, rooms_to_rejoin, true, token),
            },
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

    // 认证成功，状态已转换为已认证
    debug!(
        "Connection state transitioned to Authenticated for user: {}",
        user_id
    );

    // 记录认证成功日志
    if is_reconnect {
        info!(
            "WebSocket reconnected for user: {} ({}), rooms: {:?}, ip: {}",
            username, user_id, rooms_to_rejoin, ctx.client_ip
        );
        StructuredLogger::websocket_connect(
            user_id,
            &username,
            Some(&ctx.client_ip.ip().to_string()),
        );
        // 发送重连成功消息
        let reconnect_success = WebSocketMessage::reconnect_success(rooms_to_rejoin.clone());
        if let Ok(json) = reconnect_success.to_json() {
            let _ = sender.send(Message::Text(json)).await;
        }
    } else {
        info!(
            "WebSocket authenticated for user: {} ({}), ip: {}",
            username, user_id, ctx.client_ip
        );
        StructuredLogger::websocket_connect(
            user_id,
            &username,
            Some(&ctx.client_ip.ip().to_string()),
        );
        // 发送认证成功消息
        let auth_success = WebSocketMessage::auth_success();
        if let Ok(json) = auth_success.to_json() {
            let _ = sender.send(Message::Text(json)).await;
        }
    }

    // 先断开旧连接（如有），避免旧连接的 cleanup 覆盖新连接数据
    if state.ws_manager().is_user_connected(user_id) {
        state.ws_manager().disconnect(user_id);
    }

    // 注册连接到管理器
    state
        .ws_manager()
        .connect(user_id, username.clone(), tx.clone());

    // 记录连接指标
    state.metrics_collector().record_connection();

    // 自动订阅用户所有已加入房间的消息摘要
    if let Ok(user_rooms) = state.room_service().get_user_rooms(user_id).await {
        let room_ids: Vec<Uuid> = user_rooms.iter().map(|r| r.id).collect();
        state.ws_manager().subscribe_user_rooms(user_id, room_ids);
        debug!(
            "User {} subscribed to {} rooms for message summaries",
            user_id,
            user_rooms.len()
        );
    }

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

    // 从配置读取 WebSocket 心跳配置
    let ws_config = state.config_manager.get_config().await.websocket.clone();
    let heartbeat_interval_secs = ws_config.heartbeat_interval_secs;
    let heartbeat_timeout_secs = ws_config.heartbeat_timeout_secs;

    // 克隆 state 用于发送任务
    let state_for_send = Arc::clone(&state);

    // 启动消息发送任务
    let mut send_task = tokio::spawn(async move {
        // 发送心跳间隔（从配置读取）
        let mut heartbeat_interval = interval(Duration::from_secs(heartbeat_interval_secs));
        // 心跳超时时间（从配置读取）
        let heartbeat_timeout = Duration::from_secs(heartbeat_timeout_secs);
        // Token 验证间隔（每 5 分钟验证一次）
        let mut token_check_interval = interval(Duration::from_secs(300));

        loop {
            tokio::select! {
                // 从通道接收消息并发送给客户端
                // 使用 try_send 类似策略：写超时则丢弃当前消息不阻塞
                Some(message) = rx.recv() => {
                    match tokio::time::timeout(
                        Duration::from_secs(5),
                        sender.send(Message::Text(message)),
                    ).await {
                        Ok(Ok(())) => {}
                        Ok(Err(_)) => break,  // WebSocket 关闭
                        Err(_) => {
                            // 客户端消费慢，丢弃当前消息继续
                            // 避免单客户端慢消费阻塞全系统广播管道
                            debug!("Dropped message for user {} (slow client)", user_id);
                        }
                    }
                }
                // 定时验证 Token 是否过期
                _ = token_check_interval.tick() => {
                    if let Err(e) = state_for_send.auth_service().verify_access_token(&token) {
                        warn!("Token expired for user: {}, disconnecting: {}", user_id, e);
                        // 发送 Token 过期错误消息
                        let expired_msg = WebSocketMessage::error("TOKEN_EXPIRED", "Token expired, please reconnect");
                        if let Ok(json) = expired_msg.to_json() {
                            let _ = sender.send(Message::Text(json)).await;
                        }
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

    // 初始化连接状态为已认证（因为 wait_for_auth 已经成功返回）
    let connection_state = ConnectionState::Authenticated;

    // 启动消息接收任务
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(message)) = receiver.next().await {
            match message {
                Message::Text(text) => match WebSocketMessage::from_json(&text) {
                    Ok(ws_msg) => {
                        // 检查消息是否允许在当前状态下发送
                        if !connection_state.is_message_allowed(&ws_msg) {
                            warn!(
                                "Unauthorized message from user {} in {:?} state: {:?}",
                                user_id, connection_state, ws_msg
                            );
                            let error_msg = WebSocketMessage::error(
                                "UNAUTHORIZED",
                                "Message not allowed before authentication",
                            );
                            if let Ok(json) = error_msg.to_json() {
                                let _ = tx_clone.send(json).await;
                            }
                            continue;
                        }

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

    // 记录断开连接指标
    state.metrics_collector().record_disconnect();
}

/// 认证结果类型
enum AuthResult {
    /// 新连接
    NewConnection {
        user_id: Uuid,
        username: String,
        token: String,
    },
    /// 重连（携带需要恢复的房间列表）
    Reconnection {
        user_id: Uuid,
        username: String,
        rooms_to_rejoin: Vec<Uuid>,
        token: String,
    },
}

/// 等待客户端发送认证或重连消息
async fn wait_for_auth(
    receiver: &mut futures::stream::SplitStream<WebSocket>,
    state: &AppState,
) -> anyhow::Result<AuthResult> {
    // 从配置读取认证超时时间
    let auth_timeout_secs = state
        .config_manager
        .get_config()
        .await
        .websocket
        .auth_timeout_secs;
    let auth_timeout = Duration::from_secs(auth_timeout_secs);

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
                                token: token.clone(),
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
///
/// 优化：优先从JWT claims中获取用户名，避免数据库查询
async fn authenticate_token(token: &str, state: &AppState) -> anyhow::Result<(Uuid, String)> {
    debug!("Authenticating token");

    match state.auth_service().verify_access_token(token) {
        Ok(claims) => {
            debug!("Token verified successfully");
            let user_id = state.auth_service().extract_user_id(&claims).map_err(|e| {
                error!("Failed to extract user ID from claims: {}", e);
                anyhow::anyhow!("Invalid user ID: {}", e)
            })?;

            // 优化：优先从JWT claims中获取用户名，避免数据库查询
            if let Some(username) = claims.username {
                debug!(
                    "User authenticated from JWT claims: {} ({})",
                    username, user_id
                );
                return Ok((user_id, username));
            }

            // 兼容旧token：如果claims中没有用户名，则查询数据库
            warn!(
                "JWT claims missing username, falling back to database query for user: {}",
                user_id
            );
            match state.user_service().get_user_by_id(user_id).await {
                Ok(Some(user)) => {
                    debug!(
                        "User authenticated from database: {} ({})",
                        user.username, user_id
                    );
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
        token: token.to_string(),
    })
}

/// 处理 WebSocket 消息
async fn handle_message(
    msg: WebSocketMessage,
    user_id: Uuid,
    username: &str,
    state: &Arc<AppState>,
    tx: &mpsc::Sender<String>,
    last_pong: &Arc<std::sync::Mutex<Instant>>,
) -> anyhow::Result<()> {
    match msg {
        // 心跳请求 - 回复 Pong（支持客户端主导的心跳）
        WebSocketMessage::Ping => {
            let pong = WebSocketMessage::Pong;
            if let Ok(json) = pong.to_json() {
                let _ = tx.send(json).await;
            }
            debug!("Received ping from user: {}, sent pong", user_id);
        }

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

        // ========== 通知系统 ==========
        // 获取离线通知
        WebSocketMessage::GetOfflineNotifications {
            last_notification_id,
            limit,
        } => {
            handle_get_offline_notifications(user_id, last_notification_id, limit, state, tx)
                .await?;
        }

        // ========== 待办通知系统 ==========
        // 获取待办列表
        WebSocketMessage::GetPendingActions { action_type } => {
            handle_get_pending_actions(user_id, action_type, state, tx).await?;
        }

        // ========== 系统日志流 ==========
        // 订阅系统日志
        WebSocketMessage::SubscribeLogs { level, module } => {
            handle_subscribe_logs(user_id, level, module, state, tx).await?;
        }

        // 取消订阅系统日志
        WebSocketMessage::UnsubscribeLogs => {
            handle_unsubscribe_logs(user_id, state, tx).await?;
        }

        // 其他消息类型
        _ => {
            warn!("Unhandled message type from user {}: {:?}", user_id, msg);
        }
    }

    Ok(())
}

/// 处理获取待办列表
async fn handle_get_pending_actions(
    user_id: Uuid,
    _action_type: Option<String>,
    state: &AppState,
    tx: &mpsc::Sender<String>,
) -> anyhow::Result<()> {
    debug!(
        "Getting pending actions for user {}, action_type={:?}",
        user_id, _action_type
    );

    match state
        .notification_service()
        .get_pending_actions(user_id)
        .await
    {
        Ok(actions) => {
            let total: usize = actions.len();
            let pending_actions_list = WebSocketMessage::PendingActionsList { actions, total };

            if let Ok(json) = pending_actions_list.to_json() {
                let _ = tx.send(json).await;
            }

            debug!("Sent {} pending actions to user {}", total, user_id);
        }
        Err(e) => {
            warn!("Failed to get pending actions: {}", e);
            let error_msg =
                WebSocketMessage::error("FETCH_FAILED", "Failed to fetch pending actions");
            if let Ok(json) = error_msg.to_json() {
                let _ = tx.send(json).await;
            }
        }
    }

    Ok(())
}

/// 处理订阅系统日志
///
/// 注意：此函数会阻塞直到连接断开，因为需要保持日志订阅活跃
async fn handle_subscribe_logs(
    user_id: Uuid,
    level: Option<String>,
    module: Option<String>,
    state: &AppState,
    tx: &mpsc::Sender<String>,
) -> anyhow::Result<()> {
    debug!(
        "User {} subscribing to logs with level={:?}, module={:?}",
        user_id, level, module
    );

    // 获取日志广播器
    let broadcaster = state.log_broadcaster();

    // 创建日志接收器
    let mut log_receiver = broadcaster.subscribe();

    // 发送订阅确认
    let confirm = WebSocketMessage::LogSubscriptionConfirmed {
        success: true,
        message: format!("Subscribed to logs: level={:?}, module={:?}", level, module),
    };
    if let Ok(json) = confirm.to_json() {
        let _ = tx.send(json).await;
    }

    // 创建取消令牌，用于在连接断开时终止任务
    let cancel_token = CancellationToken::new();
    let cancel_token_clone = cancel_token.clone();
    let tx_for_logs = tx.clone();

    // 启动日志转发任务
    let log_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                biased;

                // 检查取消信号
                _ = cancel_token_clone.cancelled() => {
                    debug!("Log subscription cancelled for user {}", user_id);
                    break;
                }

                // 接收日志条目
                entry = log_receiver.recv() => {
                    match entry {
                        Ok(log_entry) => {
                            // 应用过滤器
                            let should_send = match (&level, &module) {
                                (None, None) => true,
                                (Some(l), None) => l == "all" || l == &log_entry.level,
                                (None, Some(m)) => m == "all" || m == &log_entry.target,
                                (Some(l), Some(m)) => {
                                    (l == "all" || l == &log_entry.level)
                                    && (m == "all" || m == &log_entry.target)
                                }
                            };

                            if should_send {
                                let log_msg = WebSocketMessage::LogEntry {
                                    level: log_entry.level,
                                    target: log_entry.target,
                                    message: log_entry.message,
                                    timestamp: log_entry.timestamp,
                                    fields: log_entry.fields,
                                };
                                if let Ok(json) = log_msg.to_json() {
                                    if tx_for_logs.send(json).await.is_err() {
                                        break;
                                    }
                                }
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            debug!("Log receiver lagged behind by {} messages", n);
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            debug!("Log channel closed, stopping log forwarding");
                            break;
                        }
                    }
                }
            }
        }
    });

    // 等待日志任务完成或连接断开
    // 使用 tokio::select 来同时监听两个事件
    tokio::select! {
        _ = log_task => {
            debug!("Log task completed for user {}", user_id);
        }
        _ = async {
            // 定期检查 tx 是否关闭
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            loop {
                interval.tick().await;
                // 检查 tx 是否关闭 - 使用 is_closed() 方法
                if tx.is_closed() {
                    break;
                }
            }
        } => {
            debug!("Connection closed, cancelling log subscription for user {}", user_id);
            cancel_token.cancel();
        }
    }

    Ok(())
}

/// 处理取消订阅系统日志
async fn handle_unsubscribe_logs(
    user_id: Uuid,
    #[allow(unused_variables)] state: &AppState,
    tx: &mpsc::Sender<String>,
) -> anyhow::Result<()> {
    debug!("User {} unsubscribing from logs", user_id);

    let confirm = WebSocketMessage::LogSubscriptionConfirmed {
        success: true,
        message: "Unsubscribed from logs".to_string(),
    };
    if let Ok(json) = confirm.to_json() {
        let _ = tx.send(json).await;
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
    let mut _timer = PerformanceTimer::new("handle_join_room");
    debug!("User {} joining room {}", user_id, room_id);

    // 检查房间是否存在
    match state.room_service().get_room_by_id(room_id).await {
        Ok(Some(room)) => {
            // 检查用户是否是房间成员
            match state.room_service().get_room_member(room_id, user_id).await {
                Ok(Some(_member)) => {
                    // 已经是成员，直接加入
                    do_join_room(room_id, user_id, username, state, tx).await;
                }
                Ok(None) => {
                    // 不是成员，检查房间是否公开
                    if room.is_private {
                        // 私有房间，需要邀请
                        let error_msg = WebSocketMessage::error(
                            "NOT_MEMBER",
                            "You are not a member of this room",
                        );
                        if let Ok(json) = error_msg.to_json() {
                            let _ = tx.send(json).await;
                        }
                    } else {
                        // 公开房间，自动加入
                        if let Err(e) = state.room_service().join_room(room_id, user_id).await {
                            let error_msg = WebSocketMessage::error(
                                "JOIN_FAILED",
                                &format!("Failed to join room: {}", e),
                            );
                            if let Ok(json) = error_msg.to_json() {
                                let _ = tx.send(json).await;
                            }
                            return Err(e.into());
                        }
                        do_join_room(room_id, user_id, username, state, tx).await;
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

/// 执行加入房间操作（提取公共逻辑）
async fn do_join_room(
    room_id: Uuid,
    user_id: Uuid,
    username: &str,
    state: &AppState,
    tx: &mpsc::Sender<String>,
) {
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

/// 处理离开房间
async fn handle_leave_room(
    room_id: Uuid,
    user_id: Uuid,
    username: &str,
    state: &AppState,
    tx: &mpsc::Sender<String>,
) -> anyhow::Result<()> {
    let mut _timer = PerformanceTimer::new("handle_leave_room");
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
    state: &Arc<AppState>,
    tx: &mpsc::Sender<String>,
) -> anyhow::Result<()> {
    let mut _timer = PerformanceTimer::new("handle_chat_message");
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

    // 3. 如果指定了 reply_to，验证被回复的消息
    if let Some(reply_to_id) = reply_to {
        if let Err(e) = state
            .message_service()
            .validate_reply_message(reply_to_id, room_id)
            .await
        {
            let error_msg = WebSocketMessage::error("INVALID_REPLY", &e.to_string());
            if let Ok(json) = error_msg.to_json() {
                let _ = tx.send(json).await;
            }
            return Ok(());
        }
    }

    // 4. 保存消息到数据库
    match state
        .message_service()
        .create_text_message(room_id, user_id, &content, reply_to)
        .await
    {
        Ok(message) => {
            // 获取被引用消息的信息（如果有）
            let reply_to_message = if let Some(reply_to_id) = message.reply_to {
                state
                    .message_service()
                    .get_reply_to_info(reply_to_id)
                    .await
                    .ok()
                    .flatten()
                    .map(|info| ReplyToInfo {
                        id: info.id,
                        sender_id: info.sender.id,
                        sender_name: info.sender.username.clone(),
                        content: info.content,
                        created_at: info.created_at,
                    })
            } else {
                None
            };

            // 构造广播数据
            let new_message = WebSocketMessage::NewMessage {
                message_id: message.id,
                room_id,
                sender_id: user_id,
                sender_name: username.to_string(),
                content: message.content.clone(),
                reply_to: message.reply_to,
                reply_to_message,
                created_at: message.created_at,
            };

            let broadcast_json = new_message.to_json().ok();

            let message_preview = crate::models::room::MessagePreview {
                id: message.id,
                content: message.content.clone(),
                sender_name: username.to_string(),
                created_at: message.created_at,
            };
            let summary = WebSocketMessage::RoomMessageSummary {
                room_id,
                last_message: message_preview,
                unread_count: 0,
            };
            let summary_json = summary.to_json().ok();

            let content_for_mentions = content.clone();
            let msg_id_for_mentions = message.id;
            let created_at_for_mentions = message.created_at;
            let username_for_mentions = username.to_string();
            let user_id_for_mentions = user_id;

            // 后台异步执行：广播 + 摘要 + 提及通知
            // 不阻塞 recv_task，确保消息处理流水线畅通
            let state_for_async = Arc::clone(state);
            tokio::spawn(async move {
                // 广播消息给房间所有成员
                if let Some(json) = broadcast_json {
                    state_for_async
                        .ws_manager()
                        .broadcast_to_room_all(room_id, json)
                        .await;
                }

                // 推送消息摘要
                if let Some(json) = summary_json {
                    state_for_async
                        .ws_manager()
                        .broadcast_room_summary(room_id, json)
                        .await;
                }

                // 检测并处理@提及通知
                handle_mentions(
                    &content_for_mentions,
                    msg_id_for_mentions,
                    room_id,
                    user_id_for_mentions,
                    &username_for_mentions,
                    &created_at_for_mentions,
                    &*state_for_async,
                )
                .await;
            });

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

/// 处理@提及通知
async fn handle_mentions(
    content: &str,
    message_id: Uuid,
    room_id: Uuid,
    sender_id: Uuid,
    sender_name: &str,
    created_at: &chrono::DateTime<chrono::Utc>,
    state: &AppState,
) {
    use crate::services::notification_service::MentionInfo;
    use crate::utils::mention;

    // 检查消息中是否包含@提及
    let mentions = mention::extract_mentions(content);
    if mentions.is_empty() {
        return;
    }

    debug!("Found {} mentions in message", mentions.len());

    // 过滤掉发送者自己
    let mentions = mention::filter_self_mentions(mentions, sender_name);
    if mentions.is_empty() {
        return;
    }

    // 解析用户名为用户ID
    let mentioned_user_ids = match mention::resolve_usernames_to_ids(mentions, state.db()).await {
        Ok(ids) => ids,
        Err(e) => {
            warn!("Failed to resolve usernames to IDs: {}", e);
            return;
        }
    };

    if mentioned_user_ids.is_empty() {
        return;
    }

    // 生成内容预览（限制长度）
    let content_preview = if content.len() > 100 {
        format!("{}...", &content[..100])
    } else {
        content.to_string()
    };

    // 创建提及信息
    let mention_info = MentionInfo {
        message_id,
        room_id,
        mentioned_by: sender_id,
        mentioned_by_name: sender_name.to_string(),
        content_preview,
        created_at: *created_at,
    };

    // 发送提及通知
    if let Err(e) = state
        .notification_service()
        .send_mentions(mentioned_user_ids, mention_info)
        .await
    {
        warn!("Failed to send mention notifications: {}", e);
    }
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
    state: &AppState,
    tx: &mpsc::Sender<String>,
) -> anyhow::Result<()> {
    debug!("User {} marking message {} as read", user_id, message_id);

    // 获取消息以确定房间 ID
    let room_id = match state.message_service().get_message_by_id(message_id).await {
        Ok(Some(msg)) => msg.room_id,
        Ok(None) => {
            let error_msg = WebSocketMessage::error("NOT_FOUND", "Message not found");
            if let Ok(json) = error_msg.to_json() {
                let _ = tx.send(json).await;
            }
            return Ok(());
        }
        Err(e) => {
            warn!("Failed to get message {}: {}", message_id, e);
            let error_msg = WebSocketMessage::error("FETCH_FAILED", "Failed to get message");
            if let Ok(json) = error_msg.to_json() {
                let _ = tx.send(json).await;
            }
            return Ok(());
        }
    };

    let receipt = WebSocketMessage::MessageReadReceipt {
        message_id,
        user_id,
    };

    // 广播已读回执给房间其他成员
    if let Ok(json) = receipt.to_json() {
        state
            .ws_manager()
            .broadcast_to_room(room_id, json, Some(user_id))
            .await;
    }

    // 确认已读给发送者自己
    if let Ok(json) = receipt.to_json() {
        let _ = tx.send(json).await;
    }

    debug!(
        "Message {} marked as read by user {} in room {}",
        message_id, user_id, room_id
    );
    Ok(())
}

/// 处理编辑消息
async fn handle_edit_message(
    message_id: Uuid,
    user_id: Uuid,
    new_content: String,
    state: &AppState,
    tx: &mpsc::Sender<String>,
) -> anyhow::Result<()> {
    debug!("User {} editing message {}", user_id, message_id);

    // 获取原始消息以确定房间 ID
    let orig_msg = match state.message_service().get_message_by_id(message_id).await {
        Ok(Some(msg)) => msg,
        Ok(None) => {
            let error_msg = WebSocketMessage::error("NOT_FOUND", "Message not found");
            if let Ok(json) = error_msg.to_json() {
                let _ = tx.send(json).await;
            }
            return Ok(());
        }
        Err(e) => {
            warn!("Failed to get message {}: {}", message_id, e);
            let error_msg = WebSocketMessage::error("FETCH_FAILED", "Failed to get message");
            if let Ok(json) = error_msg.to_json() {
                let _ = tx.send(json).await;
            }
            return Ok(());
        }
    };
    let room_id = orig_msg.room_id;

    // 调用 service 层编辑（内部验证发送者身份）
    match state
        .message_service()
        .edit_message(message_id, user_id, &new_content)
        .await
    {
        Ok(updated) => {
            let edited_at = updated
                .edited_at
                .expect("edited_at should be set after successful edit");
            let edited_msg = WebSocketMessage::MessageEdited {
                message_id,
                new_content: updated.content.clone(),
                edited_at,
            };

            // 广播编辑结果给房间所有成员
            if let Ok(json) = edited_msg.to_json() {
                state
                    .ws_manager()
                    .broadcast_to_room_all(room_id, json)
                    .await;
            }

            info!(
                "Message {} edited by user {} in room {}",
                message_id, user_id, room_id
            );
        }
        Err(e) => {
            warn!("Failed to edit message {}: {}", message_id, e);
            let error_msg = WebSocketMessage::error("EDIT_FAILED", &e.to_string());
            if let Ok(json) = error_msg.to_json() {
                let _ = tx.send(json).await;
            }
        }
    }

    Ok(())
}

/// 处理删除消息
async fn handle_delete_message(
    message_id: Uuid,
    user_id: Uuid,
    state: &AppState,
    tx: &mpsc::Sender<String>,
) -> anyhow::Result<()> {
    debug!("User {} deleting message {}", user_id, message_id);

    // 获取原始消息以确定房间 ID
    let orig_msg = match state.message_service().get_message_by_id(message_id).await {
        Ok(Some(msg)) => msg,
        Ok(None) => {
            let error_msg = WebSocketMessage::error("NOT_FOUND", "Message not found");
            if let Ok(json) = error_msg.to_json() {
                let _ = tx.send(json).await;
            }
            return Ok(());
        }
        Err(e) => {
            warn!("Failed to get message {}: {}", message_id, e);
            let error_msg = WebSocketMessage::error("FETCH_FAILED", "Failed to get message");
            if let Ok(json) = error_msg.to_json() {
                let _ = tx.send(json).await;
            }
            return Ok(());
        }
    };
    let room_id = orig_msg.room_id;

    // 调用 service 层删除（内部验证发送者身份）
    match state
        .message_service()
        .delete_message(message_id, user_id)
        .await
    {
        Ok(_) => {
            let deleted_msg = WebSocketMessage::MessageDeleted { message_id };

            // 广播删除结果给房间所有成员
            if let Ok(json) = deleted_msg.to_json() {
                state
                    .ws_manager()
                    .broadcast_to_room_all(room_id, json)
                    .await;
            }

            info!(
                "Message {} deleted by user {} in room {}",
                message_id, user_id, room_id
            );
        }
        Err(e) => {
            warn!("Failed to delete message {}: {}", message_id, e);
            let error_msg = WebSocketMessage::error("DELETE_FAILED", &e.to_string());
            if let Ok(json) = error_msg.to_json() {
                let _ = tx.send(json).await;
            }
        }
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
    let mut _timer = PerformanceTimer::new("handle_get_missed_messages");
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
        .map(|msg| {
            let reply_to_message = msg.reply_to_message.map(|info| ReplyToInfo {
                id: info.id,
                sender_id: info.sender.id,
                sender_name: info.sender.username.clone(),
                content: info.content,
                created_at: info.created_at,
            });

            MissedMessage {
                message_id: msg.id,
                room_id: msg.room_id,
                sender_id: msg.sender.id,
                sender_name: msg.sender.username.clone(),
                content: msg.content,
                reply_to: msg.reply_to,
                reply_to_message,
                created_at: msg.created_at,
            }
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
    let mut _timer = PerformanceTimer::new("handle_update_status");
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
                // 发送给发送者自己（确认）
                let _ = tx.send(json.clone()).await;

                // 广播给房间成员
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

/// 处理获取离线通知
async fn handle_get_offline_notifications(
    user_id: Uuid,
    #[allow(unused_variables)] last_notification_id: Option<Uuid>,
    limit: Option<i64>,
    state: &AppState,
    tx: &mpsc::Sender<String>,
) -> anyhow::Result<()> {
    debug!("Getting offline notifications for user {}", user_id);

    let limit = limit.unwrap_or(50);

    // 获取未读通知
    match state
        .notification_service()
        .get_unread_notifications(user_id, limit)
        .await
    {
        Ok(notifications) => {
            let has_more = notifications.len() as i64 >= limit;

            let offline_notifications = WebSocketMessage::OfflineNotifications {
                notifications,
                has_more,
            };

            if let Ok(json) = offline_notifications.to_json() {
                let _ = tx.send(json).await;
            }

            debug!("Sent offline notifications to user {}", user_id);
        }
        Err(e) => {
            warn!("Failed to get offline notifications: {}", e);
            let error_msg =
                WebSocketMessage::error("FETCH_FAILED", "Failed to fetch notifications");
            if let Ok(json) = error_msg.to_json() {
                let _ = tx.send(json).await;
            }
        }
    }

    Ok(())
}
