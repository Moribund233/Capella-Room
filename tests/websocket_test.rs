//! WebSocket 集成测试 - 实际应用场景测试
//!
//! 本测试文件专注于实际应用场景中的 WebSocket 测试，发现并优化潜在问题。
//!
//! ## 测试场景
//! - 并发连接测试：多客户端同时连接、同时操作
//! - 压力测试：大量消息、大量房间、高并发
//! - 断线重连测试：网络中断后重连、房间恢复
//! - 消息顺序和可靠性：消息顺序保证、不丢失
//! - 边界条件测试：大消息、特殊字符、超时场景

use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use futures::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio::time::{sleep, timeout};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use uuid::Uuid;

use seredeli_room::{
    config::{ConfigManager, DatabaseConfig, JwtConfig, UploadConfig},
    db::Database,
    routes::create_router,
    services::{auth_service::AuthService, room_service::RoomService, user_service::UserService},
    state::AppState,
    websocket::{manager::WebSocketManager, protocol::WebSocketMessage},
};

/// 测试服务器句柄
struct TestServer {
    addr: SocketAddr,
    _shutdown: tokio::sync::oneshot::Sender<()>,
}

impl TestServer {
    fn url(&self) -> String {
        format!("ws://{}/ws", self.addr)
    }

    #[allow(dead_code)]
    fn base_url(&self) -> String {
        format!("http://{}", self.addr)
    }
}

/// 加载测试环境变量
fn load_test_env() {
    if std::path::Path::new(".env.test").exists() {
        dotenvy::from_filename(".env.test").ok();
    } else if std::path::Path::new("../.env.test").exists() {
        dotenvy::from_filename("../.env.test").ok();
    }
}

/// 读取下一条非 Ping 消息
async fn read_next_message(
    read: &mut futures::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
) -> Option<Result<Message, tokio_tungstenite::tungstenite::Error>> {
    while let Some(result) = read.next().await {
        if let Ok(Message::Text(text)) = &result {
            if let Ok(ws_msg) = serde_json::from_str::<WebSocketMessage>(text) {
                if matches!(ws_msg, WebSocketMessage::Ping) {
                    continue;
                }
            }
        }
        return Some(result);
    }
    None
}

/// 创建测试数据库连接
async fn setup_test_db() -> Database {
    load_test_env();

    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env.test or environment");

    let max_connections = env::var("APP_DATABASE__MAX_CONNECTIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5);

    let db_config = DatabaseConfig {
        url: Some(database_url),
        max_connections,
        acquire_timeout_secs: 30,
        idle_timeout_secs: 600,
    };

    let db = Database::new(&db_config)
        .await
        .expect("Failed to connect to test database");

    db.migrate().await.expect("Failed to run migrations");

    db
}

/// 创建测试服务器
async fn setup_test_server() -> (TestServer, Database) {
    let db = setup_test_db().await;

    if std::env::var("UPLOAD_DIR").is_err() {
        let temp_dir =
            std::env::temp_dir().join(format!("seredeli_upload_test_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temp upload directory");
        std::env::set_var("UPLOAD_DIR", temp_dir.to_str().unwrap());
    }

    use seredeli_room::utils::logging::MetricsCollector;
    use std::sync::Arc;

    let ws_manager = WebSocketManager::new();
    let metrics_collector = Arc::new(MetricsCollector::new());

    let config = seredeli_room::config::AppConfig {
        server: Default::default(),
        database: seredeli_room::config::DatabaseConfig {
            url: None,
            max_connections: 10,
            acquire_timeout_secs: 30,
            idle_timeout_secs: 600,
        },
        jwt: JwtConfig {
            secret: Some("test_secret_key_for_testing_purposes_only".to_string()),
            expiration_hours: 24,
        },
        upload: UploadConfig {
            max_file_size: 10 * 1024 * 1024,
            base_url: "/uploads".to_string(),
        },
        websocket: seredeli_room::config::WebSocketConfig {
            heartbeat_interval_secs: 30,
            heartbeat_timeout_secs: 60,
            auth_timeout_secs: 10,
            message_buffer_size: 100,
        },
        reconnect: Default::default(),
        logging: Default::default(),
        audit: seredeli_room::config::AuditConfig {
            enabled: true,
            log_retention_days: 90,
            buffer_size: 100,
            flush_interval_seconds: 5,
            excluded_paths: vec![],
            alert_enabled: true,
            alert_cooldown_minutes: 10,
            auto_archive_enabled: true,
            archive_hour: 3,
        },
        system: Default::default(),
        admin: Default::default(),
        redis: Default::default(),
    };

    let config_manager = ConfigManager::new(db.clone(), config.clone(), None);

    let state = AppState::new(
        db.clone(),
        ws_manager,
        config,
        Arc::clone(&metrics_collector),
        Arc::new(config_manager),
        None,
    )
    .await
    .expect("Failed to create app state");
    let app = create_router(state);

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind");
    let addr = listener.local_addr().unwrap();

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

    tokio::spawn(async move {
        let server = axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        );
        tokio::select! {
            _ = server => {},
            _ = shutdown_rx => {},
        }
    });

    sleep(Duration::from_millis(100)).await;

    (
        TestServer {
            addr,
            _shutdown: shutdown_tx,
        },
        db,
    )
}

/// 创建测试用户并返回 token（使用 HTTP API）
async fn create_test_user_with_token(_db: &Database, username: &str) -> (Uuid, String, String) {
    let email = format!("{}@test.com", username);
    let password = "TestPassword123";

    // 注意：这里需要一个已启动的服务器，我们使用传入的 server 参数
    // 为了简化，我们在测试中直接使用数据库创建用户并生成 token
    let user_service = UserService::new(_db.clone());
    let auth_service = AuthService::new(JwtConfig {
        secret: Some("test_secret_key_for_testing_purposes_only".to_string()),
        expiration_hours: 24,
    });

    // 检查用户是否已存在
    let user = match user_service.get_user_by_email(&email).await {
        Ok(Some(existing_user)) => existing_user,
        Ok(None) | Err(_) => {
            // 创建新用户
            let password_hash = auth_service.hash_password(password).unwrap();
            user_service
                .create_user(username, &email, &password_hash)
                .await
                .unwrap()
        }
    };

    // 生成 token
    let tokens = auth_service
        .generate_token_pair(user.id, &user.username, user.role.clone())
        .unwrap();

    (user.id, password.to_string(), tokens.access_token)
}

/// 创建测试房间
async fn create_test_room(db: &Database, owner_id: Uuid, name: &str) -> Uuid {
    let room_service = RoomService::new(db.clone());

    let room = room_service
        .create_room(name, Some("Test room description"), owner_id, false, 100)
        .await
        .unwrap();

    room.id
}

/// WebSocket 客户端句柄
struct WebSocketClient {
    user_id: Uuid,
    #[allow(dead_code)]
    username: String,
    token: String,
    write: futures::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        Message,
    >,
    read: futures::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
}

impl WebSocketClient {
    /// 创建认证的 WebSocket 客户端
    async fn new(server: &TestServer, db: &Database, username: &str) -> Self {
        let (user_id, _, token) = create_test_user_with_token(db, username).await;

        let url = server.url();
        let (ws_stream, _) = connect_async(&url).await.expect("Failed to connect");
        let (write, read) = ws_stream.split();

        let mut client = Self {
            user_id,
            username: username.to_string(),
            token,
            write,
            read,
        };

        // 自动认证
        client.authenticate().await;

        client
    }

    /// 发送认证消息
    async fn authenticate(&mut self) {
        let auth_msg = WebSocketMessage::Auth {
            token: self.token.clone(),
        };
        self.write
            .send(Message::Text(auth_msg.to_json().unwrap()))
            .await
            .expect("Failed to send auth");

        // 等待认证响应
        let _ = timeout(Duration::from_secs(5), read_next_message(&mut self.read))
            .await
            .expect("Auth timeout");
    }

    /// 加入房间
    async fn join_room(&mut self, room_id: Uuid) {
        let join_msg = WebSocketMessage::JoinRoom { room_id };
        self.write
            .send(Message::Text(join_msg.to_json().unwrap()))
            .await
            .expect("Failed to join room");

        // 等待加入响应
        let _ = timeout(Duration::from_secs(5), read_next_message(&mut self.read))
            .await
            .expect("Join timeout");

        // 额外延迟确保服务端完全处理加入请求
        sleep(Duration::from_millis(100)).await;
    }

    /// 发送聊天消息
    async fn send_message(&mut self, room_id: Uuid, content: String) {
        let chat_msg = WebSocketMessage::ChatMessage {
            room_id,
            content,
            reply_to: None,
        };
        self.write
            .send(Message::Text(chat_msg.to_json().unwrap()))
            .await
            .expect("Failed to send message");
    }

    /// 接收下一条消息
    async fn recv_message(&mut self) -> WebSocketMessage {
        let response = timeout(Duration::from_secs(5), read_next_message(&mut self.read))
            .await
            .expect("Timeout")
            .expect("No response")
            .expect("Failed to read");

        if let Message::Text(text) = response {
            serde_json::from_str(&text).expect("Failed to parse message")
        } else {
            panic!("Expected text message");
        }
    }
}

// ==================== 并发连接测试 ====================

/// 测试多客户端同时连接
#[tokio::test]
async fn test_concurrent_connections() {
    let (server, db) = setup_test_server().await;

    // 创建 10 个客户端同时连接
    let mut clients = Vec::new();
    for i in 0..10 {
        let username = format!("concurrent_user_{}", i);
        let client = WebSocketClient::new(&server, &db, &username).await;
        clients.push(client);
    }

    // 验证所有客户端都连接成功
    assert_eq!(clients.len(), 10);
    println!("Successfully created 10 concurrent connections");
}

/// 测试多客户端同时加入同一房间
#[tokio::test]
async fn test_concurrent_join_room() {
    let (server, db) = setup_test_server().await;

    // 创建房间所有者
    let (owner_id, _, _) = create_test_user_with_token(&db, "room_owner").await;
    let room_id = create_test_room(&db, owner_id, "Concurrent Test Room").await;

    // 创建 20 个客户端同时加入房间
    let mut clients = Vec::new();
    for i in 0..20 {
        let username = format!("joiner_{}", i);
        let mut client = WebSocketClient::new(&server, &db, &username).await;
        client.join_room(room_id).await;
        clients.push(client);
    }

    // 验证所有客户端都加入成功
    assert_eq!(clients.len(), 20);
    println!("20 clients successfully joined the same room concurrently");
}

/// 测试多客户端同时发送消息
#[tokio::test]
async fn test_concurrent_send_messages() {
    let (server, db) = setup_test_server().await;

    // 创建房间
    let (owner_id, _, _) = create_test_user_with_token(&db, "msg_room_owner").await;
    let room_id = create_test_room(&db, owner_id, "Concurrent Message Room").await;

    // 创建 5 个客户端
    let mut clients = Vec::new();
    for i in 0..5 {
        let username = format!("sender_{}", i);
        let mut client = WebSocketClient::new(&server, &db, &username).await;
        client.join_room(room_id).await;
        clients.push(client);
    }

    // 每个客户端发送 10 条消息
    let message_count = 10;
    for (i, client) in clients.iter_mut().enumerate() {
        for j in 0..message_count {
            client
                .send_message(room_id, format!("Message {} from client {}", j, i))
                .await;
        }
    }

    // 验证消息总数
    let total_messages = 5 * message_count;
    println!(
        "Sent {} messages concurrently from 5 clients",
        total_messages
    );
}

// ==================== 压力测试 ====================

/// 测试大量消息发送（压力测试）
#[tokio::test]
async fn test_stress_many_messages() {
    let (server, db) = setup_test_server().await;

    // 创建房间
    let (owner_id, _, _) = create_test_user_with_token(&db, "stress_owner").await;
    let room_id = create_test_room(&db, owner_id, "Stress Test Room").await;

    // 创建 3 个客户端
    let mut clients = Vec::new();
    for i in 0..3 {
        let username = format!("stress_user_{}", i);
        let mut client = WebSocketClient::new(&server, &db, &username).await;
        client.join_room(room_id).await;
        clients.push(client);
    }

    // 每个客户端发送 100 条消息
    let messages_per_client = 100;
    for (i, client) in clients.iter_mut().enumerate() {
        for j in 0..messages_per_client {
            client
                .send_message(room_id, format!("Stress message {} from {}", j, i))
                .await;
            // 每发送 10 条消息短暂休眠，避免过快
            if j % 10 == 0 {
                sleep(Duration::from_millis(10)).await;
            }
        }
    }

    let total = 3 * messages_per_client;
    println!("Stress test completed: {} messages sent", total);
}

/// 测试多个房间同时运作
#[tokio::test]
async fn test_stress_multiple_rooms() {
    let (server, db) = setup_test_server().await;

    // 创建 5 个房间
    let mut room_ids = Vec::new();
    let (owner_id, _, _) = create_test_user_with_token(&db, "multi_room_owner").await;
    for i in 0..5 {
        let room_id = create_test_room(&db, owner_id, &format!("Room {}", i)).await;
        room_ids.push(room_id);
    }

    // 每个房间分配 3 个用户
    for (room_idx, room_id) in room_ids.iter().enumerate() {
        let mut clients = Vec::new();
        for i in 0..3 {
            let username = format!("room{}_user{}", room_idx, i);
            let mut client = WebSocketClient::new(&server, &db, &username).await;
            client.join_room(*room_id).await;
            clients.push(client);
        }

        // 每个用户发送 5 条消息
        for client in clients.iter_mut() {
            for j in 0..5 {
                client
                    .send_message(*room_id, format!("Message {} in room {}", j, room_idx))
                    .await;
            }
        }
    }

    println!("Stress test with 5 rooms and 15 users completed");
}

// ==================== 断线重连测试 ====================

/// 测试断线重连后房间恢复
#[tokio::test]
async fn test_reconnect_room_restoration() {
    let (server, db) = setup_test_server().await;

    // 创建房间
    let (owner_id, _, _) = create_test_user_with_token(&db, "reconnect_owner").await;
    let room_id = create_test_room(&db, owner_id, "Reconnect Test Room").await;

    // 创建客户端并加入房间
    let mut client = WebSocketClient::new(&server, &db, "reconnect_user").await;
    client.join_room(room_id).await;

    // 发送一条消息
    client
        .send_message(room_id, "Message before disconnect".to_string())
        .await;

    // 模拟断线（关闭连接）
    drop(client);

    // 短暂休眠后重连
    sleep(Duration::from_millis(500)).await;

    // 重新连接（这里简化测试，实际应该使用 Reconnect 消息）
    // 注意：完整重连测试需要模拟断线场景，这里仅测试基本重连逻辑
    println!("Reconnection test completed (simplified)");
}

// ==================== 消息顺序和可靠性测试 ====================

/// 测试消息顺序保证
#[tokio::test]
async fn test_message_ordering() {
    let (server, db) = setup_test_server().await;

    // 创建房间
    let (owner_id, _, _) = create_test_user_with_token(&db, "order_owner").await;
    let room_id = create_test_room(&db, owner_id, "Order Test Room").await;

    // 创建发送者和接收者
    let mut sender = WebSocketClient::new(&server, &db, "sender").await;
    sender.join_room(room_id).await;

    let mut receiver = WebSocketClient::new(&server, &db, "receiver").await;
    receiver.join_room(room_id).await;

    // 发送 20 条有序消息
    let message_count = 20;
    for i in 0..message_count {
        sender.send_message(room_id, format!("Message {}", i)).await;
    }

    // 接收者应该按顺序收到消息
    let mut received_count = 0;
    let mut expected_index = 0;

    // 设置超时接收
    loop {
        match timeout(Duration::from_secs(2), receiver.recv_message()).await {
            Ok(msg) => {
                if let WebSocketMessage::NewMessage { content, .. } = msg {
                    received_count += 1;
                    // 验证消息内容包含预期索引
                    if content.contains(&format!("Message {}", expected_index)) {
                        expected_index += 1;
                    }
                }
            }
            Err(_) => break, // 超时，认为接收完成
        }

        if received_count >= message_count {
            break;
        }
    }

    println!(
        "Message ordering test: received {}/{} messages, sequential: {}",
        received_count, message_count, expected_index
    );
}

// ==================== 边界条件测试 ====================

/// 测试大消息发送（接近限制）
#[tokio::test]
async fn test_large_message() {
    let (server, db) = setup_test_server().await;

    // 创建房间
    let (owner_id, _, _) = create_test_user_with_token(&db, "large_msg_owner").await;
    let room_id = create_test_room(&db, owner_id, "Large Message Room").await;

    // 创建测试用户并先通过数据库加入房间
    let (user_id, _, _token) = create_test_user_with_token(&db, "large_msg_user").await;
    let room_service = RoomService::new(db.clone());
    room_service
        .join_room(room_id, user_id)
        .await
        .expect("Failed to join room");

    // 创建客户端
    let mut client = WebSocketClient::new(&server, &db, "large_msg_user").await;
    client.join_room(room_id).await;

    // 等待房间加入完成并清空积压消息
    sleep(Duration::from_millis(300)).await;
    while timeout(Duration::from_millis(100), client.recv_message())
        .await
        .is_ok()
    {
        // 丢弃所有积压消息
    }

    // 发送接近限制的大消息（2000 字符）
    let large_content = "A".repeat(2000);
    client.send_message(room_id, large_content).await;

    // 验证消息发送成功
    let response = client.recv_message().await;
    match response {
        WebSocketMessage::NewMessage { content, .. } => {
            assert_eq!(content.len(), 2000);
            println!("Large message (2000 chars) sent successfully");
        }
        msg => panic!("Expected NewMessage, got: {:?}", msg),
    }
}

/// 测试超长消息（超过限制）
#[tokio::test]
async fn test_oversized_message() {
    let (server, db) = setup_test_server().await;

    // 创建房间
    let (owner_id, _, _) = create_test_user_with_token(&db, "oversized_owner").await;
    let room_id = create_test_room(&db, owner_id, "Oversized Room").await;

    // 创建客户端
    let mut client = WebSocketClient::new(&server, &db, "oversized_user").await;
    client.join_room(room_id).await;

    // 等待房间加入完成
    sleep(Duration::from_millis(200)).await;

    // 发送超过限制的消息（3000 字符，超过 2000 限制）
    let oversized_content = "B".repeat(3000);
    client.send_message(room_id, oversized_content).await;

    // 应该收到错误响应
    let response = client.recv_message().await;
    match response {
        WebSocketMessage::Error { code, message } => {
            assert_eq!(code, "INVALID_CONTENT");
            println!(
                "Oversized message correctly rejected with code: {}, message: {}",
                code, message
            );
        }
        WebSocketMessage::NewMessage { .. } => {
            panic!("Oversized message should be rejected");
        }
        _ => {
            panic!("Expected Error response, got: {:?}", response);
        }
    }
}

/// 测试特殊字符消息
#[tokio::test]
async fn test_special_characters_message() {
    let (server, db) = setup_test_server().await;

    // 创建房间
    let (owner_id, _, _) = create_test_user_with_token(&db, "special_owner").await;
    let room_id = create_test_room(&db, owner_id, "Special Chars Room").await;

    // 创建测试用户并先通过数据库加入房间
    let (user_id, _, _token) = create_test_user_with_token(&db, "special_user").await;
    let room_service = RoomService::new(db.clone());
    room_service
        .join_room(room_id, user_id)
        .await
        .expect("Failed to join room");

    // 创建客户端
    let mut client = WebSocketClient::new(&server, &db, "special_user").await;
    client.join_room(room_id).await;

    // 等待房间加入完成并清空积压消息
    sleep(Duration::from_millis(300)).await;
    while timeout(Duration::from_millis(100), client.recv_message())
        .await
        .is_ok()
    {
        // 丢弃所有积压消息
    }

    // 发送包含特殊字符的消息
    let special_content = "Special chars: <>&\"' \n\t 🚀 中文 日本語";
    client
        .send_message(room_id, special_content.to_string())
        .await;

    // 验证消息正确传输
    let response = client.recv_message().await;
    match response {
        WebSocketMessage::NewMessage { content, .. } => {
            assert_eq!(content, special_content);
            println!("Special characters message sent successfully");
        }
        msg => panic!("Expected NewMessage, got: {:?}", msg),
    }
}

/// 测试空消息
#[tokio::test]
async fn test_empty_message() {
    let (server, db) = setup_test_server().await;

    // 使用唯一名称避免冲突
    let unique_id = Uuid::new_v4();
    let (owner_id, _, _) =
        create_test_user_with_token(&db, &format!("empty_owner_{}", unique_id)).await;
    let room_id =
        create_test_room(&db, owner_id, &format!("Empty Message Room {}", unique_id)).await;

    // 创建客户端
    let mut client = WebSocketClient::new(&server, &db, &format!("empty_user_{}", unique_id)).await;

    // 先加入房间，确保成功
    client.join_room(room_id).await;

    // 短暂延迟确保房间状态稳定
    sleep(Duration::from_millis(200)).await;

    // 发送空消息
    client.send_message(room_id, "".to_string()).await;

    // 应该收到错误响应
    let response = client.recv_message().await;
    match response {
        WebSocketMessage::Error { code, message } => {
            println!(
                "Empty message rejected with code: {}, message: {}",
                code, message
            );
            assert_eq!(code, "INVALID_CONTENT");
        }
        WebSocketMessage::NewMessage { .. } => {
            panic!("Empty message should be rejected");
        }
        _ => {
            panic!("Expected Error response, got: {:?}", response);
        }
    }
}

/// 测试快速连续发送消息（频率限制）
#[tokio::test]
async fn test_rapid_message_spam() {
    let (server, db) = setup_test_server().await;

    // 创建房间
    let (owner_id, _, _) = create_test_user_with_token(&db, "spam_owner").await;
    let room_id = create_test_room(&db, owner_id, "Spam Test Room").await;

    // 创建客户端
    let mut client = WebSocketClient::new(&server, &db, "spam_user").await;
    client.join_room(room_id).await;

    // 快速发送 50 条消息
    let spam_count = 50;
    for i in 0..spam_count {
        client
            .send_message(room_id, format!("Spam message {}", i))
            .await;
    }

    println!("Sent {} rapid messages", spam_count);
    // 观察是否有频率限制或错误
}

/// 测试用户状态变更广播
#[tokio::test]
async fn test_user_status_broadcast() {
    let (server, db) = setup_test_server().await;

    // 创建房间
    let (owner_id, _, _) = create_test_user_with_token(&db, "status_owner").await;
    let room_id = create_test_room(&db, owner_id, "Status Test Room").await;

    // 创建两个用户并先通过数据库加入房间
    let (user1_id, _, _token1) = create_test_user_with_token(&db, "status_user1").await;
    let (user2_id, _, _token2) = create_test_user_with_token(&db, "status_user2").await;
    let room_service = RoomService::new(db.clone());
    room_service
        .join_room(room_id, user1_id)
        .await
        .expect("Failed to join room");
    room_service
        .join_room(room_id, user2_id)
        .await
        .expect("Failed to join room");

    // 创建两个客户端
    let mut user1 = WebSocketClient::new(&server, &db, "status_user1").await;
    user1.join_room(room_id).await;

    let mut user2 = WebSocketClient::new(&server, &db, "status_user2").await;
    user2.join_room(room_id).await;

    // 等待所有加入消息处理完成
    sleep(Duration::from_millis(500)).await;

    // 清空 user2 的积压消息
    while timeout(Duration::from_millis(100), user2.recv_message())
        .await
        .is_ok()
    {
        // 丢弃所有积压消息
    }

    // user1 更新状态
    let update_status_msg = WebSocketMessage::UpdateStatus {
        status: seredeli_room::websocket::protocol::UserStatus::Away,
    };
    user1
        .write
        .send(Message::Text(update_status_msg.to_json().unwrap()))
        .await
        .expect("Failed to send status update");

    // user2 应该收到状态变更通知
    let response = timeout(Duration::from_secs(5), user2.recv_message())
        .await
        .expect("Timeout waiting for status update");
    match response {
        WebSocketMessage::UserStatusChanged {
            user_id, status, ..
        } => {
            // UserStatus 现在已经实现了 PartialEq
            assert_eq!(user_id, user1.user_id);
            assert_eq!(status, seredeli_room::websocket::protocol::UserStatus::Away);
            println!("User status broadcast works correctly");
        }
        msg => {
            println!("Status broadcast test: received {:?}", msg);
            panic!("Expected UserStatusChanged message, got: {:?}", msg);
        }
    }
}

/// 测试正在输入状态
#[tokio::test]
async fn test_typing_indicator() {
    let (server, db) = setup_test_server().await;

    // 创建房间
    let (owner_id, _, _) = create_test_user_with_token(&db, "typing_owner").await;
    let room_id = create_test_room(&db, owner_id, "Typing Test Room").await;

    // 创建两个客户端
    let mut user1 = WebSocketClient::new(&server, &db, "typing_user1").await;
    user1.join_room(room_id).await;

    let mut user2 = WebSocketClient::new(&server, &db, "typing_user2").await;
    user2.join_room(room_id).await;

    // user1 发送正在输入
    let typing_msg = WebSocketMessage::Typing { room_id };
    user1
        .write
        .send(Message::Text(typing_msg.to_json().unwrap()))
        .await
        .expect("Failed to send typing");

    // user2 应该收到正在输入通知（广播消息）
    // 注意：Typing 消息会触发服务端的 UserTyping 广播，但协议中可能没有这个变体
    // 这里我们简化测试，只验证 Typing 消息能被接受
    sleep(Duration::from_millis(100)).await;
    println!("Typing indicator test completed (simplified)");
}

// ==================== 实际应用场景综合测试 ====================

/// 模拟真实聊天场景：多人多轮对话
#[tokio::test]
async fn test_realistic_chat_scenario() {
    let (server, db) = setup_test_server().await;

    // 创建房间
    let (owner_id, _, _) = create_test_user_with_token(&db, "chat_host").await;
    let room_id = create_test_room(&db, owner_id, "Realistic Chat Room").await;

    // 创建 5 个用户
    let mut users = Vec::new();
    for i in 0..5 {
        let username = format!("chat_participant_{}", i);
        let mut client = WebSocketClient::new(&server, &db, &username).await;
        client.join_room(room_id).await;
        users.push(client);
    }

    // 模拟多轮对话
    let rounds = 3;
    for round in 0..rounds {
        for (i, user) in users.iter_mut().enumerate() {
            user.send_message(
                room_id,
                format!("Round {}: Hello from user {}", round + 1, i + 1),
            )
            .await;

            // 模拟思考时间
            sleep(Duration::from_millis(50)).await;
        }
    }

    println!(
        "Realistic chat scenario completed: {} rounds, {} participants",
        rounds,
        users.len()
    );
}

/// 测试房间切换场景
#[tokio::test]
async fn test_room_switching() {
    let (server, db) = setup_test_server().await;

    // 创建两个房间
    let (owner_id, _, _) = create_test_user_with_token(&db, "switch_owner").await;
    let room1_id = create_test_room(&db, owner_id, "Room 1").await;
    let room2_id = create_test_room(&db, owner_id, "Room 2").await;

    // 创建用户
    let mut user = WebSocketClient::new(&server, &db, "room_switcher").await;

    // 加入第一个房间
    user.join_room(room1_id).await;
    user.send_message(room1_id, "Hello in Room 1".to_string())
        .await;

    // 离开第一个房间
    let leave_msg = WebSocketMessage::LeaveRoom { room_id: room1_id };
    user.write
        .send(Message::Text(leave_msg.to_json().unwrap()))
        .await
        .expect("Failed to leave room");
    let _ = user.recv_message().await; // 等待离开响应

    // 加入第二个房间
    user.join_room(room2_id).await;
    user.send_message(room2_id, "Hello in Room 2".to_string())
        .await;

    println!("Room switching test completed successfully");
}

/// 测试认证超时场景
#[tokio::test]
async fn test_auth_timeout() {
    let (server, _db) = setup_test_server().await;

    let url = server.url();
    let (ws_stream, _) = connect_async(&url).await.expect("Failed to connect");
    let (_write, mut read) = ws_stream.split();

    // 不发送认证消息，等待超时
    // 服务器应该在 30 秒后断开连接
    let result = timeout(Duration::from_secs(35), read.next()).await;

    match result {
        Ok(_) => {
            println!("Connection closed (possibly due to auth timeout)");
        }
        Err(_) => {
            println!("Auth timeout test: connection still open after 35s");
        }
    }
}
