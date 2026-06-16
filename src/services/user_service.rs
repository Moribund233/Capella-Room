use uuid::Uuid;

use crate::{
    db::Database,
    error::{AppError, Result},
    models::user::{
        FriendRequest, FriendRequestResponse, FriendRequestStatus, FriendResponse, Friendship,
        SendFriendRequest, User, UserInfo, UserRole, UserStatus,
    },
};

/// 用户服务
#[derive(Clone)]
pub struct UserService {
    db: Database,
}

impl UserService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// 创建用户
    pub async fn create_user(
        &self,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (username, email, password_hash, status, is_active, role)
            VALUES ($1, $2, $3, 'offline', true, 'user')
            RETURNING id, username, email, password_hash, avatar_url, status, is_active, role, email_verified, email_verified_at, created_at, updated_at
            "#
        )
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .fetch_one(self.db.pool())
        .await
        .map_err(|e| match &e {
            sqlx::Error::Database(db_err) => {
                let constraint = db_err.constraint();
                if constraint.is_some() && constraint.unwrap().contains("email") {
                    AppError::Conflict("邮箱已被注册".to_string())
                } else if constraint.is_some() && constraint.unwrap().contains("username") {
                    AppError::Conflict("用户名已被使用".to_string())
                } else {
                    AppError::Database(e)
                }
            }
            _ => AppError::Database(e),
        })?;

        Ok(user)
    }

    /// 创建超级管理员
    pub async fn create_super_admin(
        &self,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (username, email, password_hash, status, is_active, role)
            VALUES ($1, $2, $3, 'offline', true, 'super_admin')
            ON CONFLICT (email) DO UPDATE SET role = 'super_admin'
            RETURNING id, username, email, password_hash, avatar_url, status, is_active, role, email_verified, email_verified_at, created_at, updated_at
            "#
        )
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .fetch_one(self.db.pool())
        .await
        .map_err(|e| match &e {
            sqlx::Error::Database(db_err) => {
                let constraint = db_err.constraint();
                if constraint.is_some() && constraint.unwrap().contains("email") {
                    AppError::Conflict("邮箱已被注册".to_string())
                } else if constraint.is_some() && constraint.unwrap().contains("username") {
                    AppError::Conflict("用户名已被使用".to_string())
                } else {
                    AppError::Database(e)
                }
            }
            _ => AppError::Database(e),
        })?;

        Ok(user)
    }

    /// 创建管理员
    pub async fn create_admin(
        &self,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (username, email, password_hash, status, is_active, role)
            VALUES ($1, $2, $3, 'offline', true, 'admin')
            ON CONFLICT (email) DO UPDATE SET role = 'admin'
            RETURNING id, username, email, password_hash, avatar_url, status, is_active, role, email_verified, email_verified_at, created_at, updated_at
            "#
        )
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .fetch_one(self.db.pool())
        .await
        .map_err(|e| match &e {
            sqlx::Error::Database(db_err) => {
                let constraint = db_err.constraint();
                if constraint.is_some() && constraint.unwrap().contains("email") {
                    AppError::Conflict("邮箱已被注册".to_string())
                } else if constraint.is_some() && constraint.unwrap().contains("username") {
                    AppError::Conflict("用户名已被使用".to_string())
                } else {
                    AppError::Database(e)
                }
            }
            _ => AppError::Database(e),
        })?;

        Ok(user)
    }

    /// 通过ID获取用户
    pub async fn get_user_by_id(&self, user_id: Uuid) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, password_hash, avatar_url, status, is_active, role, email_verified, email_verified_at, created_at, updated_at
            FROM users
            WHERE id = $1
            "#
        )
        .bind(user_id)
        .fetch_optional(self.db.pool())
        .await?;

        Ok(user)
    }

    /// 通过邮箱获取用户
    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, password_hash, avatar_url, status, is_active, role, email_verified, email_verified_at, created_at, updated_at
            FROM users
            WHERE email = $1
            "#
        )
        .bind(email)
        .fetch_optional(self.db.pool())
        .await?;

        Ok(user)
    }

    /// 通过用户名获取用户
    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, password_hash, avatar_url, status, is_active, role, email_verified, email_verified_at, created_at, updated_at
            FROM users
            WHERE username = $1
            "#
        )
        .bind(username)
        .fetch_optional(self.db.pool())
        .await?;

        Ok(user)
    }

    /// 更新用户信息
    pub async fn update_user(
        &self,
        user_id: Uuid,
        username: Option<&str>,
        avatar_url: Option<&str>,
    ) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET
                username = COALESCE($1, username),
                avatar_url = COALESCE($2, avatar_url),
                updated_at = NOW()
            WHERE id = $3
            RETURNING id, username, email, password_hash, avatar_url, status, is_active, role, email_verified, email_verified_at, created_at, updated_at
            "#
        )
        .bind(username)
        .bind(avatar_url)
        .bind(user_id)
        .fetch_one(self.db.pool())
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::NotFound,
            sqlx::Error::Database(db_err) if db_err.constraint().is_some() => {
                AppError::Conflict("用户名已被使用".to_string())
            }
            _ => AppError::Database(e),
        })?;

        Ok(user)
    }

    /// 更新用户状态
    pub async fn update_user_status(&self, user_id: Uuid, status: UserStatus) -> Result<()> {
        let result = sqlx::query(
            r#"
            UPDATE users
            SET status = $1, updated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(status)
        .bind(user_id)
        .execute(self.db.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }

        Ok(())
    }

    /// 更新用户角色
    pub async fn update_user_role(&self, user_id: Uuid, role: UserRole) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET role = $1, updated_at = NOW()
            WHERE id = $2
            RETURNING id, username, email, password_hash, avatar_url, status, is_active, role, email_verified, email_verified_at, created_at, updated_at
            "#
        )
        .bind(role)
        .bind(user_id)
        .fetch_one(self.db.pool())
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::NotFound,
            _ => AppError::Database(e),
        })?;

        Ok(user)
    }

    /// 获取用户列表
    pub async fn list_users(&self, limit: i64, offset: i64) -> Result<Vec<User>> {
        let users = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, password_hash, avatar_url, status, is_active, role, email_verified, email_verified_at, created_at, updated_at
            FROM users
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.db.pool())
        .await?;

        Ok(users)
    }

    /// 搜索用户（支持用户名和邮箱模糊搜索）
    pub async fn search_users(
        &self,
        query: &str,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<User>, i64)> {
        let search_pattern = format!("%{}%", query);

        // 查询用户列表
        let users = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, password_hash, avatar_url, status, is_active, role, email_verified, email_verified_at, created_at, updated_at
            FROM users
            WHERE username ILIKE $1 OR email ILIKE $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#
        )
        .bind(&search_pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.db.pool())
        .await?;

        // 查询总数
        let total: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM users
            WHERE username ILIKE $1 OR email ILIKE $1
            "#,
        )
        .bind(&search_pattern)
        .fetch_one(self.db.pool())
        .await?;

        Ok((users, total))
    }

    /// 统计用户总数
    pub async fn count_users(&self) -> Result<i64> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM users
            "#,
        )
        .fetch_one(self.db.pool())
        .await?;

        Ok(count)
    }

    /// 检查邮箱是否已存在
    pub async fn email_exists(&self, email: &str) -> Result<bool> {
        let exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM users WHERE email = $1
            )
            "#,
        )
        .bind(email)
        .fetch_one(self.db.pool())
        .await?;

        Ok(exists)
    }

    /// 检查用户名是否已存在
    pub async fn username_exists(&self, username: &str) -> Result<bool> {
        let exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM users WHERE username = $1
            )
            "#,
        )
        .bind(username)
        .fetch_one(self.db.pool())
        .await?;

        Ok(exists)
    }

    /// 获取在线用户列表
    pub async fn get_online_users(&self, limit: i64, offset: i64) -> Result<Vec<User>> {
        let users = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, password_hash, avatar_url, status, is_active, role, email_verified, email_verified_at, created_at, updated_at
            FROM users
            WHERE status = 'online'
            ORDER BY updated_at DESC
            LIMIT $1 OFFSET $2
            "#
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.db.pool())
        .await?;

        Ok(users)
    }

    /// 获取指定状态的用户列表
    pub async fn get_users_by_status(
        &self,
        status: UserStatus,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<User>> {
        let users = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, password_hash, avatar_url, status, is_active, role, email_verified, email_verified_at, created_at, updated_at
            FROM users
            WHERE status = $1
            ORDER BY updated_at DESC
            LIMIT $2 OFFSET $3
            "#
        )
        .bind(status)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.db.pool())
        .await?;

        Ok(users)
    }

    /// 更新用户头像
    pub async fn update_user_avatar(&self, user_id: Uuid, avatar_url: Option<&str>) -> Result<()> {
        let result = sqlx::query(
            r#"
            UPDATE users
            SET avatar_url = $1, updated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(avatar_url)
        .bind(user_id)
        .execute(self.db.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }

        Ok(())
    }

    /// 删除用户（软删除）
    pub async fn delete_user(&self, user_id: Uuid) -> Result<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM users WHERE id = $1
            "#,
        )
        .bind(user_id)
        .execute(self.db.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }

        Ok(())
    }

    /// 软删除用户（自服务）：标记为不活动、匿名化个人数据、清除密码
    pub async fn soft_delete_user(&self, user_id: Uuid) -> Result<()> {
        let suffix = &user_id.to_string()[..8];
        let anon_username = format!("deleted_user_{}", suffix);
        let anon_email = format!("deleted_{}@deleted", suffix);

        let result = sqlx::query(
            r#"
            UPDATE users
            SET
                is_active = false,
                username = $1,
                email = $2,
                avatar_url = NULL,
                password_hash = 'deleted',
                updated_at = NOW()
            WHERE id = $3 AND is_active = true
            "#,
        )
        .bind(&anon_username)
        .bind(&anon_email)
        .bind(user_id)
        .execute(self.db.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }

        Ok(())
    }

    /// 禁用/启用用户
    pub async fn set_user_disabled(&self, user_id: Uuid, disabled: bool) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET is_active = $1, updated_at = NOW()
            WHERE id = $2
            RETURNING id, username, email, password_hash, avatar_url, status, is_active, role, email_verified, email_verified_at, created_at, updated_at
            "#
        )
        .bind(!disabled)
        .bind(user_id)
        .fetch_one(self.db.pool())
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::NotFound,
            _ => AppError::Database(e),
        })?;

        Ok(user)
    }

    /// 检查是否有超级管理员
    pub async fn has_super_admin(&self) -> Result<bool> {
        let exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM users WHERE role = 'super_admin'
            )
            "#,
        )
        .fetch_one(self.db.pool())
        .await?;

        Ok(exists)
    }

    /// 更新用户密码
    pub async fn update_password(&self, user_id: Uuid, new_password_hash: &str) -> Result<()> {
        let result = sqlx::query(
            r#"
            UPDATE users
            SET password_hash = $1, updated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(new_password_hash)
        .bind(user_id)
        .execute(self.db.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }

        Ok(())
    }

    /// 获取用户统计信息
    pub async fn get_user_stats(&self, user_id: Uuid) -> Result<UserStats> {
        // 获取加入的聊天室数量
        let joined_rooms: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(DISTINCT room_id)
            FROM room_members
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(self.db.pool())
        .await?;

        // 获取发送的消息数量
        let total_messages: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM messages
            WHERE sender_id = $1 AND is_deleted = false
            "#,
        )
        .bind(user_id)
        .fetch_one(self.db.pool())
        .await?;

        // 计算在线时长（小时）- 基于用户创建时间和最后更新时间估算
        let online_hours: i64 = sqlx::query_scalar(
            r#"
            SELECT COALESCE(
                EXTRACT(EPOCH FROM (NOW() - created_at)) / 3600,
                0
            )::bigint
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(self.db.pool())
        .await?;

        Ok(UserStats {
            joined_rooms,
            total_messages,
            online_hours,
        })
    }
}

/// 用户统计信息
#[derive(Debug, serde::Serialize)]
pub struct UserStats {
    pub joined_rooms: i64,
    pub total_messages: i64,
    pub online_hours: i64,
}

/// 用户增长统计数据
#[derive(Debug, serde::Serialize)]
pub struct UserGrowthStats {
    pub new_users_today: i64,
    pub new_users_this_week: i64,
    pub new_users_this_month: i64,
    pub total_users: i64,
    pub growth_by_day: Vec<DailyUserCount>,
}

/// 每日用户数量
#[derive(Debug, serde::Serialize)]
pub struct DailyUserCount {
    pub date: String,
    pub count: i64,
}

/// 用户行为统计
#[derive(Debug, serde::Serialize)]
pub struct UserBehaviorStats {
    pub avg_messages_per_user: f64,
    pub avg_rooms_per_user: f64,
    pub active_users_today: i64,
    pub active_users_this_week: i64,
}

/// 好友关系统计
#[derive(Debug, serde::Serialize)]
pub struct FriendStats {
    pub total_friendships: i64,
    pub pending_requests: i64,
    pub avg_friends_per_user: f64,
    pub request_accept_rate: f64,
}

// ==================== 好友功能服务方法 ====================

impl UserService {
    /// 搜索用户（按用户名，用于好友功能）
    pub async fn search_users_by_username(
        &self,
        keyword: &str,
        limit: i64,
    ) -> Result<Vec<UserInfo>> {
        let users = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, password_hash, avatar_url, status, is_active, role, email_verified, email_verified_at, created_at, updated_at
            FROM users
            WHERE username ILIKE $1 AND is_active = true
            ORDER BY username
            LIMIT $2
            "#
        )
        .bind(format!("%{}%", keyword))
        .bind(limit)
        .fetch_all(self.db.pool())
        .await?;

        Ok(users
            .into_iter()
            .map(|u| UserInfo::new(u.id, u.username, u.avatar_url))
            .collect())
    }

    /// 发送好友申请
    pub async fn send_friend_request(
        &self,
        sender_id: Uuid,
        request: SendFriendRequest,
    ) -> Result<FriendRequest> {
        // 不能向自己发送申请
        if sender_id == request.target_user_id {
            return Err(AppError::Validation("不能向自己发送好友申请".to_string()));
        }

        // 检查目标用户是否存在
        let target_exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM users WHERE id = $1 AND is_active = true)",
        )
        .bind(request.target_user_id)
        .fetch_one(self.db.pool())
        .await?;

        if !target_exists {
            return Err(AppError::NotFound);
        }

        // 检查是否已经是好友
        let is_friend = self.are_friends(sender_id, request.target_user_id).await?;
        if is_friend {
            return Err(AppError::Conflict("你们已经是好友了".to_string()));
        }

        // 检查是否已有待处理的申请（双向检查）
        let existing_request = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM friend_requests
                WHERE ((sender_id = $1 AND receiver_id = $2) OR (sender_id = $2 AND receiver_id = $1))
                AND status = 'pending'
            )
            "#
        )
        .bind(sender_id)
        .bind(request.target_user_id)
        .fetch_one(self.db.pool())
        .await?;

        if existing_request {
            return Err(AppError::Conflict("已存在待处理的好友申请".to_string()));
        }

        // 创建好友申请
        let friend_request = sqlx::query_as::<_, FriendRequest>(
            r#"
            INSERT INTO friend_requests (sender_id, receiver_id, status, message)
            VALUES ($1, $2, 'pending', $3)
            RETURNING *
            "#,
        )
        .bind(sender_id)
        .bind(request.target_user_id)
        .bind(request.message)
        .fetch_one(self.db.pool())
        .await?;

        Ok(friend_request)
    }

    /// 获取收到的好友申请列表
    pub async fn get_received_friend_requests(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<FriendRequestResponse>> {
        let requests = sqlx::query_as::<_, FriendRequest>(
            r#"
            SELECT * FROM friend_requests
            WHERE receiver_id = $1 AND status = 'pending'
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(self.db.pool())
        .await?;

        let mut responses = Vec::new();
        for request in requests {
            if let Ok(sender) = self.get_user_info(request.sender_id).await {
                responses.push(FriendRequestResponse {
                    id: request.id,
                    sender,
                    status: request.status,
                    message: request.message,
                    created_at: request.created_at,
                });
            }
        }

        Ok(responses)
    }

    /// 获取发送的好友申请列表
    pub async fn get_sent_friend_requests(&self, user_id: Uuid) -> Result<Vec<FriendRequest>> {
        let requests = sqlx::query_as::<_, FriendRequest>(
            r#"
            SELECT * FROM friend_requests
            WHERE sender_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(self.db.pool())
        .await?;

        Ok(requests)
    }

    /// 处理好友申请（接受或拒绝）
    pub async fn handle_friend_request(
        &self,
        user_id: Uuid,
        request_id: Uuid,
        accept: bool,
    ) -> Result<()> {
        // 获取申请信息
        let request: FriendRequest = sqlx::query_as("SELECT * FROM friend_requests WHERE id = $1")
            .bind(request_id)
            .fetch_one(self.db.pool())
            .await
            .map_err(|_| AppError::NotFound)?;

        // 检查是否是接收者
        if request.receiver_id != user_id {
            return Err(AppError::Forbidden);
        }

        // 检查状态
        if !matches!(request.status, FriendRequestStatus::Pending) {
            return Err(AppError::Validation("该申请已处理".to_string()));
        }

        let mut tx = self.db.pool().begin().await?;

        if accept {
            // 更新申请状态为已接受
            sqlx::query("UPDATE friend_requests SET status = 'accepted' WHERE id = $1")
                .bind(request_id)
                .execute(&mut *tx)
                .await?;

            // 创建好友关系（确保 user_id_a < user_id_b）
            let (user_a, user_b) = if request.sender_id < request.receiver_id {
                (request.sender_id, request.receiver_id)
            } else {
                (request.receiver_id, request.sender_id)
            };

            sqlx::query(
                r#"
                INSERT INTO friendships (user_id_a, user_id_b)
                VALUES ($1, $2)
                ON CONFLICT (user_id_a, user_id_b) DO NOTHING
                "#,
            )
            .bind(user_a)
            .bind(user_b)
            .execute(&mut *tx)
            .await?;
        } else {
            // 更新申请状态为已拒绝
            sqlx::query("UPDATE friend_requests SET status = 'rejected' WHERE id = $1")
                .bind(request_id)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    /// 取消发送的好友申请
    pub async fn cancel_friend_request(&self, user_id: Uuid, request_id: Uuid) -> Result<()> {
        let result = sqlx::query(
            r#"
            UPDATE friend_requests
            SET status = 'cancelled'
            WHERE id = $1 AND sender_id = $2 AND status = 'pending'
            "#,
        )
        .bind(request_id)
        .bind(user_id)
        .execute(self.db.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }

        Ok(())
    }

    /// 获取好友列表
    pub async fn get_friends(&self, user_id: Uuid) -> Result<Vec<FriendResponse>> {
        // 查询好友关系
        let friendships: Vec<Friendship> = sqlx::query_as(
            r#"
            SELECT * FROM friendships
            WHERE user_id_a = $1 OR user_id_b = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(self.db.pool())
        .await?;

        let mut friends = Vec::new();
        for friendship in friendships {
            // 获取好友ID（不是当前用户的那个）
            let friend_id = if friendship.user_id_a == user_id {
                friendship.user_id_b
            } else {
                friendship.user_id_a
            };

            if let Ok(friend_info) = self.get_user_info(friend_id).await {
                friends.push(FriendResponse {
                    id: friendship.id,
                    friend: friend_info,
                    created_at: friendship.created_at,
                });
            }
        }

        Ok(friends)
    }

    /// 删除好友
    pub async fn remove_friend(&self, user_id: Uuid, friend_id: Uuid) -> Result<()> {
        let (user_a, user_b) = if user_id < friend_id {
            (user_id, friend_id)
        } else {
            (friend_id, user_id)
        };

        let result = sqlx::query("DELETE FROM friendships WHERE user_id_a = $1 AND user_id_b = $2")
            .bind(user_a)
            .bind(user_b)
            .execute(self.db.pool())
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }

        Ok(())
    }

    /// 检查两个用户是否是好友
    pub async fn are_friends(&self, user_a: Uuid, user_b: Uuid) -> Result<bool> {
        let (user_a, user_b) = if user_a < user_b {
            (user_a, user_b)
        } else {
            (user_b, user_a)
        };

        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM friendships WHERE user_id_a = $1 AND user_id_b = $2)",
        )
        .bind(user_a)
        .bind(user_b)
        .fetch_one(self.db.pool())
        .await?;

        Ok(exists)
    }

    /// 获取用户信息
    async fn get_user_info(&self, user_id: Uuid) -> Result<UserInfo> {
        let row = sqlx::query("SELECT id, username, avatar_url FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(self.db.pool())
            .await?;

        use sqlx::Row;
        Ok(UserInfo::new(
            row.get("id"),
            row.get("username"),
            row.get("avatar_url"),
        ))
    }

    // ==================== 管理员统计方法 ====================

    /// 获取用户增长统计
    pub async fn get_user_growth_stats(&self, days: i64) -> Result<UserGrowthStats> {
        // 获取今日、本周、本月新用户数
        let new_users_today: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM users
            WHERE created_at > NOW() - INTERVAL '1 day'
            "#,
        )
        .fetch_one(self.db.pool())
        .await?;

        let new_users_this_week: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM users
            WHERE created_at > NOW() - INTERVAL '7 days'
            "#,
        )
        .fetch_one(self.db.pool())
        .await?;

        let new_users_this_month: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM users
            WHERE created_at > NOW() - INTERVAL '30 days'
            "#,
        )
        .fetch_one(self.db.pool())
        .await?;

        let total_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(self.db.pool())
            .await?;

        // 获取每日用户增长数据
        let growth_by_day: Vec<DailyUserCount> = sqlx::query_as(
            r#"
            SELECT 
                DATE(created_at) as date,
                COUNT(*) as count
            FROM users
            WHERE created_at > NOW() - INTERVAL '1 day' * $1
            GROUP BY DATE(created_at)
            ORDER BY date ASC
            "#,
        )
        .bind(days)
        .fetch_all(self.db.pool())
        .await?
        .into_iter()
        .map(|(date, count): (chrono::NaiveDate, i64)| DailyUserCount {
            date: date.format("%Y-%m-%d").to_string(),
            count,
        })
        .collect();

        Ok(UserGrowthStats {
            new_users_today,
            new_users_this_week,
            new_users_this_month,
            total_users,
            growth_by_day,
        })
    }

    /// 获取用户行为统计
    pub async fn get_user_behavior_stats(&self) -> Result<UserBehaviorStats> {
        // 人均消息数
        let avg_messages_per_user: f64 = sqlx::query_scalar(
            r#"
            SELECT COALESCE(AVG(msg_count)::float8, 0.0)
            FROM (
                SELECT sender_id, COUNT(*) as msg_count
                FROM messages
                WHERE is_deleted = false
                GROUP BY sender_id
            ) subq
            "#,
        )
        .fetch_one(self.db.pool())
        .await?;

        // 人均加入房间数
        let avg_rooms_per_user: f64 = sqlx::query_scalar(
            r#"
            SELECT COALESCE(AVG(room_count)::float8, 0.0)
            FROM (
                SELECT user_id, COUNT(*) as room_count
                FROM room_members
                GROUP BY user_id
            ) subq
            "#,
        )
        .fetch_one(self.db.pool())
        .await?;

        // 今日活跃用户（发送过消息的用户）
        let active_users_today: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(DISTINCT sender_id)
            FROM messages
            WHERE created_at > NOW() - INTERVAL '1 day'
            AND is_deleted = false
            "#,
        )
        .fetch_one(self.db.pool())
        .await?;

        // 本周活跃用户
        let active_users_this_week: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(DISTINCT sender_id)
            FROM messages
            WHERE created_at > NOW() - INTERVAL '7 days'
            AND is_deleted = false
            "#,
        )
        .fetch_one(self.db.pool())
        .await?;

        Ok(UserBehaviorStats {
            avg_messages_per_user,
            avg_rooms_per_user,
            active_users_today,
            active_users_this_week,
        })
    }

    /// 获取好友关系统计
    pub async fn get_friend_stats(&self) -> Result<FriendStats> {
        // 总好友关系数
        let total_friendships: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM friendships")
                .fetch_one(self.db.pool())
                .await?;

        // 待处理申请数
        let pending_requests: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM friend_requests WHERE status = 'pending'",
        )
        .fetch_one(self.db.pool())
        .await?;

        // 人均好友数
        let avg_friends_per_user: f64 = sqlx::query_scalar(
            r#"
            SELECT COALESCE(AVG(friend_count)::float8, 0.0)
            FROM (
                SELECT 
                    user_id_a as user_id,
                    COUNT(*) as friend_count
                FROM friendships
                GROUP BY user_id_a
                UNION ALL
                SELECT 
                    user_id_b as user_id,
                    COUNT(*) as friend_count
                FROM friendships
                GROUP BY user_id_b
            ) subq
            "#,
        )
        .fetch_one(self.db.pool())
        .await?;

        // 好友申请接受率
        let total_requests: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM friend_requests")
                .fetch_one(self.db.pool())
                .await?;

        let accepted_requests: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM friend_requests WHERE status = 'accepted'",
        )
        .fetch_one(self.db.pool())
        .await?;

        let request_accept_rate = if total_requests > 0 {
            (accepted_requests as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };

        Ok(FriendStats {
            total_friendships,
            pending_requests,
            avg_friends_per_user,
            request_accept_rate,
        })
    }
}
