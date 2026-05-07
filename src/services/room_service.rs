use uuid::Uuid;

use crate::{
    db::Database,
    error::{AppError, Result},
    models::room::{
        DirectRoomResponse, MemberRole, MessagePreview, Room, RoomInvitation,
        RoomInvitationResponse, RoomMember, RoomResponse, RoomType,
    },
    models::user::{UserInfo, UserRole},
    utils::logging::PerformanceTimer,
};

use chrono::Utc;
use rand::Rng;

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
        let mut timer = PerformanceTimer::new("db_create_room");
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
        timer.finish();

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
        let mut timer = PerformanceTimer::new("db_list_rooms");
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
                        COUNT(rm.user_id) as member_count,
                        lm.id as last_message_id,
                        lm.content as last_message_content,
                        lm.sender_name as last_message_sender_name,
                        lm.created_at as last_message_created_at
                    FROM rooms r
                    LEFT JOIN room_members rm ON r.id = rm.room_id
                    LEFT JOIN users u ON r.owner_id = u.id
            LEFT JOIN LATERAL (
                SELECT m.id, m.content, u2.username as sender_name, m.created_at
                FROM messages m
                LEFT JOIN users u2 ON m.sender_id = u2.id
                WHERE m.room_id = r.id AND m.is_deleted = false
                ORDER BY m.created_at DESC
                LIMIT 1
            ) lm ON true
                    WHERE (r.is_private = false OR EXISTS (
                        SELECT 1 FROM room_members WHERE room_id = r.id AND user_id = $1
                    ))
                    AND r.name ILIKE $2
                    GROUP BY r.id, u.username, u.avatar_url, lm.id, lm.content, lm.sender_name, lm.created_at
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
                        COUNT(rm.user_id) as member_count,
                        lm.id as last_message_id,
                        lm.content as last_message_content,
                        lm.sender_name as last_message_sender_name,
                        lm.created_at as last_message_created_at
                    FROM rooms r
                    LEFT JOIN room_members rm ON r.id = rm.room_id
                    LEFT JOIN users u ON r.owner_id = u.id
            LEFT JOIN LATERAL (
                SELECT m.id, m.content, u2.username as sender_name, m.created_at
                FROM messages m
                LEFT JOIN users u2 ON m.sender_id = u2.id
                WHERE m.room_id = r.id AND m.is_deleted = false
                ORDER BY m.created_at DESC
                LIMIT 1
            ) lm ON true
                    WHERE r.is_private = false OR EXISTS (
                        SELECT 1 FROM room_members WHERE room_id = r.id AND user_id = $1
                    )
                    GROUP BY r.id, u.username, u.avatar_url, lm.id, lm.content, lm.sender_name, lm.created_at
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
                        COUNT(rm.user_id) as member_count,
                        lm.id as last_message_id,
                        lm.content as last_message_content,
                        lm.sender_name as last_message_sender_name,
                        lm.created_at as last_message_created_at
                    FROM rooms r
                    LEFT JOIN room_members rm ON r.id = rm.room_id
                    LEFT JOIN users u ON r.owner_id = u.id
            LEFT JOIN LATERAL (
                SELECT m.id, m.content, u2.username as sender_name, m.created_at
                FROM messages m
                LEFT JOIN users u2 ON m.sender_id = u2.id
                WHERE m.room_id = r.id AND m.is_deleted = false
                ORDER BY m.created_at DESC
                LIMIT 1
            ) lm ON true
                    WHERE r.is_private = false
                    AND r.name ILIKE $1
                    GROUP BY r.id, u.username, u.avatar_url, lm.id, lm.content, lm.sender_name, lm.created_at
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
                        COUNT(rm.user_id) as member_count,
                        lm.id as last_message_id,
                        lm.content as last_message_content,
                        lm.sender_name as last_message_sender_name,
                        lm.created_at as last_message_created_at
                    FROM rooms r
                    LEFT JOIN room_members rm ON r.id = rm.room_id
                    LEFT JOIN users u ON r.owner_id = u.id
            LEFT JOIN LATERAL (
                SELECT m.id, m.content, u2.username as sender_name, m.created_at
                FROM messages m
                LEFT JOIN users u2 ON m.sender_id = u2.id
                WHERE m.room_id = r.id AND m.is_deleted = false
                ORDER BY m.created_at DESC
                LIMIT 1
            ) lm ON true
                    WHERE r.is_private = false
                    GROUP BY r.id, u.username, u.avatar_url, lm.id, lm.content, lm.sender_name, lm.created_at
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

        timer.finish();
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
                    COUNT(rm.user_id) as member_count,
                        lm.id as last_message_id,
                        lm.content as last_message_content,
                        lm.sender_name as last_message_sender_name,
                        lm.created_at as last_message_created_at
                FROM rooms r
                LEFT JOIN room_members rm ON r.id = rm.room_id
                LEFT JOIN users u ON r.owner_id = u.id
            LEFT JOIN LATERAL (
                SELECT m.id, m.content, u2.username as sender_name, m.created_at
                FROM messages m
                LEFT JOIN users u2 ON m.sender_id = u2.id
                WHERE m.room_id = r.id AND m.is_deleted = false
                ORDER BY m.created_at DESC
                LIMIT 1
            ) lm ON true
                WHERE r.is_private = false OR EXISTS (
                    SELECT 1 FROM room_members WHERE room_id = r.id AND user_id = $1
                )
                GROUP BY r.id, u.username, u.avatar_url, lm.id, lm.content, lm.sender_name, lm.created_at
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
                    COUNT(rm.user_id) as member_count,
                        lm.id as last_message_id,
                        lm.content as last_message_content,
                        lm.sender_name as last_message_sender_name,
                        lm.created_at as last_message_created_at
                FROM rooms r
                LEFT JOIN room_members rm ON r.id = rm.room_id
                LEFT JOIN users u ON r.owner_id = u.id
            LEFT JOIN LATERAL (
                SELECT m.id, m.content, u2.username as sender_name, m.created_at
                FROM messages m
                LEFT JOIN users u2 ON m.sender_id = u2.id
                WHERE m.room_id = r.id AND m.is_deleted = false
                ORDER BY m.created_at DESC
                LIMIT 1
            ) lm ON true
                WHERE r.is_private = false
                GROUP BY r.id, u.username, u.avatar_url, lm.id, lm.content, lm.sender_name, lm.created_at
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
                COUNT(rm.user_id) as member_count,
                        lm.id as last_message_id,
                        lm.content as last_message_content,
                        lm.sender_name as last_message_sender_name,
                        lm.created_at as last_message_created_at
            FROM rooms r
            LEFT JOIN room_members rm ON r.id = rm.room_id
            LEFT JOIN users u ON r.owner_id = u.id
            LEFT JOIN LATERAL (
                SELECT m.id, m.content, u2.username as sender_name, m.created_at
                FROM messages m
                LEFT JOIN users u2 ON m.sender_id = u2.id
                WHERE m.room_id = r.id AND m.is_deleted = false
                ORDER BY m.created_at DESC
                LIMIT 1
            ) lm ON true
            WHERE r.id = $1
            GROUP BY r.id, u.username, u.avatar_url, lm.id, lm.content, lm.sender_name, lm.created_at
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

    /// 删除聊天室（级联删除相关数据）
    pub async fn delete_room(&self, room_id: Uuid) -> Result<()> {
        let mut tx = self.db.pool().begin().await?;

        // 1. 删除房间成员关联
        sqlx::query(
            r#"
            DELETE FROM room_members WHERE room_id = $1
            "#,
        )
        .bind(room_id)
        .execute(&mut *tx)
        .await?;

        // 2. 删除房间消息
        sqlx::query(
            r#"
            DELETE FROM messages WHERE room_id = $1
            "#,
        )
        .bind(room_id)
        .execute(&mut *tx)
        .await?;

        // 3. 删除房间
        let result = sqlx::query(
            r#"
            DELETE FROM rooms WHERE id = $1
            "#,
        )
        .bind(room_id)
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            tx.rollback().await?;
            return Err(AppError::NotFound);
        }

        tx.commit().await?;
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

    /// 管理员踢出成员（不检查房间成员身份）
    pub async fn admin_kick_member(
        &self,
        room_id: Uuid,
        target_user_id: Uuid,
        _admin_role: &UserRole,
    ) -> Result<()> {
        // 验证房间存在
        let _room = self
            .get_room_by_id(room_id)
            .await?
            .ok_or(AppError::NotFound)?;

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

    /// 管理员设置成员角色（不检查房间成员身份）
    pub async fn admin_set_member_role(
        &self,
        room_id: Uuid,
        target_user_id: Uuid,
        new_role: MemberRole,
        _admin_role: &UserRole,
    ) -> Result<()> {
        // 验证房间存在
        let _room = self
            .get_room_by_id(room_id)
            .await?
            .ok_or(AppError::NotFound)?;

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
                COUNT(rm2.user_id) as member_count,
                lm.id as last_message_id,
                lm.content as last_message_content,
                lm.sender_name as last_message_sender_name,
                lm.created_at as last_message_created_at
            FROM rooms r
            JOIN room_members rm ON r.id = rm.room_id
            LEFT JOIN room_members rm2 ON r.id = rm2.room_id
            LEFT JOIN users u ON r.owner_id = u.id
            LEFT JOIN LATERAL (
                SELECT m.id, m.content, u2.username as sender_name, m.created_at
                FROM messages m
                LEFT JOIN users u2 ON m.sender_id = u2.id
                WHERE m.room_id = r.id AND m.is_deleted = false
                ORDER BY m.created_at DESC
                LIMIT 1
            ) lm ON true
            WHERE rm.user_id = $1
            GROUP BY r.id, u.username, u.avatar_url, lm.id, lm.content, lm.sender_name, lm.created_at
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
                    COUNT(rm.user_id) as member_count,
                        lm.id as last_message_id,
                        lm.content as last_message_content,
                        lm.sender_name as last_message_sender_name,
                        lm.created_at as last_message_created_at
                FROM rooms r
                LEFT JOIN room_members rm ON r.id = rm.room_id
                LEFT JOIN users u ON r.owner_id = u.id
            LEFT JOIN LATERAL (
                SELECT m.id, m.content, u2.username as sender_name, m.created_at
                FROM messages m
                LEFT JOIN users u2 ON m.sender_id = u2.id
                WHERE m.room_id = r.id AND m.is_deleted = false
                ORDER BY m.created_at DESC
                LIMIT 1
            ) lm ON true
                WHERE r.name ILIKE $1
                GROUP BY r.id, u.username, u.avatar_url, lm.id, lm.content, lm.sender_name, lm.created_at
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
                    COUNT(rm.user_id) as member_count,
                        lm.id as last_message_id,
                        lm.content as last_message_content,
                        lm.sender_name as last_message_sender_name,
                        lm.created_at as last_message_created_at
                FROM rooms r
                LEFT JOIN room_members rm ON r.id = rm.room_id
                LEFT JOIN users u ON r.owner_id = u.id
            LEFT JOIN LATERAL (
                SELECT m.id, m.content, u2.username as sender_name, m.created_at
                FROM messages m
                LEFT JOIN users u2 ON m.sender_id = u2.id
                WHERE m.room_id = r.id AND m.is_deleted = false
                ORDER BY m.created_at DESC
                LIMIT 1
            ) lm ON true
                GROUP BY r.id, u.username, u.avatar_url, lm.id, lm.content, lm.sender_name, lm.created_at
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
    pub last_message_id: Option<Uuid>,
    pub last_message_content: Option<String>,
    pub last_message_sender_name: Option<String>,
    pub last_message_created_at: Option<chrono::DateTime<chrono::Utc>>,
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
            last_message: self.last_message_id.map(|id| MessagePreview {
                id,
                content: self.last_message_content.unwrap_or_default(),
                sender_name: self.last_message_sender_name.unwrap_or_default(),
                created_at: self
                    .last_message_created_at
                    .unwrap_or(chrono::DateTime::UNIX_EPOCH),
            }),
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

// ==================== 房间邀请相关方法 ====================

impl RoomService {
    /// 生成随机邀请码（8位字母数字组合）
    fn generate_invite_code() -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let mut rng = rand::thread_rng();
        (0..8)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    /// 创建房间邀请
    /// 如果邀请码冲突会自动重试，最多重试3次
    pub async fn create_invitation(
        &self,
        room_id: Uuid,
        inviter_id: Uuid,
        expires_in_hours: Option<i32>,
        max_uses: Option<i32>,
    ) -> Result<RoomInvitation> {
        // 检查房间是否存在
        let room = self.get_room_by_id(room_id).await?;
        let _room = room.ok_or(AppError::NotFound)?;

        // 检查邀请者权限（只有Owner或Admin可以创建邀请）
        let can_invite = self.can_manage_room(room_id, inviter_id).await?;
        if !can_invite {
            return Err(AppError::Forbidden);
        }

        // 计算过期时间
        let expires_at =
            expires_in_hours.map(|hours| Utc::now() + chrono::Duration::hours(hours as i64));

        // 尝试创建邀请，最多重试3次（处理邀请码冲突）
        const MAX_RETRIES: u32 = 3;

        for attempt in 0..MAX_RETRIES {
            let invite_code = Self::generate_invite_code();

            match sqlx::query_as::<_, RoomInvitation>(
                r#"
                INSERT INTO room_invitations (room_id, inviter_id, invite_code, expires_at, max_uses)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING *
                "#,
            )
            .bind(room_id)
            .bind(inviter_id)
            .bind(&invite_code)
            .bind(expires_at)
            .bind(max_uses)
            .fetch_one(self.db.pool())
            .await
            {
                Ok(invitation) => return Ok(invitation),
                Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
                    // 邀请码冲突，记录并继续重试
                    tracing::warn!(
                        "Invite code conflict on attempt {}/{}, generating new code",
                        attempt + 1,
                        MAX_RETRIES
                    );
                    continue;
                }
                Err(e) => return Err(e.into()),
            }
        }

        // 重试次数用尽，记录错误并返回内部错误
        tracing::error!(
            "Failed to generate unique invite code after {} retries",
            MAX_RETRIES
        );
        Err(AppError::Internal)
    }

    /// 获取房间的邀请列表
    pub async fn get_room_invitations(
        &self,
        room_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<RoomInvitationResponse>> {
        // 检查用户是否有权限查看邀请（成员即可查看）
        let is_member = self.is_user_in_room(room_id, user_id).await?;
        if !is_member {
            return Err(AppError::Forbidden);
        }

        let rows = sqlx::query(
            r#"
            SELECT 
                ri.id,
                ri.room_id,
                ri.inviter_id,
                ri.invite_code,
                ri.expires_at,
                ri.max_uses,
                ri.used_count,
                ri.is_active,
                ri.created_at,
                u.username as inviter_username,
                u.avatar_url as inviter_avatar_url
            FROM room_invitations ri
            JOIN users u ON ri.inviter_id = u.id
            WHERE ri.room_id = $1
            ORDER BY ri.created_at DESC
            "#,
        )
        .bind(room_id)
        .fetch_all(self.db.pool())
        .await?;

        let invitations = rows
            .into_iter()
            .map(|row| {
                use sqlx::Row;
                let invitation = RoomInvitation {
                    id: row.get("id"),
                    room_id: row.get("room_id"),
                    inviter_id: row.get("inviter_id"),
                    invite_code: row.get("invite_code"),
                    expires_at: row.get("expires_at"),
                    max_uses: row.get("max_uses"),
                    used_count: row.get("used_count"),
                    is_active: row.get("is_active"),
                    created_at: row.get("created_at"),
                };
                let inviter = UserInfo::new(
                    row.get("inviter_id"),
                    row.get("inviter_username"),
                    row.get("inviter_avatar_url"),
                );
                invitation.to_response(inviter)
            })
            .collect();

        Ok(invitations)
    }

    /// 撤销邀请
    pub async fn revoke_invitation(
        &self,
        room_id: Uuid,
        invitation_id: Uuid,
        user_id: Uuid,
    ) -> Result<()> {
        // 检查用户权限（Owner或Admin可以撤销任何邀请，其他成员只能撤销自己的）
        let can_manage = self.can_manage_room(room_id, user_id).await?;

        // 获取邀请信息
        let invitation: Option<RoomInvitation> = sqlx::query_as(
            r#"
            SELECT * FROM room_invitations WHERE id = $1 AND room_id = $2
            "#,
        )
        .bind(invitation_id)
        .bind(room_id)
        .fetch_optional(self.db.pool())
        .await?;

        let invitation = invitation.ok_or(AppError::NotFound)?;

        // 如果不是管理员，只能撤销自己的邀请
        if !can_manage && invitation.inviter_id != user_id {
            return Err(AppError::Forbidden);
        }

        // 停用邀请
        let result = sqlx::query(
            r#"
            UPDATE room_invitations SET is_active = FALSE WHERE id = $1
            "#,
        )
        .bind(invitation_id)
        .execute(self.db.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }

        Ok(())
    }

    /// 通过邀请码加入房间
    pub async fn join_by_invite_code(&self, invite_code: &str, user_id: Uuid) -> Result<Uuid> {
        // 查找邀请
        let invitation: Option<RoomInvitation> = sqlx::query_as(
            r#"
            SELECT * FROM room_invitations WHERE invite_code = $1
            "#,
        )
        .bind(invite_code)
        .fetch_optional(self.db.pool())
        .await?;

        let invitation = invitation.ok_or_else(|| AppError::NotFound)?;

        // 检查邀请是否有效
        if !invitation.is_valid() {
            return Err(AppError::Forbidden);
        }

        let room_id = invitation.room_id;

        // 检查用户是否已经是成员
        if self.is_user_in_room(room_id, user_id).await? {
            return Ok(room_id);
        }

        // 检查房间是否已满
        let room = self.get_room_by_id(room_id).await?;
        let room = room.ok_or(AppError::NotFound)?;
        let member_count = self.get_room_member_count(room_id).await?;
        if member_count >= room.max_members as i64 {
            return Err(AppError::Conflict("聊天室已满".to_string()));
        }

        // 开启事务
        let mut tx = self.db.pool().begin().await?;

        // 添加成员
        sqlx::query(
            r#"
            INSERT INTO room_members (room_id, user_id, role)
            VALUES ($1, $2, 'member')
            ON CONFLICT (room_id, user_id) DO NOTHING
            "#,
        )
        .bind(room_id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

        // 更新邀请使用次数
        sqlx::query(
            r#"
            UPDATE room_invitations 
            SET used_count = used_count + 1 
            WHERE id = $1
            "#,
        )
        .bind(invitation.id)
        .execute(&mut *tx)
        .await?;

        // 记录邀请使用
        sqlx::query(
            r#"
            INSERT INTO room_invitation_uses (invitation_id, user_id)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(invitation.id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(room_id)
    }

    /// 验证邀请码是否有效（不加入，仅验证）
    pub async fn validate_invite_code(&self, invite_code: &str) -> Result<Option<RoomInvitation>> {
        let invitation: Option<RoomInvitation> = sqlx::query_as(
            r#"
            SELECT * FROM room_invitations WHERE invite_code = $1
            "#,
        )
        .bind(invite_code)
        .fetch_optional(self.db.pool())
        .await?;

        Ok(invitation.filter(|i| i.is_valid()))
    }

    // ==================== 私聊功能相关方法 ====================

    /// 获取或创建私聊房间
    /// 如果两个用户之间已存在私聊房间，则返回现有房间
    pub async fn get_or_create_direct_room(
        &self,
        user_a_id: Uuid,
        user_b_id: Uuid,
    ) -> Result<DirectRoomResponse> {
        // 不能和自己创建私聊
        if user_a_id == user_b_id {
            return Err(AppError::Validation("不能和自己创建私聊房间".to_string()));
        }

        // 检查是否已存在私聊房间
        if let Some(room) = self.find_direct_room(user_a_id, user_b_id).await? {
            return self.to_direct_room_response(room, user_a_id).await;
        }

        // 创建新的私聊房间
        self.create_direct_room(user_a_id, user_b_id).await
    }

    /// 查找两个用户之间的私聊房间
    async fn find_direct_room(&self, user_a_id: Uuid, user_b_id: Uuid) -> Result<Option<Room>> {
        let room = sqlx::query_as::<_, Room>(
            r#"
            SELECT r.* FROM rooms r
            INNER JOIN room_members rm1 ON r.id = rm1.room_id AND rm1.user_id = $1
            INNER JOIN room_members rm2 ON r.id = rm2.room_id AND rm2.user_id = $2
            WHERE r.room_type = 'direct'
            LIMIT 1
            "#,
        )
        .bind(user_a_id)
        .bind(user_b_id)
        .fetch_optional(self.db.pool())
        .await?;

        Ok(room)
    }

    /// 创建私聊房间
    ///
    /// 注意：数据库中存储的房间名称仅作为初始值，实际展示时
    /// 通过 to_direct_room_response 方法动态获取目标用户的最新用户名
    async fn create_direct_room(
        &self,
        user_a_id: Uuid,
        user_b_id: Uuid,
    ) -> Result<DirectRoomResponse> {
        // 获取对方用户信息
        let target_user = self.get_user_info(user_b_id).await?;

        let mut tx = self.db.pool().begin().await?;

        // 创建私聊房间
        // 数据库中的名称仅作为初始值，实际展示使用动态获取的用户名
        let room = sqlx::query_as::<_, Room>(
            r#"
            INSERT INTO rooms (name, description, owner_id, is_private, max_members, room_type)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(&target_user.username) // 房间名称为对方用户名（初始值）
        .bind(None::<&str>)
        .bind(user_a_id) // 创建者为房主
        .bind(true) // 私聊房间总是私有的
        .bind(2) // 私聊房间最多2人
        .bind(RoomType::Direct)
        .fetch_one(&mut *tx)
        .await?;

        // 添加双方为成员
        sqlx::query(
            r#"
            INSERT INTO room_members (room_id, user_id, role)
            VALUES ($1, $2, 'member'), ($1, $3, 'member')
            "#,
        )
        .bind(room.id)
        .bind(user_a_id)
        .bind(user_b_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(DirectRoomResponse {
            id: room.id,
            name: target_user.username.clone(),
            target_user,
            created_at: room.created_at,
        })
    }

    /// 将房间转换为私聊房间响应
    async fn to_direct_room_response(
        &self,
        room: Room,
        current_user_id: Uuid,
    ) -> Result<DirectRoomResponse> {
        // 获取房间成员（排除当前用户）
        let target_user_id: Option<Uuid> = sqlx::query_scalar(
            r#"
            SELECT user_id FROM room_members
            WHERE room_id = $1 AND user_id != $2
            LIMIT 1
            "#,
        )
        .bind(room.id)
        .bind(current_user_id)
        .fetch_optional(self.db.pool())
        .await?;

        let target_user_id = target_user_id.ok_or(AppError::NotFound)?;
        let target_user = self.get_user_info(target_user_id).await?;

        Ok(DirectRoomResponse {
            id: room.id,
            name: target_user.username.clone(),
            target_user,
            created_at: room.created_at,
        })
    }

    /// 获取用户信息
    async fn get_user_info(&self, user_id: Uuid) -> Result<UserInfo> {
        let row = sqlx::query(
            r#"
            SELECT id, username, avatar_url FROM users WHERE id = $1
            "#,
        )
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

    /// 获取用户的私聊房间列表
    pub async fn get_user_direct_rooms(&self, user_id: Uuid) -> Result<Vec<DirectRoomResponse>> {
        let rooms = sqlx::query_as::<_, Room>(
            r#"
            SELECT r.* FROM rooms r
            INNER JOIN room_members rm ON r.id = rm.room_id
            WHERE rm.user_id = $1 AND r.room_type = 'direct'
            ORDER BY r.updated_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(self.db.pool())
        .await?;

        let mut responses = Vec::new();
        for room in rooms {
            if let Ok(response) = self.to_direct_room_response(room, user_id).await {
                responses.push(response);
            }
        }

        Ok(responses)
    }
}
