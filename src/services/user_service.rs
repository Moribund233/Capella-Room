use uuid::Uuid;

use crate::{
    db::Database,
    error::{AppError, Result},
    models::user::{User, UserResponse, UserStatus},
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
    pub async fn create_user(&self, username: &str, email: &str, password_hash: &str) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (username, email, password_hash, status)
            VALUES ($1, $2, $3, 'offline')
            RETURNING id, username, email, password_hash, avatar_url, status, created_at, updated_at
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
            SELECT id, username, email, password_hash, avatar_url, status, created_at, updated_at
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
            SELECT id, username, email, password_hash, avatar_url, status, created_at, updated_at
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
            SELECT id, username, email, password_hash, avatar_url, status, created_at, updated_at
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
            RETURNING id, username, email, password_hash, avatar_url, status, created_at, updated_at
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
            "#
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

    /// 获取用户列表
    pub async fn list_users(&self, limit: i64, offset: i64) -> Result<Vec<User>> {
        let users = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, password_hash, avatar_url, status, created_at, updated_at
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

    /// 检查邮箱是否已存在
    pub async fn email_exists(&self, email: &str) -> Result<bool> {
        let exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM users WHERE email = $1
            )
            "#
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
            "#
        )
        .bind(username)
        .fetch_one(self.db.pool())
        .await?;

        Ok(exists)
    }
}
