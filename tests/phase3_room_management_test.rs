//! 阶段三聊天室管理功能测试
//!
//! 阶段三包含以下功能：
//! - 3.1 聊天室模型 - 完善 Room 和 RoomMember 模型，实现成员角色系统
//! - 3.2 聊天室接口 - 实现创建、列表、详情等接口
//! - 3.3 成员管理 - 实现加入、离开、踢出、角色管理
//! - 3.4 权限控制 - 实现 Owner/Admin/Member 权限控制
//!
//! 验收标准：
//! ✅ 用户可以创建聊天室
//! ✅ 用户可以浏览公开聊天室列表
//! ✅ 用户可以加入/离开聊天室
//! ✅ 聊天室成员角色系统正常工作
//! ✅ 权限控制正确生效

use std::env;
use std::sync::Arc;

// 引入被测模块
use seredeli_room::{
    config::{DatabaseConfig, JwtConfig},
    db::Database,
    error::AppError,
    models::room::{CreateRoomRequest, MemberRole},
    services::{
        auth_service::{AuthService},
        room_service::RoomService,
        user_service::UserService,
    },
};
use uuid::Uuid;

/// 测试辅助函数：加载测试环境变量
fn load_test_env() {
    // 加载 .env.test 文件
    if std::path::Path::new(".env.test").exists() {
        dotenvy::from_filename(".env.test").ok();
    } else if std::path::Path::new("../.env.test").exists() {
        dotenvy::from_filename("../.env.test").ok();
    }
}

/// 测试辅助函数：创建测试数据库连接
async fn setup_test_db() -> Database {
    // 确保环境变量已加载
    load_test_env();

    // 使用 .env.test 中的 DATABASE_URL
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env.test or environment");

    let max_connections = env::var("APP_DATABASE__MAX_CONNECTIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5);

    let db_config = DatabaseConfig {
        url: database_url,
        max_connections,
    };

    let db = Database::new(&db_config).await.expect("Failed to connect to test database");
    
    // 运行数据库迁移
    db.migrate().await.expect("Failed to run migrations");
    
    db
}

/// 测试辅助函数：创建测试用户
async fn create_test_user(user_service: &UserService, username: &str) -> (Uuid, String, seredeli_room::services::auth_service::TokenPair) {
    let email = format!("{}@test.com", username);
    let password = "TestPassword123";

    // 检查用户是否已存在
    if let Ok(Some(user)) = user_service.get_user_by_email(&email).await {
        // 直接生成token
        let jwt_config = JwtConfig {
            secret: "test_secret_key_for_testing_purposes_only".to_string(),
            expiration_hours: 24,
        };
        let auth_service = AuthService::new(jwt_config);
        let tokens = auth_service.generate_token_pair(user.id).unwrap();
        return (user.id, password.to_string(), tokens);
    }

    let auth_service = AuthService::new(JwtConfig {
        secret: "test_secret_key_for_testing_purposes_only".to_string(),
        expiration_hours: 24,
    });

    let password_hash = auth_service.hash_password(password).unwrap();

    let user = user_service.create_user(username, &email, &password_hash).await.unwrap();
    let tokens = auth_service.generate_token_pair(user.id).unwrap();

    (user.id, password.to_string(), tokens)
}

/// 测试聊天室模型验证
#[cfg(test)]
mod room_model_tests {
    use super::*;
    use validator::Validate;

    /// 测试有效的创建聊天室请求
    #[test]
    fn test_valid_create_room_request() {
        let request = CreateRoomRequest {
            name: "Test Room".to_string(),
            description: Some("A test room description".to_string()),
            is_private: false,
            max_members: Some(50),
        };
        assert!(request.validate().is_ok());
    }

    /// 测试聊天室名称验证 - 不能为空
    #[test]
    fn test_room_name_validation_empty() {
        let request = CreateRoomRequest {
            name: "".to_string(),
            description: None,
            is_private: false,
            max_members: None,
        };
        assert!(request.validate().is_err());
    }

    /// 测试聊天室名称验证 - 长度限制
    #[test]
    fn test_room_name_validation_length() {
        // 超过50个字符
        let request = CreateRoomRequest {
            name: "a".repeat(51),
            description: None,
            is_private: false,
            max_members: None,
        };
        assert!(request.validate().is_err());

        // 正好50个字符
        let request = CreateRoomRequest {
            name: "a".repeat(50),
            description: None,
            is_private: false,
            max_members: None,
        };
        assert!(request.validate().is_ok());
    }

    /// 测试聊天室描述长度限制
    #[test]
    fn test_room_description_validation() {
        // 超过200个字符
        let request = CreateRoomRequest {
            name: "Test Room".to_string(),
            description: Some("a".repeat(201)),
            is_private: false,
            max_members: None,
        };
        assert!(request.validate().is_err());

        // 正好200个字符
        let request = CreateRoomRequest {
            name: "Test Room".to_string(),
            description: Some("a".repeat(200)),
            is_private: false,
            max_members: None,
        };
        assert!(request.validate().is_ok());
    }

    /// 测试成员数量限制验证
    #[test]
    fn test_max_members_validation() {
        // 小于2
        let request = CreateRoomRequest {
            name: "Test Room".to_string(),
            description: None,
            is_private: false,
            max_members: Some(1),
        };
        assert!(request.validate().is_err());

        // 大于1000
        let request = CreateRoomRequest {
            name: "Test Room".to_string(),
            description: None,
            is_private: false,
            max_members: Some(1001),
        };
        assert!(request.validate().is_err());

        // 边界值2
        let request = CreateRoomRequest {
            name: "Test Room".to_string(),
            description: None,
            is_private: false,
            max_members: Some(2),
        };
        assert!(request.validate().is_ok());

        // 边界值1000
        let request = CreateRoomRequest {
            name: "Test Room".to_string(),
            description: None,
            is_private: false,
            max_members: Some(1000),
        };
        assert!(request.validate().is_ok());
    }

    /// 测试成员角色序列化
    #[test]
    fn test_member_role_serialization() {
        let owner = MemberRole::Owner;
        let admin = MemberRole::Admin;
        let member = MemberRole::Member;

        assert_eq!(serde_json::to_string(&owner).unwrap(), "\"owner\"");
        assert_eq!(serde_json::to_string(&admin).unwrap(), "\"admin\"");
        assert_eq!(serde_json::to_string(&member).unwrap(), "\"member\"");
    }

    /// 测试成员角色反序列化
    #[test]
    fn test_member_role_deserialization() {
        let owner: MemberRole = serde_json::from_str("\"owner\"").unwrap();
        let admin: MemberRole = serde_json::from_str("\"admin\"").unwrap();
        let member: MemberRole = serde_json::from_str("\"member\"").unwrap();

        assert!(matches!(owner, MemberRole::Owner));
        assert!(matches!(admin, MemberRole::Admin));
        assert!(matches!(member, MemberRole::Member));
    }
}

/// 测试聊天室服务
#[cfg(test)]
mod room_service_tests {
    use super::*;

    /// 测试创建聊天室
    #[tokio::test]
    async fn test_create_room() {
        let db = setup_test_db().await;
        let room_service = RoomService::new(db.clone());
        let user_service = UserService::new(db.clone());

        let (user_id, _, _) = create_test_user(&user_service, "test_create_room").await;

        let room = room_service
            .create_room(
                "Test Room",
                Some("Test Description"),
                user_id,
                false,
                100,
            )
            .await
            .unwrap();

        assert_eq!(room.name, "Test Room");
        assert_eq!(room.description, Some("Test Description".to_string()));
        assert_eq!(room.owner_id, user_id);
        assert!(!room.is_private);
        assert_eq!(room.max_members, 100);

        // 验证创建者自动成为成员（Owner角色）
        let members = room_service.get_room_members(room.id).await.unwrap();
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].user_id, user_id);
        assert!(matches!(members[0].role, MemberRole::Owner));
    }

    /// 测试获取聊天室列表
    #[tokio::test]
    async fn test_list_rooms() {
        let db = setup_test_db().await;
        let room_service = RoomService::new(db.clone());
        let user_service = UserService::new(db.clone());

        let (user_id, _, _) = create_test_user(&user_service, "test_list_rooms").await;

        // 创建几个测试房间
        room_service
            .create_room("Public Room 1", None, user_id, false, 100)
            .await
            .unwrap();
        room_service
            .create_room("Public Room 2", None, user_id, false, 100)
            .await
            .unwrap();

        // 获取公开房间列表
        let rooms = room_service.list_rooms(None, None, 10, 0).await.unwrap();
        assert!(!rooms.is_empty());

        // 验证返回的房间包含成员数
        for room in &rooms {
            assert!(room.member_count >= 0);
        }
    }

    /// 测试搜索聊天室
    #[tokio::test]
    async fn test_search_rooms() {
        let db = setup_test_db().await;
        let room_service = RoomService::new(db.clone());
        let user_service = UserService::new(db.clone());

        let (user_id, _, _) = create_test_user(&user_service, "test_search_rooms").await;

        // 创建特定名称的房间
        room_service
            .create_room("UniqueSearchRoom", None, user_id, false, 100)
            .await
            .unwrap();

        // 搜索房间
        let rooms = room_service
            .list_rooms(None, Some("UniqueSearch"), 10, 0)
            .await
            .unwrap();

        assert!(!rooms.is_empty());
        assert!(rooms.iter().any(|r| r.name == "UniqueSearchRoom"));
    }

    /// 测试获取聊天室详情
    #[tokio::test]
    async fn test_get_room_detail() {
        let db = setup_test_db().await;
        let room_service = RoomService::new(db.clone());
        let user_service = UserService::new(db.clone());

        let (user_id, _, _) = create_test_user(&user_service, "test_get_room").await;

        let room = room_service
            .create_room("Detail Test Room", None, user_id, false, 100)
            .await
            .unwrap();

        let detail = room_service.get_room_detail(room.id).await.unwrap();
        assert!(detail.is_some());

        let detail = detail.unwrap();
        assert_eq!(detail.id, room.id);
        assert_eq!(detail.name, room.name);
        assert_eq!(detail.member_count, 1); // 创建者是成员
    }

    /// 测试更新聊天室
    #[tokio::test]
    async fn test_update_room() {
        let db = setup_test_db().await;
        let room_service = RoomService::new(db.clone());
        let user_service = UserService::new(db.clone());

        let (user_id, _, _) = create_test_user(&user_service, "test_update_room").await;

        let room = room_service
            .create_room("Original Name", None, user_id, false, 100)
            .await
            .unwrap();

        let updated = room_service
            .update_room(room.id, Some("Updated Name"), Some("Updated Description"), None, None)
            .await
            .unwrap();

        assert_eq!(updated.name, "Updated Name");
        assert_eq!(updated.description, Some("Updated Description".to_string()));
    }

    /// 测试删除聊天室
    #[tokio::test]
    async fn test_delete_room() {
        let db = setup_test_db().await;
        let room_service = RoomService::new(db.clone());
        let user_service = UserService::new(db.clone());

        let (user_id, _, _) = create_test_user(&user_service, "test_delete_room").await;

        let room = room_service
            .create_room("Room To Delete", None, user_id, false, 100)
            .await
            .unwrap();

        // 删除房间
        room_service.delete_room(room.id).await.unwrap();

        // 验证房间已删除
        let result = room_service.get_room_by_id(room.id).await.unwrap();
        assert!(result.is_none());
    }

    /// 测试加入聊天室
    #[tokio::test]
    async fn test_join_room() {
        let db = setup_test_db().await;
        let room_service = RoomService::new(db.clone());
        let user_service = UserService::new(db.clone());

        let (owner_id, _, _) = create_test_user(&user_service, "test_join_owner").await;
        let (joiner_id, _, _) = create_test_user(&user_service, "test_join_joiner").await;

        let room = room_service
            .create_room("Join Test Room", None, owner_id, false, 100)
            .await
            .unwrap();

        // 新用户加入
        room_service.join_room(room.id, joiner_id).await.unwrap();

        // 验证成员数
        let member_count = room_service.get_room_member_count(room.id).await.unwrap();
        assert_eq!(member_count, 2);

        // 验证新成员的角色是 Member
        let member = room_service.get_room_member(room.id, joiner_id).await.unwrap();
        assert!(member.is_some());
        assert!(matches!(member.unwrap().role, MemberRole::Member));
    }

    /// 测试重复加入聊天室
    #[tokio::test]
    async fn test_join_room_already_member() {
        let db = setup_test_db().await;
        let room_service = RoomService::new(db.clone());
        let user_service = UserService::new(db.clone());

        let (user_id, _, _) = create_test_user(&user_service, "test_join_duplicate").await;

        let room = room_service
            .create_room("Duplicate Join Test", None, user_id, false, 100)
            .await
            .unwrap();

        // 尝试再次加入（已经是Owner）
        let result = room_service.join_room(room.id, user_id).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            AppError::Conflict(_) => (), // 预期错误
            _ => panic!("Expected Conflict error"),
        }
    }

    /// 测试离开聊天室
    #[tokio::test]
    async fn test_leave_room() {
        let db = setup_test_db().await;
        let room_service = RoomService::new(db.clone());
        let user_service = UserService::new(db.clone());

        let (owner_id, _, _) = create_test_user(&user_service, "test_leave_owner").await;
        let (member_id, _, _) = create_test_user(&user_service, "test_leave_member").await;

        let room = room_service
            .create_room("Leave Test Room", None, owner_id, false, 100)
            .await
            .unwrap();

        room_service.join_room(room.id, member_id).await.unwrap();

        // 成员离开
        room_service.leave_room(room.id, member_id).await.unwrap();

        // 验证成员已离开
        let is_member = room_service.is_user_in_room(room.id, member_id).await.unwrap();
        assert!(!is_member);
    }

    /// 测试Owner不能离开聊天室
    #[tokio::test]
    async fn test_owner_cannot_leave() {
        let db = setup_test_db().await;
        let room_service = RoomService::new(db.clone());
        let user_service = UserService::new(db.clone());

        let (owner_id, _, _) = create_test_user(&user_service, "test_owner_leave").await;

        let room = room_service
            .create_room("Owner Leave Test", None, owner_id, false, 100)
            .await
            .unwrap();

        // Owner尝试离开应该失败
        let result = room_service.leave_room(room.id, owner_id).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            AppError::Forbidden => (), // 预期错误
            _ => panic!("Expected Forbidden error"),
        }
    }

    /// 测试踢出成员
    #[tokio::test]
    async fn test_kick_member() {
        let db = setup_test_db().await;
        let room_service = RoomService::new(db.clone());
        let user_service = UserService::new(db.clone());

        let (owner_id, _, _) = create_test_user(&user_service, "test_kick_owner").await;
        let (member_id, _, _) = create_test_user(&user_service, "test_kick_member").await;

        let room = room_service
            .create_room("Kick Test Room", None, owner_id, false, 100)
            .await
            .unwrap();

        room_service.join_room(room.id, member_id).await.unwrap();

        // Owner踢出成员
        room_service.kick_member(room.id, member_id, owner_id).await.unwrap();

        // 验证成员已被踢出
        let is_member = room_service.is_user_in_room(room.id, member_id).await.unwrap();
        assert!(!is_member);
    }

    /// 测试普通成员不能踢人
    #[tokio::test]
    async fn test_member_cannot_kick() {
        let db = setup_test_db().await;
        let room_service = RoomService::new(db.clone());
        let user_service = UserService::new(db.clone());

        let (owner_id, _, _) = create_test_user(&user_service, "test_kick_owner2").await;
        let (member1_id, _, _) = create_test_user(&user_service, "test_kick_member1").await;
        let (member2_id, _, _) = create_test_user(&user_service, "test_kick_member2").await;

        let room = room_service
            .create_room("Kick Permission Test", None, owner_id, false, 100)
            .await
            .unwrap();

        room_service.join_room(room.id, member1_id).await.unwrap();
        room_service.join_room(room.id, member2_id).await.unwrap();

        // 普通成员尝试踢人应该失败
        let result = room_service.kick_member(room.id, member2_id, member1_id).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            AppError::Forbidden => (), // 预期错误
            _ => panic!("Expected Forbidden error"),
        }
    }

    /// 测试不能踢出Owner
    #[tokio::test]
    async fn test_cannot_kick_owner() {
        let db = setup_test_db().await;
        let room_service = RoomService::new(db.clone());
        let user_service = UserService::new(db.clone());

        let (owner_id, _, _) = create_test_user(&user_service, "test_kick_owner3").await;
        let (admin_id, _, _) = create_test_user(&user_service, "test_kick_admin").await;

        let room = room_service
            .create_room("Kick Owner Test", None, owner_id, false, 100)
            .await
            .unwrap();

        room_service.join_room(room.id, admin_id).await.unwrap();
        room_service
            .set_member_role(room.id, admin_id, MemberRole::Admin, owner_id)
            .await
            .unwrap();

        // Admin尝试踢出Owner应该失败
        let result = room_service.kick_member(room.id, owner_id, admin_id).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            AppError::Forbidden => (), // 预期错误
            _ => panic!("Expected Forbidden error"),
        }
    }

    /// 测试设置成员角色
    #[tokio::test]
    async fn test_set_member_role() {
        let db = setup_test_db().await;
        let room_service = RoomService::new(db.clone());
        let user_service = UserService::new(db.clone());

        let (owner_id, _, _) = create_test_user(&user_service, "test_role_owner").await;
        let (member_id, _, _) = create_test_user(&user_service, "test_role_member").await;

        let room = room_service
            .create_room("Role Test Room", None, owner_id, false, 100)
            .await
            .unwrap();

        room_service.join_room(room.id, member_id).await.unwrap();

        // Owner将成员设置为Admin
        room_service
            .set_member_role(room.id, member_id, MemberRole::Admin, owner_id)
            .await
            .unwrap();

        // 验证角色已更新
        let role = room_service.get_member_role(room.id, member_id).await.unwrap();
        assert!(matches!(role, Some(MemberRole::Admin)));
    }

    /// 测试只有Owner可以设置角色
    #[tokio::test]
    async fn test_only_owner_can_set_role() {
        let db = setup_test_db().await;
        let room_service = RoomService::new(db.clone());
        let user_service = UserService::new(db.clone());

        let (owner_id, _, _) = create_test_user(&user_service, "test_role_owner2").await;
        let (admin_id, _, _) = create_test_user(&user_service, "test_role_admin").await;
        let (member_id, _, _) = create_test_user(&user_service, "test_role_member2").await;

        let room = room_service
            .create_room("Role Permission Test", None, owner_id, false, 100)
            .await
            .unwrap();

        room_service.join_room(room.id, admin_id).await.unwrap();
        room_service.join_room(room.id, member_id).await.unwrap();

        // 设置admin为Admin
        room_service
            .set_member_role(room.id, admin_id, MemberRole::Admin, owner_id)
            .await
            .unwrap();

        // Admin尝试设置其他成员角色应该失败
        let result = room_service
            .set_member_role(room.id, member_id, MemberRole::Admin, admin_id)
            .await;
        assert!(result.is_err());

        match result.unwrap_err() {
            AppError::Forbidden => (), // 预期错误
            _ => panic!("Expected Forbidden error"),
        }
    }

    /// 测试获取用户加入的聊天室
    #[tokio::test]
    async fn test_get_user_rooms() {
        let db = setup_test_db().await;
        let room_service = RoomService::new(db.clone());
        let user_service = UserService::new(db.clone());

        let (user_id, _, _) = create_test_user(&user_service, "test_user_rooms").await;

        // 创建多个房间
        room_service
            .create_room("User Room 1", None, user_id, false, 100)
            .await
            .unwrap();
        room_service
            .create_room("User Room 2", None, user_id, false, 100)
            .await
            .unwrap();

        // 获取用户加入的房间
        let rooms = room_service.get_user_rooms(user_id).await.unwrap();
        assert!(rooms.len() >= 2);
    }

    /// 测试权限检查
    #[tokio::test]
    async fn test_permission_checks() {
        let db = setup_test_db().await;
        let room_service = RoomService::new(db.clone());
        let user_service = UserService::new(db.clone());

        let (owner_id, _, _) = create_test_user(&user_service, "test_perm_owner").await;
        let (admin_id, _, _) = create_test_user(&user_service, "test_perm_admin").await;
        let (member_id, _, _) = create_test_user(&user_service, "test_perm_member").await;

        let room = room_service
            .create_room("Permission Test Room", None, owner_id, false, 100)
            .await
            .unwrap();

        room_service.join_room(room.id, admin_id).await.unwrap();
        room_service.join_room(room.id, member_id).await.unwrap();

        room_service
            .set_member_role(room.id, admin_id, MemberRole::Admin, owner_id)
            .await
            .unwrap();

        // 检查Owner权限
        assert!(room_service.is_room_owner(room.id, owner_id).await.unwrap());
        assert!(room_service.can_manage_room(room.id, owner_id).await.unwrap());

        // 检查Admin权限
        assert!(!room_service.is_room_owner(room.id, admin_id).await.unwrap());
        assert!(room_service.can_manage_room(room.id, admin_id).await.unwrap());

        // 检查普通成员权限
        assert!(!room_service.is_room_owner(room.id, member_id).await.unwrap());
        assert!(!room_service.can_manage_room(room.id, member_id).await.unwrap());
    }

    /// 测试私有房间权限
    #[tokio::test]
    async fn test_private_room_access() {
        let db = setup_test_db().await;
        let room_service = RoomService::new(db.clone());
        let user_service = UserService::new(db.clone());

        let (owner_id, _, _) = create_test_user(&user_service, "test_private_owner").await;
        let (outsider_id, _, _) = create_test_user(&user_service, "test_private_outsider").await;

        let room = room_service
            .create_room("Private Room", None, owner_id, true, 100)
            .await
            .unwrap();

        // 外部用户尝试加入私有房间应该失败
        let result = room_service.join_room(room.id, outsider_id).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            AppError::Forbidden => (), // 预期错误
            _ => panic!("Expected Forbidden error"),
        }
    }

    /// 测试房间已满
    #[tokio::test]
    async fn test_room_full() {
        let db = setup_test_db().await;
        let room_service = RoomService::new(db.clone());
        let user_service = UserService::new(db.clone());

        let (owner_id, _, _) = create_test_user(&user_service, "test_full_owner").await;
        let (user1_id, _, _) = create_test_user(&user_service, "test_full_user1").await;
        let (user2_id, _, _) = create_test_user(&user_service, "test_full_user2").await;

        // 创建最大2人的房间
        let room = room_service
            .create_room("Full Test Room", None, owner_id, false, 2)
            .await
            .unwrap();

        // 加入第一个用户（应该成功）
        room_service.join_room(room.id, user1_id).await.unwrap();

        // 加入第二个用户（应该失败，因为房间已满）
        let result = room_service.join_room(room.id, user2_id).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            AppError::Conflict(_) => (), // 预期错误
            _ => panic!("Expected Conflict error"),
        }
    }

    /// 测试获取房间成员列表
    #[tokio::test]
    async fn test_get_room_members_with_users() {
        let db = setup_test_db().await;
        let room_service = RoomService::new(db.clone());
        let user_service = UserService::new(db.clone());

        let (owner_id, _, _) = create_test_user(&user_service, "test_members_owner").await;
        let (member_id, _, _) = create_test_user(&user_service, "test_members_member").await;

        let room = room_service
            .create_room("Members Test Room", None, owner_id, false, 100)
            .await
            .unwrap();

        room_service.join_room(room.id, member_id).await.unwrap();

        // 获取带用户信息的成员列表
        let members = room_service.get_room_members_with_users(room.id).await.unwrap();
        assert_eq!(members.len(), 2);

        // 验证包含用户信息
        let owner_member = members.iter().find(|m| m.user_id == owner_id).unwrap();
        assert_eq!(owner_member.username, "test_members_owner");
        assert!(matches!(owner_member.role, MemberRole::Owner));

        let normal_member = members.iter().find(|m| m.user_id == member_id).unwrap();
        assert_eq!(normal_member.username, "test_members_member");
        assert!(matches!(normal_member.role, MemberRole::Member));
    }
}
