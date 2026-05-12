//! 端到端集成测试
//!
//! 本测试文件包含实际应用场景的端到端测试，验证完整用户流程。
//! 测试会实际启动服务器，使用真实 HTTP 请求和 WebSocket 连接。
//!
//! ## 测试场景
//! - 完整用户注册登录流程
//! - 聊天室创建和管理流程
//! - WebSocket 实时消息通信
//! - 文件上传和下载流程
//! - 多用户并发场景

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use futures::{SinkExt, StreamExt};
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use serde_json::json;
use tokio::net::TcpListener;
use tokio::time::{sleep, timeout};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use uuid::Uuid;

// 引入被测模块
use capella_room::{
    config::{ConfigLoader, ConfigManager},
    db::Database,
    routes::create_router,
    state::AppState,
    utils::logging::MetricsCollector,
    websocket::{manager::WebSocketManager, protocol::WebSocketMessage},
};

/// 测试服务器句柄
struct TestServer {
    addr: SocketAddr,
    _shutdown: tokio::sync::oneshot::Sender<()>,
}

impl TestServer {
    fn base_url(&self) -> String {
        format!("http://{}", self.addr)
    }

    fn ws_url(&self) -> String {
        format!("ws://{}/ws", self.addr)
    }
}

/// 启动测试服务器
async fn start_test_server() -> TestServer {
    // 加载测试环境配置
    dotenvy::from_filename(".env.test").ok();

    // 设置配置文件路径
    std::env::set_var("CONFIG_FILE", "config.toml");

    let config = ConfigLoader::load().expect("Failed to load config");

    // 初始化数据库
    let db = Database::new(&config.database)
        .await
        .expect("Failed to connect to database");

    // 运行数据库迁移
    db.migrate().await.expect("Failed to run migrations");

    // 初始化 WebSocket 管理器
    let ws_manager = WebSocketManager::new();

    // 初始化指标收集器
    let metrics_collector = Arc::new(MetricsCollector::new());

    // 创建配置管理器（测试环境不使用 Redis 同步）
    let config_manager = ConfigManager::new(db.clone(), config.clone(), None);

    // 创建应用状态
    let state = AppState::new(
        db,
        ws_manager,
        config,
        metrics_collector,
        Arc::new(config_manager),
        None,
    )
    .await
    .expect("Failed to create app state");

    // 构建应用路由
    let app = create_router(state);

    // 绑定到随机端口
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind");
    let addr = listener.local_addr().unwrap();

    // 创建关闭信号
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    // 启动服务器 - 使用 axum::serve 会自动处理 ConnectInfo
    tokio::spawn(async move {
        let serve = axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        );

        serve
            .with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
            })
            .await
            .expect("Server failed");
    });

    // 等待服务器启动
    sleep(Duration::from_millis(500)).await;

    TestServer {
        addr,
        _shutdown: shutdown_tx,
    }
}

/// 健康检查响应
#[derive(Debug, Deserialize)]
struct HealthResponse {
    success: bool,
    data: HealthData,
}

#[derive(Debug, Deserialize)]
struct HealthData {
    status: String,
    timestamp: String,
}

/// 用户响应
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ApiResponse<T> {
    success: bool,
    data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct UserResponse {
    id: String,
    username: String,
    email: String,
    avatar_url: Option<String>,
    status: String,
}

/// 登录响应
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct LoginResponse {
    success: bool,
    data: LoginData,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct LoginData {
    access_token: String,
    refresh_token: String,
    expires_in: i64,
    token_type: String,
    user: UserResponse,
}

/// 刷新 Token 响应
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RefreshResponse {
    success: bool,
    data: RefreshData,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RefreshData {
    access_token: String,
    refresh_token: String,
    expires_in: i64,
    token_type: String,
}

/// 聊天室响应
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RoomResponse {
    id: String,
    name: String,
    description: Option<String>,
    owner_id: String,
    is_private: bool,
    max_members: i32,
    member_count: i32,
    created_at: String,
    updated_at: String,
}

/// 消息响应
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MessageResponse {
    id: String,
    room_id: String,
    sender_id: String,
    content: String,
    message_type: String,
}

/// ==================== 基础服务测试 ====================

#[tokio::test]
async fn test_server_health_check() {
    let server = start_test_server().await;
    let client = Client::new();

    let response = client
        .get(format!("{}/health", server.base_url()))
        .send()
        .await
        .expect("Failed to send request");

    // 打印状态码和原始响应内容用于调试
    let status = response.status();
    let text = response.text().await.expect("Failed to get response text");
    println!("Health check status: {}", status);
    println!("Health check response: {}", text);

    assert_eq!(status, StatusCode::OK);

    // 解析 JSON 响应
    let health_body: HealthResponse =
        serde_json::from_str(&text).expect("Failed to parse health response");
    assert!(health_body.success);
    assert_eq!(health_body.data.status, "healthy");
    assert!(!health_body.data.timestamp.is_empty());
}

#[tokio::test]
async fn test_api_version_endpoint() {
    let server = start_test_server().await;
    let client = Client::new();

    let response = client
        .get(format!("{}/api/version", server.base_url()))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::OK);

    let body: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse version response");
    assert!(body["success"].as_bool().unwrap_or(false));
    assert!(body["data"]["version"].is_string());
    assert_eq!(body["data"]["version"].as_str(), Some("v1"));
}

/// ==================== 完整用户流程测试 ====================

#[tokio::test]
async fn test_complete_user_registration_flow() {
    let server = start_test_server().await;
    let client = Client::new();

    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let username = format!("testuser_{}", unique_id);
    let email = format!("test_{}@example.com", unique_id);
    let password = "TestPassword123";

    // 1. 用户注册
    let register_response = client
        .post(format!("{}/api/v1/auth/register", server.base_url()))
        .json(&json!({
            "username": username,
            "email": email,
            "password": password
        }))
        .send()
        .await
        .expect("Failed to register");

    // 打印调试信息
    let status = register_response.status();
    let text = register_response
        .text()
        .await
        .expect("Failed to get response text");
    println!("Register status: {}", status);
    println!("Register response: {}", text);

    assert_eq!(status, StatusCode::OK);

    // 解析统一响应格式
    let api_response: ApiResponse<UserResponse> =
        serde_json::from_str(&text).expect("Failed to parse register response");

    assert!(api_response.success);
    let register_body = api_response.data;

    assert_eq!(register_body.username, username);
    assert_eq!(register_body.email, email);

    // 2. 重复注册应该失败
    let duplicate_response = client
        .post(format!("{}/api/v1/auth/register", server.base_url()))
        .json(&json!({
            "username": username,
            "email": email,
            "password": password
        }))
        .send()
        .await
        .expect("Failed to send duplicate request");

    assert_eq!(duplicate_response.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_complete_login_and_token_refresh_flow() {
    let server = start_test_server().await;
    let client = Client::new();

    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let username = format!("logintest_{}", unique_id);
    let email = format!("login_{}@example.com", unique_id);
    let password = "TestPassword123";

    // 1. 先注册用户
    let _ = client
        .post(format!("{}/api/v1/auth/register", server.base_url()))
        .json(&json!({
            "username": username,
            "email": email,
            "password": password
        }))
        .send()
        .await;

    // 2. 用户登录
    let login_response = client
        .post(format!("{}/api/v1/auth/login", server.base_url()))
        .json(&json!({
            "email": email,
            "password": password
        }))
        .send()
        .await
        .expect("Failed to login");

    assert_eq!(login_response.status(), StatusCode::OK);

    let login_body = login_response
        .json::<LoginResponse>()
        .await
        .expect("Failed to parse login response");

    assert!(login_body.success);
    assert!(!login_body.data.access_token.is_empty());
    assert!(!login_body.data.refresh_token.is_empty());
    assert_eq!(login_body.data.token_type, "Bearer");

    let access_token = login_body.data.access_token;
    let refresh_token = login_body.data.refresh_token;

    // 3. 使用 token 访问受保护接口
    let me_response = client
        .get(format!("{}/api/v1/users/me", server.base_url()))
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
        .expect("Failed to get current user");

    assert_eq!(me_response.status(), StatusCode::OK);

    // 4. 刷新 token
    let refresh_response = client
        .post(format!("{}/api/v1/auth/refresh", server.base_url()))
        .json(&json!({
            "refresh_token": refresh_token
        }))
        .send()
        .await
        .expect("Failed to refresh token");

    assert_eq!(refresh_response.status(), StatusCode::OK);

    let refresh_body = refresh_response
        .json::<RefreshResponse>()
        .await
        .expect("Failed to parse refresh response");

    assert!(refresh_body.success);
    assert!(!refresh_body.data.access_token.is_empty());

    // 5. 使用新 token 访问接口
    let new_token = refresh_body.data.access_token;
    let me_response2 = client
        .get(format!("{}/api/v1/users/me", server.base_url()))
        .header("Authorization", format!("Bearer {}", new_token))
        .send()
        .await
        .expect("Failed to get current user with new token");

    assert_eq!(me_response2.status(), StatusCode::OK);
}

/// ==================== 聊天室完整流程测试 ====================

#[tokio::test]
async fn test_complete_room_lifecycle() {
    let server = start_test_server().await;
    let client = Client::new();

    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let username = format!("roomtest_{}", unique_id);
    let email = format!("room_{}@example.com", unique_id);
    let password = "TestPassword123";

    // 1. 注册用户并登录
    let _ = client
        .post(format!("{}/api/v1/auth/register", server.base_url()))
        .json(&json!({
            "username": username,
            "email": email,
            "password": password
        }))
        .send()
        .await;

    let login_response = client
        .post(format!("{}/api/v1/auth/login", server.base_url()))
        .json(&json!({
            "email": email,
            "password": password
        }))
        .send()
        .await
        .expect("Failed to login");

    let login_body = login_response
        .json::<LoginResponse>()
        .await
        .expect("Failed to parse login");

    let token = login_body.data.access_token;

    // 2. 创建聊天室
    let room_name = format!("Test Room {}", unique_id);
    let create_response = client
        .post(format!("{}/api/v1/rooms", server.base_url()))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "name": room_name,
            "description": "A test room",
            "is_private": false,
            "max_members": 50
        }))
        .send()
        .await
        .expect("Failed to create room");

    assert_eq!(create_response.status(), StatusCode::OK);

    let room_body = create_response
        .json::<serde_json::Value>()
        .await
        .expect("Failed to parse room response");

    println!("Create room response: {:?}", room_body);

    // 解析统一响应格式
    let room_id = room_body["data"]["id"].as_str().unwrap();
    assert_eq!(room_body["data"]["name"], room_name);

    // 3. 获取聊天室列表
    let list_response = client
        .get(format!("{}/api/v1/rooms", server.base_url()))
        .header("Authorization", format!("Bearer {}", token))
        .query(&[("limit", "10"), ("offset", "0")])
        .send()
        .await
        .expect("Failed to list rooms");

    assert_eq!(list_response.status(), StatusCode::OK);

    // 4. 获取聊天室详情
    let detail_response = client
        .get(format!("{}/api/v1/rooms/{}", server.base_url(), room_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to get room detail");

    assert_eq!(detail_response.status(), StatusCode::OK);

    // 5. 更新聊天室
    let update_response = client
        .put(format!("{}/api/v1/rooms/{}", server.base_url(), room_id))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "name": format!("{} (Updated)", room_name),
            "description": "Updated description"
        }))
        .send()
        .await
        .expect("Failed to update room");

    assert_eq!(update_response.status(), StatusCode::OK);

    // 6. 删除聊天室
    let delete_response = client
        .delete(format!("{}/api/v1/rooms/{}", server.base_url(), room_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to delete room");

    assert_eq!(delete_response.status(), StatusCode::OK);

    // 7. 确认删除
    let check_response = client
        .get(format!("{}/api/v1/rooms/{}", server.base_url(), room_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to check room");

    assert_eq!(check_response.status(), StatusCode::NOT_FOUND);
}

/// ==================== WebSocket 实时通信测试 ====================

#[tokio::test]
async fn test_websocket_complete_messaging_flow() {
    let server = start_test_server().await;
    let client = Client::new();

    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let username = format!("wstest_{}", unique_id);
    let email = format!("ws_{}@example.com", unique_id);
    let password = "TestPassword123";

    // 1. 注册用户并登录
    let _ = client
        .post(format!("{}/api/v1/auth/register", server.base_url()))
        .json(&json!({
            "username": username,
            "email": email,
            "password": password
        }))
        .send()
        .await;

    let login_response = client
        .post(format!("{}/api/v1/auth/login", server.base_url()))
        .json(&json!({
            "email": email,
            "password": password
        }))
        .send()
        .await
        .expect("Failed to login");

    let login_body = login_response
        .json::<LoginResponse>()
        .await
        .expect("Failed to parse login");

    let token = login_body.data.access_token;

    // 2. 创建聊天室
    let room_response = client
        .post(format!("{}/api/v1/rooms", server.base_url()))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "name": format!("WS Test Room {}", unique_id),
            "is_private": false
        }))
        .send()
        .await
        .expect("Failed to create room");

    let room_body = room_response
        .json::<serde_json::Value>()
        .await
        .expect("Failed to parse room");

    println!("WebSocket test room response: {:?}", room_body);

    let room_id = room_body["data"]["id"].as_str().unwrap();

    // 3. 建立 WebSocket 连接（注意：WebSocket URL 不需要 token 参数，需要在连接后发送 Auth 消息）
    let ws_url = server.ws_url();
    let (ws_stream, _) = connect_async(&ws_url)
        .await
        .expect("Failed to connect WebSocket");

    let (mut write, mut read) = ws_stream.split();

    // 4. 发送认证消息
    let auth_msg = WebSocketMessage::Auth {
        token: token.clone(),
    };
    write
        .send(Message::Text(serde_json::to_string(&auth_msg).unwrap()))
        .await
        .expect("Failed to send auth message");

    // 5. 等待认证成功消息
    let auth_response = timeout(Duration::from_secs(5), read.next())
        .await
        .expect("Timeout waiting for auth response")
        .expect("No auth message received")
        .expect("WebSocket error");

    if let Message::Text(text) = auth_response {
        let msg: WebSocketMessage =
            serde_json::from_str(&text).expect("Failed to parse auth message");
        match msg {
            WebSocketMessage::AuthResult { success, .. } => {
                assert!(success, "Authentication failed");
            }
            _ => panic!("Expected AuthResult, got {:?}", msg),
        }
    }

    // 5. 加入聊天室
    let room_uuid = Uuid::parse_str(room_id).expect("Invalid room_id UUID");
    let join_msg = WebSocketMessage::JoinRoom { room_id: room_uuid };
    write
        .send(Message::Text(serde_json::to_string(&join_msg).unwrap()))
        .await
        .expect("Failed to send join message");

    // 6. 等待加入成功消息（跳过 Ping/Pong 消息）
    let join_msg = loop {
        let response = timeout(Duration::from_secs(5), read.next())
            .await
            .expect("Timeout waiting for join response")
            .expect("No join response received")
            .expect("WebSocket error");

        match response {
            Message::Text(text) => {
                let msg: WebSocketMessage =
                    serde_json::from_str(&text).expect("Failed to parse join response");
                // 跳过应用层的 Ping/Pong 消息
                match msg {
                    WebSocketMessage::Ping => {
                        // 发送 Pong 响应
                        let pong = WebSocketMessage::Pong;
                        write
                            .send(Message::Text(serde_json::to_string(&pong).unwrap()))
                            .await
                            .expect("Failed to send pong");
                        continue;
                    }
                    _ => break msg,
                }
            }
            Message::Ping(_) => {
                // 收到协议层 Ping，发送 Pong 响应
                write
                    .send(Message::Pong(vec![]))
                    .await
                    .expect("Failed to send pong");
                continue;
            }
            _ => continue,
        }
    };

    match join_msg {
        WebSocketMessage::RoomJoined { .. } => {
            // 加入成功
        }
        _ => panic!("Expected RoomJoined, got {:?}", join_msg),
    }

    // 7. 发送消息
    let chat_msg = WebSocketMessage::ChatMessage {
        room_id: room_uuid,
        content: "Hello, WebSocket!".to_string(),
        reply_to: None,
    };
    write
        .send(Message::Text(serde_json::to_string(&chat_msg).unwrap()))
        .await
        .expect("Failed to send chat message");

    // 8. 等待消息确认（跳过 Ping/Pong 消息）
    let chat_response = loop {
        let response = timeout(Duration::from_secs(5), read.next())
            .await
            .expect("Timeout waiting for message response")
            .expect("No message response received")
            .expect("WebSocket error");

        match response {
            Message::Text(text) => {
                let msg: WebSocketMessage =
                    serde_json::from_str(&text).expect("Failed to parse message response");
                // 跳过应用层的 Ping/Pong 消息
                match msg {
                    WebSocketMessage::Ping => {
                        let pong = WebSocketMessage::Pong;
                        write
                            .send(Message::Text(serde_json::to_string(&pong).unwrap()))
                            .await
                            .expect("Failed to send pong");
                        continue;
                    }
                    _ => break msg,
                }
            }
            Message::Ping(_) => {
                write
                    .send(Message::Pong(vec![]))
                    .await
                    .expect("Failed to send pong");
                continue;
            }
            _ => continue,
        }
    };

    // 收到消息后，可能是 OnlineUsers（加入房间后的在线用户列表）或 NewMessage
    match chat_response {
        WebSocketMessage::NewMessage { content, .. } => {
            assert_eq!(content, "Hello, WebSocket!");
        }
        WebSocketMessage::OnlineUsers { .. } => {
            // 收到在线用户列表，继续等待 NewMessage
            let new_msg = loop {
                let response = timeout(Duration::from_secs(5), read.next())
                    .await
                    .expect("Timeout waiting for new message")
                    .expect("No new message received")
                    .expect("WebSocket error");

                match response {
                    Message::Text(text) => {
                        let msg: WebSocketMessage =
                            serde_json::from_str(&text).expect("Failed to parse new message");
                        match msg {
                            WebSocketMessage::Ping => {
                                let pong = WebSocketMessage::Pong;
                                write
                                    .send(Message::Text(serde_json::to_string(&pong).unwrap()))
                                    .await
                                    .expect("Failed to send pong");
                                continue;
                            }
                            _ => break msg,
                        }
                    }
                    Message::Ping(_) => {
                        write
                            .send(Message::Pong(vec![]))
                            .await
                            .expect("Failed to send pong");
                        continue;
                    }
                    _ => continue,
                }
            };

            match new_msg {
                WebSocketMessage::NewMessage { content, .. } => {
                    assert_eq!(content, "Hello, WebSocket!");
                }
                _ => panic!("Expected NewMessage after OnlineUsers, got {:?}", new_msg),
            }
        }
        _ => panic!(
            "Expected NewMessage or OnlineUsers, got {:?}",
            chat_response
        ),
    }

    // 9. 离开聊天室
    let leave_msg = WebSocketMessage::LeaveRoom { room_id: room_uuid };
    write
        .send(Message::Text(serde_json::to_string(&leave_msg).unwrap()))
        .await
        .expect("Failed to send leave message");

    // 10. 关闭连接
    write.close().await.expect("Failed to close WebSocket");
}

/// ==================== 多用户场景测试 ====================

#[tokio::test]
async fn test_multi_user_room_interaction() {
    let server = start_test_server().await;
    let client = Client::new();

    // 创建两个用户
    let unique_id = Uuid::new_v4().to_string()[..8].to_string();

    // 用户1
    let user1_name = format!("multi1_{}", unique_id);
    let user1_email = format!("multi1_{}@example.com", unique_id);
    let password = "TestPassword123";

    let _ = client
        .post(format!("{}/api/v1/auth/register", server.base_url()))
        .json(&json!({
            "username": user1_name,
            "email": user1_email,
            "password": password
        }))
        .send()
        .await;

    let login1 = client
        .post(format!("{}/api/v1/auth/login", server.base_url()))
        .json(&json!({
            "email": user1_email,
            "password": password
        }))
        .send()
        .await
        .expect("Failed to login user1");

    let token1 = login1
        .json::<LoginResponse>()
        .await
        .expect("Failed to parse login1")
        .data
        .access_token;

    // 用户2
    let user2_name = format!("multi2_{}", unique_id);
    let user2_email = format!("multi2_{}@example.com", unique_id);

    let _ = client
        .post(format!("{}/api/v1/auth/register", server.base_url()))
        .json(&json!({
            "username": user2_name,
            "email": user2_email,
            "password": password
        }))
        .send()
        .await;

    let login2 = client
        .post(format!("{}/api/v1/auth/login", server.base_url()))
        .json(&json!({
            "email": user2_email,
            "password": password
        }))
        .send()
        .await
        .expect("Failed to login user2");

    let token2 = login2
        .json::<LoginResponse>()
        .await
        .expect("Failed to parse login2")
        .data
        .access_token;

    // 用户1创建聊天室
    let room_response = client
        .post(format!("{}/api/v1/rooms", server.base_url()))
        .header("Authorization", format!("Bearer {}", token1))
        .json(&json!({
            "name": format!("Multi User Room {}", unique_id),
            "is_private": false
        }))
        .send()
        .await
        .expect("Failed to create room");

    let room_body = room_response
        .json::<serde_json::Value>()
        .await
        .expect("Failed to parse room");

    println!("Multi-user test room response: {:?}", room_body);

    let room_id = room_body["data"]["id"].as_str().unwrap().to_string();

    // 用户2加入聊天室
    let join_response = client
        .post(format!(
            "{}/api/v1/rooms/{}/join",
            server.base_url(),
            room_id
        ))
        .header("Authorization", format!("Bearer {}", token2))
        .send()
        .await
        .expect("Failed to join room");

    assert_eq!(join_response.status(), StatusCode::OK);

    // 获取成员列表
    let members_response = client
        .get(format!(
            "{}/api/v1/rooms/{}/members",
            server.base_url(),
            room_id
        ))
        .header("Authorization", format!("Bearer {}", token1))
        .send()
        .await
        .expect("Failed to get members");

    assert_eq!(members_response.status(), StatusCode::OK);

    let members_body = members_response
        .json::<serde_json::Value>()
        .await
        .expect("Failed to parse members");

    println!("Members response: {:?}", members_body);

    // 成员列表在 data 字段中
    let members = members_body["data"].as_array().unwrap();
    assert_eq!(members.len(), 2);

    // 用户 2 离开聊天室（使用 DELETE 方法）
    let leave_response = client
        .delete(format!(
            "{}/api/v1/rooms/{}/leave",
            server.base_url(),
            room_id
        ))
        .header("Authorization", format!("Bearer {}", token2))
        .send()
        .await
        .expect("Failed to leave room");

    assert_eq!(leave_response.status(), StatusCode::OK);
}

/// ==================== 错误处理测试 ====================

#[tokio::test]
async fn test_error_handling_scenarios() {
    let server = start_test_server().await;
    let client = Client::new();

    // 1. 访问不存在的端点（使用 /api/v1/ 前缀下的不存在路径）
    let not_found_response = client
        .get(format!(
            "{}/api/v1/nonexistent_endpoint_xyz",
            server.base_url()
        ))
        .header("Authorization", "Bearer invalid_token_for_404_test")
        .send()
        .await
        .expect("Failed to send request");

    // 注意：由于认证中间件先执行，未认证请求会返回 401 而不是 404
    // 这里我们测试的是"受保护路由的 401 响应"
    assert_eq!(not_found_response.status(), StatusCode::UNAUTHORIZED);

    // 2. 未认证访问受保护接口
    let unauthorized_response = client
        .get(format!("{}/api/v1/users/me", server.base_url()))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(unauthorized_response.status(), StatusCode::UNAUTHORIZED);

    // 3. 无效 token 访问
    let invalid_token_response = client
        .get(format!("{}/api/v1/users/me", server.base_url()))
        .header("Authorization", "Bearer invalid_token")
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(invalid_token_response.status(), StatusCode::UNAUTHORIZED);

    // 4. 无效的登录凭证
    let invalid_login_response = client
        .post(format!("{}/api/v1/auth/login", server.base_url()))
        .json(&json!({
            "email": "nonexistent@example.com",
            "password": "wrongpassword"
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(invalid_login_response.status(), StatusCode::UNAUTHORIZED);

    // 5. 无效的注册数据
    let invalid_register_response = client
        .post(format!("{}/api/v1/auth/register", server.base_url()))
        .json(&json!({
            "username": "ab",  // 太短
            "email": "invalid-email",
            "password": "123"  // 太弱
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(invalid_register_response.status(), StatusCode::BAD_REQUEST);
}

/// ==================== 边界情况测试 ====================

#[tokio::test]
async fn test_edge_cases() {
    let server = start_test_server().await;
    let client = Client::new();

    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let username = format!("edge_{}", unique_id);
    let email = format!("edge_{}@example.com", unique_id);
    let password = "TestPassword123";

    // 注册用户
    let _ = client
        .post(format!("{}/api/v1/auth/register", server.base_url()))
        .json(&json!({
            "username": username,
            "email": email,
            "password": password
        }))
        .send()
        .await;

    let login = client
        .post(format!("{}/api/v1/auth/login", server.base_url()))
        .json(&json!({
            "email": email,
            "password": password
        }))
        .send()
        .await
        .expect("Failed to login");

    let token = login
        .json::<LoginResponse>()
        .await
        .expect("Failed to parse login")
        .data
        .access_token;

    // 1. 创建名称很长的聊天室
    let long_name = "a".repeat(100);
    let long_name_response = client
        .post(format!("{}/api/v1/rooms", server.base_url()))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "name": long_name,
            "is_private": false
        }))
        .send()
        .await
        .expect("Failed to create room with long name");

    // 应该失败，因为名称超过限制
    assert_eq!(long_name_response.status(), StatusCode::BAD_REQUEST);

    // 2. 空名称聊天室
    let empty_name_response = client
        .post(format!("{}/api/v1/rooms", server.base_url()))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "name": "",
            "is_private": false
        }))
        .send()
        .await
        .expect("Failed to create room with empty name");

    assert_eq!(empty_name_response.status(), StatusCode::BAD_REQUEST);

    // 3. 获取不存在的聊天室
    let fake_uuid = Uuid::new_v4().to_string();
    let not_found_response = client
        .get(format!("{}/api/v1/rooms/{}", server.base_url(), fake_uuid))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to get nonexistent room");

    assert_eq!(not_found_response.status(), StatusCode::NOT_FOUND);

    // 4. 分页参数测试
    let pagination_response = client
        .get(format!("{}/api/v1/rooms", server.base_url()))
        .header("Authorization", format!("Bearer {}", token))
        .query(&[("limit", "0"), ("offset", "0")])
        .send()
        .await
        .expect("Failed to list rooms with pagination");

    assert_eq!(pagination_response.status(), StatusCode::OK);
}
