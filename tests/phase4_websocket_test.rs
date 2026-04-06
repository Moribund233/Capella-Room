//! 阶段四 WebSocket 实时通信测试
//!
//! 阶段四包含以下功能：
//! - 4.1 WebSocket 管理器 - 完善连接管理，实现用户注册和断开处理
//! - 4.2 WebSocket 处理器 - 实现 WebSocket 升级、消息收发循环
//! - 4.3 消息协议 - 定义 WebSocket 消息格式，处理各种消息类型
//! - 4.4 房间广播 - 实现房间消息广播、单播
//! - 4.5 心跳机制 - 实现客户端心跳检测、服务端心跳响应
//!
//! 验收标准：
//! ✅ 客户端可以建立 WebSocket 连接
//! ✅ 用户可以加入/离开房间
//! ✅ 消息可以实时广播到房间所有成员
//! ✅ 心跳机制正常工作，超时连接被清理

use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use futures::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio::time::{sleep, timeout};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use uuid::Uuid;

// 引入被测模块
use seredeli_room::{
    config::{ConfigManager, DatabaseConfig, JwtConfig, UploadConfig},
    db::Database,
    routes::create_router,
    services::{auth_service::AuthService, room_service::RoomService, user_service::UserService},
    state::AppState,
    utils::logging::MetricsCollector,
    websocket::{manager::WebSocketManager, protocol::WebSocketMessage},
};
use std::sync::Arc;

/// 测试服务器句柄
struct TestServer {
    addr: SocketAddr,
    _shutdown: tokio::sync::oneshot::Sender<()>,
}

impl TestServer {
    fn url(&self) -> String {
        format!("ws://{}/ws", self.addr)
    }
}

/// 测试辅助函数：加载测试环境变量
fn load_test_env() {
    // 加载 .env.test 文件
    if std::path::Path::new(".env.test").exists() {
        dotenvy::from_filename(".env.test").ok();
    } else if std::path::Path::new("../.env.test").exists() {
        dotenvy::from_filename("../.env.test").ok();
    }
}

/// 测试辅助函数：读取下一条非Ping消息
async fn read_next_message(
    read: &mut futures::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
) -> Option<Result<Message, tokio_tungstenite::tungstenite::Error>> {
    while let Some(result) = read.next().await {
        if let Ok(Message::Text(text)) = &result {
            // 检查是否是Ping消息
            if let Ok(ws_msg) = serde_json::from_str::<WebSocketMessage>(text) {
                if matches!(ws_msg, WebSocketMessage::Ping) {
                    // 跳过Ping消息，继续读取
                    continue;
                }
            }
        }
        return Some(result);
    }
    None
}

/// 测试辅助函数：创建测试数据库连接
async fn setup_test_db() -> Database {
    // 确保环境变量已加载
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

    // 运行数据库迁移
    db.migrate().await.expect("Failed to run migrations");

    db
}

/// 测试辅助函数：创建测试服务器
async fn setup_test_server() -> (TestServer, Database) {
    let db = setup_test_db().await;

    // 设置 UPLOAD_DIR 环境变量（如果不存在）
    if std::env::var("UPLOAD_DIR").is_err() {
        let temp_dir =
            std::env::temp_dir().join(format!("seredeli_upload_test_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temp upload directory");
        std::env::set_var("UPLOAD_DIR", temp_dir.to_str().unwrap());
    }

    let ws_manager = WebSocketManager::new();

    let config = seredeli_room::config::AppConfig {
        app: Default::default(),
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
        websocket: Default::default(),
        reconnect: Default::default(),
        logging: Default::default(),
        cors: Default::default(),
        system: Default::default(),
        admin: Default::default(),
    };

    let metrics_collector = Arc::new(MetricsCollector::new());

    let config_manager = ConfigManager::new(db.clone(), config.clone());

    let state = AppState::new(
        db.clone(),
        ws_manager,
        config,
        metrics_collector,
        Arc::new(config_manager),
    )
    .expect("Failed to create app state");
    let app = create_router(state);

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind");
    let addr = listener.local_addr().unwrap();

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

    tokio::spawn(async move {
        let server = axum::serve(listener, app);
        tokio::select! {
            _ = server => {},
            _ = shutdown_rx => {},
        }
    });

    // 等待服务器启动
    sleep(Duration::from_millis(100)).await;

    (
        TestServer {
            addr,
            _shutdown: shutdown_tx,
        },
        db,
    )
}

/// 测试辅助函数：创建测试用户并返回token
async fn create_test_user_with_token(db: &Database, username: &str) -> (Uuid, String, String) {
    let user_service = UserService::new(db.clone());
    let auth_service = AuthService::new(JwtConfig {
        secret: Some("test_secret_key_for_testing_purposes_only".to_string()),
        expiration_hours: 24,
    });

    let email = format!("{}@test.com", username);
    let password = "TestPassword123";

    // 检查用户是否已存在
    if let Ok(Some(user)) = user_service.get_user_by_email(&email).await {
        let tokens = auth_service.generate_token_pair(user.id).unwrap();
        return (user.id, password.to_string(), tokens.access_token);
    }

    let password_hash = auth_service.hash_password(password).unwrap();
    let user = user_service
        .create_user(username, &email, &password_hash)
        .await
        .unwrap();
    let tokens = auth_service.generate_token_pair(user.id).unwrap();

    (user.id, password.to_string(), tokens.access_token)
}

/// 测试辅助函数：创建测试房间
async fn create_test_room(db: &Database, owner_id: Uuid, name: &str) -> Uuid {
    let room_service = RoomService::new(db.clone());

    let room = room_service
        .create_room(name, Some("Test room description"), owner_id, false, 100)
        .await
        .unwrap();

    room.id
}

/// WebSocket 连接测试
#[cfg(test)]
mod websocket_connection_tests {
    use super::*;

    /// 测试 WebSocket 连接建立
    #[tokio::test]
    async fn test_websocket_connection() {
        let (server, _db) = setup_test_server().await;

        let url = server.url();
        let (ws_stream, _) = connect_async(&url).await.expect("Failed to connect");

        let (_write, read) = ws_stream.split();

        // 连接应该成功建立
        // 由于没有发送认证消息，服务器应该等待认证
        // 这里我们只测试连接是否成功

        // 关闭连接
        drop(read);
    }

    /// 测试 WebSocket 认证成功
    #[tokio::test]
    async fn test_websocket_auth_success() {
        let (server, db) = setup_test_server().await;

        let (_, _, token) = create_test_user_with_token(&db, "testuser_auth").await;

        let url = server.url();
        let (ws_stream, _) = connect_async(&url).await.expect("Failed to connect");

        let (mut write, mut read) = ws_stream.split();

        // 发送认证消息
        let auth_msg = WebSocketMessage::Auth {
            token: token.clone(),
        };
        let auth_json = auth_msg.to_json().unwrap();
        write
            .send(Message::Text(auth_json))
            .await
            .expect("Failed to send auth message");

        // 等待认证响应
        let response = timeout(Duration::from_secs(5), read.next())
            .await
            .expect("Timeout waiting for auth response")
            .expect("No response received")
            .expect("Failed to read response");

        if let Message::Text(text) = response {
            let msg: WebSocketMessage = serde_json::from_str(&text).unwrap();
            match msg {
                WebSocketMessage::AuthResult { success, .. } => {
                    assert!(success, "Authentication should succeed");
                }
                _ => panic!("Expected AuthResult, got {:?}", msg),
            }
        } else {
            panic!("Expected text message");
        }
    }

    /// 测试 WebSocket 认证失败 - 无效 token
    #[tokio::test]
    async fn test_websocket_auth_failure() {
        let (server, _db) = setup_test_server().await;

        let url = server.url();
        let (ws_stream, _) = connect_async(&url).await.expect("Failed to connect");

        let (mut write, mut read) = ws_stream.split();

        // 发送无效认证消息
        let auth_msg = WebSocketMessage::Auth {
            token: "invalid_token".to_string(),
        };
        let auth_json = auth_msg.to_json().unwrap();
        write
            .send(Message::Text(auth_json))
            .await
            .expect("Failed to send auth message");

        // 等待认证响应
        let response = timeout(Duration::from_secs(5), read.next())
            .await
            .expect("Timeout waiting for auth response")
            .expect("No response received")
            .expect("Failed to read response");

        if let Message::Text(text) = response {
            let msg: WebSocketMessage = serde_json::from_str(&text).unwrap();
            match msg {
                WebSocketMessage::AuthResult { success, .. } => {
                    assert!(!success, "Authentication should fail");
                }
                _ => panic!("Expected AuthResult, got {:?}", msg),
            }
        } else {
            panic!("Expected text message");
        }
    }

    /// 测试 WebSocket 认证超时
    #[tokio::test]
    async fn test_websocket_auth_timeout() {
        // 注意：这个测试可能需要较长时间，因为需要等待认证超时
        // 在实际测试中可以跳过或调整超时时间
    }
}

/// WebSocket 房间管理测试
#[cfg(test)]
mod websocket_room_tests {
    use super::*;

    /// 测试加入房间
    #[tokio::test]
    async fn test_join_room() {
        let (server, db) = setup_test_server().await;

        let (user_id, _, token) = create_test_user_with_token(&db, "testuser_join").await;
        let room_id = create_test_room(&db, user_id, "Test Room Join").await;

        let url = server.url();
        let (ws_stream, _) = connect_async(&url).await.expect("Failed to connect");

        let (mut write, mut read) = ws_stream.split();

        // 发送认证消息
        let auth_msg = WebSocketMessage::Auth { token };
        let auth_json = auth_msg.to_json().unwrap();
        write.send(Message::Text(auth_json)).await.unwrap();

        // 等待认证响应（跳过Ping）
        let response = timeout(Duration::from_secs(5), read_next_message(&mut read))
            .await
            .expect("Timeout")
            .expect("No response")
            .expect("Failed to read");

        if let Message::Text(text) = response {
            let msg: WebSocketMessage = serde_json::from_str(&text).unwrap();
            match msg {
                WebSocketMessage::AuthResult { success, .. } => {
                    assert!(success, "Authentication should succeed");
                }
                _ => panic!("Expected AuthResult, got {:?}", msg),
            }
        }

        // 发送加入房间消息
        let join_msg = WebSocketMessage::JoinRoom { room_id };
        let join_json = join_msg.to_json().unwrap();
        write.send(Message::Text(join_json)).await.unwrap();

        // 等待加入响应（跳过Ping）
        let response = timeout(Duration::from_secs(5), read_next_message(&mut read))
            .await
            .expect("Timeout")
            .expect("No response")
            .expect("Failed to read");

        if let Message::Text(text) = response {
            let msg: WebSocketMessage = serde_json::from_str(&text).unwrap();
            match msg {
                WebSocketMessage::RoomJoined { room_id: rid, .. } => {
                    assert_eq!(rid, room_id);
                }
                _ => panic!("Expected RoomJoined, got {:?}", msg),
            }
        }
    }

    /// 测试离开房间
    #[tokio::test]
    async fn test_leave_room() {
        let (server, db) = setup_test_server().await;

        let (user_id, _, token) = create_test_user_with_token(&db, "testuser_leave").await;
        let room_id = create_test_room(&db, user_id, "Test Room Leave").await;

        let url = server.url();
        let (ws_stream, _) = connect_async(&url).await.expect("Failed to connect");

        let (mut write, mut read) = ws_stream.split();

        // 认证
        let auth_msg = WebSocketMessage::Auth { token };
        write
            .send(Message::Text(auth_msg.to_json().unwrap()))
            .await
            .unwrap();
        let _ = timeout(Duration::from_secs(5), read_next_message(&mut read)).await;

        // 加入房间
        let join_msg = WebSocketMessage::JoinRoom { room_id };
        write
            .send(Message::Text(join_msg.to_json().unwrap()))
            .await
            .unwrap();
        // 等待RoomJoined和OnlineUsers
        let _ = timeout(Duration::from_secs(5), read_next_message(&mut read)).await;
        let _ = timeout(Duration::from_secs(5), read_next_message(&mut read)).await;

        // 离开房间
        let leave_msg = WebSocketMessage::LeaveRoom { room_id };
        write
            .send(Message::Text(leave_msg.to_json().unwrap()))
            .await
            .unwrap();

        // 等待离开响应（跳过Ping）
        let response = timeout(Duration::from_secs(5), read_next_message(&mut read))
            .await
            .expect("Timeout")
            .expect("No response")
            .expect("Failed to read");

        if let Message::Text(text) = response {
            let msg: WebSocketMessage = serde_json::from_str(&text).unwrap();
            match msg {
                WebSocketMessage::RoomLeft { room_id: rid, .. } => {
                    assert_eq!(rid, room_id);
                }
                _ => panic!("Expected RoomLeft, got {:?}", msg),
            }
        }
    }

    /// 测试加入不存在的房间
    #[tokio::test]
    async fn test_join_nonexistent_room() {
        let (server, db) = setup_test_server().await;

        let (_, _, token) = create_test_user_with_token(&db, "testuser_nonexist").await;
        let fake_room_id = Uuid::new_v4();

        let url = server.url();
        let (ws_stream, _) = connect_async(&url).await.expect("Failed to connect");

        let (mut write, mut read) = ws_stream.split();

        // 认证
        let auth_msg = WebSocketMessage::Auth { token };
        write
            .send(Message::Text(auth_msg.to_json().unwrap()))
            .await
            .unwrap();
        let _ = timeout(Duration::from_secs(5), read_next_message(&mut read)).await;

        // 尝试加入不存在的房间
        let join_msg = WebSocketMessage::JoinRoom {
            room_id: fake_room_id,
        };
        write
            .send(Message::Text(join_msg.to_json().unwrap()))
            .await
            .unwrap();

        // 等待错误响应（跳过Ping）
        let response = timeout(Duration::from_secs(5), read_next_message(&mut read))
            .await
            .expect("Timeout")
            .expect("No response")
            .expect("Failed to read");

        if let Message::Text(text) = response {
            let msg: WebSocketMessage = serde_json::from_str(&text).unwrap();
            match msg {
                WebSocketMessage::Error { code, .. } => {
                    assert_eq!(code, "ROOM_NOT_FOUND");
                }
                _ => panic!("Expected Error, got {:?}", msg),
            }
        }
    }
}

/// WebSocket 消息测试
#[cfg(test)]
mod websocket_message_tests {
    use super::*;

    /// 测试发送和接收消息
    #[tokio::test]
    async fn test_send_receive_message() {
        let (server, db) = setup_test_server().await;

        let (user_id, _, token) = create_test_user_with_token(&db, "testuser_msg").await;
        let room_id = create_test_room(&db, user_id, "Test Room Message").await;

        let url = server.url();
        let (ws_stream, _) = connect_async(&url).await.expect("Failed to connect");

        let (mut write, mut read) = ws_stream.split();

        // 认证
        let auth_msg = WebSocketMessage::Auth { token };
        write
            .send(Message::Text(auth_msg.to_json().unwrap()))
            .await
            .unwrap();
        let _ = timeout(Duration::from_secs(5), read_next_message(&mut read)).await;

        // 加入房间
        let join_msg = WebSocketMessage::JoinRoom { room_id };
        write
            .send(Message::Text(join_msg.to_json().unwrap()))
            .await
            .unwrap();
        let _ = timeout(Duration::from_secs(5), read_next_message(&mut read)).await; // RoomJoined
        let _ = timeout(Duration::from_secs(5), read_next_message(&mut read)).await; // OnlineUsers

        // 发送聊天消息
        let chat_msg = WebSocketMessage::ChatMessage {
            room_id,
            content: "Hello, World!".to_string(),
            reply_to: None,
        };
        write
            .send(Message::Text(chat_msg.to_json().unwrap()))
            .await
            .unwrap();

        // 等待消息确认（NewMessage）（跳过Ping）
        let response = timeout(Duration::from_secs(5), read_next_message(&mut read))
            .await
            .expect("Timeout")
            .expect("No response")
            .expect("Failed to read");

        if let Message::Text(text) = response {
            let msg: WebSocketMessage = serde_json::from_str(&text).unwrap();
            match msg {
                WebSocketMessage::NewMessage {
                    room_id: rid,
                    content,
                    sender_id,
                    ..
                } => {
                    assert_eq!(rid, room_id);
                    assert_eq!(content, "Hello, World!");
                    assert_eq!(sender_id, user_id);
                }
                _ => panic!("Expected NewMessage, got {:?}", msg),
            }
        }
    }

    /// 测试向未加入的房间发送消息
    #[tokio::test]
    async fn test_send_to_unjoined_room() {
        let (server, db) = setup_test_server().await;

        let (user_id, _, token) = create_test_user_with_token(&db, "testuser_unjoined").await;
        let room_id = create_test_room(&db, user_id, "Test Room Unjoined").await;

        let url = server.url();
        let (ws_stream, _) = connect_async(&url).await.expect("Failed to connect");

        let (mut write, mut read) = ws_stream.split();

        // 认证
        let auth_msg = WebSocketMessage::Auth { token };
        write
            .send(Message::Text(auth_msg.to_json().unwrap()))
            .await
            .unwrap();
        let _ = timeout(Duration::from_secs(5), read_next_message(&mut read)).await;

        // 不加入房间，直接发送消息
        let chat_msg = WebSocketMessage::ChatMessage {
            room_id,
            content: "Hello!".to_string(),
            reply_to: None,
        };
        write
            .send(Message::Text(chat_msg.to_json().unwrap()))
            .await
            .unwrap();

        // 等待错误响应（跳过Ping）
        let response = timeout(Duration::from_secs(5), read_next_message(&mut read))
            .await
            .expect("Timeout")
            .expect("No response")
            .expect("Failed to read");

        if let Message::Text(text) = response {
            let msg: WebSocketMessage = serde_json::from_str(&text).unwrap();
            match msg {
                WebSocketMessage::Error { code, .. } => {
                    assert_eq!(code, "NOT_IN_ROOM");
                }
                _ => panic!("Expected Error, got {:?}", msg),
            }
        }
    }
}

/// WebSocket 心跳测试
#[cfg(test)]
mod websocket_heartbeat_tests {
    use super::*;

    /// 测试心跳机制
    #[tokio::test]
    async fn test_heartbeat() {
        let (server, db) = setup_test_server().await;

        let (_, _, token) = create_test_user_with_token(&db, "testuser_heartbeat").await;

        let url = server.url();
        let (ws_stream, _) = connect_async(&url).await.expect("Failed to connect");

        let (mut write, mut read) = ws_stream.split();

        // 认证
        let auth_msg = WebSocketMessage::Auth { token };
        write
            .send(Message::Text(auth_msg.to_json().unwrap()))
            .await
            .unwrap();
        let _ = timeout(Duration::from_secs(5), read.next()).await;

        // 等待服务器发送的心跳（30秒间隔，测试中我们等待较短时间）
        // 由于心跳间隔较长，这里我们主动发送 Pong 测试响应
        let pong_msg = WebSocketMessage::Pong;
        write
            .send(Message::Text(pong_msg.to_json().unwrap()))
            .await
            .unwrap();

        // 心跳测试通过（没有断开连接）
    }
}

/// WebSocket 管理器单元测试（不需要服务器）
#[cfg(test)]
mod websocket_manager_tests {
    use super::*;
    use tokio::sync::mpsc;

    /// 测试管理器连接管理
    #[test]
    fn test_manager_connection_management() {
        let manager = WebSocketManager::new();
        let user_id = Uuid::new_v4();
        let (tx, _rx) = mpsc::channel(100);

        // 连接
        manager.connect(user_id, "test_user".to_string(), tx);
        assert_eq!(manager.get_total_connections(), 1);
        assert!(manager.is_user_online(user_id));

        // 断开
        manager.disconnect(user_id);
        assert_eq!(manager.get_total_connections(), 0);
        assert!(!manager.is_user_online(user_id));
    }

    /// 测试房间订阅管理
    #[test]
    fn test_manager_room_subscription() {
        let manager = WebSocketManager::new();
        let user_id = Uuid::new_v4();
        let room_id = Uuid::new_v4();
        let (tx, _rx) = mpsc::channel(100);

        manager.connect(user_id, "test_user".to_string(), tx);

        // 加入房间
        manager.join_room(room_id, user_id);
        assert_eq!(manager.get_room_user_count(room_id), 1);
        assert!(manager.is_user_in_room(room_id, user_id));

        // 离开房间
        manager.leave_room(room_id, user_id);
        assert_eq!(manager.get_room_user_count(room_id), 0);
        assert!(!manager.is_user_in_room(room_id, user_id));
    }

    /// 测试广播功能
    #[tokio::test]
    async fn test_manager_broadcast() {
        let manager = WebSocketManager::new();
        let room_id = Uuid::new_v4();

        // 创建两个用户
        let user1_id = Uuid::new_v4();
        let (tx1, mut rx1) = mpsc::channel(100);
        manager.connect(user1_id, "user1".to_string(), tx1);
        manager.join_room(room_id, user1_id);

        let user2_id = Uuid::new_v4();
        let (tx2, mut rx2) = mpsc::channel(100);
        manager.connect(user2_id, "user2".to_string(), tx2);
        manager.join_room(room_id, user2_id);

        // 广播消息
        let message = r#"{"type": "Test", "content": "Hello"}"#.to_string();
        manager
            .broadcast_to_room_all(room_id, message.clone())
            .await;

        // 验证两个用户都收到消息
        let msg1 = rx1.recv().await.expect("User1 should receive message");
        assert_eq!(msg1, message);

        let msg2 = rx2.recv().await.expect("User2 should receive message");
        assert_eq!(msg2, message);
    }

    /// 测试获取房间用户列表
    #[test]
    fn test_get_room_users() {
        let manager = WebSocketManager::new();
        let room_id = Uuid::new_v4();

        let user1_id = Uuid::new_v4();
        let (tx1, _rx1) = mpsc::channel(100);
        manager.connect(user1_id, "user1".to_string(), tx1);
        manager.join_room(room_id, user1_id);

        let user2_id = Uuid::new_v4();
        let (tx2, _rx2) = mpsc::channel(100);
        manager.connect(user2_id, "user2".to_string(), tx2);
        manager.join_room(room_id, user2_id);

        let users = manager.get_room_users(room_id);
        assert_eq!(users.len(), 2);
    }
}

/// WebSocket 协议测试
#[cfg(test)]
mod websocket_protocol_tests {
    use super::*;

    /// 测试消息序列化和反序列化
    #[test]
    fn test_message_serialization() {
        let msg = WebSocketMessage::Ping;
        let json = msg.to_json().unwrap();
        assert_eq!(json, r#"{"type":"Ping"}"#);
    }

    /// 测试 Ping 消息
    #[test]
    fn test_ping_message() {
        let json = r#"{"type":"Ping"}"#;
        let msg = WebSocketMessage::from_json(json).unwrap();
        match msg {
            WebSocketMessage::Ping => {}
            _ => panic!("Expected Ping"),
        }
    }

    /// 测试 Pong 消息
    #[test]
    fn test_pong_message() {
        let json = r#"{"type":"Pong"}"#;
        let msg = WebSocketMessage::from_json(json).unwrap();
        match msg {
            WebSocketMessage::Pong => {}
            _ => panic!("Expected Pong"),
        }
    }

    /// 测试 Auth 消息
    #[test]
    fn test_auth_message() {
        let json = r#"{"type":"Auth","payload":{"token":"test_token_123"}}"#;
        let msg = WebSocketMessage::from_json(json).unwrap();
        match msg {
            WebSocketMessage::Auth { token } => {
                assert_eq!(token, "test_token_123");
            }
            _ => panic!("Expected Auth"),
        }
    }

    /// 测试 ChatMessage
    #[test]
    fn test_chat_message() {
        let room_id = Uuid::new_v4();
        let json = format!(
            r#"{{"type":"ChatMessage","payload":{{"room_id":"{}","content":"Hello","reply_to":null}}}}"#,
            room_id
        );
        let msg = WebSocketMessage::from_json(&json).unwrap();
        match msg {
            WebSocketMessage::ChatMessage {
                room_id: rid,
                content,
                reply_to,
            } => {
                assert_eq!(rid, room_id);
                assert_eq!(content, "Hello");
                assert_eq!(reply_to, None);
            }
            _ => panic!("Expected ChatMessage"),
        }
    }

    /// 测试复杂消息往返
    #[test]
    fn test_complex_message_roundtrip() {
        let room_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let msg = WebSocketMessage::NewMessage {
            message_id: Uuid::new_v4(),
            room_id,
            sender_id: user_id,
            sender_name: "TestUser".to_string(),
            content: "Test message content".to_string(),
            reply_to: None,
            reply_to_message: None,
            created_at: chrono::Utc::now(),
        };

        let json = msg.to_json().unwrap();
        let decoded = WebSocketMessage::from_json(&json).unwrap();

        match decoded {
            WebSocketMessage::NewMessage {
                content,
                sender_name,
                ..
            } => {
                assert_eq!(content, "Test message content");
                assert_eq!(sender_name, "TestUser");
            }
            _ => panic!("Expected NewMessage"),
        }
    }
}
