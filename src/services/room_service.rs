use uuid::Uuid;

use crate::{
    db::Database,
    error::{AppError, Result},
    models::room::{MemberRole, Room, RoomMember, RoomResponse},
    models::user::UserInfo,
};

/// 聊天室服务
pub struct RoomService {
    db: Database,
}

impl RoomService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// 创建聊天室
    /// 创建房间并自动将创建者添加为Owner
    pub async fn create_room(
        &self,
        name: &str,
        description: Option<&str>,
        owner_id: Uuid,
        is_private: bool,
        max_members: i32,
    ) -> Result<Room> {
        let mut tx = self.db.pool().begin().await?;

        // 创建聊天室
        let room = sqlx::query_as::<_, Room>(
            r#"
            INSERT INTO rooms (name, description, owner_id, is_private, max_members)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(name)
        .bind(description)
        .bind(owner_id)
        .bind(is_private)
        .bind(max_members)
        .fetch_one(&mut *tx)
        .await?;

        // 添加创建者为房间成员（Owner角色）
        sqlx::query(
            r#"
            INSERT INTO room_members (room_id, user_id, role)
            VALUES ($1, $2, 'owner')
            "#,
        )
        .bind(room.id)
        .bind(owner_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(room)
    }

    /// 获取聊天室列表
    /// 公开房间所有人可见，私有房间只有成员可见
    pub async fn list_rooms(
        &self,
        user_id: Option<Uuid>,
        search: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<RoomResponse>> {
        let rows = if let Some(uid) = user_id {
            // 登录用户：可以看到所有公开房间 + 自己加入的私有房间
            if let Some(query) = search {
                sqlx::query_as::<_, RoomRow>(
                    r#"
                    SELECT 
                        r.id,
                        r.name,
                        r.description,
                        r.owner_id,
                        u.username as owner_username,
                        u.avatar_url as owner_avatar_url,
                        r.is_private,
                        r.max_members,
                        r.created_at,
                        r.updated_at,
                        COUNT(rm.user_id) as member_count
                    FROM rooms r
                    LEFT JOIN room_members rm ON r.id = rm.room_id
                    LEFT JOIN users u ON r.owner_id = u.id
                    WHERE (r.is_private = false OR EXISTS (
                        SELECT 1 FROM room_members WHERE room_id = r.id AND user_id = $1
                    ))
                    AND r.name ILIKE $2
                    GROUP BY r.id, u.username, u.avatar_url
                    ORDER BY r.created_at DESC
                    LIMIT $3 OFFSET $4
                    "#,
                )
                .bind(uid)
                .bind(format!("%{}%", query))
                .bind(limit)
                .bind(offset)
                .fetch_all(self.db.pool())
                .await?
            } else {
                sqlx::query_as::<_, RoomRow>(
                    r#"
                    SELECT 
                        r.id,
                        r.name,
                        r.description,
                        r.owner_id,
                        u.username as owner_username,
                        u.avatar_url as owner_avatar_url,
                        r.is_private,
                        r.max_members,
                        r.created_at,
                        r.updated_at,
                        COUNT(rm.user_id) as member_count
                    FROM rooms r
                    LEFT JOIN room_members rm ON r.id = rm.room_id
                    LEFT JOIN users u ON r.owner_id = u.id
                    WHERE r.is_private = false OR EXISTS (
                        SELECT 1 FROM room_members WHERE room_id = r.id AND user_id = $1
                    )
                    GROUP BY r.id, u.username, u.avatar_url
                    ORDER BY r.created_at DESC
                    LIMIT $2 OFFSET $3
                    "#,
                )
                .bind(uid)
                .bind(limit)
                .bind(offset)
                .fetch_all(self.db.pool())
                .await?
            }
        } else {
            // 未登录用户：只能看到公开房间
            if let Some(query) = search {
                sqlx::query_as::<_, RoomRow>(
                    r#"
                    SELECT 
                        r.id,
                        r.name,
                        r.description,
                        r.owner_id,
                        u.username as owner_username,
                        u.avatar_url as owner_avatar_url,
                        r.is_private,
                        r.max_members,
                        r.created_at,
                        r.updated_at,
                        COUNT(rm.user_id) as member_count
                    FROM rooms r
                    LEFT JOIN room_members rm ON r.id = rm.room_id
                    LEFT JOIN users u ON r.owner_id = u.id
                    WHERE r.is_private = false
                    AND r.name ILIKE $1
                    GROUP BY r.id, u.username, u.avatar_url
                    ORDER BY r.created_at DESC
                    LIMIT $2 OFFSET $3
                    "#,
                )
                .bind(format!("%{}%", query))
                .bind(limit)
                .bind(offset)
                .fetch_all(self.db.pool())
                .await?
            } else {
                sqlx::query_as::<_, RoomRow>(
                    r#"
                    SELECT 
                        r.id,
                        r.name,
                        r.description,
                        r.owner_id,
                        u.username as owner_username,
                        u.avatar_url as owner_avatar_url,
                        r.is_private,
                        r.max_members,
                        r.created_at,
                        r.updated_at,
                        COUNT(rm.user_id) as member_count
                    FROM rooms r
                    LEFT JOIN room_members rm ON r.id = rm.room_id
                    LEFT JOIN users u ON r.owner_id = u.id
                    WHERE r.is_private = false
                    GROUP BY r.id, u.username, u.avatar_url
                    ORDER BY r.created_at DESC
                    LIMIT $1 OFFSET $2
                    "#,
                )
                .bind(limit)
                .bind(offset)
                .fetch_all(self.db.pool())
                .await?
            }
        };

        Ok(rows.into_iter().map(|r| r.into_response()).collect())
    }

    /// 获取最近更新的聊天室列表
    /// 按 updated_at 降序排序，返回最近活跃的房间
    pub async fn list_recent_rooms(
        &self,
        user_id: Option<Uuid>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<RoomResponse>> {
        let rows = if let Some(uid) = user_id {
            // 登录用户：可以看到所有公开房间 + 自己加入的私有房间
            sqlx::query_as::<_, RoomRow>(
                r#"
                SELECT
                    r.id,
                    r.name,
                    r.description,
                    r.owner_id,
                    u.username as owner_username,
                    u.avatar_url as owner_avatar_url,
                    r.is_private,
                    r.max_members,
                    r.created_at,
                    r.updated_at,
                    COUNT(rm.user_id) as member_count
                FROM rooms r
                LEFT JOIN room_members rm ON r.id = rm.room_id
                LEFT JOIN users u ON r.owner_id = u.id
                WHERE r.is_private = false OR EXISTS (
                    SELECT 1 FROM room_members WHERE room_id = r.id AND user_id = $1
                )
                GROUP BY r.id, u.username, u.avatar_url
                ORDER BY r.updated_at DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(uid)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.db.pool())
            .await?
        } else {
            // 未登录用户：只能看到公开房间
            sqlx::query_as::<_, RoomRow>(
                r#"
                SELECT
                    r.id,
                    r.name,
                    r.description,
                    r.owner_id,
                    u.username as owner_username,
                    u.avatar_url as owner_avatar_url,
                    r.is_private,
                    r.max_members,
                    r.created_at,
                    r.updated_at,
                    COUNT(rm.user_id) as member_count
                FROM rooms r
                LEFT JOIN room_members rm ON r.id = rm.room_id
                LEFT JOIN users u ON r.owner_id = u.id
                WHERE r.is_private = false
                GROUP BY r.id, u.username, u.avatar_url
                ORDER BY r.updated_at DESC
                LIMIT $1 OFFSET $2
                "#,
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(self.db.pool())
            .await?
        };

        Ok(rows.into_iter().map(|r| r.into_response()).collect())
    }

    /// 通过ID获取聊天室
    pub async fn get_room_by_id(&self, room_id: Uuid) -> Result<Option<Room>> {
        let room = sqlx::query_as::<_, Room>(
            r#"
            SELECT * FROM rooms WHERE id = $1
            "#,
        )
        .bind(room_id)
        .fetch_optional(self.db.pool())
        .await?;

        Ok(room)
    }

    /// 获取聊天室详情（包含成员数）
    pub async fn get_room_detail(&self, room_id: Uuid) -> Result<Option<RoomResponse>> {
        let row = sqlx::query_as::<_, RoomRow>(
            r#"
            SELECT
                r.id,
                r.name,
                r.description,
                r.owner_id,
                u.username as owner_username,
                u.avatar_url as owner_avatar_url,
                r.is_private,
                r.max_members,
                r.created_at,
                r.updated_at,
                COUNT(rm.user_id) as member_count
            FROM rooms r
            LEFT JOIN room_members rm ON r.id = rm.room_id
            LEFT JOIN users u ON r.owner_id = u.id
            WHERE r.id = $1
            GROUP BY r.id, u.username, u.avatar_url
            "#,
        )
        .bind(room_id)
        .fetch_optional(self.db.pool())
        .await?;

        Ok(row.map(|r| r.into_response()))
    }

    /// 更新聊天室信息
    pub async fn update_room(
        &self,
        room_id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        is_private: Option<bool>,
        max_members: Option<i32>,
    ) -> Result<Room> {
        let room = sqlx::query_as::<_, Room>(
            r#"
            UPDATE rooms
            SET 
                name = COALESCE($1, name),
                description = COALESCE($2, description),
                is_private = COALESCE($3, is_private),
                max_members = COALESCE($4, max_members)
            WHERE id = $5
            RETURNING *
            "#,
        )
        .bind(name)
        .bind(description)
        .bind(is_private)
        .bind(max_members)
        .bind(room_id)
        .fetch_one(self.db.pool())
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::NotFound,
            _ => e.into(),
        })?;

        Ok(room)
    }

    /// 删除聊天室
    pub async fn delete_room(&self, room_id: Uuid) -> Result<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM rooms WHERE id = $1
            "#,
        )
        .bind(room_id)
        .execute(self.db.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }

        Ok(())
    }

    /// 加入聊天室
    pub async fn join_room(&self, room_id: Uuid, user_id: Uuid) -> Result<()> {
        // 检查房间是否存在
        let room = self.get_room_by_id(room_id).await?;
        let room = room.ok_or(AppError::NotFound)?;

        // 检查是否已经是成员 - 幂等处理：如果已经是成员，直接返回成功
        if self.is_user_in_room(room_id, user_id).await? {
            return Ok(());
        }

        // 检查房间是否已满
        let member_count = self.get_room_member_count(room_id).await?;
        if member_count >= room.max_members as i64 {
            return Err(AppError::Conflict("聊天室已满".to_string()));
        }

        // 如果是私有房间，检查是否有权限加入（这里简化处理，私有房间需要邀请）
        if room.is_private {
            return Err(AppError::Forbidden);
        }

        // 添加成员
        sqlx::query(
            r#"
            INSERT INTO room_members (room_id, user_id, role)
            VALUES ($1, $2, 'member')
            "#,
        )
        .bind(room_id)
        .bind(user_id)
        .execute(self.db.pool())
        .await?;

        Ok(())
    }

    /// 离开聊天室
    pub async fn leave_room(&self, room_id: Uuid, user_id: Uuid) -> Result<()> {
        // 检查是否是成员
        if !self.is_user_in_room(room_id, user_id).await? {
            return Err(AppError::NotFound);
        }

        // 检查是否是Owner
        let member = self.get_room_member(room_id, user_id).await?;
        if let Some(m) = member {
            if matches!(m.role, MemberRole::Owner) {
                return Err(AppError::Forbidden);
            }
        }

        // 删除成员记录
        sqlx::query(
            r#"
            DELETE FROM room_members WHERE room_id = $1 AND user_id = $2
            "#,
        )
        .bind(room_id)
        .bind(user_id)
        .execute(self.db.pool())
        .await?;

        Ok(())
    }

    /// 踢出成员
    pub async fn kick_member(
        &self,
        room_id: Uuid,
        target_user_id: Uuid,
        operator_id: Uuid,
    ) -> Result<()> {
        // 检查操作者权限
        let operator = self.get_room_member(room_id, operator_id).await?;
        let operator = operator.ok_or(AppError::Forbidden)?;

        if !operator.is_admin_or_owner() {
            return Err(AppError::Forbidden);
        }

        // 不能踢出Owner
        let target = self.get_room_member(room_id, target_user_id).await?;
        if let Some(t) = target {
            if matches!(t.role, MemberRole::Owner) {
                return Err(AppError::Forbidden);
            }
        }

        // 删除成员记录
        let result = sqlx::query(
            r#"
            DELETE FROM room_members WHERE room_id = $1 AND user_id = $2
            "#,
        )
        .bind(room_id)
        .bind(target_user_id)
        .execute(self.db.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }

        Ok(())
    }

    /// 设置成员角色
    pub async fn set_member_role(
        &self,
        room_id: Uuid,
        target_user_id: Uuid,
        new_role: MemberRole,
        operator_id: Uuid,
    ) -> Result<()> {
        // 检查操作者权限（只有Owner可以设置角色）
        let operator = self.get_room_member(room_id, operator_id).await?;
        let operator = operator.ok_or(AppError::Forbidden)?;

        if !matches!(operator.role, MemberRole::Owner) {
            return Err(AppError::Forbidden);
        }

        // 不能修改自己的角色
        if target_user_id == operator_id {
            return Err(AppError::Forbidden);
        }

        // 更新角色
        let result = sqlx::query(
            r#"
            UPDATE room_members
            SET role = $1
            WHERE room_id = $2 AND user_id = $3
            "#,
        )
        .bind(new_role)
        .bind(room_id)
        .bind(target_user_id)
        .execute(self.db.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }

        Ok(())
    }

    /// 检查用户是否在聊天室
    pub async fn is_user_in_room(&self, room_id: Uuid, user_id: Uuid) -> Result<bool> {
        let exists: (bool,) = sqlx::query_as(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM room_members WHERE room_id = $1 AND user_id = $2
            )
            "#,
        )
        .bind(room_id)
        .bind(user_id)
        .fetch_one(self.db.pool())
        .await?;

        Ok(exists.0)
    }

    /// 获取聊天室成员
    pub async fn get_room_members(&self, room_id: Uuid) -> Result<Vec<RoomMember>> {
        let members = sqlx::query_as::<_, RoomMember>(
            r#"
            SELECT * FROM room_members WHERE room_id = $1
            ORDER BY joined_at ASC
            "#,
        )
        .bind(room_id)
        .fetch_all(self.db.pool())
        .await?;

        Ok(members)
    }

    /// 获取聊天室成员详情（包含用户信息）
    pub async fn get_room_members_with_users(
        &self,
        room_id: Uuid,
    ) -> Result<Vec<RoomMemberWithUser>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                rm.room_id,
                rm.user_id,
                rm.role as "role",
                rm.joined_at,
                u.username,
                u.email,
                u.avatar_url,
                u.status as "user_status"
            FROM room_members rm
            JOIN users u ON rm.user_id = u.id
            WHERE rm.room_id = $1
            ORDER BY rm.joined_at ASC
            "#,
        )
        .bind(room_id)
        .fetch_all(self.db.pool())
        .await?;

        let members = rows
            .into_iter()
            .map(|row| {
                use sqlx::Row;
                RoomMemberWithUser {
                    room_id: row.get("room_id"),
                    user_id: row.get("user_id"),
                    role: row.get("role"),
                    joined_at: row.get("joined_at"),
                    username: row.get("username"),
                    email: row.get("email"),
                    avatar_url: row.get("avatar_url"),
                    user_status: row.get("user_status"),
                }
            })
            .collect();

        Ok(members)
    }

    /// 获取特定成员信息
    pub async fn get_room_member(
        &self,
        room_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<RoomMember>> {
        let member = sqlx::query_as::<_, RoomMember>(
            r#"
            SELECT * FROM room_members WHERE room_id = $1 AND user_id = $2
            "#,
        )
        .bind(room_id)
        .bind(user_id)
        .fetch_optional(self.db.pool())
        .await?;

        Ok(member)
    }

    /// 获取成员角色
    pub async fn get_member_role(
        &self,
        room_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<MemberRole>> {
        let role: Option<(MemberRole,)> = sqlx::query_as(
            r#"
            SELECT role FROM room_members WHERE room_id = $1 AND user_id = $2
            "#,
        )
        .bind(room_id)
        .bind(user_id)
        .fetch_optional(self.db.pool())
        .await?;

        Ok(role.map(|r| r.0))
    }

    /// 获取聊天室成员数量
    pub async fn get_room_member_count(&self, room_id: Uuid) -> Result<i64> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM room_members WHERE room_id = $1
            "#,
        )
        .bind(room_id)
        .fetch_one(self.db.pool())
        .await?;

        Ok(count.0)
    }

    /// 获取用户加入的聊天室
    pub async fn get_user_rooms(&self, user_id: Uuid) -> Result<Vec<RoomResponse>> {
        let rows = sqlx::query_as::<_, RoomRow>(
            r#"
            SELECT
                r.id,
                r.name,
                r.description,
                r.owner_id,
                u.username as owner_username,
                u.avatar_url as owner_avatar_url,
                r.is_private,
                r.max_members,
                r.created_at,
                r.updated_at,
                COUNT(rm2.user_id) as member_count
            FROM rooms r
            JOIN room_members rm ON r.id = rm.room_id
            LEFT JOIN room_members rm2 ON r.id = rm2.room_id
            LEFT JOIN users u ON r.owner_id = u.id
            WHERE rm.user_id = $1
            GROUP BY r.id, u.username, u.avatar_url
            ORDER BY r.created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(self.db.pool())
        .await?;

        Ok(rows.into_iter().map(|r| r.into_response()).collect())
    }

    /// 检查用户是否有权限管理房间（Owner或Admin）
    pub async fn can_manage_room(&self, room_id: Uuid, user_id: Uuid) -> Result<bool> {
        let role = self.get_member_role(room_id, user_id).await?;
        Ok(matches!(
            role,
            Some(MemberRole::Owner) | Some(MemberRole::Admin)
        ))
    }

    /// 检查用户是否是房间所有者
    pub async fn is_room_owner(&self, room_id: Uuid, user_id: Uuid) -> Result<bool> {
        let role = self.get_member_role(room_id, user_id).await?;
        Ok(matches!(role, Some(MemberRole::Owner)))
    }

    /// 管理员：获取所有房间列表（包括私有房间）
    pub async fn list_all_rooms(
        &self,
        search: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<RoomResponse>> {
        let rows = if let Some(query) = search {
            sqlx::query_as::<_, RoomRow>(
                r#"
                SELECT
                    r.id,
                    r.name,
                    r.description,
                    r.owner_id,
                    u.username as owner_username,
                    u.avatar_url as owner_avatar_url,
                    r.is_private,
                    r.max_members,
                    r.created_at,
                    r.updated_at,
                    COUNT(rm.user_id) as member_count
                FROM rooms r
                LEFT JOIN room_members rm ON r.id = rm.room_id
                LEFT JOIN users u ON r.owner_id = u.id
                WHERE r.name ILIKE $1
                GROUP BY r.id, u.username, u.avatar_url
                ORDER BY r.created_at DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(format!("%{}%", query))
            .bind(limit)
            .bind(offset)
            .fetch_all(self.db.pool())
            .await?
        } else {
            sqlx::query_as::<_, RoomRow>(
                r#"
                SELECT
                    r.id,
                    r.name,
                    r.description,
                    r.owner_id,
                    u.username as owner_username,
                    u.avatar_url as owner_avatar_url,
                    r.is_private,
                    r.max_members,
                    r.created_at,
                    r.updated_at,
                    COUNT(rm.user_id) as member_count
                FROM rooms r
                LEFT JOIN room_members rm ON r.id = rm.room_id
                LEFT JOIN users u ON r.owner_id = u.id
                GROUP BY r.id, u.username, u.avatar_url
                ORDER BY r.created_at DESC
                LIMIT $1 OFFSET $2
                "#,
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(self.db.pool())
            .await?
        };

        Ok(rows.into_iter().map(|r| r.into_response()).collect())
    }

    /// 管理员：统计所有房间数
    pub async fn count_all_rooms(&self) -> Result<i64> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM rooms
            "#,
        )
        .fetch_one(self.db.pool())
        .await?;

        Ok(count.0)
    }

    /// 管理员：强制删除房间（不检查权限）
    pub async fn force_delete_room(&self, room_id: Uuid) -> Result<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM rooms WHERE id = $1
            "#,
        )
        .bind(room_id)
        .execute(self.db.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }

        Ok(())
    }
}

/// 用于查询的房间行（包含成员数和所有者信息）
#[derive(Debug, Clone, sqlx::FromRow)]
#[allow(dead_code)]
struct RoomRow {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: Uuid,
    pub owner_username: String,
    pub owner_avatar_url: Option<String>,
    pub is_private: bool,
    pub max_members: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub member_count: i64,
}

impl RoomRow {
    fn into_response(self) -> RoomResponse {
        RoomResponse {
            id: self.id,
            name: self.name,
            description: self.description,
            owner: UserInfo::new(self.owner_id, self.owner_username, self.owner_avatar_url),
            is_private: self.is_private,
            max_members: self.max_members,
            member_count: self.member_count,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

/// 成员详情（包含用户信息）
#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct RoomMemberWithUser {
    pub room_id: Uuid,
    pub user_id: Uuid,
    pub role: MemberRole,
    pub joined_at: chrono::DateTime<chrono::Utc>,
    pub username: String,
    pub email: String,
    pub avatar_url: Option<String>,
    #[sqlx(rename = "user_status")]
    pub user_status: crate::models::user::UserStatus,
}
